use std::cmp::max;
use std::iter::repeat_n;
use std::ops::Mul;
use ark_ff::{BigInteger, PrimeField};
use multilinear::multilinear::MultivariatePoly;
use multilinear::composite::{Composite, OP as COMPOSITE_OP};
use crate::circut::{ Circuit, OP as CIRCUIT_OP, Gate};
use prime_polynomail::DensePolynomial;
use transcript::transcript::{Transcript, HashTrait, TranscriptTrait};
use std::marker::PhantomData;
use sumcheck::sumcheck::{add_data_to_transcript, generate_partial_proof, verify_partial_proof, verify_partial_proof_2};
use transcript::transcript::KeccakWrapper;


//the number of vairable depends on the number of ceofficents so if is 2 coeff is 1 vairable and if is 4 is 2 vairable and so on


#[derive(Debug)]
struct GKR_PROOF<F: PrimeField> {
  claimed_sums: Vec<F>,
  round_polys: Vec<Vec<Vec<F>>>,
  evaluations: Vec<(F, F)>,
  output: Vec<F>
}

fn generate_proof<F: PrimeField, H: HashTrait, T: TranscriptTrait<F>>(circuit: &mut Circuit<F>, inputs: &Vec<F>, transcript: &mut T) -> GKR_PROOF<F> {
  circuit.evaluate(inputs);
  let mut gkr_proof = GKR_PROOF {
      claimed_sums: vec![],
      round_polys: vec![],
      evaluations: vec![],
      output: vec![],
  };

  let mut add_and_muls = vec![];
  get_add_and_muls(&circuit, &mut add_and_muls);

  let mut _w = circuit.layers[0].clone();

  if _w.len() == 1 {
      _w = vec![_w[0], F::zero()];
  }
  let num_variables = (circuit.layers[0].len() as f64).log2().ceil() as usize;
  let w_i = MultivariatePoly::new(_w, num_variables);

  let challenges_length = next_pow_of_2(w_i.coeffs.len());
  let mut challenges = vec![F::zero(); challenges_length];
  add_data_to_transcript::<F, H, T>(&w_i.coeffs, transcript);
  let squeezed = transcript.squeeze();
  let squeezed_bytes = squeezed.into_bigint().to_bytes_be();
  challenges = challenges.iter().map(|_| F::from_be_bytes_mod_order(&squeezed_bytes)).collect();

  for i in 0..circuit.gates.len() {
      let (mut add_poly, mut mul_poly) = add_and_muls[i].clone();

      let num_variables = (circuit.layers[i + 1].len() as f64).log2().ceil() as usize;
      let w_i_plus_1 = MultivariatePoly::new(circuit.layers[i + 1].clone(), num_variables);
      let blows = next_pow_of_2(w_i_plus_1.coeffs.len()) as u32;
      // blow ups
      let w_b = w_i_plus_1.blow_up_right(blows); // blow up for c
      let w_c = w_i_plus_1.blow_up_left(blows); // blow up for b
      let num_variables = (w_b.coeffs.len() as f64).log2().ceil() as usize;
      let w_plus = MultivariatePoly::new(w_b.coeffs.clone(), num_variables) + MultivariatePoly::new(w_c.coeffs.clone(), num_variables);
      let w_mul = MultivariatePoly::new(w_b.coeffs.clone(), num_variables) * MultivariatePoly::new(w_c.coeffs.clone(), num_variables);

      if i != 0 {
          let alpha = F::from_be_bytes_mod_order(&transcript.squeeze().into_bigint().to_bytes_be());
          let beta = F::from_be_bytes_mod_order(&transcript.squeeze().into_bigint().to_bytes_be());
          add_poly = apply_alpha_beta(alpha, beta, &challenges, &add_poly);
          mul_poly = apply_alpha_beta(alpha, beta, &challenges, &mul_poly);
      } else {
          add_poly = add_poly.solve(&challenges.iter().map(|x| Some(*x)).collect());
          mul_poly = mul_poly.solve(&challenges.iter().map(|x| Some(*x)).collect());
      }

      let hypercubes = vec![
          add_poly,
          MultivariatePoly::new(w_plus.coeffs.clone(), (w_plus.coeffs.len() as f64).log2().ceil() as usize),
          mul_poly,
          MultivariatePoly::new(w_mul.coeffs.clone(), (w_mul.coeffs.len() as f64).log2().ceil() as usize),
      ]
      .iter()
      .map(|x| x.coeffs.clone())
      .collect();

      let f_poly = Composite::new(
          &hypercubes,
          vec![COMPOSITE_OP::MUL, COMPOSITE_OP::ADD, COMPOSITE_OP::MUL],
      );
      let mut round_polys = vec![];
      challenges = vec![];
      // returns challenges and initial claimed sum
      let sum = generate_partial_proof::<F, H, T>(&f_poly, transcript, &mut round_polys, &mut challenges);

      let w_b_eval = w_i_plus_1.solve(&challenges.iter().take(blows as usize).map(|x| Some(*x)).collect()).coeffs[0];
      let w_c_eval = w_i_plus_1.solve(&challenges.iter().skip(blows as usize).map(|x| Some(*x)).collect()).coeffs[0];

      add_data_to_transcript::<F, H, T>(&vec![w_b_eval, w_c_eval], transcript);

      gkr_proof.claimed_sums.push(sum);
      gkr_proof.round_polys.push(round_polys.iter().map(|poly| poly.coefficients.clone()).collect());
      gkr_proof.evaluations.push((w_b_eval, w_c_eval));
  }

  gkr_proof.output = circuit.layers[0].clone();

  gkr_proof
}



