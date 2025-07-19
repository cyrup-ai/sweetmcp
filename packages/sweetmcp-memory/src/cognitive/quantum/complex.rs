//! Complex number implementation for quantum amplitude calculations

use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Div, Mul, Sub};

/// Complex number representation for quantum amplitudes
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Complex64 {
    pub real: f64,
    pub imaginary: f64,
}

impl Complex64 {
    /// Create a new complex number
    pub fn new(real: f64, imaginary: f64) -> Self {
        Self { real, imaginary }
    }

    /// Calculate the magnitude (absolute value) of the complex number
    pub fn magnitude(&self) -> f64 {
        (self.real * self.real + self.imaginary * self.imaginary).sqrt()
    }

    /// Calculate the phase angle of the complex number
    pub fn phase(&self) -> f64 {
        self.imaginary.atan2(self.real)
    }

    /// Get the complex conjugate
    pub fn conjugate(&self) -> Self {
        Self::new(self.real, -self.imaginary)
    }

    /// Normalize the complex number to unit magnitude
    pub fn normalize(&mut self) {
        let magnitude = self.magnitude();
        if magnitude > 0.0 {
            self.real /= magnitude;
            self.imaginary /= magnitude;
        }
    }

    /// Create a complex number from polar coordinates
    pub fn from_polar(magnitude: f64, phase: f64) -> Self {
        Self::new(magnitude * phase.cos(), magnitude * phase.sin())
    }

    /// Multiply two complex numbers
    pub fn multiply(&self, other: &Complex64) -> Self {
        Self::new(
            self.real * other.real - self.imaginary * other.imaginary,
            self.real * other.imaginary + self.imaginary * other.real,
        )
    }
}

impl Default for Complex64 {
    fn default() -> Self {
        Self::new(0.0, 0.0)
    }
}

impl Add for Complex64 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.real + other.real, self.imaginary + other.imaginary)
    }
}

impl Sub for Complex64 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(self.real - other.real, self.imaginary - other.imaginary)
    }
}

impl Mul for Complex64 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        self.multiply(&other)
    }
}

impl Mul<f64> for Complex64 {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self {
        Self::new(self.real * scalar, self.imaginary * scalar)
    }
}

impl Div<f64> for Complex64 {
    type Output = Self;

    fn div(self, scalar: f64) -> Self {
        Self::new(self.real / scalar, self.imaginary / scalar)
    }
}

impl AddAssign for Complex64 {
    fn add_assign(&mut self, other: Self) {
        self.real += other.real;
        self.imaginary += other.imaginary;
    }
}

impl Complex64 {
    /// Calculate e^(self) for complex exponential
    pub fn exp(&self) -> Self {
        let exp_real = self.real.exp();
        Self::new(
            exp_real * self.imaginary.cos(),
            exp_real * self.imaginary.sin(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complex_operations() {
        let c1 = Complex64::new(3.0, 4.0);
        let c2 = Complex64::new(1.0, 2.0);

        assert_eq!(c1.magnitude(), 5.0);
        assert_eq!(c1.conjugate(), Complex64::new(3.0, -4.0));

        let sum = c1 + c2;
        assert_eq!(sum, Complex64::new(4.0, 6.0));

        let product = c1 * c2;
        assert_eq!(product, Complex64::new(-5.0, 10.0));
    }

    #[test]
    fn test_normalization() {
        let mut c = Complex64::new(3.0, 4.0);
        c.normalize();
        assert!((c.magnitude() - 1.0).abs() < 1e-10);
    }
}
