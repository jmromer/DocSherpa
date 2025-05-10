/**
* """Mock docstring for function calculateSum.
* Parameters: a, b"""
*/
function calculateSum(a, b) {
    // This function adds two numbers and returns the result
    return a + b;
}

class Calculator {
    constructor(initialValue = 0) {
        this.value = initialValue;
    }
    
    add(x) {
        this.value += x;
        return this.value;
    }
    
    subtract(x) {
        this.value -= x;
        return this.value;
    }
}

/**
 * This is an existing docstring that might be outdated
 */
/**
* """Mock docstring for function complexFunction.
* Parameters: data, filterValue = null, transform = true"""
*/
/**
* """Mock docstring for class Calculator."""
*/
    /**
    * """Mock docstring for method add.
    * Parameters: x"""
    */
    /**
    * """Mock docstring for method subtract.
    * Parameters: x"""
    */
function complexFunction(data, filterValue = null, transform = true) {
    const result = [];
    for (const item of data) {
        if (filterValue === null || item > filterValue) {
            const processedItem = transform ? item * 2 : item;
            result.push(processedItem);
        }
    }
    return result;
}