fn verify_proof<F: PrimeField, H: HashTrait, T: TranscriptTrait<F>> (circuit: &mut Circuit<F>, inputs: &Vec<F>, transcript: &mut T, gkr_proof: GKR_PROOF<F>) -> bool {

  let mut add_and_muls = vec![];
  get_add_and_muls(&circuit, &mut add_and_muls);

  let evaluations = gkr_proof.evaluations;
  let claimed_sums = gkr_proof.claimed_sums;
  let round_polys: Vec<Vec<DensePolynomial<F>>> = gkr_proof.round_polys.iter()
      .map(|poly_vec| poly_vec.iter().map(|coeffs| DensePolynomial::new(coeffs.clone())).collect())
      .collect();

  let mut _w = gkr_proof.output;
  if _w.len() == 1 { _w.push(F::zero()) }
  let num_variables = (_w.len() as f64).log2().ceil() as usize;
  let w_i = MultivariatePoly::new(_w, num_variables);

  let challenges_length = next_pow_of_2(w_i.coeffs.len());  
  let mut challenges = vec![F::zero(); challenges_length];

  add_data_to_transcript::<F, H, T>(&w_i.coeffs, transcript);
  challenges = challenges.iter().map(|_| F::from_be_bytes_mod_order(&transcript.squeeze().into_bigint().to_bytes_be())).collect();  

  let last_index = circuit.gates.len()-1;
  for i in 0..circuit.gates.len(){
    // follows order of transcript call to ensure it gets the same challenges as prover
    // so alpha and beta are fetched before verify_partial_proof is called even though they aren't used
    let (mut alpha, mut beta)  = (F::zero(), F::zero());
    if i != 0 {
      alpha = F::from_be_bytes_mod_order(&transcript.squeeze().into_bigint().to_bytes_be());
      beta = F::from_be_bytes_mod_order(&transcript.squeeze().into_bigint().to_bytes_be()); 
    }
    let polys_2: Vec<Vec<F>> = round_polys[i].iter().map(|p| p.coefficients.clone()).collect();
    let (sum, new_challenges, success) = verify_partial_proof_2::<F, H, T>(claimed_sums[i], &polys_2, transcript);
    if !success { return false; }
    let (mut add_poly, mut mul_poly) = add_and_muls[i].clone();

    let (w_b_eval, w_c_eval, w_plus, w_mul);
    if i < last_index {
      (w_b_eval, w_c_eval) = evaluations[i];
      (w_plus , w_mul) = (w_b_eval + w_c_eval, w_b_eval * w_c_eval);
    } else {
      // last layer 
      let num_variables = (inputs.len() as f64).log2().ceil() as usize;
      let w_inputs = MultivariatePoly::new(inputs.clone(), num_variables);
      let challenges_len = new_challenges.len() / 2;
      let b_challenges = new_challenges.iter().take(challenges_len).map(|x| Some(*x)).collect();
      let c_challenges = new_challenges.iter().skip(challenges_len).take(challenges_len).map(|x| Some(*x)).collect();
      w_b_eval = w_inputs.solve(&b_challenges).coeffs[0];
      w_c_eval = w_inputs.solve(&c_challenges).coeffs[0];
      (w_plus, w_mul) = (w_b_eval + w_c_eval, w_b_eval * w_c_eval);
    }

    add_data_to_transcript::<F, H, T>(&vec![w_b_eval, w_c_eval], transcript);
    
    
    if i != 0 {
      mul_poly = apply_alpha_beta(alpha, beta, &challenges, &mul_poly);
      add_poly = apply_alpha_beta(alpha, beta, &challenges, &add_poly);
    } else {
      mul_poly = mul_poly.solve(&challenges.iter().map(|x| Some(*x)).collect());            
      add_poly = add_poly.solve(&challenges.iter().map(|x| Some(*x)).collect());
    }

      mul_poly = mul_poly.scalar_mul( w_mul);
      add_poly = add_poly.scalar_mul( w_plus);

    let f_poly = mul_poly + add_poly;
    let evaluated_sum = f_poly.solve(&new_challenges.iter().map(|x| Some(*x)).collect()).coeffs[0];
    if sum != evaluated_sum {
      return false;
    }

    challenges = new_challenges;
  }

  return true;  
}

