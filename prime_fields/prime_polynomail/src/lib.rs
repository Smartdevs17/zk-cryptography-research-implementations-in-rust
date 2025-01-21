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

    // pub fn interpolate(points: &[(F, F)]) -> Self {
    //     if points.is_empty() {
    //         return DensePolynomial::new(vec![F::zero()]);
    //     }

    //     let n = points.len();
    //     let mut result = vec![F::zero(); n];

    //     for (i, &(x_i, y_i)) in points.iter().enumerate() {
    //         let mut term = y_i;
    //         for (j, &(x_j, _)) in points.iter().enumerate() {
    //             if i != j {
    //                 term *= (x_i - x_j).inverse().unwrap();
    //             }
    //         }

    //         // Calculate the coefficients for this term
    //         let mut current = vec![F::zero(); n];
    //         current[0] = F::one();

    //         for (j, &(x_j, _)) in points.iter().enumerate() {
    //             if i != j {
    //                 // Multiply by (X - x_j)
    //                 let mut new = vec![F::zero(); n];
    //                 for k in 0..current.len() {
    //                     if k > 0 {
    //                         new[k] += current[k - 1];
    //                     }
    //                     new[k] -= current[k] * x_j;
    //                 }
    //                 current = new;
    //             }
    //         }

    //         // Add to result
    //         for k in 0..n {
    //             result[k] += current[k] * term;
    //         }
    //     }

    //     DensePolynomial::new(result)
    // }

    // Helper function to compute the Lagrange basis denominator
    fn compute_lagrange_denominator(x_i: F, points: &[(F, F)], i: usize) -> F {
        points.iter().enumerate()
            .filter(|&(j, _)| j != i)
            .map(|(_, &(x_j, _))| x_i - x_j)
            .product::<F>()
    }

    // Helper function to compute one term of basis polynomial (X - x_j)
    fn compute_linear_term(x_j: F, current: &[F]) -> Vec<F> {
        let n = current.len();
        let mut new_term = vec![F::zero(); n];
        
        // Multiply polynomial by (X - x_j)
        for k in 0..n {
            if k > 0 {
                new_term[k] += current[k - 1];  // Add X * current
            }
            new_term[k] -= current[k] * x_j;    // Subtract x_j * current
        }
        new_term
    }

    // Compute single Lagrange basis polynomial l_i(X)
    fn compute_lagrange_basis(i: usize, points: &[(F, F)]) -> Vec<F> {
        let n = points.len();
        let mut basis = vec![F::zero(); n];
        basis[0] = F::one();  // Start with 1

        // Multiply by (X - x_j) for each j != i
        for (j, &(x_j, _)) in points.iter().enumerate() {
            if i != j {
                basis = Self::compute_linear_term(x_j, &basis);
            }
        }
        basis
    }

    // Compute full Lagrange interpolation
    pub fn interpolate(points: &[(F, F)]) -> Self {
        if points.is_empty() {
            return DensePolynomial::new(vec![F::zero()]);
        }

        let n = points.len();
        let mut result = vec![F::zero(); n];

        // p(X) = Σ y_i * l_i(X)
        //         where l_i(X) = ∏ (X - x_j)/(x_i - x_j) for j ≠ i

        // For each point (x_i, y_i)
        for (i, &(x_i, y_i)) in points.iter().enumerate() {
            // Compute denominator for Lagrange basis
            let denominator = Self::compute_lagrange_denominator(x_i, points, i);
            let term = y_i * denominator.inverse().unwrap();

            // Compute Lagrange basis polynomial
            let basis = Self::compute_lagrange_basis(i, points);

            // Add contribution to result
            for k in 0..n {
                result[k] += basis[k] * term;
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