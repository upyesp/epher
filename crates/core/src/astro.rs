//! The astronomy module (ADR-0037): the calculator's time, angle, and
//! optics functions, and — from the ephemeris slice — the accessor
//! functions over the `solar-ephemeris` facade.
//!
//! Conventions, per the ADR:
//! - Functions return **counts in natural units** (degrees, hours, days,
//!   astronomical units, magnitudes, janskys); unit suffixes convert
//!   counts to SI. `sin` speaks radians, so `sin(30 deg)` composes.
//! - Dates are plain numbers: `jd(2000, 1, 1, 12)` — the language has no
//!   strings to spare.
//! - Everything here is shadowable like `pi`: resolution order is user
//!   variable, user constant, builtin.

use crate::{domain_error, one_float, three_floats, two_floats, EpherError, Value};

/// Dispatch an astronomy builtin. Returns `None` when the name is not an
/// astronomy function, so the caller can fall through to its own unknown-name
/// error.
pub(crate) fn call(name: &str, args: Vec<Value>) -> Option<Result<Value, EpherError>> {
    let result = match name {
        "jd" => calendar_jd(name, &args, None),
        "mjd" => calendar_jd(name, &args, Some(-2_400_000.5)),
        "now" => now(&args),
        "hms2deg" => three_floats(name, &args)
            .map(|(h, m, s)| (h + m / 60.0 + s / 3600.0) * 15.0)
            .map(Value::Float),
        "dms2deg" => three_floats(name, &args)
            .map(|(d, m, s)| d.signum() * (d.abs() + m / 60.0 + s / 3600.0))
            .map(Value::Float),
        "deg2hms" => one_float(name, &args).map(degrees_to_hms).map(Value::Str),
        "deg2dms" => one_float(name, &args).map(degrees_to_dms).map(Value::Str),
        "lst" => lst(name, &args),
        "delta_t" => delta_t(name, &args),
        "airmass" => airmass(name, &args),
        "dawes" => dawes(name, &args),
        "dist_mod" => one_float(name, &args)
            .map(|mu| 10f64.powf(1.0 + mu / 5.0))
            .map(Value::Float),
        "kepler" => two_floats(name, &args).and_then(|(m, e)| kepler(name, m, e)),
        "mag2jy" => one_float(name, &args)
            .map(|m| 3631.0 * 10f64.powf(-0.4 * m))
            .map(Value::Float),
        "jy2mag" => jy2mag(name, &args),
        _ => return None,
    };
    Some(result)
}

