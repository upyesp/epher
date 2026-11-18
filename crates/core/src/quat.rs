//! Hamilton quaternions (ADR-0085): the algebra behind
//! `Value::Quaternion`.
//!
//! Scalar first, `w + xi + yj + zk`, the SPICE / MATLAB / Octave
//! convention the ADR fixed. Everything here is plain f64 math with
//! `Option` for the undefined cases (a zero quaternion has no inverse,
//! logarithm, or axis); the evaluator owns the wording of the errors and
//! the display spelling.

/// A Hamilton quaternion, scalar first.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternion {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Quaternion {
    pub const fn new(w: f64, x: f64, y: f64, z: f64) -> Self {
        Self { w, x, y, z }
    }

    /// A real quaternion, the embedding of a plain number.
    pub const fn real(w: f64) -> Self {
        Self::new(w, 0.0, 0.0, 0.0)
    }

    pub fn parts(self) -> [f64; 4] {
        [self.w, self.x, self.y, self.z]
    }

    pub fn vector(self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }

    pub fn is_real(self) -> bool {
        self.x == 0.0 && self.y == 0.0 && self.z == 0.0
    }

    pub fn scale(self, k: f64) -> Self {
        Self::new(self.w * k, self.x * k, self.y * k, self.z * k)
    }

    pub fn conj(self) -> Self {
        Self::new(self.w, -self.x, -self.y, -self.z)
    }

    pub fn dot(self, o: Self) -> f64 {
        self.w * o.w + self.x * o.x + self.y * o.y + self.z * o.z
    }

    pub fn norm_sqr(self) -> f64 {
        self.dot(self)
    }

    pub fn norm(self) -> f64 {
        self.norm_sqr().sqrt()
    }

    /// The multiplicative inverse, `conj(q) / |q|^2`; `None` for the
    /// zero quaternion.
    pub fn inv(self) -> Option<Self> {
        let n = self.norm_sqr();
        if n == 0.0 || !n.is_finite() {
            return None;
        }
        Some(self.conj().scale(1.0 / n))
    }

    /// The unit representative; `None` for the zero quaternion.
    pub fn unit(self) -> Option<Self> {
        let n = self.norm();
        if n == 0.0 || !n.is_finite() {
            return None;
        }
        Some(self.scale(1.0 / n))
    }

    /// `exp(w) (cos |v| + sin |v| v/|v|)`, the map from an
    /// angular-velocity quaternion to a rotation.
    pub fn exp(self) -> Self {
        let v = self.norm_of_vector();
        if v == 0.0 {
            return Self::real(self.w.exp());
        }
        let s = self.w.exp() * (v.sin() / v);
        Self::new(self.w.exp() * v.cos(), self.x * s, self.y * s, self.z * s)
    }

    /// The principal logarithm, `ln |q| + acos(w/|q|) v/|v|`; `None` for
    /// zero. A negative real has no vector part to point along, so the
    /// `i` axis carries the turn (`ln(-4)` is `ln 4 + pi i`).
    pub fn ln(self) -> Option<Self> {
        let n = self.norm();
        if n == 0.0 || !n.is_finite() {
            return None;
        }
        let v = self.norm_of_vector();
        let theta = (self.w / n).clamp(-1.0, 1.0).acos();
        let scale = if v == 0.0 {
            if theta == 0.0 {
                0.0
            } else {
                theta
            }
        } else {
            theta / v
        };
        let (x, y, z) = if v == 0.0 {
            (1.0, 0.0, 0.0)
        } else {
            (self.x, self.y, self.z)
        };
        Some(Self::new(n.ln(), x * scale, y * scale, z * scale))
    }

    /// `exp(exponent * ln q)`; `None` where `ln` is undefined.
    pub fn pow(self, exponent: Self) -> Option<Self> {
        Some((exponent * self.ln()?).exp())
    }

    /// The principal square root, `exp(ln q / 2)`.
    pub fn sqrt(self) -> Option<Self> {
        Some(self.ln()?.scale(0.5).exp())
    }

    /// The rotation angle in radians, `2 acos(w/|q|)`; `None` for zero.
    pub fn angle(self) -> Option<f64> {
        let n = self.norm();
        if n == 0.0 || !n.is_finite() {
            return None;
        }
        Some(2.0 * (self.w / n).clamp(-1.0, 1.0).acos())
    }

    /// The unit axis of the vector part; `None` when it has none (a real
    /// quaternion names no axis).
    pub fn axis(self) -> Option<[f64; 3]> {
        let v = self.norm_of_vector();
        if v == 0.0 || !v.is_finite() {
            return None;
        }
        Some([self.x / v, self.y / v, self.z / v])
    }

    /// The unit quaternion for a turn of `angle` radians about `axis`;
    /// `None` for a zero axis.
    pub fn axis_angle(axis: [f64; 3], angle: f64) -> Option<Self> {
        let n = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        if n == 0.0 || !n.is_finite() {
            return None;
        }
        let s = (angle / 2.0).sin() / n;
        Some(Self::new(
            (angle / 2.0).cos(),
            axis[0] * s,
            axis[1] * s,
            axis[2] * s,
        ))
    }

    /// `q v q^-1`, the vector `v` turned by the rotation `q` (the
    /// conjugation is scale-free, so a non-unit `q` turns the same
    /// vector as its unit representative); `None` for zero.
    pub fn rotate(self, v: [f64; 3]) -> Option<[f64; 3]> {
        let turned = self * Self::new(0.0, v[0], v[1], v[2]) * self.inv()?;
        Some(turned.vector())
    }

    /// The 3x3 rotation matrix of the unit representative; `None` for
    /// zero.
    pub fn to_matrix(self) -> Option<[[f64; 3]; 3]> {
        let q = self.unit()?;
        let (w, x, y, z) = (q.w, q.x, q.y, q.z);
        Some([
            [
                1.0 - 2.0 * (y * y + z * z),
                2.0 * (x * y - w * z),
                2.0 * (x * z + w * y),
            ],
            [
                2.0 * (x * y + w * z),
                1.0 - 2.0 * (x * x + z * z),
                2.0 * (y * z - w * x),
            ],
            [
                2.0 * (x * z - w * y),
                2.0 * (y * z + w * x),
                1.0 - 2.0 * (x * x + y * y),
            ],
        ])
    }

    /// The quaternion of a rotation matrix, the scalar part `>= 0` (the
    /// SPICE `m2q` rule): Shepperd's four-case extraction, which stays
    /// stable near a 180 degree turn where the trace form loses
    /// precision.
    pub fn from_matrix(m: [[f64; 3]; 3]) -> Self {
        let trace = m[0][0] + m[1][1] + m[2][2];
        let q = if trace > 0.0 {
            let s = 0.5 / (trace + 1.0).sqrt();
            Self::new(
                0.25 / s,
                (m[2][1] - m[1][2]) * s,
                (m[0][2] - m[2][0]) * s,
                (m[1][0] - m[0][1]) * s,
            )
        } else if m[0][0] > m[1][1] && m[0][0] > m[2][2] {
            let s = 2.0 * (1.0 + m[0][0] - m[1][1] - m[2][2]).sqrt();
            Self::new(
                (m[2][1] - m[1][2]) / s,
                0.25 * s,
                (m[0][1] + m[1][0]) / s,
                (m[0][2] + m[2][0]) / s,
            )
        } else if m[1][1] > m[2][2] {
            let s = 2.0 * (1.0 + m[1][1] - m[0][0] - m[2][2]).sqrt();
            Self::new(
                (m[0][2] - m[2][0]) / s,
                (m[0][1] + m[1][0]) / s,
                0.25 * s,
                (m[1][2] + m[2][1]) / s,
            )
        } else {
            let s = 2.0 * (1.0 + m[2][2] - m[0][0] - m[1][1]).sqrt();
            Self::new(
                (m[1][0] - m[0][1]) / s,
                (m[0][2] + m[2][0]) / s,
                (m[1][2] + m[2][1]) / s,
                0.25 * s,
            )
        };
        if q.w < 0.0 {
            -q
        } else {
            q
        }
    }

    /// Spherical interpolation on the shortest path (`other` negated
    /// when the dot product is negative); `None` for a zero operand.
    pub fn slerp(self, other: Self, t: f64) -> Option<Self> {
        let a = self.unit()?;
        let mut b = other.unit()?;
        let mut dot = a.dot(b);
        if dot < 0.0 {
            b = -b;
            dot = -dot;
        }
        let dot = dot.clamp(-1.0, 1.0);
        if dot > 1.0 - 1e-12 {
            return (a.scale(1.0 - t) + b.scale(t)).unit();
        }
        let theta = dot.acos();
        let sin_theta = theta.sin();
        let wa = ((1.0 - t) * theta).sin() / sin_theta;
        let wb = (t * theta).sin() / sin_theta;
        Some(a.scale(wa) + b.scale(wb))
    }

    fn norm_of_vector(self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
}

