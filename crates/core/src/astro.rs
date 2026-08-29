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
        "ra" => ra_fn(name, &args),
        "decl" => decl_fn(name, &args),
        "dist" => dist_fn(name, &args),
        "alt" => alt_fn(name, &args),
        "az" => az_fn(name, &args),
        "rise" => event_fn(name, &args, "rise_jd"),
        "set" => event_fn(name, &args, "set_jd"),
        "transit" => event_fn(name, &args, "transit_jd"),
        "mag" => mag_fn(name, &args),
        "phase" => phase_fn(name, &args),
        "illum" => illum_fn(name, &args),
        "diam" => diam_fn(name, &args),
        "march_equinox" => march_equinox(name, &args),
        "june_solstice" => june_solstice(name, &args),
        "september_equinox" => september_equinox(name, &args),
        "december_solstice" => december_solstice(name, &args),
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

// ===== Ephemeris accessors (ADR-0037) =====
//
// The ephemeris backend is `solar-ephemeris` 0.2.0, exact-pinned, and
// reached only through this module (ADR-0037's facade rule). The crate's
// supported external contract is its two versioned JSON snapshots
// (`ephemeris-snapshot.v2` for the sky, `system-snapshot.v1` for the
// heliocentric system view); its deeper modules that the facade also
// reads (`time`, `coords`, `physics`, `top2013`, `timescales`) are
// public and stable. Pluto is the facade's own: the crate stops at
// Neptune, so Pluto rides JPL's approximate Keplerian elements
// (1800-2050, arcminute grade, honestly documented).

use solar_ephemeris::coords::AU_KM;

/// One solar-system body: its DSL number, the crate's name for it (the
/// JSON contract's key and the magnitude table's key), and its radius
/// for angular sizes the facade computes itself.
struct BodyDef {
    number: i64,
    name: &'static str,
    radius_km: f64,
}

const BODIES: [BodyDef; 11] = [
    BodyDef { number: 1, name: "Mercury", radius_km: 2439.7 },
    BodyDef { number: 2, name: "Venus", radius_km: 6051.8 },
    BodyDef { number: 3, name: "Earth", radius_km: 6378.137 },
    BodyDef { number: 4, name: "Mars", radius_km: 3389.5 },
    BodyDef { number: 5, name: "Jupiter", radius_km: 69911.0 },
    BodyDef { number: 6, name: "Saturn", radius_km: 58232.0 },
    BodyDef { number: 7, name: "Uranus", radius_km: 25362.0 },
    BodyDef { number: 8, name: "Neptune", radius_km: 24622.0 },
    BodyDef { number: 9, name: "Pluto", radius_km: 1188.3 },
    BodyDef { number: 10, name: "Sun", radius_km: 695700.0 },
    BodyDef { number: 11, name: "Moon", radius_km: 1737.4 },
];

fn body_from_number(number: i64) -> Option<&'static BodyDef> {
    BODIES.iter().find(|b| b.number == number)
}

/// Resolve the first argument as a body number. Earth is the observer:
/// it has no geocentric place to report, so the observable bodies are
/// 1, 2, 4..11.
fn body_arg(name: &str, args: &[Value]) -> Result<(&'static BodyDef, f64), EpherError> {
    let (number, jd) = match args {
        [Value::Float(a), Value::Float(b), ..] => (crate::float_to_int(*a), *b),
        _ => {
            return Err(EpherError::Type(format!(
                "{name} expects (body, jd, ...): a body number and a Julian Date"
            )))
        }
    };
    let number = number.ok_or_else(|| {
        EpherError::Type(format!("{name} expects a whole-number body, got non-integer"))
    })?;
    let body = body_from_number(number)
    .ok_or_else(|| {
        domain_error(format!(
            "unknown body {number}: Mercury 1..Neptune 8, Pluto 9, Sun 10, Moon 11"
        ))
    })?;
    if body.number == 3 {
        return Err(domain_error(
            "Earth is the observer, not a target: pick a body 1, 2, 4..11",
        ));
    }
    Ok((body, jd))
}

