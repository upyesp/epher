//! The satellites of Jupiter and Saturn (ADR-0037's facade): the
//! Galileans from Meeus's E5 theory ("Astronomical Algorithms" 2nd ed.,
//! ch. 44) and Saturn's eight major moons from Meeus ch. 46. The
//! `solar-ephemeris` crate has no satellite ephemerides, so - like
//! Pluto - these are the facade's own: published tables transcribed
//! expression-for-expression, fed by the crate's real Earth and planet
//! positions, and validated against JPL Horizons (see
//! `tests/satellites.rs`).
//!
//! Credit where due: the term tables follow the MIT-licensed
//! `astronomia` transcription (Sonia Keys' reading of Meeus); this is
//! an independent Rust port of the same published algorithms, not a
//! derivative of that code.
//!
//! Conventions (both planets, Meeus's own): returned coordinates are
//! in the planet's EQUATORIAL radii, x positive to the WEST, y
//! positive to the North, z positive toward the observer - so
//! sqrt(x^2+y^2) < 1 with z > 0 is a transit, with z < 0 an
//! occultation, and the shadow geometry rides on the Sun direction the
//! module returns alongside each moon. Validated against JPL Horizons
//! on 2026-09-17: all twelve tabulated moons agree with the observed
//! sky offsets to 0.2-3 arcseconds.
//!
//! Both theories want ecliptic coordinates of their reference frame;
//! everything here runs uniformly in the J2000 ecliptic frame (the
//! system snapshot's heliocentric vectors), with the one published
//! B1950 conversion applied where the tables expect it (ch. 46).

use crate::astro::{jd_tt_of, system_snapshot};
use crate::{domain_error, EpherError};
use solar_ephemeris::coords::AU_KM;

const LIGHT_TIME_DAYS_PER_AU: f64 = 0.0057755183;
const D2R: f64 = std::f64::consts::PI / 180.0;
/// Jupiter's equatorial radius (km).

/// A moon's apparent position in planet equatorial radii (x positive
/// to the WEST, y positive to the North, z toward the observer), with
/// the Sun's direction from the planet in the same sky frame (unit
/// vector; the shadow axis is -sun) and the planet's geocentric
/// distance.
#[derive(Clone, Copy, Debug)]
pub struct SatView {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub sun: [f64; 3],
    pub delta_au: f64,
}

impl SatView {
    /// Apparent separation from the planet's center, in arcseconds as
    /// seen by the observer.
    pub fn separation_arcsec(&self, planet_equatorial_radius_km: f64) -> f64 {
        let rho = (self.x * self.x + self.y * self.y).sqrt();
        let radians = rho * planet_equatorial_radius_km / (self.delta_au * AU_KM);
        radians.asin() * 206264.806247
    }

    /// The moon's state: 0 visible, 1 in transit across the disk,
    /// 2 occulted by the disk, 3 in the planet's shadow, 4 casting its
    /// own shadow onto the disk.
    pub fn phenomenon(&self) -> f64 {
        let rho = (self.x * self.x + self.y * self.y).sqrt();
        let s = self.sun;
        if rho < 1.0 {
            return if self.z > 0.0 { 1.0 } else { 2.0 };
        }
        if self.z < 0.0 {
            // behind the disk's plane: eclipsed iff the moon sits inside
            // the shadow cylinder along the anti-Sun axis
            let along = -(self.x * s[0] + self.y * s[1] + self.z * s[2]);
            let perp2 =
                (self.x * self.x + self.y * self.y + self.z * self.z) - along * along;
            if along > 0.0 && perp2 < 1.0 {
                return 3.0;
            }
        }
        // near side: does the moon's shadow fall on the disk? Project
        // the moon along the anti-Sun ray onto the disk plane (z = 0).
        if self.z > 0.0 && s[2].abs() > 1e-9 {
            let k = -self.z / -s[2];
            let sh_x = self.x + k * -s[0];
            let sh_y = self.y + k * -s[1];
            if sh_x * sh_x + sh_y * sh_y < 1.0 {
                return 4.0;
            }
        }
        0.0
    }
}

fn unit(v: [f64; 3]) -> [f64; 3] {
    let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / n, v[1] / n, v[2] / n]
}

fn body_xyz(snapshot: &serde_json::Value, name: &str) -> Result<[f64; 3], EpherError> {
    let e = crate::astro::snapshot_body(snapshot, name)?;
    Ok([
        crate::astro::json_f64(e, "x_au")?,
        crate::astro::json_f64(e, "y_au")?,
        crate::astro::json_f64(e, "z_au")?,
    ])
}

fn snap_f64(snapshot: &serde_json::Value, name: &str, key: &str) -> Result<f64, EpherError> {
    let e = crate::astro::snapshot_body(snapshot, name)?;
    crate::astro::json_f64(e, key)
}

/// E5 steps 1-5 (Meeus p. 312): rotate a planet-frame vector through
/// the equator-to-orbit inclination, the psi-node of Jupiter's equator
/// on its orbit, the orbit's inclination and node, and orient on the
/// observer's geocentric direction (lambda0, beta0). Returns (A, B, C);
/// the caller applies the final D roll.
#[allow(clippy::too_many_arguments)]
fn project_e5(
    v: [f64; 3],
    inc: f64,
    psi_node: f64,
    orbit_inc: f64,
    orbit_node: f64,
    lambda0: f64,
    beta0: f64,
) -> [f64; 3] {
    let [x, y, z] = v;
    let (si, ci) = inc.sin_cos();
    let (sph, cph) = psi_node.sin_cos();
    let (so, co) = orbit_node.sin_cos();
    let (sij, cij) = orbit_inc.sin_cos();
    let (sl0, cl0) = lambda0.sin_cos();
    let (sb0, cb0) = beta0.sin_cos();
    // step 1: the equator-to-orbit inclination I
    let mut a = x;
    let mut b = y * ci - z * si;
    let mut c = y * si + z * ci;
    // step 2: psi - Omega
    let a2 = a * cph - b * sph;
    b = a * sph + b * cph;
    a = a2;
    // step 3: the orbit inclination
    let b3 = b * cij - c * sij;
    c = b * sij + c * cij;
    b = b3;
    // step 4: the orbit's ascending node
    let a4 = a * co - b * so;
    b = a * so + b * co;
    a = a4;
    // step 5: orient on the observer
    let a5 = a * sl0 - b * cl0;
    b = a * cl0 + b * sl0;
    [a5, c * sb0 + b * cb0, c * cb0 - b * sb0]
}

