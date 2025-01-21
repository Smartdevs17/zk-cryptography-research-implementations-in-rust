use prime_polynomail::{self, DensePolynomial};
use ark_ff::PrimeField;
use rand::Rng;


fn create_polynomial<F: PrimeField>(secret: F, degree: usize) -> DensePolynomial<F> {
    let mut rng = rand::thread_rng();
    let mut coefficients = vec![secret.clone()];
    
    for _ in 0..degree {
        coefficients.push(F::rand(&mut rng));
    }
    
    DensePolynomial::new(coefficients)
}

fn split_secret<F: PrimeField>(secret: F, total_shares: usize, threshold: usize) -> Vec<(F, F)> {
    let polynomial = create_polynomial(secret, threshold - 1);
    dbg!(polynomial.degree());
    
    let mut shares = Vec::new();
    for i in 1..=total_shares {
        let x: F = F::from(i as u64);
        let y = polynomial.evaluate(x);
        shares.push((x, y));
    }
    shares
}

fn recover_secret<F: PrimeField>(shares: &[(F, F)], threshold: usize) -> F {
    // let points: Vec<(F, F)> = shares.iter()
    //     .cloned()
    //     .map(|(x, y)| (x, y))
    //     .collect();
    
    let polynomial = DensePolynomial::interpolate(shares);
    polynomial.evaluate(F::one())
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_basic_sharing() {
//         let secret = BigInt::from(12345);
//         let shares = split_secret(&secret, 5, 3);
//         let recovered = recover_secret(&shares, 3);
//         assert_eq!(recovered, secret);
//     }

//     #[test]
//     fn test_different_share_combinations() {
//         let secret = BigInt::from(67890);
//         let shares = split_secret(&secret, 5, 3);
        
//         // Test first three shares
//         let recovered1 = recover_secret(&shares[0..3], 3);
//         assert_eq!(recovered1, secret);
        
//         // Test last three shares
//         let recovered2 = recover_secret(&shares[2..5], 3);
//         assert_eq!(recovered2, secret);
//     }

//     #[test]
//     fn test_large_number() {
//         let secret = BigInt::from(1234567890);
//         let shares = split_secret(&secret, 6, 4);
//         let recovered = recover_secret(&shares, 4);
//         assert_eq!(recovered, secret % BigInt::from(1u64 << 53));
//     }
// }

fn main() {
    use ark_bn254::Fq;
    let secret = Fq::from(1234567890);
    let shares = split_secret(secret, 5, 3);
    println!("Shares: {:?}", shares);
    
    let recovered = recover_secret(&shares, 3);
    println!("Original Secret: {}", secret);
    println!("Recovered Secret: {}", recovered);
}