/// The crate's sky snapshot (its `ephemeris-snapshot.v2` contract) as
/// parsed JSON: apparent places, distances, angular sizes, and the
/// rise/transit/set events for the observer's local mean-solar day.
/// The full snapshot carries every body's event scan, so one build
/// costs tens of milliseconds; a one-entry memo makes the common shape
/// (several accessors at the same instant, the solar3d scene per
/// playback tick) pay it once.
fn sky_snapshot(jd: f64, lat: f64, lon: f64) -> Result<serde_json::Value, EpherError> {
    thread_local! {
        static SKY: std::cell::RefCell<Option<serde_json::Value>> =
            const { std::cell::RefCell::new(None) };
    }
    if let Some(cached) = SKY.with(|cell| cell.borrow().clone()) {
        if cached
            .get("_memo_jd")
            .and_then(serde_json::Value::as_f64)
            == Some(jd)
            && cached.get("_memo_lat").and_then(serde_json::Value::as_f64) == Some(lat)
            && cached.get("_memo_lon").and_then(serde_json::Value::as_f64) == Some(lon)
        {
            return Ok(cached);
        }
    }
    let json = solar_ephemeris::sky_snapshot_json(jd, lat, lon, 0.0);
    let mut parsed: serde_json::Value =
        serde_json::from_str(&json).map_err(|e| EpherError::Domain(format!(
            "ephemeris snapshot unreadable: {e}"
        )))?;
    if let Some(obj) = parsed.as_object_mut() {
        obj.insert("_memo_jd".into(), serde_json::json!(jd));
        obj.insert("_memo_lat".into(), serde_json::json!(lat));
        obj.insert("_memo_lon".into(), serde_json::json!(lon));
    }
    SKY.with(|cell| *cell.borrow_mut() = Some(parsed.clone()));
    Ok(parsed)
}

/// The crate's system snapshot (`system-snapshot.v1`): heliocentric
/// ecliptic-J2000 positions (AU), magnitudes, phases, and osculating
/// elements for the eight planets plus the Moon. One-entry memo, same
/// rationale as [`sky_snapshot`].
fn system_snapshot(jd: f64) -> Result<serde_json::Value, EpherError> {
    thread_local! {
        static SYSTEM: std::cell::RefCell<Option<(f64, serde_json::Value)>> =
            const { std::cell::RefCell::new(None) };
    }
    if let Some((cached_jd, cached)) = SYSTEM.with(|cell| cell.borrow().clone()) {
        if cached_jd == jd {
            return Ok(cached);
        }
    }
    let json = solar_ephemeris::system_snapshot_json(jd);
    let parsed: serde_json::Value = serde_json::from_str(&json).map_err(|e| {
        EpherError::Domain(format!("system snapshot unreadable: {e}"))
    })?;
    SYSTEM.with(|cell| *cell.borrow_mut() = Some((jd, parsed.clone())));
    Ok(parsed)
}

fn json_f64(obj: &serde_json::Value, key: &str) -> Result<f64, EpherError> {
    obj.get(key)
        .and_then(|v| v.as_f64())
        .ok_or_else(|| EpherError::Domain(format!("ephemeris snapshot missing {key}")))
}

/// Find one body's entry in a snapshot's bodies array.
fn snapshot_body<'a>(
    snapshot: &'a serde_json::Value,
    body_name: &str,
) -> Result<&'a serde_json::Value, EpherError> {
    snapshot
        .get("bodies")
        .and_then(|bodies| bodies.as_array())
        .and_then(|bodies| {
            bodies
                .iter()
                .find(|b| b.get("name").and_then(|n| n.as_str()) == Some(body_name))
        })
        .ok_or_else(|| {
            EpherError::Domain(format!("ephemeris snapshot has no entry for {body_name}"))
        })
}

/// (ra, dec) in degrees, geocentric apparent of date, from the sky
/// snapshot's explicit geocentric fields.
fn geocentric_radec(body: &BodyDef, jd: f64) -> Result<(f64, f64), EpherError> {
    let snapshot = sky_snapshot(jd, 0.0, 0.0)?;
    let entry = snapshot_body(&snapshot, body.name)?;
    Ok((
        json_f64(entry, "geocentric_apparent_ra_deg")?,
        json_f64(entry, "geocentric_apparent_dec_deg")?,
    ))
}