fn get_add_and_muls<F: PrimeField> (circuit: &Circuit<F>, add_and_muls: &mut Vec<(MultivariatePoly<F>, MultivariatePoly<F>)> ) {
  for i in 0..circuit.gates.len() {
    let gates_length = circuit.gates[i].len();
    let layer_length;
    if circuit.gates.len() <= i + 1 {
      layer_length = circuit.gates[i].iter().map(|x| max(x.left_input, x.right_input)).max().unwrap();
    } else {
      layer_length = circuit.gates[i+1].len();
    }
    let max_layer_bits = next_pow_of_2(layer_length);
    let max_gates_bits = next_pow_of_2(gates_length);

    let points_len = 1 << max_gates_bits + (max_layer_bits*2);
    let mut add_poly = vec![F::zero(); points_len];
    let mut mul_poly = vec![F::zero(); points_len];

    for (j, gate) in circuit.gates[i].iter().enumerate() {
      let index = (j << max_layer_bits * 2) // gate bits
          + (gate.left_input << max_layer_bits) // left_input bits
          + gate.right_input; // right_input bits
      match gate.op {
        CIRCUIT_OP::ADD => add_poly[index] = F::one(),
        CIRCUIT_OP::MUL => mul_poly[index] = F::one()
      }
    }

    let num_variables = (add_poly.len() as f64).log2().ceil() as usize;
    add_and_muls.push((MultivariatePoly::new(add_poly, num_variables), MultivariatePoly::new(mul_poly, num_variables)));

    // f_polys.push(FPOLY::new(mul_poly, add_poly, layer.clone()))
  }  
}

fn next_pow_of_2 (no: usize) -> usize {
  let toOne = |x: usize| -> usize { if x == 0 {1} else {x}};
  toOne((no as f64).log2().ceil() as usize)
}

fn apply_alpha_beta <F: PrimeField> (alpha: F, beta: F, challenges: &Vec<F>, former_op_poly: &MultivariatePoly<F>) -> MultivariatePoly<F> {
  let no_of_challenges = challenges.len()/2;
  let mut polys = vec![];

  for  skip in [0, no_of_challenges] {
    let no_of_variables = (former_op_poly.coeffs.len() as f64).log2() as usize;
    let mut _challenges: Vec<Option<F>> = challenges
        .iter()
        .skip(skip)
        .take(no_of_challenges)
        .map(|x| Some(*x))
        .collect();
    _challenges.extend(&vec![None; no_of_variables - no_of_challenges]);
    dbg!(&_challenges);
    polys.push(former_op_poly.solve(&_challenges));
  }

  
  polys[0].scalar_mul(alpha) + polys[1].scalar_mul(beta)
}


