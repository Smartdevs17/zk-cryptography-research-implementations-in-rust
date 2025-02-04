use ark_ff::PrimeField;


#[derive(Clone, Debug)]
pub struct MultivariatePoly<F: PrimeField> {
    pub(crate) coeffs: Vec<F>,
    pub(crate) num_vars: usize,
}

impl<F: PrimeField> MultivariatePoly<F> {
   pub fn new(coeffs: Vec<F>, num_vars: usize) -> Self {
        if coeffs.len() != 2usize.pow(num_vars as u32) {
            panic!("Invalid number of coefficients");
        }
        Self { coeffs, num_vars }
    }

    pub fn evaluate(&self, point: &Vec<F>) -> F {
        if point.len() != self.num_vars {
            panic!("Invalid number of variables");
        }
        let mut result = F::zero();
        for i in 0..self.coeffs.len() {
            let mut term = self.coeffs[i];
            for j in 0..self.num_vars {
                if i & (1 << j) != 0 {
                    term *= point[j];
                }
            }
            result += term;
        }
        result
    }

   pub fn sum_over_boolean_hypercube(&self) -> F {
        let mut sum = F::zero();
        let num_points = 1 << self.num_vars; // 2^num_vars

        for i in 0..num_points {
            let mut point = vec![F::zero(); self.num_vars];
            for j in 0..self.num_vars {
                if (i >> j) & 1 == 1 {
                    point[j] = F::one();
                }
            }
            sum += self.evaluate(&point);
        }
        sum
    }

   pub fn evaluate_at_round(&self, round: usize, partial_evaluation: &[F], x: F) -> F {
        let mut point = partial_evaluation[0..round].to_vec();
        point.push(x);
        point.extend(vec![F::zero(); self.num_vars - round - 1]);

        let mut sum = F::zero();
        let remaining_vars = self.num_vars - round - 1;
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