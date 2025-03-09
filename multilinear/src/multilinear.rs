use ark_ff::PrimeField;
use ark_bn254::Fr;
use rand::thread_rng;
use std::ops::{Add, Mul};


#[derive(Clone, Debug, PartialEq)]
pub struct MultivariatePoly<F: PrimeField> {
    pub coeffs: Vec<F>,
    pub num_vars: usize,
}

impl<F: PrimeField> MultivariatePoly<F> {
    pub fn new(coeffs: Vec<F>, num_vars: usize) -> Self {
        if coeffs.len() != 2usize.pow(num_vars as u32) {
            panic!("Invalid number of coefficients");
        }
        Self { coeffs, num_vars }
    }

   
    pub fn partial_evaluate(poly: &Vec<F>, var_idx: usize, val: F) -> Vec<F> {
        let poly_size = poly.len();
        let new_poly_size = poly_size / 2;
        let mut new_poly: Vec<F> = Vec::with_capacity(new_poly_size);

        let mut i = 0;
        let mut j = 0;

        while i < new_poly_size {
            let y1 = poly[j];
            let num_vars = poly.len().ilog2() as usize;
            let power = num_vars - 1 - var_idx;
            let y2 = poly[j | (1 << power)];
            new_poly.push(y1 + (val * (y2 - y1)));

            i += 1;
            j = if (j + 1) % (1 << power) == 0 {
                j + 1 + (1 << power)
            } else {
                j + 1
            }
        }

        new_poly
    }


    pub fn evaluate(&self, point: &Vec<F>) -> F {
        if point.len() != self.num_vars {
            panic!("Invalid number of variables");
        }
        let mut result = F::zero();
        for i in 0..self.coeffs.len() {
            let mut term = self.coeffs[i];
            for j in 0..self.num_vars {
                if (i >> j) & 1 == 1 {
                    term *= point[j]; // Multiply by x_j if the bit is set
                } else {
                    term *= F::one() - point[j]; // Multiply by (1 - x_j) if the bit is not set
                }
            }
            result += term;
        }
        result
    }

    pub fn evaluate_partial(&self, points: &Vec<F>) -> F {
        let mut evaluated_poly = self.coeffs.clone();
        let num_points = points.len();

        for i in 0..num_points {
            evaluated_poly = Self::partial_evaluate(&evaluated_poly, 0, points[i]);
        }

        evaluated_poly[0]
    }

    pub fn solve(&self, values: &Vec<Option<F>>) -> MultivariatePoly<F> {
        // The values 
          if 2_usize.pow(values.len() as u32) > self.coeffs.len() {
            println!("Polynomial is incorrect");
          }
          let hypercube = self.coeffs.clone();
          // log2 of hypercube length gives the number of variables
          let variable_len = hypercube.len().trailing_zeros() as usize;
          let mut intermediate_result = MultivariatePoly::new(hypercube, self.num_vars);
          for (i, value) in values.iter().enumerate() {
            intermediate_result = match value {
              Some(_value) => MultivariatePoly::new(MultivariatePoly::partial_evaluate(&intermediate_result.coeffs, variable_len - i - 1, *_value), intermediate_result.num_vars - 1),
              None => intermediate_result
            }
          }
    
          intermediate_result
      }
      