fn ra_fn(name: &str, args: &[Value]) -> Result<Value, EpherError> {
    let (body, jd) = body_arg(name, args)?;
    Ok(Value::Float(geocentric_radec(body, jd)?.0))
}

fn decl_fn(name: &str, args: &[Value]) -> Result<Value, EpherError> {
    let (body, jd) = body_arg(name, args)?;
    Ok(Value::Float(geocentric_radec(body, jd)?.1))
}

fn dist_fn(name: &str, args: &[Value]) -> Result<Value, EpherError> {
    let (body, jd) = body_arg(name, args)?;
    if body.name == "Pluto" {
        let (_, delta_au) = pluto_geometry(jd)?;
        return Ok(Value::Float(delta_au));
    }
    let snapshot = sky_snapshot(jd, 0.0, 0.0)?;
    let entry = snapshot_body(&snapshot, body.name)?;
    let km = json_f64(entry, "distance_km")?;
    Ok(Value::Float(km / AU_KM))
}

/// Topocentric altitude (true, unrefracted) at the observer.
fn alt_fn(name: &str, args: &[Value]) -> Result<Value, EpherError> {
    let ((lat, lon), body, jd) = observer_args(name, args)?;
    if body.name == "Pluto" {
        let (alt, _) = pluto_altaz(jd, lat, lon)?;
        return Ok(Value::Float(alt));
    }
    let snapshot = sky_snapshot(jd, lat, lon)?;
    let entry = snapshot_body(&snapshot, body.name)?;
    Ok(Value::Float(json_f64(entry, "alt_deg")?))
}

fn az_fn(name: &str, args: &[Value]) -> Result<Value, EpherError> {
    let ((lat, lon), body, jd) = observer_args(name, args)?;
    if body.name == "Pluto" {
        let (_, az) = pluto_altaz(jd, lat, lon)?;
        return Ok(Value::Float(az));
    }
    let snapshot = sky_snapshot(jd, lat, lon)?;
    let entry = snapshot_body(&snapshot, body.name)?;
    Ok(Value::Float(json_f64(entry, "az_deg")?))
}

/// `(lat, lon), body, jd` for the horizontal-accessor signatures
/// `f(body, jd, lat, lon)`.
fn observer_args(
    name: &str,
    args: &[Value],
) -> Result<((f64, f64), &'static BodyDef, f64), EpherError> {
    let (body, jd) = body_arg(name, args)?;
    match args {
        [_, _, Value::Float(lat), Value::Float(lon)] => {
            if !(-90.0..=90.0).contains(lat) {
                return Err(domain_error(format!(
                    "{name} needs a latitude in -90..90 degrees, got {lat}"
                )));
            }
            Ok(((*lat, *lon), body, jd))
        }
        _ => Err(EpherError::Type(format!(
            "{name} expects (body, jd, lat, lon), got {} argument(s)",
            args.len()
        ))),
    }
}

/// Rise / transit / set from the sky snapshot's event block (JDs within
/// the observer's local mean-solar day of `jd`; a body that never rises
/// or sets that day is a domain error, not a NaN).
fn event_fn(name: &str, args: &[Value], key: &str) -> Result<Value, EpherError> {
    let ((lat, lon), body, jd) = observer_args(name, args)?;
    let snapshot = sky_snapshot(jd, lat, lon)?;
    let entry = snapshot_body(&snapshot, body.name)?;
    let value = entry
        .get(key)
        .ok_or_else(|| EpherError::Domain(format!("ephemeris snapshot missing {key}")))?;
    match value.as_f64() {
        Some(x) if x.is_finite() => Ok(Value::Float(x)),
        _ => Err(domain_error(format!(
            "{} never {}s on that local day at that latitude",
            body.name,
            key.strip_suffix("_jd").unwrap_or(key)
        ))),
    }
}

