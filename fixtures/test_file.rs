fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}

struct Calculator {
    value: i32,
}

impl Calculator {
    fn new(initial_value: i32) -> Self {
        Calculator { value: initial_value }
    }
    
    fn add(&mut self, x: i32) -> i32 {
        self.value += x;
        self.value
    }
    
    fn subtract(&mut self, x: i32) -> i32 {
        self.value -= x;
        self.value
    }
}

/// This is an existing docstring that might be outdated
fn complex_function(data: Vec<i32>, filter_value: Option<i32>, transform: bool) -> Vec<i32> {
    let mut result = Vec::new();
    for item in data {
        if filter_value.is_none() || item > filter_value.unwrap() {
            let processed_item = if transform { item * 2 } else { item };
            result.push(processed_item);
        }
    }
    result
}