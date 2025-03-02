use ark_ff::PrimeField;
use sha3::{Keccak256, Digest};
use std::marker::PhantomData;

// A transcript is a hash function that can be used to generate a random field element
pub struct Transcript<K: HashTrait, F: PrimeField> {
    _field: PhantomData<F>, // Placeholder to hold the field even if we are not using it
    hash_function: K,
}

impl<K: HashTrait, F: PrimeField> Transcript<K, F> {
    pub fn new(hash_function: K) -> Self {
        Transcript {
            _field: PhantomData,
            hash_function,
        }
    }

    // Function to absorb data into the hash function
    pub fn absorb(&mut self, data: &[u8]) {
        self.hash_function.append(data);
    }

    // Squeeze will return a field element
    pub fn squeeze(&mut self) -> F {
        let hash_output = self.hash_function.generate_hash();
        F::from_be_bytes_mod_order(&hash_output)
    }

    // Fiat-Shamir challenge generation
    pub fn generate_challenge(&mut self) -> F {
        self.squeeze()
    }
}

// A vector is a growable array, but a slice is a fixed-size array you can only push to a specific index
pub trait HashTrait {
    fn append(&mut self, data: &[u8]);
    fn generate_hash(&self) -> Vec<u8>;
}

pub struct KeccakWrapper {
    keccak: Keccak256,
}

impl HashTrait for KeccakWrapper {
    fn append(&mut self, data: &[u8]) {
        self.keccak.update(data);
    }

    fn generate_hash(&self) -> Vec<u8> {
        self.keccak.clone().finalize().to_vec()
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use super::*;
    use ark_bn254::Fr;
    use sha3::Keccak256;

    #[test]
    fn test_transcript() {
        let mut transcript = Transcript::<KeccakWrapper, Fr>::new(KeccakWrapper {
            keccak: Keccak256::new(),
        });
        let data = "hello world".as_bytes(); // Converts to bytes
        let data2 = b"hello world"; // Converts to bytes
        transcript.absorb(data);
        let output = transcript.squeeze();
        println!("output: {:?}", output);
    }

    #[test]
    fn test_fiat_shamir() {
        let mut transcript = Transcript::<KeccakWrapper, Fr>::new(KeccakWrapper {
            keccak: Keccak256::new(),
        });

        // Simulate a prover's commitment
        let commitment = b"prover_commitment";
        transcript.absorb(commitment);

        // Generate a Fiat-Shamir challenge
        let challenge = transcript.generate_challenge();
        println!("Fiat-Shamir Challenge: {:?}", challenge);

        // Simulate a prover's response
        let response = b"prover_response";
        transcript.absorb(response);

        // Generate another challenge if needed
        let challenge2 = transcript.generate_challenge();
        println!("Fiat-Shamir Challenge 2: {:?}", challenge2);
    }
}