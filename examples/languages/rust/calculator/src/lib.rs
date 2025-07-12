use wasm_bindgen::prelude::*;

// Import the `console.log` function from the `console` object
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro to easily call console.log
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

/// Add two numbers together
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    console_log!("Adding {} + {} = {}", a, b, a + b);
    a + b
}

/// Subtract two numbers
#[wasm_bindgen]
pub fn subtract(a: i32, b: i32) -> i32 {
    console_log!("Subtracting {} - {} = {}", a, b, a - b);
    a - b
}

/// Multiply two numbers
#[wasm_bindgen]
pub fn multiply(a: i32, b: i32) -> i32 {
    console_log!("Multiplying {} * {} = {}", a, b, a * b);
    a * b
}

/// Divide two numbers (returns f64 for precision)
#[wasm_bindgen]
pub fn divide(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        console_log!("Error: Division by zero!");
        f64::NAN
    } else {
        let result = a / b;
        console_log!("Dividing {} / {} = {}", a, b, result);
        result
    }
}

/// Calculate factorial (demonstrates recursion in WASM)
#[wasm_bindgen]
pub fn factorial(n: u32) -> u64 {
    if n <= 1 {
        1
    } else {
        (n as u64) * factorial(n - 1)
    }
}

/// A more complex function that processes an array of numbers
#[wasm_bindgen]
pub fn sum_array(numbers: &[i32]) -> i32 {
    let sum = numbers.iter().sum();
    console_log!("Sum of array with {} elements: {}", numbers.len(), sum);
    sum
}

/// Demonstrates string processing
#[wasm_bindgen]
pub fn reverse_string(input: &str) -> String {
    let reversed: String = input.chars().rev().collect();
    console_log!("Reversed '{}' to '{}'", input, reversed);
    reversed
}

/// Entry point that will be called when the module is initialized
#[wasm_bindgen(start)]
pub fn main() {
    console_log!("Calculator WASM module initialized!");
}
