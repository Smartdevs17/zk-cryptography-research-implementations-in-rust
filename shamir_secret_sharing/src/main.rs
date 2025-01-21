use polynomial::{self, DensePolynomial};
use rand::Rng;
use num_bigint::BigInt;
use num_traits::{ToPrimitive, FromPrimitive};

fn create_polynomial(secret: &BigInt, degree: usize) -> DensePolynomial {
    let mut rng = rand::thread_rng();
    let mut coefficients = vec![secret.clone()];
    
    for _ in 0..degree {
        let random_value: u32 = rng.gen();
        coefficients.push(BigInt::from(random_value));
    }
    
    let max_value = BigInt::from(1u64 << 53);
    let coefficients_f64: Vec<f64> = coefficients.iter()
        .map(|c| (c % &max_value).to_f64().unwrap_or(0.0))
        .collect();
    
    DensePolynomial::new(coefficients_f64)
}

fn split_secret(secret: &BigInt, total_shares: usize, threshold: usize) -> Vec<(BigInt, BigInt)> {
    let max_value = BigInt::from(1u64 << 53);
    let secret_mod = secret % &max_value;
    let polynomial = create_polynomial(&secret_mod, threshold - 1);
    
    let mut shares = Vec::new();
    for i in 1..=total_shares {
        let x = BigInt::from(i);
        let y = polynomial.evaluate(x.to_f64().unwrap());
        shares.push((x, BigInt::from_f64(y).unwrap_or(BigInt::from(0))));
    }
    shares
}

fn recover_secret(shares: &[(BigInt, BigInt)], threshold: usize) -> BigInt {
    let points: Vec<(f64, f64)> = shares.iter()
        .map(|(x, y)| (x.to_f64().unwrap_or(0.0), y.to_f64().unwrap_or(0.0)))
        .collect();
    
    let polynomial = DensePolynomial::interpolate(&points[..threshold]);
    BigInt::from_f64(polynomial.evaluate(0.0)).unwrap_or(BigInt::from(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_sharing() {
        let secret = BigInt::from(12345);
        let shares = split_secret(&secret, 5, 3);
        let recovered = recover_secret(&shares, 3);
        assert_eq!(recovered, secret);
    }

    #[test]
    fn test_different_share_combinations() {
        let secret = BigInt::from(67890);
        let shares = split_secret(&secret, 5, 3);
        
        // Test first three shares
        let recovered1 = recover_secret(&shares[0..3], 3);
        assert_eq!(recovered1, secret);
        
        // Test last three shares
        let recovered2 = recover_secret(&shares[2..5], 3);
        assert_eq!(recovered2, secret);
    }

    #[test]
    fn test_large_number() {
        let secret = BigInt::from(1234567890);
        let shares = split_secret(&secret, 6, 4);
        let recovered = recover_secret(&shares, 4);
        assert_eq!(recovered, secret % BigInt::from(1u64 << 53));
    }
}

fn main() {
    let secret = BigInt::from(1234567890);
    let shares = split_secret(&secret, 5, 3);
    println!("Shares: {:?}", shares);
    
    let recovered = recover_secret(&shares, 3);
    println!("Original Secret: {}", secret);
    println!("Recovered Secret: {}", recovered);
}