/// The four Galilean moons, E5 (Meeus ch. 44): `sat` 1=Io, 2=Europa,
/// 3=Ganymede, 4=Callisto. Earth and Jupiter heliocentric
/// ecliptic-J2000 positions come from the system snapshot (the crate's
/// VSOP2013/TOP2013 stand-ins for Meeus's VSOP87 inputs), so one call
/// costs a couple of cheap snapshot builds, not the full sky.
pub fn galilean(sat: usize, jd_utc: f64) -> Result<SatView, EpherError> {
    if !(1..=4).contains(&sat) {
        return Err(domain_error(
            "Jupiter's satellites are 1 Io, 2 Europa, 3 Ganymede, 4 Callisto",
        ));
    }
    let jde = jd_tt_of(jd_utc);
    let snap = system_snapshot(jde)?;
    let earth = body_xyz(&snap, "Earth")?;
    let jup1 = body_xyz(&snap, "Jupiter")?;
    // geocentric Jupiter for the light time, then iterate once more
    let geo1 = [jup1[0] - earth[0], jup1[1] - earth[1], jup1[2] - earth[2]];
    let d1 = (geo1[0] * geo1[0] + geo1[1] * geo1[1] + geo1[2] * geo1[2]).sqrt();
    let tau = d1 * LIGHT_TIME_DAYS_PER_AU;
    let snap2 = system_snapshot(jde - tau)?;
    let jup = body_xyz(&snap2, "Jupiter")?;
    let inc_deg = snap_f64(&snap2, "Jupiter", "inc_deg")?;
    let node_deg = snap_f64(&snap2, "Jupiter", "node_deg")?;
    let geo = [jup[0] - earth[0], jup[1] - earth[1], jup[2] - earth[2]];
    let delta = (geo[0] * geo[0] + geo[1] * geo[1] + geo[2] * geo[2]).sqrt();
    let lambda0 = geo[1].atan2(geo[0]);
    let beta0 = (geo[2] / (geo[0] * geo[0] + geo[1] * geo[1]).sqrt()).atan();

    let t = jde - 2443000.5 - tau;
    let l1 = 106.07719 + 203.48895579 * t;
    let l2 = 175.73161 + 101.374724735 * t;
    let l3 = 120.55883 + 50.317609207 * t;
    let l4 = 84.44459 + 21.571071177 * t;
    let pi1 = 97.0881 + 0.16138586 * t;
    let pi2 = 154.8663 + 0.04726307 * t;
    let pi3 = 188.184 + 0.00712734 * t;
    let pi4 = 335.2868 + 0.00184 * t;
    let w1 = 312.3346 - 0.13279386 * t;
    let w2 = 100.4411 - 0.03263064 * t;
    let w3 = 119.1942 - 0.00717703 * t;
    let w4 = 322.6186 - 0.00175934 * t;
    let gamma = 0.33033 * (163.679 + 0.0010512 * t).to_radians().sin()
        + 0.03439 * (34.486 - 0.0161731 * t).to_radians().sin();
    let phi_l = 199.6766 + 0.1737919 * t;
    let psi = 316.5182 - 0.00000208 * t;
    let g = 30.23756 + 0.0830925701 * t + gamma;
    let g2 = 31.97853 + 0.0334597339 * t;
    const PI_CAP: f64 = 13.469942;

    let s = |a: f64| (a * D2R).sin();
    let c = |a: f64| (a * D2R).cos();

    let sigma1 = 0.47259 * s(2.0 * (l1 - l2))
        + -0.03478 * s(pi3 - pi4)
        + 0.01081 * s(l2 - 2.0 * l3 + pi3)
        + 0.00738 * s(phi_l)
        + 0.00713 * s(l2 - 2.0 * l3 + pi2)
        + -0.00674 * s(pi1 + pi3 - 2.0 * PI_CAP - 2.0 * g)
        + 0.00666 * s(l2 - 2.0 * l3 + pi4)
        + 0.00445 * s(l1 - pi3)
        + -0.00354 * s(l1 - l2)
        + -0.00317 * s(2.0 * psi - 2.0 * PI_CAP)
        + 0.00265 * s(l1 - pi4)
        + -0.00186 * s(g)
        + 0.00162 * s(pi2 - pi3)
        + 0.00158 * s(4.0 * (l1 - l2))
        + -0.00155 * s(l1 - l3)
        + -0.00138 * s(psi + w3 - 2.0 * PI_CAP - 2.0 * g)
        + -0.00115 * s(2.0 * (l1 - 2.0 * l2 + w2))
        + 0.00089 * s(pi2 - pi4)
        + 0.00085 * s(l1 + pi3 - 2.0 * PI_CAP - 2.0 * g)
        + 0.00083 * s(w2 - w3)
        + 0.00053 * s(psi - w2);
    let sigma2 = 1.06476 * s(2.0 * (l2 - l3))
        + 0.04256 * s(l1 - 2.0 * l2 + pi3)
        + 0.03581 * s(l2 - pi3)
        + 0.02395 * s(l1 - 2.0 * l2 + pi4)
        + 0.01984 * s(l2 - pi4)
        + -0.01778 * s(phi_l)
        + 0.01654 * s(l2 - pi2)
        + 0.01334 * s(l2 - 2.0 * l3 + pi2)
        + 0.01294 * s(pi3 - pi4)
        + -0.01142 * s(l2 - l3)
        + -0.01057 * s(g)
        + -0.00775 * s(2.0 * (psi - PI_CAP))
        + 0.00524 * s(2.0 * (l1 - l2))
        + -0.0046 * s(l1 - l3)
        + 0.00316 * s(psi - 2.0 * g + w3 - 2.0 * PI_CAP)
        + -0.00203 * s(pi1 + pi3 - 2.0 * PI_CAP - 2.0 * g)
        + 0.00146 * s(psi - w3)
        + -0.00145 * s(2.0 * g)
        + 0.00125 * s(psi - w4)
        + -0.00115 * s(l1 - 2.0 * l3 + pi3)
        + -0.00094 * s(2.0 * (l2 - w2))
        + 0.00086 * s(2.0 * (l1 - 2.0 * l2 + w2))
        + -0.00086 * s(5.0 * g2 - 2.0 * g + 52.225)
        + -0.00078 * s(l2 - l4)
        + -0.00064 * s(3.0 * l3 - 7.0 * l4 + 4.0 * pi4)
        + 0.00064 * s(pi1 - pi4)
        + -0.00063 * s(l1 - 2.0 * l3 + pi4)
        + 0.00058 * s(w3 - w4)
        + 0.00056 * s(2.0 * (psi - PI_CAP - g))
        + 0.00056 * s(2.0 * (l2 - l4))
        + 0.00055 * s(2.0 * (l1 - l3))
        + 0.00052 * s(3.0 * l3 - 7.0 * l4 + pi3 + 3.0 * pi4)
        + -0.00043 * s(l1 - pi3)
        + 0.00041 * s(5.0 * (l2 - l3))
        + 0.00041 * s(pi4 - PI_CAP)
        + 0.00032 * s(w2 - w3)
        + 0.00032 * s(2.0 * (l3 - g - PI_CAP));
    let sigma3 = 0.1649 * s(l3 - pi3)
        + 0.09081 * s(l3 - pi4)
        + -0.06907 * s(l2 - l3)
        + 0.03784 * s(pi3 - pi4)
        + 0.01846 * s(2.0 * (l3 - l4))
        + -0.0134 * s(g)
        + -0.01014 * s(2.0 * (psi - PI_CAP))
        + 0.00704 * s(l2 - 2.0 * l3 + pi3)
        + -0.0062 * s(l2 - 2.0 * l3 + pi2)
        + -0.00541 * s(l3 - l4)
        + 0.00381 * s(l2 - 2.0 * l3 + pi4)
        + 0.00235 * s(psi - w3)
        + 0.00198 * s(psi - w4)
        + 0.00176 * s(phi_l)
        + 0.0013 * s(3.0 * (l3 - l4))
        + 0.00125 * s(l1 - l3)
        + -0.00119 * s(5.0 * g2 - 2.0 * g + 52.225)
        + 0.00109 * s(l1 - l2)
        + -0.001 * s(3.0 * l3 - 7.0 * l4 + 4.0 * pi4)
        + 0.00091 * s(w3 - w4)
        + 0.0008 * s(3.0 * l3 - 7.0 * l4 + pi3 + 3.0 * pi4)
        + -0.00075 * s(2.0 * l2 - 3.0 * l3 + pi3)
        + 0.00072 * s(pi1 + pi3 - 2.0 * PI_CAP - 2.0 * g)
        + 0.00069 * s(pi4 - PI_CAP)
        + -0.00058 * s(2.0 * l3 - 3.0 * l4 + pi4)
        + -0.00057 * s(l3 - 2.0 * l4 + pi4)
        + 0.00056 * s(l3 + pi3 - 2.0 * PI_CAP - 2.0 * g)
        + -0.00052 * s(l2 - 2.0 * l3 + pi1)
        + -0.00050 * s(pi2 - pi3)
        + 0.00048 * s(l3 - 2.0 * l4 + pi3)
        + -0.00045 * s(2.0 * l2 - 3.0 * l3 + pi4)
        + -0.00041 * s(pi2 - pi4)
        + -0.00038 * s(2.0 * g)
        + -0.00037 * s(pi3 - pi4 + w3 - w4)
        + -0.00032 * s(3.0 * l3 - 7.0 * l4 + 2.0 * pi3 + 2.0 * pi4)
        + 0.0003 * s(4.0 * (l3 - l4))
        + 0.00029 * s(l3 + pi4 - 2.0 * PI_CAP - 2.0 * g)
        + -0.00028 * s(w3 + psi - 2.0 * PI_CAP - 2.0 * g)
        + 0.00026 * s(l3 - PI_CAP - g)
        + 0.00024 * s(l2 - 3.0 * l3 + 2.0 * l4)
        + 0.00021 * s(2.0 * (l3 - PI_CAP - g))
        + -0.00021 * s(l3 - pi2)
        + 0.00017 * s(2.0 * (l3 - pi3));
    let sigma4 = 0.84287 * s(l4 - pi4)
        + 0.03431 * s(pi4 - pi3)
        + -0.03305 * s(2.0 * (psi - PI_CAP))
        + -0.03211 * s(g)
        + -0.01862 * s(l4 - pi3)
        + 0.01186 * s(psi - w4)
        + 0.00623 * s(l4 + pi4 - 2.0 * g - 2.0 * PI_CAP)
        + 0.00387 * s(2.0 * (l4 - pi4))
        + -0.00284 * s(5.0 * g2 - 2.0 * g + 52.225)
        + -0.00234 * s(2.0 * (psi - pi4))
        + -0.00223 * s(l3 - l4)
        + -0.00208 * s(l4 - PI_CAP)
        + 0.00178 * s(psi + w4 - 2.0 * pi4)
        + 0.00134 * s(pi4 - PI_CAP)
        + 0.00125 * s(2.0 * (l4 - g - PI_CAP))
        + -0.00117 * s(2.0 * g)
        + -0.00112 * s(2.0 * (l3 - l4))
        + 0.00107 * s(3.0 * l3 - 7.0 * l4 + 4.0 * pi4)
        + 0.00102 * s(l4 - g - PI_CAP)
        + 0.00096 * s(2.0 * l4 - psi - w4)
        + 0.00087 * s(2.0 * (psi - w4))
        + -0.00085 * s(3.0 * l3 - 7.0 * l4 + pi3 + 3.0 * pi4)
        + 0.00085 * s(l3 - 2.0 * l4 + pi4)
        + -0.00081 * s(2.0 * (l4 - psi))
        + 0.00071 * s(l4 + pi4 - 2.0 * PI_CAP - 3.0 * g)
        + 0.00061 * s(l1 - l4)
        + -0.00056 * s(psi - w3)
        + -0.00054 * s(l3 - 2.0 * l4 + pi3)
        + 0.00051 * s(l2 - l4)
        + 0.00042 * s(2.0 * (psi - g - PI_CAP))
        + 0.00039 * s(2.0 * (pi4 - w4))
        + 0.00036 * s(psi + PI_CAP - pi4 - w4)
        + 0.00035 * s(2.0 * g2 - g + 188.37)
        + -0.00035 * s(l4 - pi4 + 2.0 * PI_CAP - 2.0 * psi)
        + -0.00032 * s(l4 + pi4 - 2.0 * PI_CAP - g)
        + 0.0003 * s(2.0 * g2 - 2.0 * g + 149.15)
        + 0.00029 * s(3.0 * l3 - 7.0 * l4 + 2.0 * pi3 + 2.0 * pi4)
        + 0.00028 * s(l4 - pi4 + 2.0 * psi - 2.0 * PI_CAP)
        + -0.00028 * s(2.0 * (l4 - w4))
        + -0.00027 * s(pi3 - pi4 + w3 - w4)
        + -0.00026 * s(5.0 * g2 - 3.0 * g + 188.37)
        + 0.00025 * s(w4 - w3)
        + -0.00025 * s(l2 - 3.0 * l3 + 2.0 * l4)
        + -0.00023 * s(3.0 * (l3 - l4))
        + 0.00021 * s(2.0 * l4 - 2.0 * PI_CAP - 3.0 * g)
        + -0.00021 * s(2.0 * l3 - 3.0 * l4 + pi4)
        + 0.00019 * s(l4 - pi4 - g)
        + -0.00019 * s(2.0 * l4 - pi3 - pi4)
        + -0.00018 * s(l4 - pi4 + g)
        + -0.00016 * s(l4 + pi3 - 2.0 * PI_CAP - 2.0 * g);

    let big_l = [l1 + sigma1, l2 + sigma2, l3 + sigma3, l4 + sigma4];
    let b_i = [
        (0.0006393 * s(big_l[0] - w1)
            + 0.0001825 * s(big_l[0] - w2)
            + 0.0000329 * s(big_l[0] - w3)
            + -0.0000311 * s(big_l[0] - psi)
            + 0.0000093 * s(big_l[0] - w4)
            + 0.0000075 * s(3.0 * big_l[0] - 4.0 * l2 - 1.9927 * sigma1 + w2)
            + 0.0000046 * s(big_l[0] + psi - 2.0 * PI_CAP - 2.0 * g))
            .atan(),
        (0.0081004 * s(big_l[1] - w2)
            + 0.0004512 * s(big_l[1] - w3)
            + -0.0003284 * s(big_l[1] - psi)
            + 0.0001160 * s(big_l[1] - w4)
            + 0.0000272 * s(l1 - 2.0 * l3 + 1.0146 * sigma2 + w2)
            + -0.0000144 * s(big_l[1] - w1)
            + 0.0000143 * s(big_l[1] + psi - 2.0 * PI_CAP - 2.0 * g)
            + 0.0000035 * s(big_l[1] - psi + g)
            + -0.0000028 * s(l1 - 2.0 * l3 + 1.0146 * sigma2 + w3))
            .atan(),
        (0.0032402 * s(big_l[2] - w3)
            + -0.0016911 * s(big_l[2] - psi)
            + 0.0006847 * s(big_l[2] - w4)
            + -0.0002797 * s(big_l[2] - w2)
            + 0.0000321 * s(big_l[2] + psi - 2.0 * PI_CAP - 2.0 * g)
            + 0.0000051 * s(big_l[2] - psi + g)
            + -0.0000045 * s(big_l[2] - psi - g)
            + -0.0000045 * s(big_l[2] + psi - 2.0 * PI_CAP)
            + 0.0000037 * s(big_l[2] + psi - 2.0 * PI_CAP - 3.0 * g)
            + 0.000003 * s(2.0 * l2 - 3.0 * big_l[2] + 4.03 * sigma3 + w2)
            + -0.0000021 * s(2.0 * l2 - 3.0 * big_l[2] + 4.03 * sigma3 + w3))
            .atan(),
        (-0.0076579 * s(big_l[3] - psi)
            + 0.0044134 * s(big_l[3] - w4)
            + -0.0005112 * s(big_l[3] - w3)
            + 0.0000773 * s(big_l[3] + psi - 2.0 * PI_CAP - 2.0 * g)
            + 0.0000104 * s(big_l[3] - psi + g)
            + -0.0000102 * s(big_l[3] - psi - g)
            + 0.0000088 * s(big_l[3] + psi - 2.0 * PI_CAP - 3.0 * g)
            + -0.0000038 * s(big_l[3] + psi - 2.0 * PI_CAP - g))
            .atan(),
    ];
    let r_i = [
        5.90569
            * (1.0
                + -0.0041339 * c(2.0 * (l1 - l2))
                + -0.0000387 * c(l1 - pi3)
                + -0.0000214 * c(l1 - pi4)
                + 0.000017 * c(l1 - l2)
                + -0.0000131 * c(4.0 * (l1 - l2))
                + 0.0000106 * c(l1 - l3)
                + -0.0000066 * c(l1 + pi3 - 2.0 * PI_CAP - 2.0 * g)),
        9.39657
            * (1.0
                + 0.0093848 * c(l1 - l2)
                + -0.0003116 * c(l2 - pi3)
                + -0.0001744 * c(l2 - pi4)
                + -0.0001442 * c(l2 - pi2)
                + 0.0000553 * c(l2 - l3)
                + 0.0000523 * c(l1 - l3)
                + -0.0000290 * c(2.0 * (l1 - l2))
                + 0.0000164 * c(2.0 * (l2 - w2))
                + 0.0000107 * c(l1 - 2.0 * l3 + pi3)
                + -0.0000102 * c(l2 - pi1)
                + -0.0000091 * c(2.0 * (l1 - l3))),
        14.98832
            * (1.0
                + -0.0014388 * c(l3 - pi3)
                + -0.0007917 * c(l3 - pi4)
                + 0.0006342 * c(l2 - l3)
                + -0.0001761 * c(2.0 * (l3 - l4))
                + 0.0000294 * c(l3 - l4)
                + -0.0000156 * c(3.0 * (l3 - l4))
                + 0.0000156 * c(l1 - l3)
                + -0.0000153 * c(l1 - l2)
                + 0.000007 * c(2.0 * l2 - 3.0 * l3 + pi3)
                + -0.0000051 * c(l3 + pi3 - 2.0 * PI_CAP - 2.0 * g)),
        26.36273
            * (1.0
                + -0.0073546 * c(l4 - pi4)
                + 0.0001621 * c(l4 - pi3)
                + 0.0000974 * c(l3 - l4)
                + -0.0000543 * c(l4 + pi4 - 2.0 * PI_CAP - 2.0 * g)
                + -0.0000271 * c(2.0 * (l4 - pi4))
                + 0.0000182 * c(l4 - PI_CAP)
                + 0.0000177 * c(2.0 * (l3 - l4))
                + -0.0000167 * c(2.0 * l4 - psi - w4)
                + 0.0000167 * c(psi - w4)
                + -0.0000155 * c(2.0 * (l4 - PI_CAP - g))
                + 0.0000142 * c(2.0 * (l4 - psi))
                + 0.0000105 * c(l1 - l4)
                + 0.0000092 * c(l2 - l4)
                + -0.0000089 * c(l4 - PI_CAP - g)
                + -0.0000062 * c(l4 + pi4 - 2.0 * PI_CAP - 3.0 * g)
                + 0.0000048 * c(2.0 * (l4 - w4))),
    ];

    // precession of the node since B1950 (Meeus p. 311)
    let t0 = (jde - 2433282.423) / 36525.0;
    let p_deg = (1.3966626 + 0.0003088 * t0) * t0;
    let big_l = [
        big_l[0] + p_deg,
        big_l[1] + p_deg,
        big_l[2] + p_deg,
        big_l[3] + p_deg,
    ];
    let psi_r = (psi + p_deg) * D2R;
    let t_c = (jde - 2415020.0) / 36525.0;
    let inc = (3.120262 + 0.0006 * t_c) * D2R;
    let orbit_inc = inc_deg.to_radians();
    let orbit_node = node_deg.to_radians();

    // satellite vectors in Jupiter's equatorial frame
    let psi_deg = psi + p_deg;
    let mut xyz = [[0.0f64; 3]; 5];
    for i in 0..4 {
        let (sl_ps, cl_ps) = (big_l[i] - psi_deg).to_radians().sin_cos();
        let (sb, cb) = b_i[i].sin_cos();
        xyz[i] = [r_i[i] * cl_ps * cb, r_i[i] * sl_ps * cb, r_i[i] * sb];
    }
    xyz[4] = [0.0, 0.0, 1.0]; // the observer

    let pre4 = project_e5(
        xyz[4], inc, psi_r - orbit_node, orbit_inc, orbit_node, lambda0, beta0,
    );
    let d_roll = pre4[0].atan2(pre4[2]);
    let (sd, cd) = d_roll.sin_cos();

    // the Sun's direction from Jupiter through the same chain
    // Jupiter -> Sun is simply -Jupiter's heliocentric vector
    let sun_pre = project_e5(
        unit([-jup[0], -jup[1], -jup[2]]),
        inc,
        psi_r - orbit_node,
        orbit_inc,
        orbit_node,
        lambda0,
        beta0,
    );
    let sun_sky = unit([
        sun_pre[0] * cd - sun_pre[2] * sd,
        sun_pre[0] * sd + sun_pre[2] * cd,
        sun_pre[1],
    ]);

    let pre = project_e5(
        xyz[sat - 1],
        inc,
        psi_r - orbit_node,
        orbit_inc,
        orbit_node,
        lambda0,
        beta0,
    );
    let k_light = [17295.0, 21819.0, 27558.0, 36548.0];
    let mut x = pre[0] * cd - pre[2] * sd;
    let y = pre[0] * sd + pre[2] * cd;
    let z = pre[1];
    // differential light time
    let dl = x / r_i[sat - 1];
    x += z.abs() / k_light[sat - 1] * (1.0 - dl * dl).sqrt();
    // perspective effect
    let w = delta / (delta + z / 2095.0);

    Ok(SatView {
        x: x * w,
        y: y * w,
        z,
        sun: sun_sky,
        delta_au: delta,
    })
}