#[cfg(test)]
mod test {
  use super::*;
  use ark_bn254::Fq;
  use sha3::{Keccak256, Digest};  

  #[test]
  fn test_get_add_and_muls() {
        let gates = vec![
      // layer 1
      vec![
        Gate::new(0, 1, CIRCUIT_OP::ADD, 0),
      ],
      vec![
        Gate::new(0, 1, CIRCUIT_OP::ADD, 0),
        Gate::new(2, 3, CIRCUIT_OP::MUL, 1),
      ]
    ];

    let mut circuit: Circuit<Fq> = Circuit::new(
      gates
    );

    let inputs: Vec<Fq> = vec![ 1, 2, 3, 4 ].iter().map(|x| Fq::from(*x)).collect();
    let mut add_and_muls = vec![];
    get_add_and_muls(&circuit, &mut add_and_muls);

    assert_eq!(
      add_and_muls[0].0.coeffs,
      vec![ 0, 1, 0, 0, 0, 0, 0, 0].iter().map(|x| Fq::from(*x as u64)).collect::<Vec<Fq>>()
    );

    assert_eq!(
      add_and_muls[0].1.coeffs,
      vec![ 0, 0, 0, 0, 0, 0, 0, 0].iter().map(|x| Fq::from(*x as u64)).collect::<Vec<Fq>>()
    );

    assert_eq!(
      add_and_muls[1].0.coeffs,
      vec![ 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ].iter().map(|x| Fq::from(*x as u64)).collect::<Vec<Fq>>()
    );

    assert_eq!(
      add_and_muls[1].1.coeffs,
      vec![ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0,].iter().map(|x| Fq::from(*x as u64)).collect::<Vec<Fq>>()
    );
  }

  // 4b + 2a
  #[test]
  fn test_apply_alpha_beta() {
    let poly = MultivariatePoly::new(
      vec![0, 4, 3, 7, 2, 6, 5, 9].iter().map(|x| Fq::from(*x)).collect(),
      3
    );
    let new_poly: MultivariatePoly<Fq> = 
      apply_alpha_beta(Fq::from(2), Fq::from(3), &vec![Fq::from(2), Fq::from(3)], &poly);

    assert_eq!(
      new_poly.coeffs,
      vec![26, 46, 41, 61].iter().map(|x| Fq::from(*x)).collect::<Vec<Fq>>()
    )
  }

  // #[test]
  // fn test_generate_proof() {
  //   let gates = vec![
  //     // layer 1
  //     vec![
  //       Gate::new(0, 1, CIRCUIT_OP::MUL, 0),
  //     ],   
  //     vec![
  //       Gate::new(0, 1, CIRCUIT_OP::ADD, 0),
  //       Gate::new(2, 3, CIRCUIT_OP::MUL, 1),        
  //     ],
  //     vec![
  //       Gate::new(0, 1, CIRCUIT_OP::ADD, 0),
  //       Gate::new(2, 3, CIRCUIT_OP::MUL, 1),
  //       Gate::new(4, 5, CIRCUIT_OP::MUL, 2),
  //       Gate::new(6, 7, CIRCUIT_OP::ADD, 3)      
  //     ]
  //   ];

  //   let mut circuit: Circuit<Fq> = Circuit::new(
  //     gates
  //   );

  //   let inputs: Vec<Fq> = vec![ 1, 2, 3, 4, 5, 6, 7, 8 ].iter().map(|x| Fq::from(*x)).collect();
    
  //   let mut hasher = KeccakWrapper { keccak: Keccak256::new() };
  //   let mut transcript = Transcript::new(hasher);
  //   let gkr_proof = generate_proof::<Fq, KeccakWrapper, Transcript<KeccakWrapper, Fq>>(&mut circuit, &inputs, &mut transcript);
    
  //   hasher = KeccakWrapper { keccak: Keccak256::new() };
  //   transcript = Transcript::new(hasher);
  //   assert_eq!(
  //     true, 
  //     verify_proof::<Fq, KeccakWrapper, Transcript<KeccakWrapper, Fq>>(&mut circuit, &inputs, &mut transcript, gkr_proof)
  //   );
  // }
}