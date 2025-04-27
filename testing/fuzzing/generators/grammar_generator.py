#!/usr/bin/env python3
"""
Grammar Generator for Anarchy Inference Fuzzing.

This module provides functionality for generating inputs based on the Anarchy Inference grammar
for fuzzing the Anarchy Inference language implementation.
"""

import os
import sys
import time
import random
import string
import logging
from typing import Dict, List, Any, Optional, Tuple, Callable
from dataclasses import dataclass

# Add parent directory to path to import fuzzing framework
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# Import from fuzzing framework
from fuzzing.fuzzing_framework import TestCase, GeneratorType

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger("grammar_generator")

class GrammarGenerator:
    """Generates inputs based on the Anarchy Inference grammar for fuzzing."""
    
    def __init__(self, config: Optional[Dict[str, Any]] = None):
        """Initialize the grammar generator.
        
        Args:
            config: Optional configuration for the generator
        """
        self.config = config or {}
        
        # Default configuration values
        self.max_depth = self.config.get("max_depth", 5)
        self.max_statements = self.config.get("max_statements", 20)
        self.error_probability = self.config.get("error_probability", 0.1)
        self.include_comments = self.config.get("include_comments", True)
        
        # Grammar rules
        self.grammar = self._initialize_grammar()
        
        # Variable tracking
        self.variables = set()
        
        # Function tracking
        self.functions = set()
    
    def generate(self, parent: Optional[TestCase] = None) -> TestCase:
        """Generate a test case based on grammar.
        
        Args:
            parent: Optional parent test case (ignored for grammar generation)
            
        Returns:
            Generated test case
        """
        # Generate a random ID
        test_id = f"grammar_{int(time.time())}_{random.randint(0, 1000000)}"
        
        # Reset state
        self.variables = set()
        self.functions = set()
        
        # Generate content based on grammar
        content = self._generate_program()
        
        # Create metadata
        metadata = {
            "generator": "grammar",
            "max_depth": self.max_depth,
            "max_statements": self.max_statements,
            "error_probability": self.error_probability,
            "generation_time": time.time()
        }
        
        # Create test case
        return TestCase(
            id=test_id,
            content=content,
            generator_type=GeneratorType.GRAMMAR,
            metadata=metadata,
            parent_id=parent.id if parent else None
        )
    
    def _initialize_grammar(self) -> Dict[str, Callable]:
        """Initialize the grammar rules.
        
        Returns:
            Dictionary of grammar production rules
        """
        grammar = {
            "program": self._generate_program,
            "statement": self._generate_statement,
            "expression": self._generate_expression,
            "variable_declaration": self._generate_variable_declaration,
            "function_declaration": self._generate_function_declaration,
            "assignment": self._generate_assignment,
            "if_statement": self._generate_if_statement,
            "for_loop": self._generate_for_loop,
            "while_loop": self._generate_while_loop,
            "return_statement": self._generate_return_statement,
            "print_statement": self._generate_print_statement,
            "binary_expression": self._generate_binary_expression,
            "unary_expression": self._generate_unary_expression,
            "function_call": self._generate_function_call,
            "variable_reference": self._generate_variable_reference,
            "literal": self._generate_literal,
            "number_literal": self._generate_number_literal,
            "string_literal": self._generate_string_literal,
            "boolean_literal": self._generate_boolean_literal,
            "array_literal": self._generate_array_literal,
            "object_literal": self._generate_object_literal,
            "comment": self._generate_comment
        }
        return grammar
    
    def _generate_program(self) -> str:
        """Generate a complete program.
        
        Returns:
            Generated program
        """
        # Add a comment at the beginning
        lines = []
        if self.include_comments:
            lines.append(self._generate_comment())
        
        # Generate a random number of statements
        num_statements = random.randint(1, self.max_statements)
        
        # Generate statements
        for _ in range(num_statements):
            lines.append(self._generate_statement(0))
        
        # Ensure there's at least one return statement
        if not any("return" in line for line in lines):
            lines.append(self._generate_return_statement(0))
        
        # Join lines
        return '\n'.join(lines)
    
    def _generate_statement(self, depth: int) -> str:
        """Generate a statement.
        
        Args:
            depth: Current recursion depth
            
        Returns:
            Generated statement
        """
        # Limit recursion depth
        if depth >= self.max_depth:
            return self._generate_simple_statement(depth)
        
        # Choose statement type
        statement_types = [
            (self._generate_variable_declaration, 3),
            (self._generate_assignment, 3),
            (self._generate_if_statement, 2),
            (self._generate_for_loop, 1),
            (self._generate_while_loop, 1),
            (self._generate_return_statement, 1),
            (self._generate_print_statement, 2),
            (self._generate_function_declaration, 1),
            (self._generate_expression, 2)
        ]
        
        # Weight the selection based on depth
        if depth > 2:
            # Prefer simpler statements at greater depths
            statement_types = [
                (self._generate_variable_declaration, 1),
                (self._generate_assignment, 2),
                (self._generate_return_statement, 3),
                (self._generate_print_statement, 3),
                (self._generate_expression, 3)
            ]
        
        # Select a statement type based on weights
        total_weight = sum(weight for _, weight in statement_types)
        r = random.uniform(0, total_weight)
        cumulative_weight = 0
        
        for generator_func, weight in statement_types:
            cumulative_weight += weight
            if r <= cumulative_weight:
                # Generate the statement
                statement = generator_func(depth)
                
                # Maybe add a comment
                if self.include_comments and random.random() < 0.1:
                    statement += " " + self._generate_comment()
                
                return statement
        
        # Fallback
        return self._generate_simple_statement(depth)
    
    def _generate_simple_statement(self, depth: int) -> str:
        """Generate a simple statement.
        
        Args:
            depth: Current recursion depth
            
        Returns:
            Generated simple statement
        """
        # Choose a simple statement type
        simple_types = [
            self._generate_variable_declaration,
            self._generate_assignment,
            self._generate_return_statement,
            self._generate_print_statement,
            self._generate_expression
        ]
        
        generator_func = random.choice(simple_types)
        return generator_func(depth)
    
    def _generate_block(self, depth: int) -> str:
        """Generate a block of statements.
        
        Args:
            depth: Current recursion depth
            
        Returns:
            Generated block
        """
        # Limit recursion depth
        if depth >= self.max_depth:
            return "{ " + self._generate_simple_statement(depth) + " }"
        
        # Generate a random number of statements
        num_statements = random.randint(1, max(1, self.max_statements // (depth + 1)))
        
        # Generate statements
        statements = []
        for _ in range(num_statements):
            statements.append(self._generate_statement(depth + 1))
        
        # Join statements
        if statements:
            return "{\n" + "\n".join(f"    {stmt}" for stmt in statements) + "\n}"
        else:
            return "{ }"
    
    def _generate_expression(self, depth: int) -> str:
        """Generate an expression.
        
        Args:
            depth: Current recursion depth
            
        Returns:
            Generated expression
        """
        # Limit recursion depth
        if depth >= self.max_depth:
            return self._generate_simple_expression(depth)
        
        # Choose expression type
        expr_types = [
            (self._generate_binary_expression, 3),
            (self._generate_unary_expression, 2),
            (self._generate_function_call, 2),
            (self._generate_variable_reference, 3),
            (self._generate_literal, 3)
        ]
        
        # Weight the selection based on depth
        if depth > 2:
            # Prefer simpler expressions at greater depths
            expr_types = [
                (self._generate_variable_reference, 3),
                (self._generate_literal, 4)
            ]
        
        # Select an expression type based on weights
        total_weight = sum(weight for _, weight in expr_types)
        r = random.uniform(0, total_weight)
        cumulative_weight = 0
        
        for generator_func, weight in expr_types:
            cumulative_weight += weight
            if r <= cumulative_weight:
                return generator_func(depth)
        
        # Fallback
        return self._generate_simple_expression(depth)
    
    def _generate_simple_expression(self, depth: int) -> str:
        """Generate a simple expression.
        
        Args:
            depth: Current recursion depth
            
        Returns:
            Generated simple expression
        """
        # Choose a simple expression type
        simple_types = [
            self._generate_variable_reference,
            self._generate_literal
        ]
        
        generator_func = random.choice(simple_types)
        return generator_func(depth)
    
    def _generate_variable_declaration(self, depth: int) -> str:
        """Generate a variable declaration.
        
        Args:
            depth: Current recursion depth
            
        Returns:
            Generated variable declaration
        """
        # Generate a variable name
        var_name = self._generate_variable_name()
        
        # Add to variables
        self.variables.add(var_name)
        
        # Generate an expression
        expr = self._generate_expression(depth + 1)
        
        # Generate the declaration
        return f"{var_name} ← {expr}"
    
    def _generate_function_declaration(self, depth: int) -> str:
        """Generate a function declaration.
        
        Args:
            depth: Current recursion depth
            
        Returns:
            Generated function declaration
        """
        # Limit recursion depth
        if depth >= self.max_depth - 1:
            return self._generate_simple_statement(depth)
        
        # Generate a function name
        func_name = self._generate_function_name()
        
        # Add to functions
        self.functions.add(func_name)
        
        # Generate parameters
        num_params = random.randint(0, 3)
        params = []
        
        for _ in range(num_params):
            param_name = self._generate_variable_name()
            params.append(param_name)
            self.variables.add(param_name)
        
        # Generate the function body
        body = self._generate_block(depth + 1)
        
        # Generate the declaration
        return f"λ⟨{func_name}⟩({', '.join(params)}) {body}"
    
    def _generate_assignment(self, depth: int) -> str:
        """Generate an assignment statement.
        
        Args:
            depth: Current recursion depth
            
        Returns:
            Generated assignment statement
        """
        # Generate a variable name
        if self.variables and random.random() < 0.8:
            # Use an existing variable
            var_name = random.choice(list(self.variables))
        else:
            # Create a new variable
            var_name = self._generate_variable_name()
            self.variables.add(var_name)
        
        # Generate an expression
        expr = self._generate_expression(depth + 1)
        
        # Generate the assignment
        return f"{var_name} ← {expr}"
    
    def _generate_if_statement(self, depth: int) -> str:
        """Generate an if statement.
        
        Args:
            depth: Current recursion depth
            
        Returns:
            Generated if statement
        """
        # Generate a condition
        condition = self._generate_expression(depth + 1)
        
        # Generate the then block
        then_block = self._generate_block(depth + 1)
        
        # Maybe generate an else block
        if random.random() < 0.5:
            else_block = self._generate_block(depth + 1)
            return f"if {condition} {then_block} else {else_block}"
        else:
            return f"if {condition} {then_block}"
    
    def _generate_for_loop(self, depth: int) -> str:
        """Generate a for loop.
        
        Args:
            depth: Current recursion depth
            
        Returns:
            Generated for loop
        """
        # Generate a loop variable
        var_name = self._generate_variable_name()
        self.variables.add(var_name)
        
        # Generate a range
        range_value = random.randint(1, 10)
        
        # Generate the loop body
        body = self._generate_block(depth + 1)
        
        # Generate the for loop
        return f"for {var_name} in range({range_value}) {body}"
    
    def _generate_while_loop(self, depth: int) -> str:
        """Generate a while loop.
        
        Args:
            depth: Current recursion depth
            
        Returns:
            Generated while loop
        """
        # Generate a condition
        condition = self._generate_expression(depth + 1)
        
        # Generate the loop body
        body = self._generate_block(depth + 1)
        
        # Generate the while loop
        return f"while {condition} {body}"
    
    def _generate_return_statement(self, depth: int) -> str:
        """Generate a return statement.
        
        Args:
            depth: Current recursion depth
            
        Returns:
            Generated return statement
        """
        # Generate an expression
        expr = self._generate_expression(depth + 1)
        
        # Generate the return statement
        return f"return {expr}"
    
    def _generate_print_statement(self, depth: int) -> str:
        """Generate a print statement.
        
        Args:
            depth: Current recursion depth
            
        Returns:
            Generated print statement
        """
        # Generate an expression
        expr = self._generate_expression(depth + 1)
        
        # Generate the print statement
        return f"⌽({expr})"
    
    def _generate_binary_expression(self, depth: int) -> str:
        """Generate a binary expression.
        
        Args:
            depth: Current recursion depth
            
        Returns:
            Generated binary expression
        """
        # Limit recursion depth
        if depth >= self.max_depth - 1:
            return self._generate_simple_expression(depth)
        
        # Generate left and right expressions
        left = self._generate_expression(depth + 1)
        right = self._generate_expression(depth + 1)
        
        # Choose an operator
        operators = ["+", "-", "*", "/", "%", "==", "!=", "<", ">", "<=", ">=", "&&", "||"]
        op = random.choice(operators)
        
        # Generate the binary expression
        return f"({left} {op} {right})"
    
    def _generate_unary_expression(self, depth: int) -> str:
        """Generate a unary expression.
        
        Args:
            depth: Current recursion depth
            
        Returns:
            Generated unary expression
        """
        # Limit recursion depth
        if depth >= self.max_depth - 1:
            return self._generate_simple_expression(depth)
        
        # Generate an expression
        expr = self._generate_expression(depth + 1)
        
        # Choose an operator
        operators = ["-", "!", "~"]
        op = random.choice(operators)
        
        # Generate the unary expression
        return f"{op}({expr})"
    
    def _generate_function_call(self, depth: int) -> str:
        """Generate a function call.
        
        Args:
            depth: Current recursion depth
            
        Returns:
            Generated function call
        """
        # Choose a function name
        if self.functions and random.random() < 0.8:
            # Use an existing function
            func_name = random.choice(list(self.functions))
        else:
            # Use a built-in function
            built_ins = ["print", "len", "range", "min", "max", "sum", "abs"]
            func_name = random.choice(built_ins)
        
        # Generate arguments
        num_args = random.randint(0, 3)
        args = []
        
        for _ in range(num_args):
            args.append(self._generate_expression(depth + 1))
        
        # Generate the function call
        return f"{func_name}({', '.join(args)})"
    
    def _generate_variable_reference(self, depth: int) -> str:
        """Generate a variable reference.
        
        Args:
            depth: Current recursion depth
            
        Returns:
            Generated variable reference
        """
        # Choose a variable name
        if self.variables and random.random() < 0.8:
            # Use an existing variable
            return random.choice(list(self.variables))
        else:
            # Use a new variable (this might be an error, which is good for fuzzing)
            return self._generate_variable_name()
    
    def _generate_literal(self, depth: int) -> str:
        """Generate a literal value.
        
        Args:
            depth: Current recursion depth
            
        Returns:
            Generated literal
        """
        # Choose a literal type
        literal_types = [
            self._generate_number_literal,
            self._generate_string_literal,
            self._generate_boolean_literal,
            self._generate_array_literal,
            self._generate_object_literal
        ]
        
        # Weight the selection based on depth
        if depth > 2:
            # Prefer simpler literals at greater depths
            literal_types = [
                self._generate_number_literal,
                self._generate_string_literal,
                self._generate_boolean_literal
            ]
        
        generator_func = random.choice(literal_types)
        return generator_func(depth)
    
    def _generate_number_literal(self, depth: int) -> str:
        """Generate a number literal.
        
        Args:
            depth: Current recursion depth
            
        Returns:
            Generated number literal
        """
        # Choose a number type
        number_type = random.choice(["integer", "float"])
        
        if number_type == "integer":
            # Generate an integer
            return str(random.randint(-1000, 1000))
        else:
            # Generate a float
            return str(random.uniform(-1000.0, 1000.0))
    
    def _generate_string_literal(self, depth: int) -> str:
        """Generate a string literal.
        
        Args:
            depth: Current recursion depth
            
        Returns:
            Generated string literal
        """
        # Generate a random string
        length = random.randint(0, 20)
        chars = string.ascii_letters + string.digits + " "
        content = ''.join(random.choice(chars) for _ in range(length))
        
        # Generate the string literal
        return f'"{content}"'
    
    def _generate_boolean_literal(self, depth: int) -> str:
        """Generate a boolean literal.
        
        Args:
            depth: Current recursion depth
            
        Returns:
            Generated boolean literal
        """
        # Choose a boolean value
        return random.choice(["⊤", "⊥"])
    
    def _generate_array_literal(self, depth: int) -> str:
        """Generate an array literal.
        
        Args:
            depth: Current recursion depth
            
        Returns:
            Generated array literal
        """
        # Limit recursion depth
        if depth >= self.max_depth - 1:
            return "[]"
        
        # Generate array elements
        num_elements = random.randint(0, 5)
        elements = []
        
        for _ in range(num_elements):
            elements.append(self._generate_expression(depth + 1))
        
        # Generate the array literal
        return f"[{', '.join(elements)}]"
    
    def _generate_object_literal(self, depth: int) -> str:
        """Generate an object literal.
        
        Args:
            depth: Current recursion depth
            
        Returns:
            Generated object literal
        """
        # Limit recursion depth
        if depth >= self.max_depth - 1:
            return "{}"
        
        # Generate object properties
        num_properties = random.randint(0, 3)
        properties = []
        
        for _ in range(num_properties):
            key = self._generate_string_literal(depth + 1)
            value = self._generate_expression(depth + 1)
            properties.append(f"{key}: {value}")
        
        # Generate the object literal
        return f"{{{', '.join(properties)}}}"
    
    def _generate_comment(self) -> str:
        """Generate a comment.
        
        Returns:
            Generated comment
        """
        # Choose a comment type
        comment_type = random.choice(["line", "block"])
        
        if comment_type == "line":
            # Generate a line comment
            return f"// {self._generate_comment_text()}"
        else:
            # Generate a block comment
            return f"/* {self._generate_comment_text()} */"
    
    def _generate_comment_text(self) -> str:
        """Generate comment text.
        
        Returns:
            Generated comment text
        """
        # Choose a comment template
        templates = [
            "This is a comment",
            "TODO: Fix this later",
            "FIXME: This might be broken",
            "This code does X",
            "Variable {var} is used for Y",
            "Function {func} calculates Z",
            "This is a test case for fuzzing",
            "Generated by grammar-based fuzzer",
            "Anarchy Inference test program",
            "This might cause an error"
        ]
        
        template = random.choice(templates)
        
        # Fill in placeholders
        if "{var}" in template and self.variables:
            template = template.replace("{var}", random.choice(list(self.variables)))
        
        if "{func}" in template and self.functions:
            template = template.replace("{func}", random.choice(list(self.functions)))
        
        return template
    
    def _generate_variable_name(self) -> str:
        """Generate a variable name.
        
        Returns:
            Generated variable name
        """
        # Choose a variable name template
        templates = [
            "x", "y", "z", "a", "b", "c", "i", "j", "k",
            "count", "index", "value", "result", "temp",
            "sum", "total", "average", "min", "max"
        ]
        
        name = random.choice(templates)
        
        # Maybe add a suffix
        if random.random() < 0.3:
            name += str(random.randint(1, 100))
        
        return name
    
    def _generate_function_name(self) -> str:
        """Generate a function name.
        
        Returns:
            Generated function name
        """
        # Choose a function name template
        templates = [
            "func", "calculate", "compute", "process", "handle",
            "get", "set", "update", "create", "delete",
            "find", "search", "sort", "filter", "map"
        ]
        
        name = random.choice(templates)
        
        # Maybe add a suffix
        if random.random() < 0.3:
            name += str(random.randint(1, 100))
        
        return name


def main():
    """Main entry point for testing the grammar generator."""
    # Create a grammar generator
    generator = GrammarGenerator()
    
    # Generate a test case
    test_case = generator.generate()
    
    # Print the test case
    print(f"Test Case ID: {test_case.id}")
    print(f"Generator Type: {test_case.generator_type.value}")
    print(f"Metadata: {test_case.metadata}")
    print(f"Content:\n{test_case.content}")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