impl std::ops::Add for Quaternion {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(
            self.w + other.w,
            self.x + other.x,
            self.y + other.y,
            self.z + other.z,
        )
    }
}

impl std::ops::Sub for Quaternion {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(
            self.w - other.w,
            self.x - other.x,
            self.y - other.y,
            self.z - other.z,
        )
    }
}

/// Hamilton's product.
impl std::ops::Mul for Quaternion {
    type Output = Self;

    fn mul(self, o: Self) -> Self {
        Self::new(
            self.w * o.w - self.x * o.x - self.y * o.y - self.z * o.z,
            self.w * o.x + self.x * o.w + self.y * o.z - self.z * o.y,
            self.w * o.y - self.x * o.z + self.y * o.w + self.z * o.x,
            self.w * o.z + self.x * o.y - self.y * o.x + self.z * o.w,
        )
    }
}

impl std::ops::Neg for Quaternion {
    type Output = Self;

    fn neg(self) -> Self {
        self.scale(-1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Quaternion;
    use std::f64::consts::FRAC_PI_2;

    fn close(a: [f64; 4], b: [f64; 4]) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-12)
    }

    #[test]
    fn products_follow_hamilton() {
        let i = Quaternion::new(0.0, 1.0, 0.0, 0.0);
        let j = Quaternion::new(0.0, 0.0, 1.0, 0.0);
        let k = Quaternion::new(0.0, 0.0, 0.0, 1.0);
        assert_eq!(i * j, k);
        assert_eq!(j * i, -k);
        assert_eq!(k * k, Quaternion::real(-1.0));
    }

