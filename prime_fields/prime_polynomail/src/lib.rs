use ark_ff::PrimeField;

#[derive(Debug, Clone)]
pub struct DensePolynomial<F: PrimeField> {
   pub coefficients: Vec<F>,
}

impl<F: PrimeField> DensePolynomial<F> {
    pub fn new(coefficients: Vec<F>) -> Self {
        let mut coeffs = coefficients;
        while coeffs.len() > 1 && coeffs.last().map_or(false, |&x| x.is_zero()) {
            coeffs.pop();
        }
        DensePolynomial { coefficients: coeffs }
    }

    pub fn evaluate(&self, x: F) -> F {
        self.coefficients
            .iter()
            .enumerate()
            .map(|(i, &coef)| coef * x.pow([i as u64]))
            .sum()
    }

    pub fn degree(&self) -> usize {
        if self.coefficients.is_empty() {
            0
        } else {
            self.coefficients.len() - 1
        }
    }

    pub fn interpolate(points: &[(F, F)]) -> Self {
        if points.is_empty() {
            return DensePolynomial::new(vec![F::zero()]);
        }

        let n = points.len();
        let mut result = vec![F::zero(); n];

        for (i, &(x_i, y_i)) in points.iter().enumerate() {
            let mut term = y_i;
            for (j, &(x_j, _)) in points.iter().enumerate() {
                if i != j {
                    term *= (x_i - x_j).inverse().unwrap();
                }
            }

            // Calculate the coefficients for this term
            let mut current = vec![F::zero(); n];
            current[0] = F::one();

            for (j, &(x_j, _)) in points.iter().enumerate() {
                if i != j {
                    // Multiply by (X - x_j)
                    let mut new = vec![F::zero(); n];
                    for k in 0..current.len() {
                        if k > 0 {
                            new[k] += current[k - 1];
                        }
                        new[k] -= current[k] * x_j;
                    }
                    current = new;
                }
            }

            // Add to result
            for k in 0..n {
                result[k] += current[k] * term;
            }
        }

        DensePolynomial::new(result)
    }
}

#[cfg(test)]
/// This module contains tests for polynomial interpolation and evaluation
/// using the `DensePolynomial` struct from the `ark_poly` crate.
///
/// # Tests
///
/// - `test_linear_interpolation`: Tests linear interpolation with points (1, 2) and (2, 4).
///   The resulting polynomial should be `2x`.
///
/// - `test_quadratic_interpolation`: Tests quadratic interpolation with points (0, 0), (1, 1), and (2, 4).
///   The resulting polynomial should be `x^2`.
///
/// - `test_cubic_interpolation`: Tests cubic interpolation with points (0, 0), (1, 1), (2, 8), and (3, 27).
///   The resulting polynomial should be `x^3`.
///
/// - `test_degree`: Tests the degree of a polynomial created with coefficients [1, 2, 3].
///   The resulting polynomial should be `1 + 2x + 3x^2` and its degree should be 2.
///
/// - `test_constant_polynomial`: Tests constant polynomial interpolation with points (1, 5) and (2, 5).
///   The resulting polynomial should be `5`.
mod tests {
    use super::*;
    use ark_bn254::Fr;

    #[test]
    fn test_linear_interpolation() {
        let points = vec![(Fr::from(1u64), Fr::from(2u64)), (Fr::from(2u64), Fr::from(4u64))];
        let poly = DensePolynomial::interpolate(&points);
        
        assert_eq!(poly.evaluate(Fr::from(0u64)), Fr::from(0u64));
        assert_eq!(poly.evaluate(Fr::from(1u64)), Fr::from(2u64));
        assert_eq!(poly.evaluate(Fr::from(2u64)), Fr::from(4u64));
    }

    #[test]
    fn test_quadratic_interpolation() {
        let points = vec![
            (Fr::from(0u64), Fr::from(0u64)),
            (Fr::from(1u64), Fr::from(1u64)),
            (Fr::from(2u64), Fr::from(4u64)),
        ];
        let poly = DensePolynomial::interpolate(&points);
        
        assert_eq!(poly.evaluate(Fr::from(0u64)), Fr::from(0u64));
        assert_eq!(poly.evaluate(Fr::from(1u64)), Fr::from(1u64));
        assert_eq!(poly.evaluate(Fr::from(2u64)), Fr::from(4u64));
    }

    #[test]
    fn test_cubic_interpolation() {
        let points = vec![
            (Fr::from(0u64), Fr::from(0u64)),
            (Fr::from(1u64), Fr::from(1u64)),
            (Fr::from(2u64), Fr::from(8u64)),
            (Fr::from(3u64), Fr::from(27u64)),
        ];
        let poly = DensePolynomial::interpolate(&points);
        
        assert_eq!(poly.evaluate(Fr::from(0u64)), Fr::from(0u64));
        assert_eq!(poly.evaluate(Fr::from(1u64)), Fr::from(1u64));
        assert_eq!(poly.evaluate(Fr::from(2u64)), Fr::from(8u64));
        assert_eq!(poly.evaluate(Fr::from(3u64)), Fr::from(27u64));
    }

    #[test]
    fn test_degree() {
        let poly = DensePolynomial::new(vec![Fr::from(1u64), Fr::from(2u64), Fr::from(3u64)]);
        assert_eq!(poly.degree(), 2);
    }

    #[test]
    fn test_constant_polynomial() {
        let points = vec![(Fr::from(1u64), Fr::from(5u64)), (Fr::from(2u64), Fr::from(5u64))];
        let poly = DensePolynomial::interpolate(&points);
        
        assert_eq!(poly.evaluate(Fr::from(0u64)), Fr::from(5u64));
        assert_eq!(poly.evaluate(Fr::from(1u64)), Fr::from(5u64));
        assert_eq!(poly.evaluate(Fr::from(2u64)), Fr::from(5u64));
    }
}