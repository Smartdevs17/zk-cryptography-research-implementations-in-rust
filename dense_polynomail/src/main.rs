struct DensePolynomial{
    coefficients: Vec<f64>,
}

impl DensePolynomial{
    fn new(coefficients: Vec<f64>) -> DensePolynomial{ DensePolynomial{coefficients}}

    fn degree(&self) -> usize {self.coefficients.len() - 1}

    fn evaluate(&self, x: f64) -> f64 {
        self.coefficients.iter().enumerate().map(|(i, &coef)| coef * x.powi(i as i32)).sum()
    }
}

fn main(){
    let result = DensePolynomial::new(vec![5.0, 2.0]);
    println!("The degree is: {}", result.degree());
    println!("The computed result: {}", result.evaluate(2.0));
}