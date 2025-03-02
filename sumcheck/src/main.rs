use ark_ff::{BigInteger, PrimeField};
use multilinear::multilinear::MultivariatePoly;
use multilinear::composite::{Composite, OP};
use prime_polynomail::{self, DensePolynomial};
use transcript::transcript::{HashTrait, Transcript, TranscriptTrait};
use std::marker::PhantomData;
use std::iter::repeat_n;

/// The Sum-Check protocol is a protocol for verifying that the sum of a polynomial over a
/// boolean hypercube is equal to a claimed value.
/// 
pub fn generate_partial_proof<F: PrimeField, H: HashTrait, T: TranscriptTrait<F>>(poly: &Composite<F>, transcript: &mut T, round_polys: &mut Vec<DensePolynomial<F>>,  challenges: &mut Vec<F>) -> F {
    let mut poly_eval = poly.clone();
    let degree = 2;
    let rounds = poly_eval.polys[0].num_vars as usize;   
    print!("rounds={:?}", rounds); 
    let mut partial_evals = vec![];
    let mut final_eval = F::zero();

    for i in 0..rounds {
        let mut reduced_poly = poly_eval.reduce();
        let extra_points = reduced_poly.coeffs.len()/2;
        let mut index = 0;

        repeat_n(0, extra_points).for_each(|_| {
            let mut values = vec![Some(F::zero()); rounds-i];
            values = values.iter().enumerate().map( |x| {
                if x.0 == 0 {
                    return Some(F::from(2));
                } else {
                    // shift to right and find modulus to get the value at that point.
                    return Some(F::from(index >> (rounds-i - x.0 - 1) & 1));
                }
            }).collect();

            let result = poly_eval.evaluate(&values);
            reduced_poly.coeffs.push(result);
            index += 1;
        });

        let mut round_poly = vec![];
        for j in 0..(degree + 1) {
            round_poly.push(reduced_poly.coeffs.iter().skip(j * extra_points).take(extra_points).sum());
        }

        final_eval = round_poly[0] + round_poly[1];
        // dbg!(&round_poly, final_eval);        
        partial_evals.push(final_eval);
        let mut data = vec![final_eval];
        data.extend(&round_poly);
        let challenge = add_data_to_transcript::<F, H, T>(&data, transcript);
        // dbg!(&challenge);

        challenges.push(challenge);

        poly_eval = poly_eval.partial_evaluate(&vec![challenge], 0);
        round_polys.push(DensePolynomial { coefficients: round_poly });

    }

    partial_evals[0]

  

}

//write a verify_partial_proof function that takes in the initial sum, the round polynomials, and the transcript, and returns the final sum
pub fn verify_partial_proof<F: PrimeField, H: HashTrait, T: TranscriptTrait<F>>(initial_sum: F, round_polys: &Vec<DensePolynomial<F>>, transcript: &mut T) -> (F, Vec<F>) {
    let mut final_sum = initial_sum;
    let mut challenges = vec![];
    for (i, round_poly) in round_polys.iter().enumerate() {
        if final_sum != round_poly.coefficients[0] + round_poly.coefficients[1] {
            // dbg!(i, final_sum, round_poly.coefficients[0] + round_poly.coefficients[1]);
            panic!("Invalid proof");
            return (F::zero(), vec![]);
        }        
        let mut data = vec![final_sum];
        data.extend(&round_poly.coefficients);
        let challenge
        = add_data_to_transcript::<F, H, T>(&data, transcript);
        dbg!(&data);
        challenges.push(challenge);

        let points = round_polys[i].coefficients.iter().enumerate().map( |x| (F::from(x.0 as u64), x.1.clone())).collect::<Vec<(F, F)>>();
        let univariate_poly = DensePolynomial::interpolate(&points);
        dbg!(&univariate_poly);
        print!("================>>>>>>>>>>>>>>");
        dbg!(&points);
        final_sum = univariate_poly.evaluate(challenge);
        dbg!(&final_sum, challenge);
    }
    (final_sum, challenges)
}

pub fn verify_partial_proof_2<F: PrimeField, H: HashTrait, T: TranscriptTrait<F>> (sum: F, polys: &Vec<Vec<F>>, transcript: &mut T) -> (F , Vec<F>) {
    let mut challenges = vec![];
    let mut challenge;
    let mut sum = sum;

    for i in 0..polys.len() {
        if sum != polys[i][0] + polys[i][1] {
            panic!("Invalid proof for partial sum check");
        }

        let mut data = vec![sum];
        data.extend(&polys[i]);
        challenge = add_data_to_transcript::<F, H, T>(&data, transcript);
        dbg!(&data);
        challenges.push(challenge);

        let points = polys[i].iter().enumerate().map( |x| (F::from(x.0 as u64), x.1.clone())).collect::<Vec<(F, F)>>();
        let univariate_poly = DensePolynomial::interpolate(&points);
        println!("Univariate poly from Nonse");
        dbg!(&univariate_poly);
        dbg!(&points);
        sum = DensePolynomial::evaluate(&univariate_poly, challenge);
        dbg!(&sum, challenge);
    }

    (sum, challenges)
}

pub fn add_data_to_transcript <F: PrimeField, H: HashTrait, T: TranscriptTrait<F>> (data: &Vec<F>, transcript: &mut T) -> F {
    let mut bytes = vec![];
    data.iter().for_each(|x| {
        bytes.extend(x.into_bigint().to_bytes_be())
    });
    transcript.absorb(&bytes);
    let squeezed = transcript.squeeze();
    let squeezed_bytes = squeezed.into_bigint().to_bytes_be();
    let challenge = F::from_be_bytes_mod_order(&squeezed_bytes);
    return challenge;
}





