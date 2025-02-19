use ark_ff::PrimeField;
use ark_bn254::Fr;
use std::marker::PhantomData;
use multilinear::{self, MultivariatePoly};


#[derive(Debug, Clone)]
enum Gate {
    Add(usize, usize, usize), // Addition gate with indices
    Mul(usize, usize, usize), // Multiplication gate with indices
}

// Add this enum to specify gate type
#[derive(Debug, Clone, Copy)]
enum GateType {
    Addition,
    Multiplication,
}

#[derive(Debug, Clone)]
struct Circuit<F: PrimeField> {
    layers: Vec<Vec<Gate>>,
    _marker: PhantomData<F>,
}

impl<F: PrimeField> Circuit<F> {
    fn new() -> Self {
        Self {
            layers: Vec::new(),
            _marker: PhantomData,
        }
    }

    fn add_layer(&mut self, layer: Vec<Gate>) {
        self.layers.push(layer);
    }

    fn evaluate(&self, inputs: Vec<F>) -> Vec<Vec<F>> {
        let mut evaluation_steps = vec![inputs.clone()];
        let mut all_values = inputs; // Contains all values: inputs + intermediate results

        for layer in &self.layers {
            let mut new_values = Vec::with_capacity(layer.len());
            
            for gate in layer {
                let result = match gate {
                    Gate::Add(a, b, _) => all_values[*a] + all_values[*b],
                    Gate::Mul(a, b, _) => all_values[*a] * all_values[*b],
                };
                new_values.push(result);
            }
            
            evaluation_steps.push(new_values.clone());
            all_values.extend(new_values); // Add new results to all_values
        }

        evaluation_steps
    }

    fn get_layer_evaluation(&self, inputs: Vec<F>, layer_index: usize) -> Option<Vec<F>> {
        let evaluation_steps = self.evaluate(inputs);
        if layer_index < evaluation_steps.len() {
            Some(evaluation_steps[layer_index].clone())
        } else {
            None
        }
    }

    fn get_max_index_in_layer(&self, layer_index: usize) -> usize {
        let mut max_index = 0;
        for gate in &self.layers[layer_index] {
            match gate {
                Gate::Add(left, right, output) | Gate::Mul(left, right, output) => {
                    max_index = max_index.max(*left).max(*right).max(*output);
                }
            }
        }
        max_index
    }

    fn num_of_layer_variables(max_index: usize) -> usize {
        let bits_needed = (max_index + 1).next_power_of_two().trailing_zeros() as usize;
        3 * bits_needed
    }

    fn convert_to_binary_and_to_decimal(
        max_index: usize,
        output_index: usize,
        left_index: usize,
        right_index: usize,
    ) -> usize {
        let bits_per_index = (max_index + 1).next_power_of_two().trailing_zeros() as usize;
        let mask = (1 << bits_per_index) - 1;
        
        assert!(left_index <= max_index && right_index <= max_index && output_index <= max_index,
            "Indices must not exceed max_index");
            
        (output_index & mask) << (2 * bits_per_index) |
        (left_index & mask) << bits_per_index |
        (right_index & mask)
    }

    fn addi(&self, layer_index: usize) -> MultivariatePoly<F> {
        let max_index = self.get_max_index_in_layer(layer_index);
        let num_variables = Self::num_of_layer_variables(max_index);
        let boolean_hypercube_combinations = 1 << num_variables;
        let mut add_i_values = vec![F::zero(); boolean_hypercube_combinations];

        println!("Layer {}: max_index = {}, num_variables = {}, combinations = {}", 
            layer_index, max_index, num_variables, boolean_hypercube_combinations);

        for gate in &self.layers[layer_index] {
            if let Gate::Add(left, right, output) = gate {
                let position_index = Self::convert_to_binary_and_to_decimal(
                    max_index,
                    *output,
                    *left,
                    *right,
                );
                println!("Gate Add({}, {}, {}) -> position_index = {}", 
                    left, right, output, position_index);
                add_i_values[position_index] = F::one();
            }
        }

        MultivariatePoly::new(add_i_values, num_variables)
    }

    fn muli(&self, layer_index: usize) -> MultivariatePoly<F> {
        let max_index = self.get_max_index_in_layer(layer_index);
        let num_variables = Self::num_of_layer_variables(max_index);
        let boolean_hypercube_combinations = 1 << num_variables;
        let mut mul_i_values = vec![F::zero(); boolean_hypercube_combinations];

        for gate in &self.layers[layer_index] {
            if let Gate::Mul(left, right, output) = gate {
                let position_index = Self::convert_to_binary_and_to_decimal(
                    max_index,
                    *output,
                    *left,
                    *right,
                );
                mul_i_values[position_index] = F::one();
            }
        }

        MultivariatePoly::new(mul_i_values, num_variables)
    }