/// Apparent magnitude. The planets come from the system snapshot (the
/// crate's Meeus ch. 41 tables, Saturn's rings included); the Moon from
/// Meeus ch. 48 through the facade; the Sun from its distance; Pluto
/// from its elements.
fn mag_fn(name: &str, args: &[Value]) -> Result<Value, EpherError> {
    let (body, jd) = body_arg(name, args)?;
    match body.name {
        "Sun" => {
            let snapshot = system_snapshot(jd)?;
            let earth = snapshot_body(&snapshot, "Earth")?;
            let r = json_f64(earth, "dist_au")?;
            Ok(Value::Float(-26.74 + 10.0 * r.log10()))
        }
        "Moon" => {
            let snapshot = system_snapshot(jd)?;
            let moon = snapshot_body(&snapshot, "Moon")?;
            let alpha = json_f64(moon, "phase_angle_deg")?;
            // Meeus ch. 48.4
            Ok(Value::Float(-12.7 + 0.026 * alpha + 4e-9 * alpha * alpha * alpha * alpha))
        }
        "Pluto" => pluto_mag(jd).map(Value::Float),
        _ => {
            let snapshot = system_snapshot(jd)?;
            let entry = snapshot_body(&snapshot, body.name)?;
            match entry.get("magnitude").and_then(|m| m.as_f64()) {
                Some(m) => Ok(Value::Float(m)),
                None => Err(domain_error(format!("no magnitude for {}", body.name))),
            }
        }
    }
}

fn phase_fn(name: &str, args: &[Value]) -> Result<Value, EpherError> {
    let (body, jd) = body_arg(name, args)?;
    if body.name == "Pluto" {
        return Ok(Value::Float(pluto_geometry(jd)?.0));
    }
    let snapshot = system_snapshot(jd)?;
    let entry = snapshot_body(&snapshot, body.name)?;
    Ok(Value::Float(json_f64(entry, "phase_angle_deg")?))
}

fn illum_fn(name: &str, args: &[Value]) -> Result<Value, EpherError> {
    let (body, jd) = body_arg(name, args)?;
    if body.name == "Pluto" {
        let (phase, _) = pluto_geometry(jd)?;
        return Ok(Value::Float(
            solar_ephemeris::physics::illuminated_fraction(phase),
        ));
    }
    let snapshot = system_snapshot(jd)?;
    let entry = snapshot_body(&snapshot, body.name)?;
    Ok(Value::Float(json_f64(entry, "illuminated_fraction")?))
}

fn diam_fn(name: &str, args: &[Value]) -> Result<Value, EpherError> {
    let (body, jd) = body_arg(name, args)?;
    if body.name == "Pluto" {
        let (_, delta_au) = pluto_geometry(jd)?;
        let semi = (body.radius_km / (delta_au * AU_KM)).asin().to_degrees();
        return Ok(Value::Float(2.0 * semi));
    }
    let snapshot = sky_snapshot(jd, 0.0, 0.0)?;
    let entry = snapshot_body(&snapshot, body.name)?;
    Ok(Value::Float(json_f64(entry, "angular_size_arcsec")? / 3600.0))
}

// --- Pluto: the facade's own approximate ephemeris ---
//
// JPL's approximate Keplerian elements with rates (valid 1800-2050).
// Arcminute-grade, far below the crate's arcsecond class, and
// documented as such wherever the guide mentions Pluto.

#[derive(Clone)]
struct OrbitElements {
    a: f64,
    e: f64,
    inc: f64,
    node: f64,
    argp: f64,
    mean_anomaly: f64,
}

fn pluto_elements(jy2k: f64) -> OrbitElements {
    let t = jy2k / 100.0; // centuries past J2000
    let a = 39.482_116_75 - 0.000_315_96 * t;
    let e = 0.248_827_30 + 0.000_051_70 * t;
    let inc = 17.140_012_06 + 0.000_048_18 * t;
    let lon_peri = 224.068_916_29 - 0.040_629_42 * t;
    let node = 110.303_936_84 - 0.011_834_82 * t;
    let mean_longitude = 238.929_038_33 + 145.207_805_15 * t;
    OrbitElements {
        a,
        e,
        inc,
        node,
        argp: lon_peri - node,
        mean_anomaly: (mean_longitude - lon_peri).rem_euclid(360.0),
    }
}

