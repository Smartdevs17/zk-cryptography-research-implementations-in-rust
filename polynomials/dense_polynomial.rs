struct Polynomail {
    coefficients: Vec<f64>,
}

impl Polynomail {
    fn new(coefficients: Vec<f64>) -> Polynomail {Polynomail{coefficients}}

    fn degree(&self) -> usize {self.coefficients.len() -1}
}

fn main() {
    let coefficients = Polynomail::new(vec![5.0,2.0]);
    println!("this is the degree: {}", coefficients.degree());
}