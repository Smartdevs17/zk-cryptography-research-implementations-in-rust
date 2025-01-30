use ark_ff::PrimeField;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial, Polynomial};
use std::marker::PhantomData;

#[derive(Debug, Clone)]
struct SumCheckProof<F: PrimeField> {
    claimed_sum: F,
    polynomials: Vec<DensePolynomial<F>>,
}

struct Prover<F: PrimeField> {
    polynomial: DensePolynomial<F>,
    claimed_sum: F,
}

impl<F: PrimeField> Prover<F> {
    fn new(polynomial: DensePolynomial<F>, claimed_sum: F) -> Self {
        Prover {
            polynomial,
            claimed_sum,
        }
    }

    fn generate_proof(&self) -> SumCheckProof<F> {
        let polynomials = vec![self.polynomial.clone()];  // For this example, we just use the original polynomial
        
        SumCheckProof {
            claimed_sum: self.claimed_sum,
            polynomials,
        }
    }
}

struct Verifier<F: PrimeField> {
    _field: PhantomData<F>,
}

impl<F: PrimeField> Verifier<F> {
    fn new() -> Self {
        Verifier {
            _field: PhantomData,
        }
    }

    fn verify_proof(&self, proof: SumCheckProof<F>, polynomial: &DensePolynomial<F>) -> bool {
        // Compute the actual sum over {0,1}
        let actual_sum = self.compute_actual_sum(polynomial);
        
        // Compare with the claimed sum
        proof.claimed_sum == actual_sum
    }

    fn compute_actual_sum(&self, poly: &DensePolynomial<F>) -> F {
        // Evaluate polynomial at x=0 and x=1 and sum the results
        let eval_at_0 = poly.evaluate(&F::from(0u64));
        let eval_at_1 = poly.evaluate(&F::from(1u64));
        eval_at_0 + eval_at_1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr;

    #[test]
    fn test_sumcheck_protocol() {
        // Define polynomial f(x) = 3x^2 + 2x + 1
        let poly = DensePolynomial::from_coefficients_vec(vec![
            Fr::from(1u64),  // constant term
            Fr::from(2u64),  // coefficient of x
            Fr::from(3u64),  // coefficient of x^2
        ]);

        // Calculate the expected sum: f(0) + f(1)
        // f(0) = 1
        // f(1) = 3 + 2 + 1 = 6
        // Total sum = 7
        let claimed_sum = Fr::from(7u64);
        
        let prover = Prover::new(poly.clone(), claimed_sum);
        let proof = prover.generate_proof();
        
        let verifier = Verifier::new();
        let is_valid = verifier.verify_proof(proof, &poly);
        
        assert!(is_valid, "Sum-Check proof verification failed!");
    }
}