/// Heliocentric ecliptic-J2000 position (AU) at the elements' own mean
/// anomaly.
fn elements_xyz(el: &OrbitElements) -> [f64; 3] {
    elements_xyz_at(el, el.mean_anomaly)
}

/// Pluto's (phase angle deg, geocentric distance AU), from its elements
/// and the snapshot's Earth position.
fn pluto_geometry(jd: f64) -> Result<(f64, f64), EpherError> {
    let system = system_snapshot(jd)?;
    let earth = snapshot_body(&system, "Earth")?;
    let earth_xyz = [
        json_f64(earth, "x_au")?,
        json_f64(earth, "y_au")?,
        json_f64(earth, "z_au")?,
    ];
    let jy2k = (jd_tt_of(jd) - solar_ephemeris::time::J2000) / 365.25;
    let xyz = elements_xyz(&pluto_elements(jy2k));
    let r = (xyz[0] * xyz[0] + xyz[1] * xyz[1] + xyz[2] * xyz[2]).sqrt();
    let d = [
        xyz[0] - earth_xyz[0],
        xyz[1] - earth_xyz[1],
        xyz[2] - earth_xyz[2],
    ];
    let delta = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    let sun_earth = (earth_xyz[0] * earth_xyz[0]
        + earth_xyz[1] * earth_xyz[1]
        + earth_xyz[2] * earth_xyz[2])
        .sqrt();
    let phase = solar_ephemeris::physics::phase_angle_deg(r, delta, sun_earth);
    Ok((phase, delta))
}

/// TT Julian Date of a UTC Julian Date, through the crate's own
/// Delta-T policy.
fn jd_tt_of(jd_utc: f64) -> f64 {
    solar_ephemeris::timescales::AstroTime::from_jd_utc(jd_utc).jd_tt
}

fn pluto_mag(jd: f64) -> Result<f64, EpherError> {
    // H = -1.0 with a negligible phase term (documented approximation)
    let (_, delta) = pluto_geometry(jd)?;
    let jy2k = (jd_tt_of(jd) - solar_ephemeris::time::J2000) / 365.25;
    let xyz = elements_xyz(&pluto_elements(jy2k));
    let r = (xyz[0] * xyz[0] + xyz[1] * xyz[1] + xyz[2] * xyz[2]).sqrt();
    Ok(-1.0 + 5.0 * (r * delta).log10())
}

/// Pluto's apparent (ra, dec) of date, through the crate's public
/// reduction functions: elements to heliocentric J2000, minus the
/// snapshot's Earth, precessed to date with nutation in longitude.
/// Light-time and aberration are skipped (arcminute body, documented).
fn pluto_radec(jd: f64) -> Result<(f64, f64), EpherError> {
    let system = system_snapshot(jd)?;
    let earth = snapshot_body(&system, "Earth")?;
    let earth_xyz = [
        json_f64(earth, "x_au")?,
        json_f64(earth, "y_au")?,
        json_f64(earth, "z_au")?,
    ];
    let astro = solar_ephemeris::timescales::AstroTime::from_jd_utc(jd);
    let t = solar_ephemeris::time::centuries(astro.jd_tt);
    let (dpsi, deps) = solar_ephemeris::time::nutation_deg(t);
    let jy2k = (astro.jd_tt - solar_ephemeris::time::J2000) / 365.25;
    let xyz = elements_xyz(&pluto_elements(jy2k));
    let g = [xyz[0] - earth_xyz[0], xyz[1] - earth_xyz[1], xyz[2] - earth_xyz[2]];
    let lon_j2000 = g[1].atan2(g[0]).to_degrees();
    let lat_j2000 = g[2].atan2((g[0] * g[0] + g[1] * g[1]).sqrt()).to_degrees();
    let (lon_date, lat_date) = solar_ephemeris::coords::precess_ecliptic_from_j2000(
        lon_j2000,
        lat_j2000,
        t,
    );
    let eps_true = solar_ephemeris::time::mean_obliquity_deg(t) + deps;
    Ok(solar_ephemeris::coords::ecl_to_equ(
        (lon_date + dpsi).rem_euclid(360.0),
        lat_date,
        eps_true,
    ))
}

