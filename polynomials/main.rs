#[derive(Debug, Clone)]
struct DensePolynomial {
    coefficients: Vec<f64>,
}

impl DensePolynomial {
    fn new(coefficients: Vec<f64>) -> Self {
        let mut coeffs = coefficients;
        while coeffs.len() > 1 && coeffs.last().map_or(false, |&x| x.abs() < 1e-10) {
            coeffs.pop();
        }
        DensePolynomial { coefficients: coeffs }
    }

    fn evaluate(&self, x: f64) -> f64 {
        self.coefficients
            .iter()
            .enumerate()
            .map(|(i, &coef)| coef * x.powi(i as i32))
            .sum()
    }

    fn degree(&self) -> usize {
        if self.coefficients.is_empty() {
            0
        } else {
            self.coefficients.len() - 1
        }
    }

    fn interpolate(points: &[(f64, f64)]) -> Self {
        if points.is_empty() {
            return DensePolynomial::new(vec![0.0]);
        }

        let n = points.len();
        let mut result = vec![0.0; n];
        //L(X) = y1L(x) + y2L(x)

        // For each point, calculate its contribution
        for i in 0..n {
            let (xi, yi) = points[i];
            
            // Calculate the Lagrange basis polynomial
            let mut basis = vec![1.0];  // Start with constant term 1
            let mut factor = 1.0;

            // Build up the basis polynomial
            for j in 0..n {
                if i != j {
                    let (xj, _) = points[j];
                    factor *= xi - xj;
                    
                    // Multiply by (x - xj)
                    let mut new_basis = vec![0.0; basis.len() + 1];
                    for k in 0..basis.len() {
                        new_basis[k + 1] += basis[k];         // x term
                        new_basis[k] -= basis[k] * xj;        // constant term
                    }
                    basis = new_basis;
                }
            }

            // Scale the basis polynomial by yi / factor
            let scale = yi / factor;
            for k in 0..basis.len() {
                result[k] += basis[k] * scale;
            }
        }

        // Clean up any very small coefficients that might be numerical noise
        for coef in result.iter_mut() {
            if coef.abs() < 1e-10 {
                *coef = 0.0;
            }
        }

        DensePolynomial::new(result)
    }
}

fn main() {
    let test_cases = vec![
        // Linear (Degree 1)
        vec![(0.0, 1.0), (2.0, 5.0)],
        
        // Quadratic (Degree 2)
        vec![(0.0, 1.0), (1.0, 2.0), (2.0, 5.0)],
        
        // Cubic (Degree 3)
        vec![(0.0, 1.0), (1.0, 3.0), (2.0, 5.0), (3.0, 4.0)],
        
        // Quartic (Degree 4)
        vec![(0.0, 1.0), (1.0, 3.0), (2.0, 0.0), (3.0, 4.0), (4.0, 1.0)],
    ];

    for (i, points) in test_cases.iter().enumerate() {
        println!("\nTest Case {} (Degree {})", i + 1, points.len() - 1);
        println!("Points: {:?}", points);
        
        let poly = DensePolynomial::interpolate(points);
        println!("Coefficients: {:?}", poly.coefficients);
        
        println!("Verification:");
        for &(x, y) in points {
            let evaluated = poly.evaluate(x);
            println!("f({}) = {:.6} (expected: {})", x, evaluated, y);
        }
        
        let x = points[0].0 + 0.5;
        println!("Intermediate point f({}) = {:.6}", x, poly.evaluate(x));
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
    fn test_quadratic_interpolation() {
        let points = vec![(0.0, 1.0), (1.0, 2.0), (2.0, 5.0)];
        let poly = DensePolynomial::interpolate(&points);
        for &(x, y) in &points {
            assert!((poly.evaluate(x) - y).abs() < 1e-10);
        }
    }
}