use prime_polynomail::{self, DensePolynomial};
use ark_ff::PrimeField;
use rand;

fn create_polynomial<F: PrimeField>(secret: F, degree: usize) -> DensePolynomial<F> {
    let mut rng = rand::thread_rng();
    let mut coefficients = vec![secret];
    
    for _ in 0..degree {
        coefficients.push(F::rand(&mut rng));
    }
    
    DensePolynomial::new(coefficients)
}

fn split_secret<F: PrimeField>(secret: F, total_shares: usize, threshold: usize) -> Vec<(F, F)> {
    let polynomial = create_polynomial(secret, threshold - 1);
    
    let mut shares = Vec::new();
    for i in 1..=total_shares {
        let x = F::from(i as u64);
        let y = polynomial.evaluate(x);
        shares.push((x, y));
    }
    shares
}

fn recover_secret<F: PrimeField>(shares: &[(F, F)], threshold: usize) -> F {
    let shares = &shares[..threshold]; // Only use the first 'threshold' shares
    let polynomial = DensePolynomial::interpolate(shares);
    polynomial.evaluate(F::zero()) // Evaluate at 0 to get the secret
}

fn main() {
    use ark_bn254::Fr; // Using Fr instead of Fq for better numerical properties
    let secret = Fr::from(1234567890u64);
    let shares = split_secret(secret, 5, 3);
    println!("Shares: {:?}", shares);
    
    let recovered = recover_secret(&shares, 3);
    println!("Original Secret: {}", secret);
    println!("Recovered Secret: {}", recovered);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fq;

    #[test]
    fn test_create_polynomial() {
        let secret = Fq::from(1234567890u64);
        let polynomial = create_polynomial(secret, 2);
        assert_eq!(polynomial.degree(), 2);
        assert_eq!(polynomial.coefficients[0], secret);
    }

    #[test]
    fn test_split_secrest() {
        let secret = Fq::from(1234567890u64);
        let shares = split_secret(secret, 5, 3);
        assert_eq!(shares.len(), 5);
        // Check that the first share is not zero
        assert_ne!(shares[0].1, Fq::from(0));
    }

    #[test]
    fn test_recover_secret() {
        let secret = Fq::from(1234567890u64);
        let shares = split_secret(secret, 5, 3);
        let recovered = recover_secret(&shares, 3);
        assert_eq!(recovered, secret);
    }

    #[test]
    fn test_recover_secret_with_extra_shares() {
        // Test recovering the secret with more shares than the threshold
        let secret = Fq::from(1234567890u64);
        let shares = split_secret(secret, 5, 3);
        let recovered = recover_secret(&shares, 4);
        assert_eq!(recovered, secret);
    }

    #[test]
    fn test_recover_secret_with_minimum_shares() {
        // Test recovering the secret with exactly the threshold number of shares
        let secret = Fq::from(1234567890u64);
        let shares = split_secret(secret, 5, 3);
        let recovered = recover_secret(&shares[..3], 3);
        assert_eq!(recovered, secret);
    }

    #[test]
    fn test_recover_secret_with_lower_threshold(){
        let secret = Fq::from(1234567890u64);
        let shares = split_secret(secret,5,3);
        let recovered = recover_secret(&shares, 2);
        assert_ne!(recovered, secret);
    }
}