/// Pluto's topocentric (alt, az), true and unrefracted, through the
/// crate's alt_az with the observer's local apparent sidereal time.
fn pluto_altaz(jd: f64, lat: f64, lon: f64) -> Result<(f64, f64), EpherError> {
    let astro = solar_ephemeris::timescales::AstroTime::from_jd_utc(jd);
    let t = solar_ephemeris::time::centuries(astro.jd_tt);
    let (dpsi, deps) = solar_ephemeris::time::nutation_deg(t);
    let eps_true = solar_ephemeris::time::mean_obliquity_deg(t) + deps;
    let gast = solar_ephemeris::time::gast_deg(astro.jd_ut1, dpsi, eps_true);
    let lst = (gast + lon).rem_euclid(360.0);
    let (ra, dec) = pluto_radec(jd)?;
    Ok(solar_ephemeris::coords::alt_az(ra, dec, lst, lat))
}

// --- Seasons (ADR-0037) ---
//
// The apparent solar longitude (the crate's sun_apparent_ecliptic,
// aberration and nutation included) crosses 0/90/180/270 degrees. A
// bisection over a generous window finds the crossing to sub-minute
// precision.

fn apparent_solar_longitude(jd: f64) -> f64 {
    let astro = solar_ephemeris::timescales::AstroTime::from_jd_utc(jd);
    let t = solar_ephemeris::time::centuries(astro.jd_tt);
    let (dpsi, _deps) = solar_ephemeris::time::nutation_deg(t);
    solar_ephemeris::planets::sun_apparent_ecliptic(astro.jd_tt, dpsi).0
}

fn season_jd(year: i32, target_deg: f64, start: (i32, u8), end: (i32, u8)) -> Result<Value, EpherError> {
    let signed = |jd: f64| {
        let diff = (apparent_solar_longitude(jd) - target_deg).rem_euclid(360.0);
        if diff > 180.0 {
            diff - 360.0
        } else {
            diff
        }
    };
    let jd = |y: i32, m: u8| match calendar_jd("jd", &[Value::float(y as f64), Value::float(m as f64), Value::float(1.0)], None) {
        Ok(Value::Float(x)) => x,
        _ => unreachable!("season windows use valid months"),
    };
    let (mut a, mut b) = (jd(year, start.1), jd(end.0, end.1));
    let (mut fa, mut fb) = (signed(a), signed(b));
    if !(fa < 0.0 && fb > 0.0) {
        return Err(domain_error(format!(
            "the season crossing for {year} fell outside its search window"
        )));
    }
    for _ in 0..60 {
        let m = 0.5 * (a + b);
        let fm = signed(m);
        if (fm < 0.0) == (fa < 0.0) {
            a = m;
            fa = fm;
        } else {
            b = m;
            fb = fm;
        }
    }
    let _ = fb;
    Ok(Value::Float(0.5 * (a + b)))
}

fn march_equinox(name: &str, args: &[Value]) -> Result<Value, EpherError> {
    let year = year_arg(name, args)?;
    season_jd(year, 0.0, (year, 1), (year, 4))
}

fn june_solstice(name: &str, args: &[Value]) -> Result<Value, EpherError> {
    let year = year_arg(name, args)?;
    season_jd(year, 90.0, (year, 4), (year, 7))
}

fn september_equinox(name: &str, args: &[Value]) -> Result<Value, EpherError> {
    let year = year_arg(name, args)?;
    season_jd(year, 180.0, (year, 7), (year, 10))
}

fn december_solstice(name: &str, args: &[Value]) -> Result<Value, EpherError> {
    let year = year_arg(name, args)?;
    season_jd(year, 270.0, (year, 10), (year + 1, 1))
}

fn year_arg(name: &str, args: &[Value]) -> Result<i32, EpherError> {
    let y = one_float(name, args)?;
    let year = crate::float_to_int(y)
        .ok_or_else(|| EpherError::Type(format!("{name} expects a whole-number year, got {y}")))?;
    i32::try_from(year)
        .map_err(|_| domain_error(format!("{name} needs a year within i32, got {year}")))
}

