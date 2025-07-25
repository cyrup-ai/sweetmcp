//! Complex number implementation for quantum amplitude calculations

use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Sub};

/// Complex number representation for quantum amplitudes
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Complex64 {
    pub re: f64,  // Real part 
    pub im: f64,  // Imaginary part
}

impl Complex64 {
    /// Create a new complex number
    pub fn new(real: f64, imaginary: f64) -> Self {
        Self { 
            re: real, 
            im: imaginary,
        }
    }

    /// Calculate the magnitude (absolute value) of the complex number
    pub fn magnitude(&self) -> f64 {
        (self.real * self.real + self.imaginary * self.imaginary).sqrt()
    }

    /// Alias for magnitude() for compatibility with other complex libraries
    pub fn norm(&self) -> f64 {
        self.magnitude()
    }

    /// Check if the complex number is finite (both real and imaginary parts are finite)
    pub fn is_finite(&self) -> bool {
        self.real.is_finite() && self.imaginary.is_finite()
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
            self.re = self.real;
            self.im = self.imaginary;
        }
    }

    /// Create a complex number from polar coordinates
    pub fn from_polar(magnitude: f64, phase: f64) -> Self {
        Self::new(magnitude * phase.cos(), magnitude * phase.sin())
    }

    /// Multiply two complex numbers
    pub fn multiply(&self, other: &Complex64) -> Self {
        Self::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
        )
    }

    /// Calculate the magnitude (norm) of the complex number
    pub fn norm(&self) -> f64 {
        (self.re * self.re + self.im * self.im).sqrt()
    }

    /// Calculate the complex conjugate
    pub fn conj(&self) -> Self {
        Self::new(self.re, -self.im)
    }

    /// Get the argument (phase) of the complex number
    pub fn arg(&self) -> f64 {
        self.im.atan2(self.re)
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
        Self::new(self.re + other.re, self.im + other.im)
    }
}

impl Sub for Complex64 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(self.re - other.re, self.im - other.im)
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
        Self::new(self.re * scalar, self.im * scalar)
    }
}

impl Div<f64> for Complex64 {
    type Output = Self;

    fn div(self, scalar: f64) -> Self {
        Self::new(self.re / scalar, self.im / scalar)
    }
}

impl AddAssign for Complex64 {
    fn add_assign(&mut self, other: Self) {
        self.re += other.re;
        self.im += other.im;
    }
}

impl MulAssign<f64> for Complex64 {
    fn mul_assign(&mut self, scalar: f64) {
        self.re *= scalar;
        self.im *= scalar;
    }
}

impl MulAssign for Complex64 {
    fn mul_assign(&mut self, other: Self) {
        let result = self.multiply(&other);
        self.re = result.re;
        self.im = result.im;
    }
}

impl Complex64 {
    /// Calculate e^(self) for complex exponential
    pub fn exp(&self) -> Self {
        let exp_real = self.re.exp();
        Self::new(
            exp_real * self.im.cos(),
            exp_real * self.im.sin(),
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

        assert_eq!(c1.norm(), 5.0);
        assert_eq!(c1.conj(), Complex64::new(3.0, -4.0));

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
