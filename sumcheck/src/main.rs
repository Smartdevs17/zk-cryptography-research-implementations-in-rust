mod multilinear;
mod transcript;

pub use crate::multilinear::MultivariatePoly;
pub use crate::transcript::Transcript;

/// The Sum-Check protocol is a protocol for verifying that the sum of a polynomial over a
/// boolean hypercube is equal to a claimed value.
use ark_ff::{BigInteger, PrimeField};
use prime_polynomail::{self, DensePolynomial};
use std::marker::PhantomData;

#[derive(Clone, Debug)]
struct SumCheckProof<F: PrimeField> {
    claimed_sum: F,
    round_polynomials: Vec<DensePolynomial<F>>,
    challenges: Vec<F>,
    final_evaluation: F,
}

/// The Prover in the Sum-Check protocol
struct Prover<F: PrimeField> {
    polynomial: MultivariatePoly<F>,
}

impl<F: PrimeField> Prover<F> {
    /// Creates a new Prover instance
    fn new(polynomial: MultivariatePoly<F>) -> Self {
        Self { polynomial }
    }

    /// Generates the univariate polynomial for a specific round
    ///
    /// # Arguments
    /// * `round` - Current round number
    /// * `partial_evaluation` - Previous challenge values
    fn generate_round_polynomial(
        &self,
        round: usize,
        partial_evaluation: &[F],
    ) -> DensePolynomial<F> {
        // Evaluate the polynomial at x = 0 and x = 1 with all previous rounds fixed
        let eval_0 = self
            .polynomial
            .evaluate_at_round(round, partial_evaluation, F::zero());
        let eval_1 = self
            .polynomial
            .evaluate_at_round(round, partial_evaluation, F::one());

        // Create degree-1 polynomial through these points:
        // f(x) = ax + b where:
        // b = f(0) = eval_0
        // a = f(1) - f(0) = eval_1 - eval_0
        let coeffs = vec![
            eval_0,          // constant term (b)
            eval_1 - eval_0, // coefficient of x (a)
        ];
        DensePolynomial {
            coefficients: coeffs,
        }
    }

    /// Generates the complete Sum-Check proof
    fn generate_proof(&self) -> SumCheckProof<F> {
        let claimed_sum = self.polynomial.sum_over_boolean_hypercube();
        let mut round_polynomials = Vec::new();
        let mut challenges = Vec::new();
        let mut partial_evaluation = Vec::new();

        let mut transcript = Transcript::new();
        transcript.append(
            self.polynomial
                .coeffs
                .iter()
                .flat_map(|f| f.into_bigint().to_bytes_be())
                .collect::<Vec<_>>()
                .as_slice(),
        );
        transcript.append(claimed_sum.into_bigint().to_bytes_be().as_slice());

        // Generate proof for each variable
        for round in 0..self.polynomial.num_vars {
            let round_poly = self.generate_round_polynomial(round, &partial_evaluation);
            round_polynomials.push(round_poly.clone());

            transcript.append(
                round_poly
                    .coefficients
                    .iter()
                    .flat_map(|f| f.into_bigint().to_bytes_be())
                    .collect::<Vec<_>>()
                    .as_slice(),
            );

            let challenge = transcript.sample_field_element();
            challenges.push(challenge);
            partial_evaluation.push(challenge);
        }

        let final_evaluation = self.polynomial.evaluate(&partial_evaluation);

        SumCheckProof {
            claimed_sum,
            round_polynomials,
            challenges,
            final_evaluation,
        }
    }
}

/// The Verifier in the Sum-Check protocol
struct Verifier<F: PrimeField> {
    _field: PhantomData<F>,
}

impl<F: PrimeField> Verifier<F> {
    /// Creates a new Verifier instance
    fn new() -> Self {
        Self {
            _field: PhantomData,
        }
    }