// ===== Saturn: Meeus ch. 46 =====

/// The per-moon elements as ch. 46's R4 (radians internally).
struct R4 {
    lambda: f64,
    r: f64,
    gamma: f64,
    node: f64,
}

/// The chapter's epoch/period constants and precomputed sines (the JS
/// Qs object).
struct Q {
    t1: f64,
    t2: f64,
    t4: f64,
    t6: f64,
    t7: f64,
    t8: f64,
    t9: f64,
    t10: f64,
    t11: f64,
    w0: f64,
    w1: f64,
    w2: f64,
    w3: f64,
    w4: f64,
    w5: f64,
    w6: f64,
    w7: f64,
    w8: f64,
    s1: f64,
    c1: f64,
    s2: f64,
    c2: f64,
    e1: f64,
}

impl Q {
    fn new(jde: f64) -> Self {
        let t1 = jde - 2411093.0;
        let t2 = t1 / 365.25;
        let t3 = (jde - 2433282.423) / 365.25 + 1950.0;
        let t4 = jde - 2411368.0;
        let t5 = t4 / 365.25;
        let t6 = jde - 2415020.0;
        let _ = (t3, t5);
        let t7 = t6 / 36525.0;
        let t8 = t6 / 365.25;
        let t9 = (jde - 2442000.5) / 365.25;
        let t10 = jde - 2409786.0;
        let t11 = t10 / 36525.0;
        Q {
            t1,
            t2,
            t4,
            t6,
            t7,
            t8,
            t9,
            t10,
            t11,
            w0: 5.095 * D2R * (t3 - 1866.39),
            w1: 74.4 * D2R + 32.39 * D2R * t2,
            w2: 134.3 * D2R + 92.62 * D2R * t2,
            w3: 42.0 * D2R - 0.5118 * D2R * t5,
            w4: 276.59 * D2R + 0.5118 * D2R * t5,
            w5: 267.2635 * D2R + 1222.1136 * D2R * t7,
            w6: 175.4762 * D2R + 1221.5515 * D2R * t7,
            w7: 2.4891 * D2R + 0.002435 * D2R * t7,
            w8: 113.35 * D2R - 0.2597 * D2R * t7,
            s1: (28.0817 * D2R).sin(),
            c1: (28.0817 * D2R).cos(),
            s2: (168.8112 * D2R).sin(),
            c2: (168.8112 * D2R).cos(),
            e1: 0.05589 - 0.000346 * t7,
        }
    }