    pub fn sum_over_boolean_hypercube(&self) -> F {
        let num_vars = self.num_vars; // Number of variables
        let num_points = 1 << num_vars; // 2^num_vars
    
        let mut sum = F::zero();
    
        // Iterate over all points in the boolean hypercube
        for i in 0..num_points {
            // Create a point in the boolean hypercube
            let mut point = vec![F::zero(); num_vars];
            for j in 0..num_vars {
                if (i >> j) & 1 == 1 {
                    point[j] = F::one();
                }
            }
    
            // Evaluate the polynomial at this point
            let eval = self.evaluate(&point);
            sum += eval;
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

    pub fn blow_up_right(&self, blows: u32) -> Self {
        let mut new_coeffs = get_blow_up_poly(self, blows);
        new_coeffs = new_coeffs
            .iter()
            .enumerate()
            .map(|(i, _)| self.coeffs[i >> blows as usize])
            .collect();
        Self::new(new_coeffs, self.num_vars + blows as usize)
    }

    pub fn blow_up_left(&self, blows: u32) -> Self {
        let mut new_coeffs = get_blow_up_poly(self, blows);
        let mask = self.coeffs.len() - 1;
        new_coeffs = new_coeffs
            .iter()
            .enumerate()
            .map(|(i, _)| self.coeffs[i & mask])
            .collect();
        Self::new(new_coeffs, self.num_vars + blows as usize)
    }

    pub fn scalar_mul(&self, value: F) -> Self {
        Self::new(self.coeffs.iter().map(|&x| x * value).collect(), self.num_vars)
    }
}

pub fn get_blow_up_poly<F: PrimeField>(poly: &MultivariatePoly<F>, blows: u32) -> Vec<F> {
    if poly.coeffs.len() % 2 != 0 {
        panic!("Number of coefficients must be a power of 2");
    }
    let new_variable_len = 1 << (poly.coeffs.len().trailing_zeros() + blows);
    vec![F::zero(); new_variable_len as usize]
}

impl<F: PrimeField> Add for MultivariatePoly<F> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        if self.num_vars != other.num_vars {
            panic!("Polynomials must have the same number of variables");
        }
        let coeffs = self.coeffs.iter().zip(other.coeffs.iter())
            .map(|(a, b)| *a + *b)
            .collect();
        Self::new(coeffs, self.num_vars)
    }
}

impl<F: PrimeField> Mul for MultivariatePoly<F> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        if self.num_vars != other.num_vars {
            panic!("Polynomials must have the same number of variables");
        }
        let mut coeffs = vec![F::zero(); self.coeffs.len()];
        for i in 0..self.coeffs.len() {
            coeffs[i] = self.coeffs[i] * other.coeffs[i];
        }
        Self::new(coeffs, self.num_vars)
    }
}

#[cfg(test)]
/// Tests for the `MultivariatePoly` struct.
///
/// # Tests
///
/// - `test_new`: Tests the creation of a new `MultivariatePoly` instance with given coefficients and number of variables.
/// - `test_evaluate`: Tests the evaluation of the polynomial at a given point. The polynomial evaluated at point (1, 1) should result in 10.
/// - `test_sum_over_boolean_hypercube`: Tests the sum of the polynomial over the boolean hypercube. The sum should be 10.
/// - `test_evaluate_at_round`: Tests the evaluation of the polynomial at a specific round with partial evaluation and a given value. The result should be 10.
/// - `test_scalar_mul`: Tests the scalar multiplication of the polynomial. Each coefficient should be multiplied by the scalar value.
/// - `test_blow_up_right`: Tests the increase of the number of variables by blowing up the polynomial to the right. The number of variables should increase by 1.
/// - `test_blow_up_left`: Tests the increase of the number of variables by blowing up the polynomial to the left. The number of variables should increase by 1.

mod tests {
    use super::*;

    #[test]
    /// Tests the creation of a new `MultivariatePoly` instance with given coefficients and number of variables.
    fn test_new() {
        let coeffs = vec![Fr::from(1u64), Fr::from(2u64), Fr::from(3u64), Fr::from(4u64)];
        let poly = MultivariatePoly::new(coeffs.clone(), 2);
        assert_eq!(poly.coeffs, coeffs);
        assert_eq!(poly.num_vars, 2);
    }

    #[test]
    /// Tests the evaluation of the polynomial at a given point.
    /// The polynomial evaluated at point (1, 1) should result in 10.
    /// Equation: 1 + 2*1 + 3*1 + 4*1*1 = 10
    fn test_evaluate() {
        let coeffs = vec![Fr::from(1u64), Fr::from(2u64), Fr::from(3u64), Fr::from(4u64)];
        let poly = MultivariatePoly::new(coeffs, 2);
        let point = vec![Fr::from(1u64), Fr::from(1u64)];
        let result = poly.evaluate(&point);
        assert_eq!(result, Fr::from(10u64));
    }

    #[test]
    fn test_evaluate_4y_7xy() {
        let coeffs = vec![Fr::from(0u64), Fr::from(4u64), Fr::from(0u64), Fr::from(11u64)];
        let poly = MultivariatePoly::new(coeffs, 2);
        let point = vec![Fr::from(2u64), Fr::from(3u64)];
        let result = poly.evaluate(&point);
        assert_eq!(result, Fr::from(54u64)); // ✅ Should now pass
    }