/// Julian Date from a calendar date `jd(y, m, d [, hr])`; `mjd` subtracts
/// the MJD offset (the MJD epoch, 1858-11-17 00:00, is 0). The calendar
/// arithmetic is the plain Gregorian recipe (Fliegel & Van Flandern),
/// independent of any ephemeris.
fn calendar_jd(name: &str, args: &[Value], offset: Option<f64>) -> Result<Value, EpherError> {
    let (y, m, d, hr) = match args.len() {
        3 => {
            let (a, b, c) = three_floats(name, args)?;
            (a, b, c, 0.0)
        }
        4 => match args {
            [Value::Float(a), Value::Float(b), Value::Float(c), Value::Float(h)] => {
                (*a, *b, *c, *h)
            }
            _ => {
                return Err(EpherError::Type(format!(
                    "{name} expects numbers, got {args:?}"
                )))
            }
        },
        n => {
            return Err(EpherError::Type(format!(
                "{name} expects 3 or 4 numbers (year, month, day [, hour]), got {n}"
            )))
        }
    };
    let (Some(y), Some(m), Some(d)) = (
        crate::float_to_int(y),
        crate::float_to_int(m),
        crate::float_to_int(d),
    ) else {
        return Err(EpherError::Type(format!(
            "{name} expects a whole-number year, month, and day, got {y}, {m}, {d}"
        )));
    };
    if !(1..=12).contains(&m) {
        return Err(domain_error(format!("month {m} is outside 1..12")));
    }
    if !(1..=31).contains(&d) {
        return Err(domain_error(format!("day {d} is outside 1..31")));
    }
    let mut yy = y;
    let mut mm = m;
    if mm <= 2 {
        yy -= 1;
        mm += 12;
    }
    let a = (yy as f64 / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();
    let jd = (365.25 * (yy as f64 + 4716.0)).floor()
        + (30.6001 * (mm as f64 + 1.0)).floor()
        + d as f64
        + b
        - 1524.5
        + hr / 24.0;
    Ok(Value::Float(jd + offset.unwrap_or(0.0)))
}

/// The current Julian Date from the host clock — epher's first
/// non-deterministic builtin (ADR-0037). Wasm reads the JavaScript Date
/// clock (the same `js_sys::Date::now()` the animation transport uses);
/// native reads the system clock.
fn now(args: &[Value]) -> Result<Value, EpherError> {
    match args {
        [] => Ok(Value::Float(2_440_587.5 + now_unix_seconds() / 86_400.0)),
        _ => Err(EpherError::Type(format!(
            "now expects 0 arguments, got {}",
            args.len()
        ))),
    }
}

#[cfg(target_arch = "wasm32")]
fn now_unix_seconds() -> f64 {
    js_sys::Date::now() / 1000.0
}

#[cfg(not(target_arch = "wasm32"))]
fn now_unix_seconds() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// Local sidereal time in hours: Greenwich apparent sidereal time plus the
/// observer's east longitude, wrapped to 0..24 (ADR-0037).
fn lst(name: &str, args: &[Value]) -> Result<Value, EpherError> {
    let (jd, lon_deg) = two_floats(name, args)?;
    let astro = solar_ephemeris::timescales::AstroTime::from_jd_utc(jd);
    let t = solar_ephemeris::time::centuries(astro.jd_tt);
    let (dpsi, deps) = solar_ephemeris::time::nutation_deg(t);
    let eps_true = solar_ephemeris::time::mean_obliquity_deg(t) + deps;
    let gast = solar_ephemeris::time::gast_deg(astro.jd_ut1, dpsi, eps_true);
    let lst_deg = (gast + lon_deg).rem_euclid(360.0);
    Ok(Value::Float(lst_deg / 15.0))
}

/// TT - UT1 in seconds (the Espenak-Meeus polynomial band, -500 to +2150),
/// read through the ephemeris crate so the whole program agrees on one
/// Earth-clock correction (ADR-0037).
fn delta_t(name: &str, args: &[Value]) -> Result<Value, EpherError> {
    let jd = one_float(name, args)?;
    let year = solar_ephemeris::time::year_from_jd(jd);
    Ok(Value::Float(solar_ephemeris::time::delta_t_seconds(year)))
}

/// Degrees to `h m s` right-ascension text — hours are 15 degrees. The
/// display-only `Str`, the same mechanism as `hex` (ADR-0022).
fn degrees_to_hms(deg: f64) -> String {
    let total_hours = deg.rem_euclid(360.0) / 15.0;
    let h = total_hours.floor();
    let m_total = (total_hours - h) * 60.0;
    let mut m = m_total.floor();
    let mut s = ((m_total - m) * 60.0).round();
    if s >= 60.0 {
        s -= 60.0;
        m += 1.0;
    }
    if m >= 60.0 {
        m -= 60.0;
    }
    format!("{}h {}m {}s", h as i64, m as i64, s as i64)
}

/// Degrees to signed `D° M' S"` sexagesimal text; the sign rides the
/// degrees component, the way declinations are written.
fn degrees_to_dms(deg: f64) -> String {
    let sign = if deg < 0.0 { "-" } else { "" };
    let a = deg.abs();
    let d = a.floor();
    let m_total = (a - d) * 60.0;
    let mut m = m_total.floor();
    let mut s = ((m_total - m) * 60.0).round();
    if s >= 60.0 {
        s -= 60.0;
        m += 1.0;
    }
    if m >= 60.0 {
        m -= 60.0;
    }
    format!("{sign}{}\u{b0} {}' {}\"", d as i64, m as i64, s as i64)
}

/// Airmass sec(z) against a positive altitude in degrees.
fn airmass(name: &str, args: &[Value]) -> Result<Value, EpherError> {
    let alt = one_float(name, args)?;
    if alt <= 0.0 || alt > 90.0 {
        return Err(domain_error(format!(
            "airmass needs an altitude in 0 < alt <= 90 degrees, got {alt}"
        )));
    }
    Ok(Value::Float(1.0 / (90.0 - alt).to_radians().cos()))
}

/// Dawes' resolving power: 116 divided by the aperture in millimetres,
/// in arcseconds.
fn dawes(name: &str, args: &[Value]) -> Result<Value, EpherError> {
    let d = one_float(name, args)?;
    if d <= 0.0 {
        return Err(domain_error(format!(
            "dawes needs a positive aperture in millimetres, got {d}"
        )));
    }
    Ok(Value::Float(116.0 / d))
}

/// Kepler's equation `E - e sin E = M`, solved by Newton-Raphson from the
/// classic starter (Meeus ch. 30). Degrees in, degrees out.
fn kepler(name: &str, m_deg: f64, e: f64) -> Result<Value, EpherError> {
    if !(0.0..1.0).contains(&e) {
        return Err(domain_error(format!(
            "{name} needs an eccentricity in 0..1, got {e}"
        )));
    }
    let m = m_deg.to_radians();
    let mut ea = m + e * m.sin();
    for _ in 0..60 {
        let residual = ea - e * ea.sin() - m;
        if residual.abs() < 1e-14 {
            break;
        }
        let slope = 1.0 - e * ea.cos();
        if slope.abs() < 1e-12 {
            break;
        }
        ea -= residual / slope;
    }
    if (ea - e * ea.sin() - m).abs() > 1e-9 {
        return Err(domain_error(format!(
            "{name} did not converge for M = {m_deg}, e = {e}"
        )));
    }
    Ok(Value::Float(ea.to_degrees().rem_euclid(360.0)))
}

/// Magnitude to flux density on the AB system: magnitude 0 is 3631 Jy, so
/// `mag2jy(20) Jy` walks the ADR's count-then-convert path.
fn jy2mag(name: &str, args: &[Value]) -> Result<Value, EpherError> {
    let jy = one_float(name, args)?;
    if jy <= 0.0 {
        return Err(domain_error(format!(
            "jy2mag needs a positive flux density in Jy, got {jy}"
        )));
    }
    Ok(Value::Float(-2.5 * (jy / 3631.0).log10()))
}