    /// The common eccentricity/pathology reduction (Meeus p. 317).
    fn subr(&self, lambda2: f64, p: f64, e: f64, a: f64, node: f64, inc: f64) -> R4 {
        let m = lambda2 - p;
        let e2 = e * e;
        let e3 = e2 * e;
        let e4 = e2 * e2;
        let e5 = e3 * e2;
        // m and cap_c are radians (the Q elements are built in radians)
        let cap_c = (2.0 * e - 0.25 * e3 + 0.0520833333 * e5) * m.sin()
            + (1.25 * e2 - 0.458333333 * e4) * (2.0 * m).sin()
            + (1.083333333 * e3 - 0.671875 * e5) * (3.0 * m).sin()
            + 1.072917 * e4 * (4.0 * m).sin()
            + 1.142708 * e5 * (5.0 * m).sin();
        let r = a * (1.0 - e2) / (1.0 + e * (m + cap_c).cos());
        let g = node - 168.8112 * D2R;
        let (si, ci) = (inc.sin(), inc.cos());
        let (sg, cg) = (g.sin(), g.cos());
        let a1 = si * sg;
        let a2 = self.c1 * si * cg - self.s1 * ci;
        let gamma = a1.hypot(a2).asin();
        let u = a1.atan2(a2);
        let psi = (self.s1 * sg).atan2(self.c1 * si - self.s1 * ci * cg);
        R4 {
            lambda: lambda2 + cap_c + u - g - psi,
            r,
            gamma,
            node: 168.8112 * D2R + u,
        }
    }