    /// Verifies a Sum-Check proof
    ///
    /// # Arguments
    /// * `proof` - The proof to verify
    /// * `polynomial` - The original polynomial
    fn verify_proof(&self, proof: &SumCheckProof<F>, polynomial: &MultivariatePoly<F>) -> bool {
        if proof.round_polynomials.len() != polynomial.num_vars {
            return false;
        }

        let mut challenges = Vec::new();
        let mut transcript = Transcript::new();

        transcript.append(
            polynomial
                .coeffs
                .iter()
                .flat_map(|f| f.into_bigint().to_bytes_be())
                .collect::<Vec<_>>()
                .as_slice(),
        );
        transcript.append(proof.claimed_sum.into_bigint().to_bytes_be().as_slice());

        let mut current_sum = proof.claimed_sum;

        for round_poly in &proof.round_polynomials {
            // Check polynomial degree is at most 1
            if round_poly.degree() > 1 {
                return false;
            }

            // Verify sum at x=0 and x=1 matches the claimed sum
            let sum_0 = round_poly.evaluate(F::zero());
            let sum_1 = round_poly.evaluate(F::one());
            if sum_0 + sum_1 != current_sum {
                return false;
            }

            transcript.append(
                round_poly
                    .coefficients
                    .iter()
                    .flat_map(|f| f.into_bigint().to_bytes_be())
                    .collect::<Vec<_>>()
                    .as_slice(),
            );

            let challenge = transcript.sample_field_element();
            current_sum = round_poly.evaluate(challenge);
            challenges.push(challenge);
        }

        // Final check: verify the claimed evaluation
        proof.final_evaluation == polynomial.evaluate(&challenges)
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr;

    #[test]
    fn test_polynomial_evaluation() {
        // Test evaluation at point (1,1)
        let coefficients = vec![
            Fr::from(0u64), // constant term
            Fr::from(1u64), // x term
            Fr::from(1u64), // y term
            Fr::from(1u64), // xy term
        ];
        let polynomial = MultivariatePoly::new(coefficients, 2);

        let point = vec![Fr::from(1u64), Fr::from(1u64)];
        let evaluation = polynomial.evaluate(&point);

        // f(1,1) = 0 + 1 + 1 + 1 = 3
        assert_eq!(evaluation, Fr::from(3u64));
    }

    #[test]
    fn test_sumcheck_protocol() {
        // Test polynomial f(x,y) = x + y + xy
        let coefficients = vec![
            Fr::from(0u64), // constant term
            Fr::from(1u64), // x term
            Fr::from(1u64), // y term
            Fr::from(1u64), // xy term
        ];
        let polynomial = MultivariatePoly::new(coefficients, 2);

        let prover = Prover::new(polynomial.clone());
        let proof = prover.generate_proof();

        let verifier = Verifier::new();
        assert!(
            verifier.verify_proof(&proof, &polynomial),
            "Sum-Check proof verification failed!"
        );
    }

    #[test]
    fn test_sumcheck() {
        let poly = MultivariatePoly::new(
            vec![
                Fr::from(0u64), // constant term
                Fr::from(0u64), // x term
                Fr::from(0u64), // y term
                Fr::from(3u64), // xy term
                Fr::from(0u64), // z term
                Fr::from(0u64), // xz term
                Fr::from(2u64), // yz term
                Fr::from(5u64), // xyz term
            ],
            3,
        );
        let prover = Prover::new(poly.clone());
        let proof = prover.generate_proof();

        let verifier = Verifier::new();
        // dbg!(verifier.verify_proof(&proof, &poly));
        assert!(
            verifier.verify_proof(&proof, &poly),
            "Sum-Check proof verification failed!"
        );
    }

    #[test]
    fn test_sumcheck_failure() {
        let poly = MultivariatePoly::new(
            vec![
                Fr::from(0u64), // constant term
                Fr::from(1u64), // x term
                Fr::from(1u64), // y term
                Fr::from(1u64), // xy term
            ],
            2,
        );
        let prover = Prover::new(poly.clone());
        let proof = prover.generate_proof();

        // Modify the proof to make it invalid
        let mut invalid_proof = proof.clone();
        invalid_proof.claimed_sum += Fr::from(1u64);

        let verifier = Verifier::new();
        assert!(
            !verifier.verify_proof(&invalid_proof, &poly),
            "Sum-Check proof verification should have failed!"
        );
    }
}
