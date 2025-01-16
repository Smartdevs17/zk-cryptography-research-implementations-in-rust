struct SparsePolynomial{
    coefficients: Vec<(u32,u32)>,
    degree: u32,
}

impl SparsePolynomial{
    fn new(coefficients: Vec<(u32,u32)>) -> SparsePolynomial{
        let degree = *coefficients.iter().map(|(_, d)| d).max().unwrap();
        SparsePolynomial{coefficients: coefficients, degree: degree}
    }

    fn degree(&self) -> u32{
        self.degree
    }

    fn evaluate(&self, x: u32) -> u32 {
        let result = self.coefficients.iter().map(|(c,d)| c * x.pow(*d)).sum();
        return result;
    }

}




fn main() {
    println!("Hello, world!");
    let result = SparsePolynomial::new(vec![(2,1),(5,0)]);
    println!("The degree is: {:?}", result.degree());
}
