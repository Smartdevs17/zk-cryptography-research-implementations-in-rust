use ark_ff::PrimeField;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial, Polynomial};
use std::marker::PhantomData;

/// Represents a multivariate polynomial over a prime field.
/// The polynomial is stored as a vector of coefficients where each bit pattern in the index
/// represents the presence/absence of variables in that term.
#[derive(Debug, Clone)]
struct MultivariatePoly<F: PrimeField> {
    coefficients: Vec<F>,     // Stores coefficients of the polynomial
    num_variables: usize,     // Number of variables in the polynomial
}

/// Represents a complete Sum-Check protocol proof.
#[derive(Debug, Clone)]
struct SumCheckProof<F: PrimeField> {
    claimed_sum: F,                           // The sum that the prover claims is correct
    round_polynomials: Vec<DensePolynomial<F>>, // Univariate polynomials for each round
    challenges: Vec<F>,                       // Random challenges from the verifier
    final_evaluation: F,                      // Final evaluation of the polynomial
}

impl<F: PrimeField> MultivariatePoly<F> {
    /// Creates a new multivariate polynomial.
    /// 
    /// # Arguments
    /// * `coefficients` - Vector of coefficients for each term
    /// * `num_variables` - Number of variables in the polynomial
    fn new(coefficients: Vec<F>, num_variables: usize) -> Self {
        Self {
            coefficients,
            num_variables,
        }
    }

    /// Evaluates the polynomial at a given point.
    /// 
    /// # Arguments
    /// * `point` - Vector of values for each variable
    /// 
    /// # Example
    /// For f(x,y) = x + y + xy and point [1,1]:
    /// - Evaluates to 1 + 1 + 1*1 = 3
    fn evaluate(&self, point: &[F]) -> F {
        assert_eq!(point.len(), self.num_variables);
        
        let mut result = F::zero();
        // Iterate through each term in the polynomial
        for (i, coeff) in self.coefficients.iter().enumerate() {
            let mut term = *coeff;
            let mut i_temp = i;
            
            // Check which variables appear in this term using bit patterns
            for j in 0..self.num_variables {
                if i_temp & 1 == 1 {
                    term *= point[j];
                }
                i_temp >>= 1;
            }
            result += term;
        }
        result
    }

    /// Computes the sum of the polynomial over the boolean hypercube {0,1}^n.
    /// 
    /// # Example
    /// For f(x,y) = x + y + xy:
    /// - f(0,0) = 0
    /// - f(0,1) = 1
    /// - f(1,0) = 1
    /// - f(1,1) = 3
    /// Total sum = 5
    fn sum_over_boolean_hypercube(&self) -> F {
        let mut sum = F::zero();
        let num_points = 1 << self.num_variables;  // 2^n points in the hypercube
        
        // Iterate over all possible boolean combinations
        for i in 0..num_points {
            let mut point = vec![F::zero(); self.num_variables];
            for j in 0..self.num_variables {
                if (i >> j) & 1 == 1 {
                    point[j] = F::one();
                }
            }
            sum += self.evaluate(&point);
        }
        sum
    }

    /// Evaluates the polynomial during a specific round of the protocol.
    /// This fixes some variables to challenges and one to x, leaving the rest free.
    /// 
    /// # Arguments
    /// * `round` - Current round number
    /// * `partial_evaluation` - Previous challenge values
    /// * `x` - Value for the current variable
    fn evaluate_at_round(&self, round: usize, partial_evaluation: &[F], x: F) -> F {
        // Construct point with:
        // - Previous rounds' challenges
        // - Current round's variable
        // - Remaining variables set to 0
        let mut point = partial_evaluation[0..round].to_vec();
        point.push(x);
        point.extend(vec![F::zero(); self.num_variables - round - 1]);
        
        let mut sum = F::zero();
        let remaining_vars = self.num_variables - round - 1;
        let num_remaining_points = 1 << remaining_vars;
        
        // Sum over all possible values of the remaining variables
        for i in 0..num_remaining_points {
            let mut full_point = point.clone();
            for j in 0..remaining_vars {
                if (i >> j) & 1 == 1 {
                    full_point[round + 1 + j] = F::one();
                }
            }
            sum += self.evaluate(&full_point);
        }
        sum
    }
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