    fn mimas(&self) -> R4 {
        let l = 127.64 * D2R + 381.994497 * D2R * self.t1 - 43.57 * D2R * self.w0.sin()
            - 0.72 * D2R * (3.0 * self.w0).sin()
            - 0.02144 * D2R * (5.0 * self.w0).sin();
        let p = 106.1 * D2R + 365.549 * D2R * self.t2;
        let m = l - p;
        let cap_c = 2.18287 * D2R * m.sin() + 0.025988 * D2R * (2.0 * m).sin()
            + 0.00043 * D2R * (3.0 * m).sin();
        R4 {
            lambda: l + cap_c,
            r: 3.06879 / (1.0 + 0.01905 * (m + cap_c).cos()),
            gamma: 1.563 * D2R,
            node: 54.5 * D2R - 365.072 * D2R * self.t2,
        }
    }

    fn enceladus(&self) -> R4 {
        let l = 200.317 * D2R + 262.7319002 * D2R * self.t1
            + 0.25667 * D2R * self.w1.sin()
            + 0.20883 * D2R * self.w2.sin();
        let p = 309.107 * D2R + 123.44121 * D2R * self.t2;
        let m = l - p;
        let cap_c = 0.55577 * D2R * m.sin() + 0.00168 * D2R * (2.0 * m).sin();
        R4 {
            lambda: l + cap_c,
            r: 3.94118 / (1.0 + 0.00485 * (m + cap_c).cos()),
            gamma: 0.0262 * D2R,
            node: 348.0 * D2R - 151.95 * D2R * self.t2,
        }
    }

