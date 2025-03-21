use ark_ff::PrimeField;
use ark_bn254::Fr;
use std::marker::PhantomData;

#[derive(Debug)]
pub enum CIRCUIT_OP{
  ADD,
  MUL
}

#[derive(Debug, Clone)]
pub enum Gate {
    Add(usize, usize), // Indexes of the values to add
    Mul(usize, usize), // Indexes of the values to multiply
}

#[derive(Debug, Clone)]
pub struct Circuit<F: PrimeField> {
  layers: Vec<Vec<Gate>>, // Each layer contains a list of gates
    _marker: PhantomData<F>,
}

impl<F: PrimeField> Circuit<F> {
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            _marker: PhantomData,
        }
    }

   pub fn add_layer(&mut self, layer: Vec<Gate>) {
        self.layers.push(layer);
    }

    pub fn evaluate(&self, inputs: Vec<F>) -> Vec<Vec<F>> {
        let mut evaluation_steps = vec![inputs.clone()];
        let mut all_values = inputs; // Contains all values: inputs + intermediate results

        for layer in &self.layers {
            let mut new_values = Vec::with_capacity(layer.len());
            
            for gate in layer {
                let result = match gate {
                    Gate::Add(a, b) => all_values[*a] + all_values[*b],
                    Gate::Mul(a, b) => all_values[*a] * all_values[*b],
                };
                new_values.push(result);
            }
            
            evaluation_steps.push(new_values.clone());
            all_values.extend(new_values); // Add new results to all_values
        }

        evaluation_steps
    }

    pub fn get_layer_evaluation(&self, inputs: Vec<F>, layer_index: usize) -> Option<Vec<F>> {
        let evaluation_steps = self.evaluate(inputs);
        if layer_index < evaluation_steps.len() {
            Some(evaluation_steps[layer_index].clone())
        } else {
            None
        }
    }

    pub fn addi(&self, layer_index: usize, all_values: &Vec<F>) -> Option<Vec<F>> {
        if layer_index >= self.layers.len() {
            return None;
        }
    
        let mut results = Vec::new();
        for gate in &self.layers[layer_index] {
            if let Gate::Add(a, b) = gate {
                // Check if indices are within bounds
                if *a >= all_values.len() || *b >= all_values.len() {
                    return None; // Return None if indices are out of bounds
                }
                results.push(all_values[*a] + all_values[*b]);
            }
        }
        Some(results)
    }

    pub fn muli(&self, layer_index: usize, all_values: &Vec<F>) -> Option<Vec<F>> {
        if layer_index >= self.layers.len() {
            return None;
        }
    
        let mut results = Vec::new();
        for gate in &self.layers[layer_index] {
            if let Gate::Mul(a, b) = gate {
                // Check if indices are within bounds
                if *a >= all_values.len() || *b >= all_values.len() {
                    return None; // Return None if indices are out of bounds
                }
                results.push(all_values[*a] * all_values[*b]);
            }
        }
    
        // Return None if there are no Mul gates in the layer
        if results.is_empty() {
            None
        } else {
            Some(results)
        }
    }


}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_evaluation() {
        let input1 = Fr::from(1);
        let input2 = Fr::from(2);
        let input3 = Fr::from(3);
        let input4 = Fr::from(4);

        let mut circuit = Circuit::new();
        // Layer 1: [1,2,3,4] -> [1+2=3, 3*4=12]
        circuit.add_layer(vec![Gate::Add(0, 1), Gate::Mul(2, 3)]);
        // Layer 2: Available values [1,2,3,4,3,12] -> [3+12=15]
        circuit.add_layer(vec![Gate::Add(4, 5)]);

        let evaluation = circuit.evaluate(vec![input1, input2, input3, input4]);

        assert_eq!(evaluation.len(), 3);
        assert_eq!(evaluation[1], vec![input1 + input2, input3 * input4]);
        assert_eq!(evaluation[2], vec![evaluation[1][0] + evaluation[1][1]]);

        for (step, values) in evaluation.iter().enumerate() {
            println!("Step {}: {:?}", step, values);
        }
    }

    #[test]
    fn test_circuit_evaluation_with_more_inputs() {
        let input1 = Fr::from(1);
        let input2 = Fr::from(2);
        let input3 = Fr::from(3);
        let input4 = Fr::from(4);
        let input5 = Fr::from(5);
        let input6 = Fr::from(6);
        let input7 = Fr::from(7);
        let input8 = Fr::from(8);

        let mut circuit = Circuit::new();
        // Layer 1: [1,2,3,4,5,6,7,8] -> [1+2=3, 3*4=12]
        circuit.add_layer(vec![Gate::Add(0, 1), Gate::Mul(2, 3)]);
        // Layer 2: Available values [1,2,3,4,5,6,7,8,3,12] -> [5+6=11, 7*8=56]
        circuit.add_layer(vec![Gate::Add(4, 5), Gate::Mul(6, 7)]);
        // Layer 3: Available values [1,2,3,4,5,6,7,8,3,12,11,56] -> [3+12=15, 11*56=616]
        circuit.add_layer(vec![Gate::Add(8, 9), Gate::Mul(10, 11)]);
        // Layer 4: Available values [1,2,3,4,5,6,7,8,3,12,11,56,15,616] -> [15+616=631]
        circuit.add_layer(vec![Gate::Add(12, 13)]);

        let evaluation = circuit.evaluate(vec![
            input1, input2, input3, input4,
            input5, input6, input7, input8
        ]);

        assert_eq!(evaluation.len(), 5);
        
        // First layer results
        let expected_layer1 = vec![input1 + input2, input3 * input4];
        assert_eq!(evaluation[1], expected_layer1);
        
        // Second layer results
        let expected_layer2 = vec![input5 + input6, input7 * input8];
        assert_eq!(evaluation[2], expected_layer2);
        
        // Third layer results
        let expected_layer3 = vec![expected_layer1[0] + expected_layer1[1], 
                                 expected_layer2[0] * expected_layer2[1]];
        assert_eq!(evaluation[3], expected_layer3);

        // Fourth layer results
        let expected_layer4 = vec![expected_layer3[0] + expected_layer3[1]];
        assert_eq!(evaluation[4], expected_layer4);

        for (step, values) in evaluation.iter().enumerate() {
            println!("Step {}: {:?}", step, values);
        }
    }

    #[test]
    fn test_circuit_evaluation_one() {
        let input1 = Fr::from(1);
        let input2 = Fr::from(2);
        let input3 = Fr::from(3);
        let input4 = Fr::from(4);
        let input5 = Fr::from(5);
        let input6 = Fr::from(6);
        let input7 = Fr::from(7);
        let input8 = Fr::from(8);

        let mut circuit = Circuit::new();
        // Layer 1: [1,2,3,4,5,6,7,8] -> [1+2=3, 3*4=12]
        circuit.add_layer(vec![Gate::Add(0, 1), Gate::Mul(2, 3)]);
        // Layer 2: Available values [1,2,3,4,5,6,7,8,3,12] -> [5*6=30, 7*8=56]
        circuit.add_layer(vec![Gate::Mul(4, 5), Gate::Mul(6, 7)]);
        // Layer 3: Available values [1,2,3,4,5,6,7,8,3,12,11,56] -> [3+12=15, 30*56=1680]
        circuit.add_layer(vec![Gate::Add(8, 9), Gate::Mul(10, 11)]);
        // Layer 4: Available values [1,2,3,4,5,6,7,8,3,12,11,56,15,616] -> [15+1680=1695]
        circuit.add_layer(vec![Gate::Add(12, 13)]);

        let evaluation = circuit.evaluate(vec![
            input1, input2, input3, input4,
            input5, input6, input7, input8
        ]);

        assert_eq!(evaluation.len(), 5);
        
        // First layer results
        let expected_layer1 = vec![input1 + input2, input3 * input4];
        assert_eq!(evaluation[1], expected_layer1);
        
        // Second layer results
        let expected_layer2 = vec![input5 * input6, input7 * input8];
        assert_eq!(evaluation[2], expected_layer2);
        
        // Third layer results
        let expected_layer3 = vec![expected_layer1[0] + expected_layer1[1], 
                                 expected_layer2[0] * expected_layer2[1]];
        assert_eq!(evaluation[3], expected_layer3);

        // Fourth layer results
        let expected_layer4 = vec![expected_layer3[0] + expected_layer3[1]];
        assert_eq!(evaluation[4], expected_layer4);

        for (step, values) in evaluation.iter().enumerate() {
            println!("Step {}: {:?}", step, values);
        }
    }

    #[test]
    fn test_get_layer_evaluation() {
        let input1 = Fr::from(1);
        let input2 = Fr::from(2);
        let input3 = Fr::from(3);
        let input4 = Fr::from(4);

        let mut circuit = Circuit::new();
        // Layer 1: [1,2,3,4] -> [1+2=3, 3*4=12]
        circuit.add_layer(vec![Gate::Add(0, 1), Gate::Mul(2, 3)]);
        // Layer 2: Available values [1,2,3,4,3,12] -> [3+12=15]
        circuit.add_layer(vec![Gate::Add(4, 5)]);

        let inputs = vec![input1, input2, input3, input4];

        // Test layer 0 (inputs)
        let layer_0_eval = circuit.get_layer_evaluation(inputs.clone(), 0);
        assert_eq!(layer_0_eval, Some(vec![input1, input2, input3, input4]));

        // Test layer 1
        let layer_1_eval = circuit.get_layer_evaluation(inputs.clone(), 1);
        assert_eq!(layer_1_eval, Some(vec![input1 + input2, input3 * input4]));

        // Test layer 2
        let layer_2_eval = circuit.get_layer_evaluation(inputs.clone(), 2);
        assert_eq!(layer_2_eval, Some(vec![input1 + input2 + input3 * input4]));

        // Test out of bounds layer
        let layer_3_eval = circuit.get_layer_evaluation(inputs, 3);
        assert_eq!(layer_3_eval, None);
    }

    #[test]
    fn test_get_layer_evaluation_with_more_inputs() {
        let input1 = Fr::from(1);
        let input2 = Fr::from(2);
        let input3 = Fr::from(3);
        let input4 = Fr::from(4);
        let input5 = Fr::from(5);
        let input6 = Fr::from(6);
        let input7 = Fr::from(7);
        let input8 = Fr::from(8);

        let mut circuit = Circuit::new();
        // Layer 1: [1,2,3,4,5,6,7,8] -> [1+2=3, 3*4=12]
        circuit.add_layer(vec![Gate::Add(0, 1), Gate::Mul(2, 3)]);
        // Layer 2: Available values [1,2,3,4,5,6,7,8,3,12] -> [5+6=11, 7*8=56]
        circuit.add_layer(vec![Gate::Add(4, 5), Gate::Mul(6, 7)]);
        // Layer 3: Available values [1,2,3,4,5,6,7,8,3,12,11,56] -> [3+12=15, 11*56=616]
        circuit.add_layer(vec![Gate::Add(8, 9), Gate::Mul(10, 11)]);
        // Layer 4: Available values [1,2,3,4,5,6,7,8,3,12,11,56,15,616] -> [15+616=631]
        circuit.add_layer(vec![Gate::Add(12, 13)]);

        let inputs = vec![
            input1, input2, input3, input4,
            input5, input6, input7, input8
        ];

        // Test layer 0 (inputs)
        let layer_0_eval = circuit.get_layer_evaluation(inputs.clone(), 0);
        assert_eq!(layer_0_eval, Some(vec![input1, input2, input3, input4, input5, input6, input7, input8]));

        // Test layer 1
        let layer_1_eval = circuit.get_layer_evaluation(inputs.clone(), 1);
        assert_eq!(layer_1_eval, Some(vec![input1 + input2, input3 * input4]));

        // Test layer 2
        let layer_2_eval = circuit.get_layer_evaluation(inputs.clone(), 2);
        assert_eq!(layer_2_eval, Some(vec![input5 + input6, input7 * input8]));

        // Test layer 3
        let layer_3_eval = circuit.get_layer_evaluation(inputs.clone(), 3);
        assert_eq!(layer_3_eval, Some(vec![input1 + input2 + input3 * input4, (input5 + input6) * (input7 * input8)]));

        // Test layer 4
        let layer_4_eval = circuit.get_layer_evaluation(inputs.clone(), 4);
        assert_eq!(layer_4_eval, Some(vec![input1 + input2 + input3 * input4 + (input5 + input6) * (input7 * input8)]));

        // Test out of bounds layer
        let layer_5_eval = circuit.get_layer_evaluation(inputs, 5);
        assert_eq!(layer_5_eval, None);
    }


    #[test]
    fn test_addi() {
        // Define a simple circuit with one layer and one Add gate
        let circuit = Circuit {
            layers: vec![vec![Gate::Add(0, 1)]],
            _marker: PhantomData,
        };

        // Define input values
        let all_values = vec![Fr::from(2), Fr::from(3)];

        // Test the addi function
        let result = circuit.addi(0, &all_values);
        assert_eq!(result, Some(vec![Fr::from(5)])); // 2 + 3 = 5

        // Test out-of-bounds indices
        let invalid_circuit = Circuit {
            layers: vec![vec![Gate::Add(2, 3)]], // Indices 2 and 3 are out of bounds
            _marker: PhantomData,
        };
        let invalid_result = invalid_circuit.addi(0, &all_values);
        assert_eq!(invalid_result, None); // Should return None for out-of-bounds indices
    }

    #[test]
    fn test_addi_with_more_inputs() {
        let input1 = Fr::from(1);
        let input2 = Fr::from(2);
        let input3 = Fr::from(3);
        let input4 = Fr::from(4);

        let circuit = Circuit {
            layers: vec![vec![Gate::Add(0, 1), Gate::Add(2, 3)]],
            _marker: PhantomData,
        };

        let all_values = vec![input1, input2, input3, input4];

        let result = circuit.addi(0, &all_values);
        assert_eq!(result, Some(vec![input1 + input2, input3 + input4])); // [1+2=3, 3+4=7]

        // Test out-of-bounds indices
        let invalid_circuit = Circuit {
            layers: vec![vec![Gate::Add(4, 5)]], // Indices 4 and 5 are out of bounds
            _marker: PhantomData,
        };
        let invalid_result = invalid_circuit.addi(0, &all_values);
        assert_eq!(invalid_result, None); // Should return None for out-of-bounds indices
    }

    #[test]
    fn test_muli() {
        let input1 = Fr::from(1);
        let input2 = Fr::from(2);
        let input3 = Fr::from(3);
        let input4 = Fr::from(4);

        let mut circuit = Circuit::new();
        // Layer 1: [1,2,3,4] -> [1+2=3, 3*4=12]
        circuit.add_layer(vec![Gate::Add(0, 1), Gate::Mul(2, 3)]);
        // Layer 2: Available values [1,2,3,4,3,12] -> [3+12=15]
        circuit.add_layer(vec![Gate::Add(4, 5)]);

        let inputs = vec![input1, input2, input3, input4];
        let evaluation = circuit.evaluate(inputs.clone());

        // Test muli for layer 1
        let muli_layer_1 = circuit.muli(0, &inputs);
        assert_eq!(muli_layer_1, Some(vec![input3 * input4]));

        // Test muli for out of bounds layer
        let muli_layer_2 = circuit.muli(1, &evaluation[1]);
        assert_eq!(muli_layer_2, None);
    }
}