    /// Generates the complete Sum-Check proof
    fn generate_proof(&self) -> SumCheckProof<F> {
        let claimed_sum = self.polynomial.sum_over_boolean_hypercube();
        let mut round_polynomials = Vec::new();
        let mut challenges = Vec::new();
        let mut partial_evaluation = Vec::new();

        // Generate proof for each variable
        for round in 0..self.polynomial.num_variables {
            let round_poly = self.generate_round_polynomial(round, &partial_evaluation);
            round_polynomials.push(round_poly);

            // In practice, these challenges come from the verifier
            let challenge = F::from(2u64);
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

    /// Generates the univariate polynomial for a specific round
    /// 
    /// # Arguments
    /// * `round` - Current round number
    /// * `partial_evaluation` - Previous challenge values
    fn generate_round_polynomial(&self, round: usize, partial_evaluation: &[F]) -> DensePolynomial<F> {
        // Evaluate the polynomial at x = 0 and x = 1 with all previous rounds fixed
        let eval_0 = self.polynomial.evaluate_at_round(round, partial_evaluation, F::zero());
        let eval_1 = self.polynomial.evaluate_at_round(round, partial_evaluation, F::one());
        
        // Create degree-1 polynomial through these points:
        // f(x) = ax + b where:
        // b = f(0) = eval_0
        // a = f(1) - f(0) = eval_1 - eval_0
        let coeffs = vec![
            eval_0,            // constant term (b)
            eval_1 - eval_0,   // coefficient of x (a)
        ];
        
        DensePolynomial::from_coefficients_vec(coeffs)
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
        let mut current_sum = proof.claimed_sum;
        let mut partial_evaluation = Vec::new();

        // Verify each round
        for (round_poly, &challenge) in proof.round_polynomials.iter().zip(proof.challenges.iter()) {
            // Check polynomial degree is at most 1
            if round_poly.degree() > 1 {
                return false;
            }

            // Verify sum at x=0 and x=1 matches the claimed sum
            let sum_0 = round_poly.evaluate(&F::zero());
            let sum_1 = round_poly.evaluate(&F::one());
            if sum_0 + sum_1 != current_sum {
                return false;
            }

            // Update for next round
            current_sum = round_poly.evaluate(&challenge);
            partial_evaluation.push(challenge);
        }

        // Final check: verify the claimed evaluation
        proof.final_evaluation == polynomial.evaluate(&partial_evaluation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr;

    #[test]
    fn test_sumcheck_protocol() {
        // Test polynomial f(x,y) = x + y + xy
        let coefficients = vec![
            Fr::from(0u64),  // constant term
            Fr::from(1u64),  // x term
            Fr::from(1u64),  // y term
            Fr::from(1u64),  // xy term
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
    fn test_polynomial_evaluation() {
        // Test evaluation at point (1,1)
        let coefficients = vec![
            Fr::from(0u64),  // constant term
            Fr::from(1u64),  // x term
            Fr::from(1u64),  // y term
            Fr::from(1u64),  // xy term
        ];
        let polynomial = MultivariatePoly::new(coefficients, 2);
        
        let point = vec![Fr::from(1u64), Fr::from(1u64)];
        let evaluation = polynomial.evaluate(&point);
        
        // f(1,1) = 0 + 1 + 1 + 1 = 3
        assert_eq!(evaluation, Fr::from(3u64));
    }

    #[test]
    fn test_boolean_hypercube_sum() {
        // Test sum over {0,1}^2
        let coefficients = vec![
            Fr::from(0u64),  // constant term
            Fr::from(1u64),  // x term
            Fr::from(1u64),  // y term
            Fr::from(1u64),  // xy term
        ];
        let polynomial = MultivariatePoly::new(coefficients, 2);
        
        let sum = polynomial.sum_over_boolean_hypercube();
        
        // Sum = f(0,0) + f(0,1) + f(1,0) + f(1,1)
        //     = 0 + 1 + 1 + 3
        //     = 5
        assert_eq!(sum, Fr::from(5u64));
    }
}