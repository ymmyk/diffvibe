#!/usr/bin/env python3
"""A simple calculator module."""

def add(a, b):
    """Add two numbers."""
    return a + b

def subtract(a, b):
    """Subtract b from a."""
    return a - b

def multiply(a, b):
    """Multiply two numbers."""
    return a * b

def divide(a, b):
    """Divide a by b."""
    if b == 0:
        raise ValueError("Cannot divide by zero")
    return a / b

class Calculator:
    """A simple calculator class."""

    def __init__(self):
        self.history = []

    def calculate(self, operation, a, b):
        """Perform a calculation and store in history."""
        if operation == "add":
            result = add(a, b)
        elif operation == "subtract":
            result = subtract(a, b)
        elif operation == "multiply":
            result = multiply(a, b)
        elif operation == "divide":
            result = divide(a, b)
        else:
            raise ValueError(f"Unknown operation: {operation}")

        self.history.append((operation, a, b, result))
        return result

    def get_history(self):
        """Return calculation history."""
        return self.history

if __name__ == "__main__":
    calc = Calculator()
    print(calc.calculate("add", 5, 3))
    print(calc.calculate("multiply", 4, 7))
    print(calc.get_history())
