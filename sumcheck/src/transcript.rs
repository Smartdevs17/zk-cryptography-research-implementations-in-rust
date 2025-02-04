use ark_ff::PrimeField;
use sha3::{digest::Update, Digest, Keccak256};

// Transcript for generating challenges using Keccak256
pub struct Transcript {
    hasher: Keccak256,
}

impl Transcript {
    // Create a new Transcript
    pub fn new() -> Self {
        Self {
            hasher: Keccak256::new(),
        }
    }

    // Append data to the transcript
    pub fn append(&mut self, new_data: &[u8]) {
        Update::update(&mut self.hasher, new_data);
    }

    // Sample a challenge (32-byte hash)
    fn sample_challenge(&mut self) -> [u8; 32] {
        let mut result_hash = [0; 32];
        result_hash.copy_from_slice(&self.hasher.finalize_reset());
        Update::update(&mut self.hasher, &result_hash);
        result_hash
    }

    // Sample a field element from the transcript
    pub fn sample_field_element<F: PrimeField>(&mut self) -> F {
        let challenge = self.sample_challenge();
        F::from_be_bytes_mod_order(&challenge)
    }

    // Sample a field element (alternative implementation)
    pub fn sample_element<F: PrimeField>(&mut self) -> F {
        let mut hash = [0; 32];
        hash.copy_from_slice(&self.hasher.finalize_reset());
        Update::update(&mut self.hasher, &hash);
        F::from_be_bytes_mod_order(&hash)
    }

    // Sample multiple field elements from the transcript
    pub fn sample_n_field_elements<F: PrimeField>(&mut self, n: usize) -> Vec<F> {
        (0..n).map(|_| self.sample_field_element::<F>()).collect()
    }
}