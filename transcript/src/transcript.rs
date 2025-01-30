use ark_ff::PrimeField;
use sha3::{Keccak256, Digest};
use std::marker::PhantomData;


//a transcript is a hash function that can be used to generate a random field element

struct Transcript<K: HashTrait, F: PrimeField> {
    _feild: PhantomData<F>,//place holder to hold the field even if we are not using it
    hash_function: K
}

impl<K: HashTrait, F: PrimeField> Transcript<K, F> {
    fn new(hash_function: K) -> Self {
        Transcript {
            _feild: PhantomData,
            hash_function
        }
    }

    fn init(hash_function: K) -> Self {
        Self{
            _feild: PhantomData,
            hash_function
        }
    }

    //function to absorb data into the hash function
    fn absorb(&mut self, data: &[u8]) {
        self.hash_function.append(data);
    }
    
   //squeeze will return a field element
    fn squeeze(&self) -> F {
        let hash_output = self.hash_function.generate_hash();
        F::from_be_bytes_mod_order(&hash_output)
    }
  
}
//a vector a growable array
//but a slice is a fixed size array you can only push to a specific index


trait HashTrait {
    fn append(&mut self, data: &[u8]);
    fn generate_hash(&self) -> Vec<u8>;
}

struct KeccakWrapper {
    keccak: Keccak256
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
mod test{

    use super::*;
    use sha3::Keccak256;

    #[test]
    fn test_transcript() {
        use ark_bn254::Fr; // Import a concrete PrimeField type
        let mut transcript = Transcript::<KeccakWrapper, Fr>::new(KeccakWrapper { keccak: Keccak256::new() });
        let data = "hello world".as_bytes(); //converts to bytes
        let data2 = b"hello world"; //converts to bytes
        transcript.absorb(data);
        let output = transcript.squeeze();
        println!("output: {:?}", output);
    }
} 