    fn tethys(&self) -> R4 {
        R4 {
            lambda: 285.306 * D2R + 190.69791226 * D2R * self.t1
                + 2.063 * D2R * self.w0.sin()
                + 0.03409 * D2R * (3.0 * self.w0).sin()
                + 0.001015 * D2R * (5.0 * self.w0).sin(),
            r: 4.880998,
            gamma: 1.0976 * D2R,
            node: 111.33 * D2R - 72.2441 * D2R * self.t2,
        }
    }

    fn dione(&self) -> R4 {
        let l = 254.712 * D2R + 131.53493193 * D2R * self.t1
            - 0.0215 * D2R * self.w1.sin()
            - 0.01733 * D2R * self.w2.sin();
        let p = 174.8 * D2R + 30.82 * D2R * self.t2;
        let m = l - p;
        let cap_c = 0.24717 * D2R * m.sin() + 0.00033 * D2R * (2.0 * m).sin();
        R4 {
            lambda: l + cap_c,
            r: 6.24871 / (1.0 + 0.002157 * (m + cap_c).cos()),
            gamma: 0.0139 * D2R,
            node: 232.0 * D2R - 30.27 * D2R * self.t2,
        }
    }

    fn rhea(&self) -> R4 {
        let p2 = 342.7 * D2R + 10.057 * D2R * self.t2;
        let a1 = 0.000265 * p2.sin() + 0.001 * self.w4.sin();
        let a2 = 0.000265 * p2.cos() + 0.001 * self.w4.cos();
        let e = a1.hypot(a2);
        let p = a1.atan2(a2);
        let n = 345.0 * D2R - 10.057 * D2R * self.t2;
        let lambda2 = 359.244 * D2R + 79.6900472 * D2R * self.t1 + 0.086754 * D2R * n.sin();
        let inc = 28.0362 * D2R + 0.346898 * D2R * n.cos() + 0.0193 * D2R * self.w3.cos();
        let node = 168.8034 * D2R + 0.736936 * D2R * n.sin() + 0.041 * D2R * self.w3.sin();
        self.subr(lambda2, p, e, 8.725924, node, inc)
    }

    fn titan(&self) -> R4 {
        let l = 261.1582 * D2R + 22.57697855 * D2R * self.t4 + 0.074025 * D2R * self.w3.sin();
        let i2 = 27.45141 * D2R + 0.295999 * D2R * self.w3.cos();
        let om2 = 168.66925 * D2R + 0.628808 * D2R * self.w3.sin();
        let om2_w8 = om2 - self.w8;
        let a1 = self.w7.sin() * om2_w8.sin();
        let a2 = self.w7.cos() * i2.sin() * om2_w8.cos() - self.w7.sin() * i2.cos();
        let g0 = 102.8623 * D2R;
        let psi = a1.atan2(a2);
        let s = a1.hypot(a2);
        let mut g = self.w4 - om2 - psi;
        let mut varpi = 0.0;
        let s2g0 = (2.0 * g0).sin();
        for _ in 0..3 {
            varpi = self.w4 + 0.37515 * D2R * ((2.0 * g).sin() - s2g0);
            g = varpi - om2 - psi;
        }
        let e2 = 0.029092 + 0.00019048 * ((2.0 * g).cos() - s2g0.cos());
        let qq = 2.0 * (self.w5 - varpi);
        let b1 = i2.sin() * om2_w8.sin();
        let b2 = self.w7.cos() * i2.cos() * om2_w8.cos() - self.w7.sin() * i2.cos();
        let theta = b1.atan2(b2) + self.w8;
        let e = e2 + 0.002778797 * e2 * qq.cos();
        let p = varpi + 0.159215 * D2R * qq.sin();
        let u = 2.0 * self.w5 - 2.0 * theta + psi;
        let h = 0.9375 * e2 * e2 * qq.sin() + 0.1875 * s * s * (2.0 * (self.w5 - theta)).sin();
        let lambda2 = l
            - 0.254744 * D2R
                * (self.e1 * self.w6.sin()
                    + 0.75 * self.e1 * self.e1 * (2.0 * self.w6).sin()
                    + h);
        let inc = i2 + 0.031843 * D2R * s * u.cos();
        let node = om2 + 0.031843 * D2R * s * u.sin() / i2.sin();
        self.subr(lambda2, p, e, 20.216193, node, inc)
    }

