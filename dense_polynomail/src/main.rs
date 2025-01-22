struct DensePolynomial {
    coefficients: Vec<f64>,
}

impl DensePolynomial {
    fn new(coefficients: Vec<f64>) -> DensePolynomial {
        DensePolynomial { coefficients }
    }

    fn degree(&self) -> usize {
        self.coefficients.len() - 1
    }

    fn evaluate(&self, x: f64) -> f64 {
        self.coefficients
            .iter()
            .enumerate()
            .map(|(i, &coef)| coef * x.powi(i as i32))
            .sum()
    }

    fn interpolate(points: Vec<(f64, f64)>) -> Self{
        if points.is_empty(){
            return DensePolynomial::new(vec![0.0])
        }

        let n = points.len();
        let mut result = vec![0.0; n];
        for i in 0..n{
            let (xi , yi) = points[i];

            let mut basis = vec![1.0];
            let mut factor = 1.0;

            for j in 0..n {
                if i != j {
                    let (xj, _) = points[j];
                    factor *= xi - xj;

                    let mut new_basis = vec![0.0; basis.len() + 1];
                    for k in 0..basis.len() {
                        new_basis[k + 1] += basis[k]; //x-term
                        new_basis[k] -= basis[k] * xj; // constant term
                    }
                    basis = new_basis;
                }
            }
            let scale = yi / factor;
            for k in 0..basis.len() {
                result[k] += scale * basis[k]
            }

            for coef in result.iter_mut() {
                if coef.abs() < 1e-10 {
                    *coef = 0.0;
                }
            }
            while result.len() > 1 && result.last().map_or(false, |&x| x == 0.0) {
                result.pop();
            }
    
        }
        DensePolynomial::new(result)
    }

    //f(x) = f(x-1) + f(x-2)
    fn fibonacci_poly(x: f64, points: Vec<(f64, f64)>) -> f64{
        let fibonacci = Self::interpolate(points);
        // let y = fibonacci.evaluate(x);
        let y1 = fibonacci.evaluate(x-1.0);
        let y2 = fibonacci.evaluate(x-2.0);
        y1 + y2


    }


}

fn main() {
    let result = DensePolynomial::new(vec![5.0, 2.0]);
    println!("The degree is: {}", result.degree());
    println!("The computed result: {}", result.evaluate(2.0));
    let points = vec![(1.0,2.0), (2.0,4.0), (4.0,8.0)];
    let poly = DensePolynomial::interpolate(points);
    println!("Coefficients: {:?}", poly.coefficients);
}

#[cfg(test)]

mod tests{
    use crate::DensePolynomial;


    #[test]
    fn test_constraint() {
        let points = vec![(0.0,1.0), (1.0, 1.0), (2.0,2.0), (3.0,3.0), (4.0, 5.0), (5.0, 8.0),(6.0,13.0), (7.0,21.0)];
        let points1 = vec![(0.0,1.0), (1.0, 1.0), (2.0,2.0), (3.0,3.0), (4.0, 5.0), (5.0, 8.0),(6.0,13.0), (7.0,21.0)];
        let poly = DensePolynomial::fibonacci_poly(7.0, points);
        let poly1 = DensePolynomial::interpolate(points1);
        let y = poly1.evaluate(7.0);
        assert_eq!(y.round(), poly.round());

    }

}