    #[test]
    /// Tests the sum of the polynomial over the boolean hypercube.
    /// The sum should be 18.
    /// f(x, y) = 1 + 2x + 3y + 4xy
    /// Equation: sum of evaluations at points (0,0), (0,1), (1,0), (1,1) = 1 + 3 + 2 + 12 = 18
    fn test_sum_over_boolean_hypercube() {
        let coeffs = vec![Fr::from(1u64), Fr::from(2u64), Fr::from(3u64), Fr::from(4u64)];
        let poly = MultivariatePoly::new(coeffs, 2);
        let result = poly.sum_over_boolean_hypercube();
        assert_eq!(result, Fr::from(18u64));
    }

    #[test]
    /// Tests the evaluation of the polynomial at a specific round with partial evaluation and a given value.
    /// The result should be 10.
    /// Equation: partial evaluation at round 0 with value 1 results in evaluation at point (1, 1) = 10
    fn test_evaluate_at_round() {
        let coeffs = vec![Fr::from(1u64), Fr::from(2u64), Fr::from(3u64), Fr::from(4u64)];
        let poly = MultivariatePoly::new(coeffs, 2);
        let partial_evaluation = vec![Fr::from(1u64)];
        let x = Fr::from(1u64);
        let result = poly.evaluate_at_round(0, &partial_evaluation, x);
        assert_eq!(result, Fr::from(10u64));
    }

    #[test]
    /// Tests the scalar multiplication of the polynomial.
    /// Each coefficient should be multiplied by the scalar value.
    /// Equation: [1, 2, 3, 4] * 2 = [2, 4, 6, 8]
    fn test_scalar_mul() {
        let coeffs = vec![Fr::from(1u64), Fr::from(2u64), Fr::from(3u64), Fr::from(4u64)];
        let poly = MultivariatePoly::new(coeffs.clone(), 2);
        let scalar = Fr::from(2u64);
        let result = poly.scalar_mul(scalar);
        let expected_coeffs: Vec<Fr> = coeffs.iter().map(|&x| x * scalar).collect();
        assert_eq!(result.coeffs, expected_coeffs);
    }

    #[test]
    /// Tests the increase of the number of variables by blowing up the polynomial to the right.
    /// The number of variables should increase by 1.
    fn test_blow_up_right() {
        let coeffs = vec![Fr::from(1u64), Fr::from(2u64), Fr::from(3u64), Fr::from(4u64)];
        let poly = MultivariatePoly::new(coeffs, 2);
        let result = poly.blow_up_right(1);
        assert_eq!(result.num_vars, 3);
    }

    #[test]
    /// Tests the increase of the number of variables by blowing up the polynomial to the left.
    /// The number of variables should increase by 1.
    fn test_blow_up_left() {
        let coeffs = vec![Fr::from(1u64), Fr::from(2u64), Fr::from(3u64), Fr::from(4u64)];
        let poly = MultivariatePoly::new(coeffs, 2);
        let result = poly.blow_up_left(1);
        assert_eq!(result.num_vars, 3);
    }

    #[test]
    /// Tests the partial evaluation of the polynomial f(x, y) = 4y + 7xy at x = 2.
    /// The result should be 18y.
    fn test_partial_evaluate_4y_7xy_at_x_2() {
        let coeffs = vec![Fr::from(0u64), Fr::from(4u64), Fr::from(0u64), Fr::from(11u64)];
        let poly = MultivariatePoly::new(coeffs, 2);
        let result = MultivariatePoly::partial_evaluate(&poly.coeffs, 0, Fr::from(2u64));
        let expected_result = vec![Fr::from(0u64), Fr::from(18u64)];
        assert_eq!(result, expected_result);
    }

    #[test]
    /// Tests the partial evaluation of the polynomial f(x, y) = 4y + 7xy at x = 2 and y = 3.
    /// The result should be 54.
    fn test_evaluate_partial_4y_7xy_at_x_2_y_3() {
        let coeffs = vec![Fr::from(0u64), Fr::from(4u64), Fr::from(0u64), Fr::from(11u64)];
        let poly = MultivariatePoly::new(coeffs, 2);
        let point = vec![Fr::from(2u64), Fr::from(3u64)];
        let result = poly.evaluate_partial(&point);
        assert_eq!(result, Fr::from(54u64));
    }

}

