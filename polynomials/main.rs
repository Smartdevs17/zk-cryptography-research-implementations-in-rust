#[derive(Debug, Clone)]
struct DensePolynomial {
    coefficients: Vec<f64>,
}

impl DensePolynomial {
    fn new(coefficients: Vec<f64>) -> Self {
        // Remove trailing zeros
        let mut coeffs = coefficients;
        while coeffs.len() > 1 && coeffs.last().map_or(false, |&x| x == 0.0) {
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

    // Fixed Lagrange interpolation method
    fn interpolate(points: &[(f64, f64)]) -> Self {
        if points.is_empty() {
            return DensePolynomial::new(vec![0.0]);
        }

        let n = points.len();
        let mut result_coeffs = vec![0.0; n];

        for i in 0..n {
            let (xi, yi) = points[i];
            let mut numerator = 1.0;
            let mut denominator = 1.0;

            for j in 0..n {
                if i != j {
                    let (xj, _) = points[j];
                    numerator *= -xj;
                    denominator *= xi - xj;
                }
            }

            let li = yi / denominator;
            result_coeffs[0] += li * numerator;

            let mut prev = numerator;
            for k in 1..n {
                let mut term = 0.0;
                for j in 0..n {
                    if i != j {
                        let (xj, _) = points[j];
                        if k == 1 {
                            term += 1.0;
                        } else {
                            term += prev / (-xj);
                        }
                    }
                }
                prev = term;
                result_coeffs[k] += li * term;
            }
        }

        DensePolynomial::new(result_coeffs)
    }
}

fn main() {
    let points = vec![(2.0, 4.0), (4.0, 8.0)];
    let interpolated = DensePolynomial::interpolate(&points);
    
    println!("Degree of polynomial: {}", interpolated.degree());
    println!("The interpolation function coefficients: {:?}", interpolated.coefficients);
    
    // Test the interpolation
    for &(x, y) in &points {
        let evaluated = interpolated.evaluate(x);
        println!("f({}) = {} (original: {})", x, evaluated, y);
    }

    // Test an intermediate point
    let x = 3.0;
    println!("f({}) = {}", x, interpolated.evaluate(x));
}