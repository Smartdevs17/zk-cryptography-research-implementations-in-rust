use ark_ff::PrimeField;

#[derive(Clone, Debug)]
struct MultilinearPoly<F: PrimeField> {
    evals: Vec<F>,
    num_vars: usize,
}

//eval
//partial_eval

impl<F: PrimeField> MultilinearPoly<F> {
    fn new(num_vars: usize, evaluations: Vec<F>) -> Self {
        assert_eq!(evaluations.len(), 1 << num_vars);
        Self { evals: evaluations, num_vars }
    }
    
    fn zero(num_vars: usize) -> Self {
        Self {
            evals: vec![F::zero(); 1 << num_vars],
            num_vars: num_vars,
        }
    }

    fn evaluate(&self, assignments: &[F]) -> F {
        if assignments.len() != self.num_vars {
            panic!("wrong number of assignments");
        }
        let mut poly = self.clone();
        for val in assignments{
            poly = poly.partial_evalulate(0, val);
        }

        poly.evals[0]
    }

fn partial_evalulate(&self, index: usize, val: &F) -> Self {
        // Use index to generate pairing
        //linear interpolate and evaluate
        //01 - (001, 101) - (1,5)
        //10 - (010, 110) - (2,6)
        //11 - (011, 111) - (3,7)
        let mut result = vec![];
        for (a , b) in Self::pairs(index, self.num_vars){
            let a = self.evals[a];
            let b = self.evals[b];
            result.push(a + (*val) * (b - a));
        }

        Self {
            evals: result,
            num_vars: self.num_vars - 1,
        }
    } 

    // fn pairs( index: usize, num_vars: usize) -> impl Iterator<Item = (usize, usize)> {
    //     let mask = 1 << index;
    //     (0..1 << self.num_vars).filter(move |&i| i & mask == 0).map(move |i| (i, i | mask))
    // }

    // 1 >> target_hc is the 2^target_hc bit
    fn pairs( index: usize, num_vars: usize) -> Vec<(usize, usize)> {
        let target_hc = num_vars - 1;
        let mut result = Vec::new();
        for i in 0..(1 << target_hc) {
            let inverted_index = num_vars -index - 1;
            let insert_zero = Self::insert_bit(i, inverted_index);
            let insert_one = Self::insert_bit(i, target_hc) | (1 << target_hc);
            result.push((insert_zero, insert_one));
        }
        result
    }
    

    //always insert 0 at index 0
    //3 insert 0 at index 1 insert_at(3, 1)
    //11 -> 101

    fn insert_bit(value: usize , index: usize) -> usize {
        // let mask = 1 << index;
        // let left = value & !(mask - 1);
        // let right = value & (mask - 1);
        // left | mask | right

        //1011 & 0011
        // 1<< 2 = 100
        //100 -1 = 11
        let high = value >> index;
        let mask = (1 << index) - 1;
        let low = value & mask;

        //high bit << index + 1 | low bit
        high << index + 1 | low
    }




}

fn main() {
    println!("Hello, world!");
}