    #[test]
    fn exp_and_ln_round_trip_a_general_quaternion() {
        let q = Quaternion::new(0.3, -0.5, 0.7, 0.2);
        let round_trip = q.ln().expect("nonzero").exp();
        assert!(close(round_trip.parts(), q.parts()));
    }

    #[test]
    fn log_of_a_negative_real_turns_about_i() {
        let q = Quaternion::real(-4.0);
        let l = q.ln().expect("nonzero");
        assert!((l.w - 4f64.ln()).abs() < 1e-12);
        assert!((l.x - std::f64::consts::PI).abs() < 1e-12);
        let root = q.sqrt().expect("nonzero");
        assert!(close(root.parts(), [0.0, 2.0, 0.0, 0.0]));
    }

    #[test]
    fn a_quarter_turn_about_z_maps_x_to_y() {
        let q = Quaternion::axis_angle([0.0, 0.0, 1.0], FRAC_PI_2).expect("axis");
        let v = q.rotate([1.0, 0.0, 0.0]).expect("nonzero");
        assert!((v[0]).abs() < 1e-12);
        assert!((v[1] - 1.0).abs() < 1e-12);
        assert!((v[2]).abs() < 1e-12);
    }

    #[test]
    fn the_matrix_round_trip_keeps_the_positive_scalar() {
        for degrees in [10.0f64, 90.0, 179.0, 181.0, 270.0] {
            let q =
                Quaternion::axis_angle([0.3, -0.4, 0.866_025_403_784_438_6], degrees.to_radians())
                    .expect("axis");
            let back = Quaternion::from_matrix(q.to_matrix().expect("nonzero"));
            assert!(back.w >= 0.0, "scalar went negative at {degrees} degrees");
            let a = q.unit().expect("nonzero");
            let b = if a.dot(back) < 0.0 { -back } else { back };
            assert!(
                close(a.parts(), b.parts()),
                "{degrees} degrees: {a:?} vs {b:?}"
            );
        }
    }
}