    // Add this helper function to create expected polynomial
    fn create_expected_poly(&self, layer_index: usize, gate_type: GateType) -> MultivariatePoly<F> {
        let max_index = self.get_max_index_in_layer(layer_index);
        let num_vars = Self::num_of_layer_variables(max_index);
        let mut expected_values = vec![F::zero(); 1 << num_vars];
        
        for gate in &self.layers[layer_index] {
            match (gate, gate_type) {
                (Gate::Add(left, right, output), GateType::Addition) |
                (Gate::Mul(left, right, output), GateType::Multiplication) => {
                    let position_index = Self::convert_to_binary_and_to_decimal(
                        max_index,
                        *output,
                        *left,
                        *right
                    );
                    expected_values[position_index] = F::from(1u64);
                }
                _ => continue,
            }
        }
        
        MultivariatePoly::new(expected_values, num_vars)
    }
}

fn main() {
    let mut circuit = Circuit::<Fr>::new();
    circuit.add_layer(vec![Gate::Add(0, 1, 2), Gate::Add(2, 3, 3)]);
    
    let add_poly = circuit.addi(0);
    let expected_poly = circuit.create_expected_poly(0, GateType::Addition);
    
    println!("Generated Addition Polynomial: {:?}", add_poly);
    println!("Expected Addition Polynomial: {:?}", expected_poly);
    assert_eq!(add_poly, expected_poly);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_addi_small_circuit() {
        let mut circuit = Circuit::<Fr>::new();
        circuit.add_layer(vec![Gate::Add(0, 1, 2), Gate::Add(2, 3, 3)]);
        
        let add_poly = circuit.addi(0);
        let expected_poly = circuit.create_expected_poly(0, GateType::Addition);
        
        println!("Generated Addition Polynomial: {:?}", add_poly);
        println!("Expected Addition Polynomial: {:?}", expected_poly);
        assert_eq!(add_poly, expected_poly);
    }

    #[test]
    fn test_addi_larger_circuit() {
        let mut circuit = Circuit::<Fr>::new();
        circuit.add_layer(vec![Gate::Add(0, 1, 2), Gate::Add(4, 5, 6)]);
        
        let add_poly = circuit.addi(0);
        let expected_poly = circuit.create_expected_poly(0, GateType::Addition);
        
        assert_eq!(add_poly, expected_poly);
    }

    // Similar updates for multiplication tests
    #[test]
    fn test_muli_small_circuit() {
        let mut circuit = Circuit::<Fr>::new();
        circuit.add_layer(vec![Gate::Mul(0, 1, 2), Gate::Mul(2, 3, 3)]);
        
        let mul_poly = circuit.muli(0);
        let expected_poly = circuit.create_expected_poly(0, GateType::Multiplication);
        
        assert_eq!(mul_poly, expected_poly);
    }


    // #[test]
    // fn test_muli_small_circuit() {
    //     let mut circuit = Circuit::<Fr>::new();
    //     circuit.add_layer(vec![Gate::Mul(0, 1, 2), Gate::Mul(2, 3, 3)]);
        
    //     let mul_poly = circuit.muli(0);
        
    //     // Max index is 3, needs 2 bits per index, total 6 variables
    //     let max_index = 3;
    //     let num_vars = Circuit::<Fr>::num_of_layer_variables(max_index);
    //     let mut expected_values = vec![Fr::from(0u64); 1 << num_vars];
        
    //     let pos1 = Circuit::<Fr>::convert_to_binary_and_to_decimal(max_index, 2, 0, 1);
    //     let pos2 = Circuit::<Fr>::convert_to_binary_and_to_decimal(max_index, 3, 2, 3);
    //     expected_values[pos1] = Fr::from(1u64);
    //     expected_values[pos2] = Fr::from(1u64);
        
    //     let expected_poly = MultivariatePoly::new(expected_values, num_vars);
    //     println!("Generated Multiplication Polynomial: {:?}", mul_poly);
    //     println!("Expected Multiplication Polynomial: {:?}", expected_poly);
    //     assert_eq!(mul_poly, expected_poly);
    // }

    // #[test]
    // fn test_muli_larger_circuit() {
    //     let mut circuit = Circuit::<Fr>::new();
    //     circuit.add_layer(vec![Gate::Mul(0, 1, 2), Gate::Mul(4, 5, 6)]);
        
    //     let mul_poly = circuit.muli(0);
        
    //     // Max index is 6, needs 3 bits per index, total 9 variables
    //     let max_index = 6;
    //     let num_vars = Circuit::<Fr>::num_of_layer_variables(max_index);
    //     let mut expected_values = vec![Fr::from(0u64); 1 << num_vars];
        
    //     let pos1 = Circuit::<Fr>::convert_to_binary_and_to_decimal(max_index, 2, 0, 1);
    //     let pos2 = Circuit::<Fr>::convert_to_binary_and_to_decimal(max_index, 6, 4, 5);
    //     expected_values[pos1] = Fr::from(1u64);
    //     expected_values[pos2] = Fr::from(1u64);
        
    //     let expected_poly = MultivariatePoly::new(expected_values, num_vars);
    //     println!("Generated Multiplication Polynomial: {:?}", mul_poly);
    //     println!("Expected Multiplication Polynomial: {:?}", expected_poly);
    //     assert_eq!(mul_poly, expected_poly);
    // }

}