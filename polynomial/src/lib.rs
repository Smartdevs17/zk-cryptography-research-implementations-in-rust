// lib.rs
#[derive(Debug, Clone)]
pub struct DensePolynomial {
    coefficients: Vec<f64>,
}

impl DensePolynomial {
    pub fn new(coefficients: Vec<f64>) -> Self {
        let mut coeffs = coefficients;
        while coeffs.len() > 1 && coeffs.last().map_or(false, |&x| x == 0.0) {
            coeffs.pop();
        }
        DensePolynomial { coefficients: coeffs }
    }

    pub fn evaluate(&self, x: f64) -> f64 {
        self.coefficients
            .iter()
            .enumerate()
            .map(|(i, &coef)| coef * x.powi(i as i32))
            .sum()
    }

    pub fn degree(&self) -> usize {
        if self.coefficients.is_empty() {
            0
        } else {
            self.coefficients.len() - 1
        }
    }

    pub fn interpolate(points: &[(f64, f64)]) -> Self {
        if points.is_empty() {
            return DensePolynomial::new(vec![0.0]);
        }

        let n = points.len();
        let mut result = vec![0.0; n];

        for i in 0..n {
            let (xi, yi) = points[i];
            let mut basis = vec![1.0];
            let mut factor = 1.0;

            for j in 0..n {
                if i != j {
                    let (xj, _) = points[j];
                    factor *= xi - xj;
                    let mut new_basis = vec![0.0; basis.len() + 1];
                    for k in 0..basis.len() {
                        new_basis[k + 1] += basis[k];
                        new_basis[k] -= basis[k] * xj;
                    }
                    basis = new_basis;
                }
            }

            let scale = yi / factor;
            for k in 0..basis.len() {
                result[k] += basis[k] * scale;
            }
        }

        // for coef in result.iter_mut() {
        //     if coef.abs() < 1e-10 {
        //         *coef = 0.0;
        //     }
        // }
        while result.len() > 1 && result.last().map_or(false, |&x| x == 0.0) {
            result.pop();
        }

        DensePolynomial::new(result)
    }

    pub fn get_coefficients(&self) -> &Vec<f64> {
        &self.coefficients
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_interpolation() {
        let points = vec![(0.0, 1.0), (2.0, 5.0)];
        let poly = DensePolynomial::interpolate(&points);
        assert!((poly.evaluate(0.0) - 1.0).abs() < 1e-10);
        assert!((poly.evaluate(2.0) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_degree() {
        let poly = DensePolynomial::new(vec![1.0, 2.0, 3.0]);
        assert_eq!(poly.degree(), 2);
    }
}