fn main(){
    println!("Hello, world!");
}

#[cfg(test)]
mod tests{
      // use super::
      use super::*;
      use ark_bn254::Fq;
      use sha3::{Keccak256, Digest};
      use transcript::transcript::KeccakWrapper;
  
    use multilinear::multilinear::MultivariatePoly;


    #[test]
    fn test_generate_partial_proof() {
        // 2a + 3
        let mut poly_a = MultivariatePoly::new(vec![3, 5].iter().map(|x| Fq::from(x.clone())).collect(), 1);
        poly_a = poly_a.blow_up_right( 2);
        print!("Polynomail a={:?}", poly_a.coeffs);

        // 4b + 2a
        let mut poly_b = MultivariatePoly::new(vec![0, 4, 2, 6].iter().map(|x| Fq::from(x.clone())).collect(), 2);
        poly_b = poly_b.blow_up_right( 1);
        print!("Polynomail b={:?}", poly_b.coeffs);

        // 3c + 2
        let mut poly_c = MultivariatePoly::new(vec![2, 5].iter().map(|x| Fq::from(x.clone())).collect(), 1);
        poly_c = poly_c.blow_up_left( 2); 
        print!("Polynomail c={:?}", poly_c.coeffs);

        // 3c + 2
        let mut poly_d = MultivariatePoly::new(vec![2, 5].iter().map(|x| Fq::from(x.clone())).collect(), 1);
        poly_d = poly_d.blow_up_left( 2);        
        print!("Polynomail d={:?}", poly_d.coeffs); 

        let composite = Composite::new(
            &vec![poly_a.coeffs, poly_b.coeffs, poly_c.coeffs, poly_d.coeffs],
            vec![OP::MUL, OP::ADD, OP::MUL]
        );
        print!("Composite={:?}", composite.polys);

        let mut round_polys: Vec<DensePolynomial<Fq>> = vec![];
        let mut transcript = Transcript::<KeccakWrapper, Fq>::new(KeccakWrapper {
            keccak: Keccak256::new(),
        });
        let mut challenges = vec![];
        let initial_sum = generate_partial_proof::<Fq, KeccakWrapper, Transcript<KeccakWrapper, Fq>>(&composite, &mut transcript, &mut round_polys, &mut challenges);

        let hasher = KeccakWrapper { keccak: Keccak256::new() };
        let mut transcript = Transcript::new(hasher);
        let (sum, challenges) = verify_partial_proof::<Fq, KeccakWrapper, Transcript<KeccakWrapper, Fq>>(initial_sum, &round_polys, &mut transcript);

        assert_eq!(
            sum,
            composite.evaluate(&challenges.iter().map(|x| Some(x.clone())).collect())
        );
    }

    #[test]
    fn test_generate_partial_proof_2() {
        // 2a + 3
        let mut poly_a = MultivariatePoly::new(vec![3, 5].iter().map(|x| Fq::from(x.clone())).collect(), 1);
        poly_a = poly_a.blow_up_right( 1);
        // print!("Polynomail a={:?}", poly_a.coeffs);

        // 4b + 2a
        let mut poly_b = MultivariatePoly::new(vec![0, 4, 2, 6].iter().map(|x| Fq::from(x.clone())).collect(), 2);
        // poly_b = poly_b.blow_up_right( 1);
        // print!("Polynomail b={:?}", poly_b.coeffs);

        let mut poly_c = MultivariatePoly::new(vec![2, 5].iter().map(|x| Fq::from(x.clone())).collect(), 1);
        poly_c = poly_c.blow_up_left( 1); 
        // print!("Polynomail c={:?}", poly_c.coeffs);

        let composite = Composite::new(
            &vec![poly_a.coeffs, poly_b.coeffs, poly_c.coeffs],
            vec![OP::MUL, OP::ADD]
        );
        // print!("Composite={:?}", composite.polys);

        let mut round_polys: Vec<DensePolynomial<Fq>> = vec![];
        let mut transcript = Transcript::<KeccakWrapper, Fq>::new(KeccakWrapper {
            keccak: Keccak256::new(),
        });
        let mut challenges = vec![];
        let initial_sum = generate_partial_proof::<Fq, KeccakWrapper, Transcript<KeccakWrapper, Fq>>(&composite, &mut transcript, &mut round_polys, &mut challenges);

        let hasher = KeccakWrapper { keccak: Keccak256::new() };
        let mut transcript = Transcript::new(hasher);

        let polys_2: Vec<Vec<Fq>> = round_polys.iter().map(|p| p.coefficients.clone()).collect();
        let (sum_2, challenges_2) = verify_partial_proof_2::<Fq, KeccakWrapper, Transcript<KeccakWrapper, Fq>>(initial_sum, &polys_2, &mut transcript);

        let hasher = KeccakWrapper { keccak: Keccak256::new() };
        let mut transcript = Transcript::new(hasher);
        let (sum, challenges) = verify_partial_proof::<Fq, KeccakWrapper, Transcript<KeccakWrapper, Fq>>(initial_sum, &round_polys, &mut transcript);

        

        assert_eq!(
            sum,
            composite.evaluate(&challenges.iter().map(|x| Some(x.clone())).collect())
        );
    }
}