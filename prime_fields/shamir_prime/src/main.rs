use ark_ff::PrimeField;
use prime_polynomail::{self, DensePolynomial};
use rand;

fn create_polynomial<F: PrimeField>(secret: F, degree: usize) -> DensePolynomial<F> {
    let mut random_value = rand::thread_rng();
    let mut coeffs = vec![secret];

    for _ in 0..degree {
        coeffs.push(F::rand(&mut random_value));
    }
    DensePolynomial {
        coefficients: coeffs,
    }
}

fn split_secret<F: PrimeField>(secret: F, total_shares: usize, threshold: usize) -> Vec<(F, F)> {
    let poly = create_polynomial(secret, threshold - 1);
    let mut all_shares = Vec::new();
    for i in 1..=total_shares {
        let x = F::from(i as u64);
        let y = poly.evaluate(x);
        all_shares.push((x, y))
    }
    all_shares
}

fn recover_secret<F: PrimeField>(shares: &[(F, F)], threshold: usize) -> F {
    let points = &shares[..threshold];
    let poly = DensePolynomial::interpolate(points);
    poly.evaluate(F::zero())
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr;

    #[test]
    fn test_create_poly() {
        let secret = Fr::from(12345u64);
        let poly = create_polynomial(secret, 2);
        assert_eq!(poly.coefficients[0], secret);
    }

    #[test]
    fn test_split_secret() {
        let secret = Fr::from(12345u64);
        let shares = split_secret(secret, 5, 3);
        let recover_secret = recover_secret(&shares, 3);
        assert_eq!(recover_secret, secret);
    }
}
