use std::iter::{Product, Sum};
use std::ops::{Add, Mul};

#[derive(Debug, PartialEq, Clone)]
struct Polynomail {
    coefficients: Vec<f64>,
}

impl Polynomail {
    fn new(coefficients: Vec<f64>) -> Polynomail {
        Polynomail { coefficients }
    }

    fn degree(&self) -> usize {
        self.coefficients.len() - 1
    }

    fn evaluate(&self, x: f64) -> f64 {
        self.coefficients
            .iter()
            .enumerate()
            .map(|(i, c)| c * x.powi(i as i32))
            .sum()
    }

    fn interpolate(xs: Vec<f64>, ys: Vec<f64>) -> Self {
        xs.iter()
            .zip(ys.iter())
            .map(|(x, y)| Self::basis(x, &xs).scalar_mul(y))
            .sum()
    }

    fn scalar_mul(&self, scalar: &f64) -> Self {
        Polynomail {
            coefficients: self.coefficients.iter().map(|c| c * scalar).collect(),
        }
    }

    fn basis(x: &f64, interpolating_set: &[f64]) -> Self {
        // numerator
        let numerator: Polynomail = interpolating_set
            .iter()
            .filter(|val| *val != x)
            .map(|x_n| Polynomail::new(vec![-x_n, 1.0]))
            .product();

        // denominator
        let denominator = 1.0 /  numerator.evaluate(*x);

        numerator.scalar_mul(&denominator)
    }
}

impl Mul for &Polynomail {
    type Output = Polynomail;

    fn mul(self, rhs: Self) -> Self::Output {
        let new_degree = self.degree() + rhs.degree();
        let mut result = vec![0.0; new_degree + 1];
        for i in 0..self.coefficients.len() {
            for j in 0..rhs.coefficients.len() {
                result[i + j] += self.coefficients[i] * rhs.coefficients[j];
            }
        }
        Polynomail {
            coefficients: result,
        }
    }
}

impl Add for &Polynomail {
    type Output = Polynomail;

    fn add(self, rhs: Self) -> Self::Output {
        let (mut bigger, smaller) = if self.degree() < rhs.degree() {
            (rhs.clone(), self)
        } else {
            (self.clone(), rhs)
        };

        for (b_coeff, s_coeff) in bigger
            .coefficients
            .iter_mut()
            .zip(smaller.coefficients.iter())
        {
            *b_coeff += s_coeff;
        }

        Polynomail::new(bigger.coefficients)
    }
}

impl Sum for Polynomail {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut result = Polynomail::new(vec![0.0]);
        for poly in iter {
            result = &result + &poly;
        }
        result
    }
}

impl Product for Polynomail {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut result = Polynomail::new(vec![1.0]); // Start with neutral element for multiplication
        for poly in iter {
            result = &result * &poly;
        }
        result
    }
}

fn main() {
    let coefficients = Polynomail::new(vec![5.0, 2.0]);
    println!("this is the degree: {}", coefficients.degree());
    println!("Evaluate at f(3)= {}", coefficients.evaluate(3.0));

    let xs_points = vec![2.0, 4.0];
    let ys_points = vec![4.0, 8.0];
    let interpolated = Polynomail::interpolate(xs_points, ys_points);

    println!("Degree of polynomial: {}", interpolated.degree());
    println!(
        "The interpolation function coefficients: {:?}",
        interpolated.coefficients
    );
}
