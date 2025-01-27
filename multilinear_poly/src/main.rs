struct MultiPoly {
    coefficients: Vec<f64>,
    num_variables: usize,
}

impl MultiPoly{
    fn new(num_variables: usize, coefficients: Vec<f64>) -> MultiPoly{
        let n = coefficients.len();
        let len = 2_usize.pow(num_variables as u32);
        if n != len{
            panic!("The number of coefficients must be 2^num_variables");
        }
        MultiPoly{coefficients, num_variables}
    }


    fn reorder_coefficients(&self, swap_index: usize) -> Vec<f64> {
        let mut reordered = vec![0.0; self.coefficients.len()];
        let num_variables = self.num_variables;
    
        for i in 0..self.coefficients.len() {
            // Convert the index to a binary representation
            let mut binary = (0..num_variables)
                .map(|bit| (i >> bit) & 1)
                .collect::<Vec<_>>();
    
            // Swap the bit at `swap_index` with the bit at the last position
            binary.swap(swap_index, num_variables - 1);
    
            // Recompute the new index after the swap
            let new_index = binary
                .iter()
                .enumerate()
                .fold(0, |acc, (bit, &value)| acc | (value << bit));
    
            // Place the coefficient in the reordered position
            reordered[new_index] = self.coefficients[i];
        }
    
        reordered
    }
    

    fn get_points_with_pairings(&self, variable_index: usize) -> Vec<(f64, f64)> {
        if variable_index >= self.num_variables {
            panic!("Variable index out of bounds");
        }
    
        // Reorder coefficients if the variable_index is not the last variable
        let reordered_coefficients = if variable_index != self.num_variables - 1 {
            self.reorder_coefficients(variable_index)
        } else {
            self.coefficients.clone()
        };
    
        let mut points = Vec::new();
        let num_outcomes = 2_usize.pow(self.num_variables as u32);
        let step = 1 << (self.num_variables - 1); // Use the last variable for pairing logic
    
        for i in (0..num_outcomes).step_by(2 * step) {
            for j in 0..step {
                let index1 = i + j;          // Even position
                let index2 = index1 + step; // Odd position
    
                if index1 < reordered_coefficients.len() && index2 < reordered_coefficients.len() {
                    let x = reordered_coefficients[index1];
                    let y = reordered_coefficients[index2];
                    points.push((x, y));
                } else {
                    eprintln!(
                        "Index out of bounds: index1 = {}, index2 = {}, len = {}",
                        index1, index2, reordered_coefficients.len()
                    );
                }
            }
        }
    
        points
    }

    fn get_unique_pairs_coefficients(arr: Vec<f64>, pos: usize) -> Vec<(f64, f64)> {
        let mask = 1 << pos; // Mask for the current bit position
        let mut coefficients = Vec::new(); // To store unique pair coefficients

        for i in 0..arr.len() {
            let pair = i ^ mask; // Calculate the pair index by flipping the bit at `pos`

            // Only process unique pairs (avoid duplicates)
            if i < pair {
                println!(
                    "Unique Pair: (i={}, pair={}) -> Values: ({}, {})",
                    i, pair, arr[i], arr[pair]
                );
                coefficients.push((arr[i], arr[pair])); // Store coefficients as pairs
            }
        }

        coefficients
    }
        
    

    fn partial_eval(&self, variable_index: usize, var: f64) -> Vec<f64> {
        if variable_index >= self.num_variables {
            panic!("Variable index out of bounds");
        }

        let points = Self::get_unique_pairs_coefficients(self.coefficients.clone(),variable_index);
        let mut result = Vec::new();

        for (y1, y2) in points {
            let value = y1 + var * (y2 - y1);
            result.push(value);
        }

        result
    }

    fn full_eval(&self, vars: Vec<f64>) -> f64 {
        if vars.len() != self.num_variables {
            panic!("Number of variables does not match the polynomial");
        }

        let mut current_coefficients = self.coefficients.clone();

        for variable_index in 0..self.num_variables {
            current_coefficients = MultiPoly {
                num_variables: self.num_variables - variable_index,
                coefficients: current_coefficients,
            }
            .partial_eval(0, vars[variable_index]);
        }

        // After evaluating all variables, only one value should remain
        if current_coefficients.len() != 1 {
            panic!("Full evaluation did not result in a single value");
        }

        current_coefficients[0]
    }

}




fn main() {
    // let coefficients1: Vec<f64> = vec![0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 2.0, 5.0];
    // let num_variables1 = 3;
    // let poly1: MultiPoly = MultiPoly::new(num_variables1, coefficients1);
    // let points = poly1.get_points_with_pairings(2);
    // for (x, y) in points{
    //     println!("{} {}", x, y);
    // }

    let coefficients = vec![0.0, 2.0,0.0, 5.0];
    let num_variables = 2;
    let poly: MultiPoly = MultiPoly::new(num_variables, coefficients);

    let points = poly.get_points_with_pairings(1);
    for (x, y) in points{
        println!("{} {}", x, y);
    }
    println!("Partial Eval: {:?}", poly.partial_eval(1, 5.0));
    println!("Full Eval: {:?}", poly.full_eval(vec![2.0, 3.0]));
}
