#!/usr/bin/env python3
"""A simple calculator module with extended functionality."""

from typing import List, Tuple
from dataclasses import dataclass

def add(a: float, b: float) -> float:
    """Add two numbers."""
    return a + b

def subtract(a: float, b: float) -> float:
    """Subtract b from a."""
    return a - b

def multiply(a: float, b: float) -> float:
    """Multiply two numbers."""
    return a * b

def divide(a: float, b: float) -> float:
    """Divide a by b."""
    if b == 0:
        raise ZeroDivisionError("Cannot divide by zero")
    return a / b

def power(a: float, b: float) -> float:
    """Raise a to the power of b."""
    return a ** b

def modulo(a: float, b: float) -> float:
    """Return a modulo b."""
    if b == 0:
        raise ZeroDivisionError("Cannot modulo by zero")
    return a % b

@dataclass
class HistoryEntry:
    """Represents a single calculation in history."""
    operation: str
    a: float
    b: float
    result: float

class Calculator:
    """A simple calculator class with extended operations."""

    OPERATIONS = {
        "add": add,
        "subtract": subtract,
        "multiply": multiply,
        "divide": divide,
        "power": power,
        "modulo": modulo,
    }

    def __init__(self):
        self.history: List[HistoryEntry] = []

    def calculate(self, operation: str, a: float, b: float) -> float:
        """Perform a calculation and store in history."""
        if operation not in self.OPERATIONS:
            raise ValueError(f"Unknown operation: {operation}")

        func = self.OPERATIONS[operation]
        result = func(a, b)

        self.history.append(HistoryEntry(operation, a, b, result))
        return result

    def get_history(self) -> List[HistoryEntry]:
        """Return calculation history."""
        return self.history

    def clear_history(self) -> None:
        """Clear the calculation history."""
        self.history = []

    def undo(self) -> HistoryEntry | None:
        """Remove and return the last calculation."""
        if self.history:
            return self.history.pop()
        return None

if __name__ == "__main__":
    calc = Calculator()
    print(calc.calculate("add", 5, 3))
    print(calc.calculate("multiply", 4, 7))
    print(calc.calculate("power", 2, 10))
    print(calc.get_history())
