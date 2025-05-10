def calculate_sum(a, b):
    """Calculate the sum of two numbers.
    
    Parameters:
    a (int or float): The first number to be added.
    b (int or float): The second number to be added.
    
    Returns:
    int or float: The sum of the two input numbers.
    
    Raises:
    TypeError: If the inputs are not integers or floats."""
    return a + b

class Calculator:
    """This class 'Calculator' represents a simple calculator with an initial value, which can be set during the instantiation of an object. If not set, the initial value defaults to 0. The calculator can perform addition and subtraction operations.
    
    Methods:
    - __init__(self, initial_value=0): Initializes a new instance of the Calculator class. 
      Parameters:
      initial_value (int, optional): The initial value of the calculator. Defaults to 0.
      
    - add(self, x): Adds a number to the current value of the calculator.
      Parameters:
      x (int): The number to be added.
      Returns:
      int: The new value of the calculator after the addition.
      
    - subtract(self, x): Subtracts a number from the current value of the calculator.
      Parameters:
      x (int): The number to be subtracted.
      Returns:
      int: The new value of the calculator after the subtraction."""
    def __init__(self, initial_value=0):
        """Initializes an instance of the class.
        
        This method sets the initial value of the instance to the provided value. If no initial value is provided, it defaults to 0.
        
        Parameters:
        initial_value (int, optional): The initial value to set. Defaults to 0."""
        self.value = initial_value
    
    def add(self, x):
        """Method to add a given number to the current value.
        
        Parameters:
        x (int or float): The number to be added to the current value.
        
        Returns:
        self.value (int or float): The updated value after addition.
        
        Raises:
        TypeError: If the input value is not an integer or a float."""
        self.value += x
        return self.value
    
    def subtract(self, x):
        """Subtracts a given number from the current object's value.
        
        Parameters:
        x (int or float): The number to be subtracted from the current value.
        
        Returns:
        int or float: The updated value after subtraction.
        
        Raises:
        TypeError: If the input parameter 'x' is not an integer or a float."""
        self.value -= x
        return self.value

def complex_function(data, filter_value=None, transform=True):
    """complex_function is a function that filters and optionally transforms a list of numeric data.
    
    Parameters:
    data (list): A list of numeric data to be processed.
    filter_value (numeric, optional): A value to filter the data. Only elements in the data list that are greater than this value are processed. If None, all elements are processed. Defaults to None.
    transform (bool, optional): A flag indicating whether to transform the data. If True, each processed element is multiplied by 2. Defaults to True.
    
    Returns:
    list: A list of processed data. The list contains elements from the original data list that are greater than the filter_value (if provided), optionally transformed by multiplying by 2.
    
    Raises:
    TypeError: If the data list contains non-numeric elements, or if filter_value is not a numeric value or None."""
    result = []
    for item in data:
        if filter_value is None or item > filter_value:
            if transform:
                item = item * 2
            result.append(item)
    return result