    fn hyperion(&self) -> R4 {
        let eta = 92.39 * D2R + 0.5621071 * D2R * self.t6;
        let zeta = 148.19 * D2R - 19.18 * D2R * self.t8;
        let theta = 184.8 * D2R - 35.41 * D2R * self.t9;
        let theta2 = theta - 7.5 * D2R;
        let as_ = 176.0 * D2R + 12.22 * D2R * self.t8;
        let bs = 8.0 * D2R + 24.44 * D2R * self.t8;
        let cs = bs + 5.0 * D2R;
        let varpi = 69.898 * D2R - 18.67088 * D2R * self.t8;
        let phi = 2.0 * (varpi - self.w5);
        let chi = 94.9 * D2R - 2.292 * D2R * self.t8;
        let a = 24.50601 - 0.08686 * eta.cos() - 0.00166 * (zeta + eta).cos()
            + 0.00175 * (zeta - eta).cos();
        let e = 0.103458 - 0.004099 * eta.cos() - 0.000167 * (zeta + eta).cos()
            + 0.000235 * (zeta - eta).cos() + 0.02303 * zeta.cos()
            - 0.00212 * (2.0 * zeta).cos() + 0.000151 * (3.0 * zeta).cos()
            + 0.00013 * phi.cos();
        let p = varpi + 0.15648 * D2R * chi.sin() - 0.4457 * D2R * eta.sin()
            - 0.2657 * D2R * (zeta + eta).sin() - 0.3573 * D2R * (zeta - eta).sin()
            - 12.872 * D2R * zeta.sin() + 1.668 * D2R * (2.0 * zeta).sin()
            - 0.2419 * D2R * (3.0 * zeta).sin() - 0.07 * D2R * phi.sin();
        let lambda2 = 177.047 * D2R + 16.91993829 * D2R * self.t6 + 0.15648 * D2R * chi.sin()
            + 9.142 * D2R * eta.sin() + 0.007 * D2R * (2.0 * eta).sin()
            - 0.014 * D2R * (3.0 * eta).sin() + 0.2275 * D2R * (zeta + eta).sin()
            + 0.2112 * D2R * (zeta - eta).sin() - 0.26 * D2R * zeta.sin()
            - 0.0098 * D2R * (2.0 * zeta).sin() - 0.013 * D2R * as_.sin()
            + 0.017 * D2R * bs.sin() - 0.0303 * D2R * phi.sin();
        let inc = 27.3347 * D2R + 0.6434886 * D2R * chi.cos() + 0.315 * D2R * self.w3.cos()
            + 0.018 * D2R * theta.cos() - 0.018 * D2R * cs.cos();
        let node = 168.6812 * D2R + 1.40136 * D2R * chi.cos() + 0.68599 * D2R * self.w3.sin()
            - 0.0392 * D2R * cs.sin() + 0.0366 * D2R * theta2.sin();
        self.subr(lambda2, p, e, a, node, inc)
    }

    fn iapetus(&self) -> R4 {
        let l = 261.1582 * D2R + 22.57697855 * D2R * self.t4;
        let varpi2 = 91.796 * D2R + 0.562 * D2R * self.t7;
        let psi = 4.367 * D2R - 0.195 * D2R * self.t7;
        let theta = 146.819 * D2R - 3.198 * D2R * self.t7;
        let phi = 60.47 * D2R + 1.521 * D2R * self.t7;
        let phi_big = 205.055 * D2R - 2.091 * D2R * self.t7;
        let e2 = 0.028298 + 0.001156 * self.t11;
        let varpi0 = 352.91 * D2R + 11.71 * D2R * self.t11;
        let mu = 76.3852 * D2R + 4.53795125 * D2R * self.t10;
        let i2 = (18.4602 - 0.9518 * self.t11 - 0.072 * self.t11 * self.t11
            + 0.0054 * self.t11 * self.t11 * self.t11)
            * D2R;
        let om2 = (143.198 - 3.919 * self.t11 + 0.116 * self.t11 * self.t11
            + 0.008 * self.t11 * self.t11 * self.t11)
            * D2R;
        let l_ = mu - varpi0;
        let g = varpi0 - om2 - psi;
        let g1 = varpi0 - om2 - phi;
        let ls = self.w5 - varpi2;
        let gs = varpi2 - theta;
        let lt = l - self.w4;
        let gt = self.w4 - phi_big;
        let u1 = 2.0 * (l_ + g - ls - gs);
        let u2 = l_ + g1 - lt - gt;
        let u3 = l_ + 2.0 * (g - ls - gs);
        let u4 = lt + gt - g1;
        let u5 = 2.0 * (ls + gs);
        let a = 58.935028 + 0.004638 * u1.cos() + 0.058222 * u2.cos();
        let e = e2 - 0.0014097 * (g1 - gt).cos() + 0.0003733 * (u5 - 2.0 * g).cos()
            + 0.000118 * u3.cos() + 0.0002408 * l_.cos()
            + 0.0002849 * (l_ + u2).cos() + 0.000619 * u4.cos();
        let w = 0.08077 * D2R * (g1 - gt).sin() + 0.02139 * D2R * (u5 - 2.0 * g).sin()
            - 0.00676 * D2R * u3.sin() + 0.0138 * D2R * l_.sin()
            + 0.01632 * D2R * (l_ + u2).sin() + 0.03547 * D2R * u4.sin();
        let p = varpi0 + w / e2;
        let lambda2 = mu - 0.04299 * D2R * u2.sin() - 0.00789 * D2R * u1.sin()
            - 0.06312 * D2R * ls.sin() - 0.00295 * D2R * (2.0 * ls).sin()
            - 0.02231 * D2R * u5.sin() + 0.0065 * D2R * (u5 + psi).sin();
        let inc = i2 + 0.04204 * D2R * (u5 + psi).cos()
            + 0.00235 * D2R * (l_ + g1 + lt + gt + phi).cos()
            + 0.0036 * D2R * (u2 + phi).cos();
        let w2 = 0.04204 * D2R * (u5 + psi).sin()
            + 0.00235 * D2R * (l_ + g1 + lt + gt + phi).sin()
            + 0.00358 * D2R * (u2 + phi).sin();
        let node = om2 + w2 / i2.sin();
        self.subr(lambda2, p, e, a, node, inc)
    }
}