// ===== The solar3d scene (ADR-0037 + the ADR-0015 amendment) =====
//
// One builder, one snapshot: the scene is the data the 3D pane renders —
// orbit curves sampled from the snapshot's osculating elements, trails
// that end where each body is now, and labelled dots. Frontends project
// through the shared View3D and draw with their existing renderers.

/// Colour per body, used by the SVG renderer and by the live legends
/// (the pane's legend checkboxes carry the same hex). Visible against
/// the dark default theme at better than 3:1, like the curve palette.
pub fn body_color(body: i64) -> &'static str {
    match body {
        1 => "#9a9ba2",   // Mercury — grey
        2 => "#ffb340",   // Venus — amber
        3 => "#4da3ff",   // Earth — blue
        4 => "#ff6b5e",   // Mars — red-orange
        5 => "#d8a25e",   // Jupiter — tan
        6 => "#e8d59b",   // Saturn — pale gold
        7 => "#7fd8d0",   // Uranus — pale cyan
        8 => "#5e7bff",   // Neptune — deep blue
        9 => "#c39dff",   // Pluto — violet
        10 => "#ffd75e",  // Sun — yellow
        _ => "#d9dade",   // Moon — silver
    }
}

/// The display name of a body number (legends, labels, alt text).
pub fn body_name(body: i64) -> &'static str {
    match body {
        1 => "Mercury",
        2 => "Venus",
        3 => "Earth",
        4 => "Mars",
        5 => "Jupiter",
        6 => "Saturn",
        7 => "Uranus",
        8 => "Neptune",
        9 => "Pluto",
        10 => "Sun",
        11 => "Moon",
        _ => "body",
    }
}

/// One body's orbit (or trail) in heliocentric ecliptic-J2000 AU.
#[derive(Debug, Clone, PartialEq)]
pub struct SolarPath {
    pub body: i64,
    pub points: Vec<[f64; 3]>,
}

/// One body's position now, in heliocentric ecliptic-J2000 AU.
#[derive(Debug, Clone, PartialEq)]
pub struct SolarDot {
    pub body: i64,
    pub xyz: [f64; 3],
}

/// The solar system at an instant: what `solar3d` renders.
#[derive(Debug, Clone, PartialEq)]
pub struct SolarScene {
    pub jd: f64,
    pub orbits: Vec<SolarPath>,
    pub trails: Vec<SolarPath>,
    pub dots: Vec<SolarDot>,
}

impl SolarScene {
    /// A camera above the ecliptic, far enough out that Neptune's orbit
    /// fits with room to spare. Orbit and zoom controls take it from
    /// here (the ADR-0015 amendment inherits the pane's controls).
    pub fn default_view(&self) -> crate::graph::View3D {
        crate::graph::View3D {
            yaw: 0.8,
            pitch: 0.9,
            camera: 120.0,
        }
    }
}