fn elements(q: &Q, sat: usize) -> R4 {
    match sat {
        1 => q.mimas(),
        2 => q.enceladus(),
        3 => q.tethys(),
        4 => q.dione(),
        5 => q.rhea(),
        6 => q.titan(),
        7 => q.hyperion(),
        _ => q.iapetus(),
    }
}

/// ch. 46's rotation chain (no orbit-element steps - the per-moon
/// gamma/node already carry the Laplace-plane geometry).
fn project46(v: [f64; 3], q: &Q, lambda0: f64, beta0: f64) -> [f64; 3] {
    let [x, y, z] = v;
    let mut a = x;
    let mut b = q.c1 * y - q.s1 * z;
    let c = q.s1 * y + q.c1 * z;
    let a0 = q.c2 * a - q.s2 * b;
    b = q.s2 * a + q.c2 * b;
    a = a0;
    let (sl0, cl0) = lambda0.sin_cos();
    let (sb0, cb0) = beta0.sin_cos();
    let a5 = a * sl0 - b * cl0;
    let b5 = a * cl0 + b * sl0;
    [a5, b5 * cb0 + c * sb0, c * cb0 - b * sb0]
}

/// One of Saturn's eight major moons (ch. 46): `sat` 1=Mimas,
/// 2=Enceladus, 3=Tethys, 4=Dione, 5=Rhea, 6=Titan, 7=Hyperion,
/// 8=Iapetus.
pub fn saturn_moon(sat: usize, jd_utc: f64) -> Result<SatView, EpherError> {
    if !(1..=8).contains(&sat) {
        return Err(domain_error(
            "Saturn's satellites are 1 Mimas, 2 Enceladus, 3 Tethys, 4 Dione, \
             5 Rhea, 6 Titan, 7 Hyperion, 8 Iapetus",
        ));
    }
    let jde = jd_tt_of(jd_utc);
    // Saturn's geocentric vector (J2000 ecliptic), iterated for light time
    let snap0 = system_snapshot(jde)?;
    let earth = body_xyz(&snap0, "Earth")?;
    let sat1 = body_xyz(&snap0, "Saturn")?;
    let geo1 = [sat1[0] - earth[0], sat1[1] - earth[1], sat1[2] - earth[2]];
    let d1 = (geo1[0] * geo1[0] + geo1[1] * geo1[1] + geo1[2] * geo1[2]).sqrt();
    let tau = d1 * LIGHT_TIME_DAYS_PER_AU;
    let snap2 = system_snapshot(jde - tau)?;
    let sat_hel = body_xyz(&snap2, "Saturn")?;
    let geo = [sat_hel[0] - earth[0], sat_hel[1] - earth[1], sat_hel[2] - earth[2]];
    let delta = (geo[0] * geo[0] + geo[1] * geo[1] + geo[2] * geo[2]).sqrt();

    // lambda0/beta0: Saturn's geocentric direction, precessed back to
    // B1950 (the epoch of the chapter's constants); the vector is
    // already in the J2000 ecliptic frame
    let lambda0_j2000 = geo[1].atan2(geo[0]);
    let beta0_j2000 = (geo[2] / (geo[0] * geo[0] + geo[1] * geo[1]).sqrt()).atan();
    let (l_b1950, b_b1950) = solar_ephemeris::coords::precess_ecliptic_from_j2000(
        lambda0_j2000.to_degrees(),
        beta0_j2000.to_degrees(),
        (2433282.423 - 2451545.0) / 36525.0,
    );
    let lambda0 = l_b1950.to_radians();
    let beta0 = b_b1950.to_radians();

    // the element tables are evaluated at the light-time-corrected
    // epoch: we see the system as it was when the light left Saturn
    let q = Q::new(jde - tau);
    let e = elements(&q, sat);
    let r = e.r;
    // satellite vector in the Laplace-plane geometry (Meeus p. 316)
    let u = e.lambda - e.node;
    let w = e.node - 168.8112 * D2R;
    let (su, cu) = u.sin_cos();
    let (sw, cw) = w.sin_cos();
    let (sg, cg) = e.gamma.sin_cos();
    let xyz = [
        r * (cu * cw - su * cg * sw),
        r * (su * cw * cg + cu * sw),
        r * su * sg,
    ];

    let pre0 = project46([0.0, 0.0, 1.0], &q, lambda0, beta0);
    let d_roll = pre0[0].atan2(pre0[2]);
    let pre = project46(xyz, &q, lambda0, beta0);
    let (sd, cd) = d_roll.sin_cos();
    let k_light = [0.0, 20947.0, 23715.0, 26382.0, 29876.0, 35313.0, 53800.0, 59222.0, 91820.0];
    let mut x = pre[0] * cd - pre[2] * sd;
    let y = pre[0] * sd + pre[2] * cd;
    let z = pre[1];
    let dl = x / e.r;
    x += z.abs() / k_light[sat] * (1.0 - dl * dl).sqrt();
    let w_persp = delta / (delta + z / 2475.0);

    // the Sun's direction from Saturn through the same chain
    let sun_pre = project46(unit([-sat_hel[0], -sat_hel[1], -sat_hel[2]]), &q, lambda0, beta0);
    let sun_sky = unit([
        sun_pre[0] * cd - sun_pre[2] * sd,
        sun_pre[0] * sd + sun_pre[2] * cd,
        sun_pre[1],
    ]);

    Ok(SatView {
        x: x * w_persp,
        y: y * w_persp,
        z,
        sun: sun_sky,
        delta_au: delta,
    })
}