/// Build the scene at a Julian Date. One system snapshot feeds dots
/// (exact positions) and orbits (osculating elements sampled as
/// ellipses); Pluto rides the facade's own elements throughout.
pub fn solar_scene(jd: f64) -> Result<SolarScene, EpherError> {
    const ORBIT_SAMPLES: usize = 512;
    const TRAIL_POINTS: usize = 48;
    let system = system_snapshot(jd)?;
    let astro = solar_ephemeris::timescales::AstroTime::from_jd_utc(jd);
    let jy2k = (astro.jd_tt - solar_ephemeris::time::J2000) / 365.25;

    let snapshot_xyz = |name: &str| -> Result<[f64; 3], EpherError> {
        let entry = snapshot_body(&system, name)?;
        Ok([
            json_f64(entry, "x_au")?,
            json_f64(entry, "y_au")?,
            json_f64(entry, "z_au")?,
        ])
    };
    let snapshot_elements = |name: &str| -> Result<OrbitElements, EpherError> {
        let entry = snapshot_body(&system, name)?;
        Ok(OrbitElements {
            a: json_f64(entry, "a_au")?,
            e: json_f64(entry, "ecc")?,
            inc: json_f64(entry, "inc_deg")?,
            node: json_f64(entry, "node_deg")?,
            argp: json_f64(entry, "argp_deg")?,
            mean_anomaly: 0.0,
        })
    };

    // Dots first: the Sun at the origin, the snapshot's bodies at their
    // exact positions, Pluto from its elements.
    let mut dots = vec![SolarDot { body: 10, xyz: [0.0; 3] }];
    for (number, name) in [
        (1, "Mercury"),
        (2, "Venus"),
        (3, "Earth"),
        (4, "Mars"),
        (5, "Jupiter"),
        (6, "Saturn"),
        (7, "Uranus"),
        (8, "Neptune"),
    ] {
        dots.push(SolarDot { body: number, xyz: snapshot_xyz(name)? });
    }
    dots.push(SolarDot { body: 9, xyz: elements_xyz(&pluto_elements(jy2k)) });
    dots.push(SolarDot { body: 11, xyz: snapshot_xyz("Moon")? });

    // Orbits: the osculating ellipse, sampled densely over the mean
    // anomaly. The Moon's orbit is sub-pixel at this scale and is not
    // drawn.
    let mut orbits = Vec::new();
    let mut trails = Vec::new();
    for body in [1, 2, 3, 4, 5, 6, 7, 8, 9] {
        let el = if body == 9 {
            pluto_elements(jy2k)
        } else {
            snapshot_elements(body_name(body))?
        };
        let points: Vec<[f64; 3]> = (0..ORBIT_SAMPLES)
            .map(|k| {
                elements_xyz_at(
                    &el,
                    360.0 * k as f64 / ORBIT_SAMPLES as f64,
                )
            })
            .collect();
        // The trail is the arc of the same ellipse ending at the body's
        // current position: find the sample nearest the dot and walk
        // backwards. Sampling is dense enough that the seam is invisible
        // (0.7 degrees of mean anomaly).
        let dot = dots
            .iter()
            .find(|d| d.body == body)
            .map(|d| d.xyz)
            .unwrap_or([0.0; 3]);
        let nearest = points
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let da = a.iter().zip(dot.iter()).map(|(x, y)| (x - y) * (x - y)).sum::<f64>();
                let db = b.iter().zip(dot.iter()).map(|(x, y)| (x - y) * (x - y)).sum::<f64>();
                da.total_cmp(&db)
            })
            .map(|(i, _)| i)
            .unwrap_or(0);
        let n = points.len();
        // oldest first, so the trail's drawing order ends at the dot
        let trail: Vec<[f64; 3]> = (0..TRAIL_POINTS)
            .map(|k| points[(nearest + n - (TRAIL_POINTS - 1 - k)) % n].clone())
            .collect();
        orbits.push(SolarPath { body, points });
        trails.push(SolarPath { body, points: trail });
    }

    Ok(SolarScene { jd, orbits, trails, dots })
}

/// Position on an orbit at a given mean anomaly (degrees) — the same
/// elements-to-ecliptic rotation [`elements_xyz`] applies, parameterized.
fn elements_xyz_at(el: &OrbitElements, mean_anomaly_deg: f64) -> [f64; 3] {
    let (ma, e) = (mean_anomaly_deg.to_radians(), el.e);
    let mut ea = ma + e * ma.sin();
    for _ in 0..60 {
        let residual = ea - e * ea.sin() - ma;
        if residual.abs() < 1e-14 {
            break;
        }
        ea -= residual / (1.0 - e * ea.cos());
    }
    let (a, i, node, argp) = (el.a, el.inc.to_radians(), el.node.to_radians(), el.argp.to_radians());
    let xp = a * (ea.cos() - e);
    let yp = a * (1.0 - e * e).sqrt() * ea.sin();
    let (so, co) = node.sin_cos();
    let (sw, cw) = argp.sin_cos();
    let (si, ci) = i.sin_cos();
    [
        (cw * co - sw * so * ci) * xp + (-sw * co - cw * so * ci) * yp,
        (cw * so + sw * co * ci) * xp + (-sw * so + cw * co * ci) * yp,
        sw * si * xp + cw * si * yp,
    ]
}
