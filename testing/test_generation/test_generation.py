"""
Automated Test Generation for Anarchy Inference

This module provides functionality to automatically generate test cases for Anarchy Inference,
improving code coverage and finding edge cases.
"""

import os
import sys
import json
import time
import random
import string
import hashlib
import re
from typing import Dict, List, Any, Optional, Tuple, Set, Callable, Union

# Add the parent directory to the path so we can import the anarchy module
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

try:
    import anarchy
except ImportError:
    print("Error: Could not import anarchy module. Make sure it's in the parent directory.")
    sys.exit(1)

class TestTemplate:
    """Represents a template for generating Anarchy Inference test cases."""
    
    def __init__(self, name: str, template: str, variables: Dict[str, List[str]] = None):
        """Initialize a test template.
        
        Args:
            name: Name of the template
            template: Template string with placeholders
            variables: Dictionary mapping variable names to possible values
        """
        self.name = name
        self.template = template
        self.variables = variables or {}
    
    def generate(self, variable_values: Dict[str, str] = None) -> str:
        """Generate a test case from the template.
        
        Args:
            variable_values: Optional dictionary of variable values to use
            
        Returns:
            The generated test case
        """
        # Start with the template
        result = self.template
        
        # If variable values are provided, use them
        if variable_values:
            for var_name, value in variable_values.items():
                placeholder = f"{{{var_name}}}"
                result = result.replace(placeholder, value)
        
        # For any remaining variables, choose random values
        for var_name, possible_values in self.variables.items():
            placeholder = f"{{{var_name}}}"
            if placeholder in result:
                value = random.choice(possible_values)
                result = result.replace(placeholder, value)
        
        return result
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'TestTemplate':
        """Create a template from a dictionary."""
        return cls(
            name=data["name"],
            template=data["template"],
            variables=data.get("variables", {})
        )
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert the template to a dictionary."""
        return {
            "name": self.name,
            "template": self.template,
            "variables": self.variables
        }


class TemplateEngine:
    """Manages templates and generates test cases from them."""
    
    def __init__(self, template_dir: str = None):
        """Initialize the template engine.
        
        Args:
            template_dir: Directory containing template files
        """
        self.template_dir = template_dir or os.path.join(
            os.path.dirname(os.path.dirname(__file__)), 
            "templates"
        )
        self.templates: Dict[str, TestTemplate] = {}
        
        # Create the template directory if it doesn't exist
        if not os.path.exists(self.template_dir):
            os.makedirs(self.template_dir)
        
        # Load templates
        self._load_templates()
    
    def _load_templates(self):
        """Load templates from the template directory."""
        for file in os.listdir(self.template_dir):
            if file.endswith(".json"):
                with open(os.path.join(self.template_dir, file), 'r') as f:
                    try:
                        data = json.load(f)
                        template = TestTemplate.from_dict(data)
                        self.templates[template.name] = template
                    except json.JSONDecodeError:
                        print(f"Error loading template from {file}: Invalid JSON")
                    except KeyError as e:
                        print(f"Error loading template from {file}: Missing key {e}")
    
    def add_template(self, template: TestTemplate):
        """Add a template to the engine.
        
        Args:
            template: The template to add
        """
        self.templates[template.name] = template
        
        # Save the template to a file
        with open(os.path.join(self.template_dir, f"{template.name}.json"), 'w') as f:
            json.dump(template.to_dict(), f, indent=2)
    
    def generate_test(self, template_name: str, variable_values: Dict[str, str] = None) -> str:
        """Generate a test case from a template.
        
        Args:
            template_name: Name of the template to use
            variable_values: Optional dictionary of variable values to use
            
        Returns:
            The generated test case
        """
        if template_name not in self.templates:
            raise ValueError(f"Template {template_name} not found")
        
        return self.templates[template_name].generate(variable_values)
    
    def generate_tests(self, count: int, template_name: str = None) -> List[str]:
        """Generate multiple test cases.
        
        Args:
            count: Number of test cases to generate
            template_name: Optional name of the template to use
            
        Returns:
            List of generated test cases
        """
        results = []
        
        for _ in range(count):
            if template_name:
                # Use the specified template
                results.append(self.generate_test(template_name))
            else:
                # Choose a random template
                template_name = random.choice(list(self.templates.keys()))
                results.append(self.generate_test(template_name))
        
        return results


class Fuzzer:
    """Generates random but valid Anarchy Inference code."""
    
    def __init__(self, interpreter: 'anarchy.Interpreter'):
        """Initialize the fuzzer.
        
        Args:
            interpreter: The Anarchy Inference interpreter to use
        """
        self.interpreter = interpreter
        
        # Define the grammar elements of Anarchy Inference
        self.symbols = {
            "arithmetic_ops": ["+", "-", "*", "/"],
            "comparison_ops": ["=", "≠", "<", ">", "≤", "≥"],
            "logical_ops": ["∧", "∨", "¬"],
            "keywords": ["ι", "ƒ", "λ", "⟼", "⌽", "÷"],
            "boolean_literals": ["⊤", "⊥"],
            "special_symbols": ["🎤"]  # User input emoji
        }
        
        # Define patterns for generating different code elements
        self.patterns = {
            "variable_decl": "ι {name} = {expr}",
            "function_decl": "ƒ {name}({params}) {body}",
            "library_decl": "λ {name} {body}",
            "return_stmt": "⟼ {expr}",
            "print_stmt": "⌽ {expr}",
            "try_catch": "÷ {try_block} {catch_block}",
            "string_dict": ":{key}",
            "user_input": "🎤"
        }
    
    def generate_random_name(self, length: int = 5) -> str:
        """Generate a random variable or function name.
        
        Args:
            length: Length of the name
            
        Returns:
            A random name
        """
        return ''.join(random.choice(string.ascii_lowercase) for _ in range(length))
    
    def generate_random_number(self, min_val: int = -100, max_val: int = 100) -> str:
        """Generate a random number.
        
        Args:
            min_val: Minimum value
            max_val: Maximum value
            
        Returns:
            A random number as a string
        """
        return str(random.randint(min_val, max_val))
    
    def generate_random_string(self, length: int = 10) -> str:
        """Generate a random string literal.
        
        Args:
            length: Length of the string
            
        Returns:
            A random string literal
        """
        chars = string.ascii_letters + string.digits + string.punctuation + ' '
        content = ''.join(random.choice(chars) for _ in range(length))
        return f'"{content}"'
    
    def generate_random_expression(self, depth: int = 0, max_depth: int = 3) -> str:
        """Generate a random expression.
        
        Args:
            depth: Current recursion depth
            max_depth: Maximum recursion depth
            
        Returns:
            A random expression
        """
        if depth >= max_depth:
            # Base case: generate a simple expression
            choices = [
                lambda: self.generate_random_number(),
                lambda: self.generate_random_string(),
                lambda: random.choice(self.symbols["boolean_literals"]),
                lambda: f":{self.generate_random_name()}",  # String dictionary reference
                lambda: self.generate_random_name()  # Variable reference
            ]
            return random.choice(choices)()
        
        # Recursive case: generate a more complex expression
        choices = [
            # Arithmetic expression
            lambda: f"{self.generate_random_expression(depth+1, max_depth)} {random.choice(self.symbols['arithmetic_ops'])} {self.generate_random_expression(depth+1, max_depth)}",
            
            # Comparison expression
            lambda: f"{self.generate_random_expression(depth+1, max_depth)} {random.choice(self.symbols['comparison_ops'])} {self.generate_random_expression(depth+1, max_depth)}",
            
            # Logical expression
            lambda: f"{self.generate_random_expression(depth+1, max_depth)} {random.choice(self.symbols['logical_ops'])} {self.generate_random_expression(depth+1, max_depth)}",
            
            # Negation
            lambda: f"¬{self.generate_random_expression(depth+1, max_depth)}",
            
            # Function call
            lambda: f"{self.generate_random_name()}({', '.join(self.generate_random_expression(depth+1, max_depth) for _ in range(random.randint(0, 3)))})",
            
            # Simple expression
            lambda: self.generate_random_expression(max_depth, max_depth)
        ]
        
        return random.choice(choices)()
    
    def generate_random_statement(self, depth: int = 0, max_depth: int = 3) -> str:
        """Generate a random statement.
        
        Args:
            depth: Current recursion depth
            max_depth: Maximum recursion depth
            
        Returns:
            A random statement
        """
        if depth >= max_depth:
            # Base case: generate a simple statement
            choices = [
                # Variable declaration
                lambda: self.patterns["variable_decl"].format(
                    name=self.generate_random_name(),
                    expr=self.generate_random_expression(max_depth, max_depth)
                ),
                
                # Print statement
                lambda: self.patterns["print_stmt"].format(
                    expr=self.generate_random_expression(max_depth, max_depth)
                ),
                
                # Return statement
                lambda: self.patterns["return_stmt"].format(
                    expr=self.generate_random_expression(max_depth, max_depth)
                ),
                
                # Expression statement (function call)
                lambda: f"{self.generate_random_name()}({', '.join(self.generate_random_expression(max_depth, max_depth) for _ in range(random.randint(0, 3)))})"
            ]
            return random.choice(choices)()
        
        # Recursive case: generate a more complex statement
        choices = [
            # Variable declaration
            lambda: self.patterns["variable_decl"].format(
                name=self.generate_random_name(),
                expr=self.generate_random_expression(depth+1, max_depth)
            ),
            
            # Function declaration
            lambda: self.patterns["function_decl"].format(
                name=self.generate_random_name(),
                params=', '.join(self.generate_random_name() for _ in range(random.randint(0, 3))),
                body=self.generate_random_block(depth+1, max_depth)
            ),
            
            # Library declaration
            lambda: self.patterns["library_decl"].format(
                name=self.generate_random_name(),
                body=self.generate_random_block(depth+1, max_depth)
            ),
            
            # Try-catch block
            lambda: self.patterns["try_catch"].format(
                try_block=self.generate_random_block(depth+1, max_depth),
                catch_block=self.generate_random_block(depth+1, max_depth)
            ),
            
            # Print statement
            lambda: self.patterns["print_stmt"].format(
                expr=self.generate_random_expression(depth+1, max_depth)
            ),
            
            # Return statement
            lambda: self.patterns["return_stmt"].format(
                expr=self.generate_random_expression(depth+1, max_depth)
            ),
            
            # User input
            lambda: self.patterns["user_input"]
        ]
        
        return random.choice(choices)()
    
    def generate_random_block(self, depth: int = 0, max_depth: int = 3) -> str:
        """Generate a random block of code.
        
        Args:
            depth: Current recursion depth
            max_depth: Maximum recursion depth
            
        Returns:
            A random block of code
        """
        num_statements = random.randint(1, 5)
        statements = [self.generate_random_statement(depth, max_depth) for _ in range(num_statements)]
        return "{\n  " + "\n  ".join(statements) + "\n}"
    
    def generate_random_program(self, max_depth: int = 3, num_statements: int = None) -> str:
        """Generate a random Anarchy Inference program.
        
        Args:
            max_depth: Maximum recursion depth
            num_statements: Number of top-level statements
            
        Returns:
            A random program
        """
        if num_statements is None:
            num_statements = random.randint(3, 10)
        
        statements = [self.generate_random_statement(0, max_depth) for _ in range(num_statements)]
        
        # Add checkpoint markers for testing
        for i, statement in enumerate(statements):
            if random.random() < 0.3:  # 30% chance to add a checkpoint
                statements[i] = f"{statement}\n# CHECKPOINT: checkpoint_{i}"
        
        return "\n".join(statements)
    
    def generate_valid_programs(self, count: int, max_depth: int = 3) -> List[str]:
        """Generate multiple valid Anarchy Inference programs.
        
        Args:
            count: Number of programs to generate
            max_depth: Maximum recursion depth
            
        Returns:
            List of generated programs
        """
        programs = []
        
        for _ in range(count):
            program = self.generate_random_program(max_depth)
            
            # Validate the program
            try:
                # Simple syntax check
                self.interpreter.parse(program)
                programs.append(program)
            except Exception:
                # If invalid, try again
                continue
        
        return programs


class MutationEngine:
    """Creates variations of existing tests through mutation."""
    
    def __init__(self, interpreter: 'anarchy.Interpreter'):
        """Initialize the mutation engine.
        
        Args:
            interpreter: The Anarchy Inference interpreter to use
        """
        self.interpreter = interpreter
        
        # Define mutation operators
        self.mutation_operators = [
            self._change_literal,
            self._swap_operator,
            self._delete_statement,
            self._duplicate_statement,
            self._swap_statements,
            self._add_statement,
            self._change_variable_name
        ]
    
    def mutate(self, program: str, num_mutations: int = 1) -> str:
        """Mutate a program.
        
        Args:
            program: The program to mutate
            num_mutations: Number of mutations to apply
            
        Returns:
            The mutated program
        """
        result = program
        
        for _ in range(num_mutations):
            # Choose a random mutation operator
            operator = random.choice(self.mutation_operators)
            
            # Apply the mutation
            try:
                result = operator(result)
            except Exception:
                # If mutation fails, keep the original
                continue
        
        return result
    
    def _change_literal(self, program: str) -> str:
        """Change a numeric or string literal in the program.
        
        Args:
            program: The program to mutate
            
        Returns:
            The mutated program
        """
        # Find numeric literals
        num_matches = list(re.finditer(r'\b\d+\b', program))
        if num_matches:
            match = random.choice(num_matches)
            old_value = match.group(0)
            new_value = str(int(old_value) + random.randint(-10, 10))
            return program[:match.start()] + new_value + program[match.end():]
        
        # Find string literals
        str_matches = list(re.finditer(r'"[^"]*"', program))
        if str_matches:
            match = random.choice(str_matches)
            old_value = match.group(0)
            # Remove quotes
            content = old_value[1:-1]
            
            # Mutate the string
            if content:
                pos = random.randint(0, len(content) - 1)
                char = random.choice(string.ascii_letters + string.digits)
                new_content = content[:pos] + char + content[pos+1:]
            else:
                new_content = random.choice(string.ascii_letters)
            
            new_value = f'"{new_content}"'
            return program[:match.start()] + new_value + program[match.end():]
        
        return program
    
    def _swap_operator(self, program: str) -> str:
        """Swap an operator in the program.
        
        Args:
            program: The program to mutate
            
        Returns:
            The mutated program
        """
        # Define operator groups
        arithmetic_ops = ["+", "-", "*", "/"]
        comparison_ops = ["=", "≠", "<", ">", "≤", "≥"]
        logical_ops = ["∧", "∨"]
        
        # Choose an operator group
        op_groups = [arithmetic_ops, comparison_ops, logical_ops]
        op_group = random.choice(op_groups)
        
        # Find operators from the chosen group
        op_pattern = '|'.join(re.escape(op) for op in op_group)
        matches = list(re.finditer(op_pattern, program))
        
        if matches:
            match = random.choice(matches)
            old_op = match.group(0)
            # Choose a different operator from the same group
            new_op = random.choice([op for op in op_group if op != old_op])
            return program[:match.start()] + new_op + program[match.end():]
        
        return program
    
    def _delete_statement(self, program: str) -> str:
        """Delete a statement from the program.
        
        Args:
            program: The program to mutate
            
        Returns:
            The mutated program
        """
        lines = program.split('\n')
        if len(lines) <= 1:
            return program
        
        # Choose a random line to delete
        line_idx = random.randint(0, len(lines) - 1)
        
        # Skip checkpoint markers
        while "# CHECKPOINT:" in lines[line_idx]:
            line_idx = random.randint(0, len(lines) - 1)
        
        # Delete the line
        del lines[line_idx]
        
        return '\n'.join(lines)
    
    def _duplicate_statement(self, program: str) -> str:
        """Duplicate a statement in the program.
        
        Args:
            program: The program to mutate
            
        Returns:
            The mutated program
        """
        lines = program.split('\n')
        if not lines:
            return program
        
        # Choose a random line to duplicate
        line_idx = random.randint(0, len(lines) - 1)
        
        # Skip checkpoint markers
        while "# CHECKPOINT:" in lines[line_idx]:
            line_idx = random.randint(0, len(lines) - 1)
        
        # Duplicate the line
        lines.insert(line_idx + 1, lines[line_idx])
        
        return '\n'.join(lines)
    
    def _swap_statements(self, program: str) -> str:
        """Swap two statements in the program.
        
        Args:
            program: The program to mutate
            
        Returns:
            The mutated program
        """
        lines = program.split('\n')
        if len(lines) <= 1:
            return program
        
        # Choose two random lines to swap
        idx1 = random.randint(0, len(lines) - 1)
        idx2 = random.randint(0, len(lines) - 1)
        
        # Skip checkpoint markers
        while "# CHECKPOINT:" in lines[idx1]:
            idx1 = random.randint(0, len(lines) - 1)
        while "# CHECKPOINT:" in lines[idx2] or idx1 == idx2:
            idx2 = random.randint(0, len(lines) - 1)
        
        # Swap the lines
        lines[idx1], lines[idx2] = lines[idx2], lines[idx1]
        
        return '\n'.join(lines)
    
    def _add_statement(self, program: str) -> str:
        """Add a new statement to the program.
        
        Args:
            program: The program to mutate
            
        Returns:
            The mutated program
        """
        lines = program.split('\n')
        
        # Create a fuzzer to generate a new statement
        fuzzer = Fuzzer(self.interpreter)
        new_statement = fuzzer.generate_random_statement()
        
        # Choose a random position to insert the new statement
        pos = random.randint(0, len(lines))
        
        # Insert the new statement
        lines.insert(pos, new_statement)
        
        return '\n'.join(lines)
    
    def _change_variable_name(self, program: str) -> str:
        """Change a variable name in the program.
        
        Args:
            program: The program to mutate
            
        Returns:
            The mutated program
        """
        # Find variable declarations
        var_matches = list(re.finditer(r'ι\s+([a-zA-Z_][a-zA-Z0-9_]*)', program))
        if not var_matches:
            return program
        
        # Choose a random variable declaration
        match = random.choice(var_matches)
        old_name = match.group(1)
        
        # Generate a new name
        new_name = old_name + "_" + ''.join(random.choice(string.ascii_lowercase) for _ in range(3))
        
        # Replace all occurrences of the variable name
        return re.sub(r'\b' + re.escape(old_name) + r'\b', new_name, program)
    
    def generate_mutants(self, program: str, count: int, mutations_per_program: int = 1) -> List[str]:
        """Generate multiple mutants of a program.
        
        Args:
            program: The program to mutate
            count: Number of mutants to generate
            mutations_per_program: Number of mutations to apply to each mutant
            
        Returns:
            List of mutated programs
        """
        mutants = []
        
        for _ in range(count):
            mutant = self.mutate(program, mutations_per_program)
            
            # Validate the mutant
            try:
                # Simple syntax check
                self.interpreter.parse(mutant)
                mutants.append(mutant)
            except Exception:
                # If invalid, try again
                continue
        
        return mutants


class TestOracle:
    """Validates generated test outputs."""
    
    def __init__(self, interpreter: 'anarchy.Interpreter'):
        """Initialize the test oracle.
        
        Args:
            interpreter: The Anarchy Inference interpreter to use
        """
        self.interpreter = interpreter
    
    def validate(self, program: str) -> Tuple[bool, str]:
        """Validate a program.
        
        Args:
            program: The program to validate
            
        Returns:
            A tuple of (is_valid, error_message)
        """
        try:
            # Parse the program
            self.interpreter.parse(program)
            return True, ""
        except Exception as e:
            return False, str(e)
    
    def execute(self, program: str) -> Tuple[bool, Any, str]:
        """Execute a program and capture the result.
        
        Args:
            program: The program to execute
            
        Returns:
            A tuple of (success, result, error_message)
        """
        try:
            # Execute the program
            result = self.interpreter.execute(program)
            return True, result, ""
        except Exception as e:
            return False, None, str(e)
    
    def compare_results(self, program1: str, program2: str) -> Tuple[bool, str]:
        """Compare the results of executing two programs.
        
        Args:
            program1: First program
            program2: Second program
            
        Returns:
            A tuple of (results_match, difference)
        """
        success1, result1, error1 = self.execute(program1)
        success2, result2, error2 = self.execute(program2)
        
        # If both failed, compare error messages
        if not success1 and not success2:
            return error1 == error2, f"Error1: {error1}\nError2: {error2}"
        
        # If one succeeded and one failed
        if success1 != success2:
            return False, f"Program1 {'succeeded' if success1 else 'failed'}, Program2 {'succeeded' if success2 else 'failed'}"
        
        # If both succeeded, compare results
        return result1 == result2, f"Result1: {result1}\nResult2: {result2}"


class TestSelector:
    """Prioritizes tests based on coverage and other metrics."""
    
    def __init__(self, interpreter: 'anarchy.Interpreter'):
        """Initialize the test selector.
        
        Args:
            interpreter: The Anarchy Inference interpreter to use
        """
        self.interpreter = interpreter
        self.test_scores: Dict[str, float] = {}
    
    def score_test(self, program: str) -> float:
        """Score a test based on various metrics.
        
        Args:
            program: The program to score
            
        Returns:
            A score between 0 and 1
        """
        score = 0.0
        
        # Calculate complexity score
        complexity_score = self._calculate_complexity(program)
        score += 0.4 * complexity_score  # 40% weight
        
        # Calculate uniqueness score
        uniqueness_score = self._calculate_uniqueness(program)
        score += 0.3 * uniqueness_score  # 30% weight
        
        # Calculate validity score
        validity_score = self._calculate_validity(program)
        score += 0.3 * validity_score  # 30% weight
        
        # Store the score
        test_hash = hashlib.md5(program.encode()).hexdigest()
        self.test_scores[test_hash] = score
        
        return score
    
    def _calculate_complexity(self, program: str) -> float:
        """Calculate the complexity of a program.
        
        Args:
            program: The program to analyze
            
        Returns:
            A score between 0 and 1
        """
        # Count lines
        lines = program.strip().split('\n')
        line_count = len(lines)
        
        # Count operators
        operator_count = sum(line.count(op) for line in lines for op in "+-*/=≠<>≤≥∧∨¬")
        
        # Count keywords
        keyword_count = sum(line.count(kw) for line in lines for kw in "ιƒλ⟼⌽÷")
        
        # Calculate complexity score
        complexity = (line_count + operator_count + keyword_count) / 100  # Normalize
        return min(1.0, complexity)  # Cap at 1.0
    
    def _calculate_uniqueness(self, program: str) -> float:
        """Calculate how unique a program is compared to existing ones.
        
        Args:
            program: The program to analyze
            
        Returns:
            A score between 0 and 1
        """
        # If no existing tests, it's unique
        if not self.test_scores:
            return 1.0
        
        # Calculate similarity to existing tests
        program_hash = hashlib.md5(program.encode()).hexdigest()
        if program_hash in self.test_scores:
            return 0.0  # Exact duplicate
        
        # Simple uniqueness heuristic based on program length
        lengths = [len(h) for h in self.test_scores.keys()]
        avg_length = sum(lengths) / len(lengths)
        program_length = len(program)
        
        # Calculate uniqueness based on length difference
        length_diff = abs(program_length - avg_length) / max(avg_length, 1)
        return min(1.0, length_diff)  # Cap at 1.0
    
    def _calculate_validity(self, program: str) -> float:
        """Calculate the validity of a program.
        
        Args:
            program: The program to analyze
            
        Returns:
            A score between 0 and 1
        """
        try:
            # Try to parse the program
            self.interpreter.parse(program)
            return 1.0  # Valid program
        except Exception:
            return 0.0  # Invalid program
    
    def select_tests(self, programs: List[str], count: int) -> List[str]:
        """Select the best tests from a list of programs.
        
        Args:
            programs: List of programs to select from
            count: Number of programs to select
            
        Returns:
            List of selected programs
        """
        # Score all programs
        scores = [(program, self.score_test(program)) for program in programs]
        
        # Sort by score in descending order
        scores.sort(key=lambda x: x[1], reverse=True)
        
        # Select the top programs
        return [program for program, _ in scores[:count]]


class TestGenerator:
    """Generates test cases for Anarchy Inference."""
    
    def __init__(self, interpreter: 'anarchy.Interpreter', output_dir: str = None):
        """Initialize the test generator.
        
        Args:
            interpreter: The Anarchy Inference interpreter to use
            output_dir: Directory to save generated tests
        """
        self.interpreter = interpreter
        self.output_dir = output_dir or os.path.join(
            os.path.dirname(os.path.dirname(__file__)), 
            "generated_tests"
        )
        
        # Create the output directory if it doesn't exist
        if not os.path.exists(self.output_dir):
            os.makedirs(self.output_dir)
        
        # Initialize components
        self.template_engine = TemplateEngine()
        self.fuzzer = Fuzzer(interpreter)
        self.mutation_engine = MutationEngine(interpreter)
        self.oracle = TestOracle(interpreter)
        self.selector = TestSelector(interpreter)
    
    def generate_tests(self, count: int, method: str = "all") -> List[str]:
        """Generate test cases using various methods.
        
        Args:
            count: Number of tests to generate
            method: Method to use (template, fuzzing, mutation, or all)
            
        Returns:
            List of generated test cases
        """
        if method == "template":
            return self._generate_from_templates(count)
        elif method == "fuzzing":
            return self._generate_from_fuzzing(count)
        elif method == "mutation":
            return self._generate_from_mutation(count)
        elif method == "all":
            # Distribute count among methods
            template_count = count // 3
            fuzzing_count = count // 3
            mutation_count = count - template_count - fuzzing_count
            
            template_tests = self._generate_from_templates(template_count)
            fuzzing_tests = self._generate_from_fuzzing(fuzzing_count)
            mutation_tests = self._generate_from_mutation(mutation_count, template_tests + fuzzing_tests)
            
            return template_tests + fuzzing_tests + mutation_tests
        else:
            raise ValueError(f"Unknown method: {method}")
    
    def _generate_from_templates(self, count: int) -> List[str]:
        """Generate tests from templates.
        
        Args:
            count: Number of tests to generate
            
        Returns:
            List of generated tests
        """
        # If no templates exist, create some default ones
        if not self.template_engine.templates:
            self._create_default_templates()
        
        # Generate tests from templates
        tests = self.template_engine.generate_tests(count)
        
        # Validate and filter tests
        valid_tests = []
        for test in tests:
            is_valid, _ = self.oracle.validate(test)
            if is_valid:
                valid_tests.append(test)
        
        return valid_tests
    
    def _generate_from_fuzzing(self, count: int) -> List[str]:
        """Generate tests using fuzzing.
        
        Args:
            count: Number of tests to generate
            
        Returns:
            List of generated tests
        """
        # Generate random programs
        return self.fuzzer.generate_valid_programs(count)
    
    def _generate_from_mutation(self, count: int, seed_programs: List[str] = None) -> List[str]:
        """Generate tests using mutation.
        
        Args:
            count: Number of tests to generate
            seed_programs: Programs to mutate
            
        Returns:
            List of generated tests
        """
        if not seed_programs:
            # If no seed programs, generate some using fuzzing
            seed_programs = self._generate_from_fuzzing(5)
        
        # Generate mutants
        mutants = []
        for program in seed_programs:
            mutants.extend(self.mutation_engine.generate_mutants(
                program, 
                count=max(1, count // len(seed_programs)),
                mutations_per_program=random.randint(1, 3)
            ))
        
        # Select the best mutants
        if len(mutants) > count:
            mutants = self.selector.select_tests(mutants, count)
        
        return mutants
    
    def _create_default_templates(self):
        """Create default templates for test generation."""
        # Arithmetic template
        arithmetic_template = TestTemplate(
            name="arithmetic",
            template="""# Arithmetic operations test
ι {var1} = {num1}
ι {var2} = {num2}
ι {result_var} = {var1} {op} {var2}
⌽ {result_var}
# CHECKPOINT: arithmetic_result""",
            variables={
                "var1": ["a", "x", "foo"],
                "var2": ["b", "y", "bar"],
                "result_var": ["result", "sum", "output"],
                "num1": ["5", "10", "42", "100"],
                "num2": ["2", "7", "13", "50"],
                "op": ["+", "-", "*", "/"]
            }
        )
        self.template_engine.add_template(arithmetic_template)
        
        # String template
        string_template = TestTemplate(
            name="string",
            template="""# String operations test
ι {var1} = {str1}
ι {var2} = {str2}
ι {result_var} = {var1} + {var2}
⌽ {result_var}
# CHECKPOINT: string_result""",
            variables={
                "var1": ["s1", "str1", "first"],
                "var2": ["s2", "str2", "second"],
                "result_var": ["result", "combined", "output"],
                "str1": ['"Hello, "', '"Testing "', '"Anarchy "'],
                "str2": ['"world!"', '"Inference"', '"is fun!"']
            }
        )
        self.template_engine.add_template(string_template)
        
        # Function template
        function_template = TestTemplate(
            name="function",
            template="""# Function test
ƒ {func_name}({param1}, {param2}) {
  ι {local_var} = {param1} {op} {param2}
  ⟼ {local_var}
}

ι {result_var} = {func_name}({arg1}, {arg2})
⌽ {result_var}
# CHECKPOINT: function_result""",
            variables={
                "func_name": ["add", "calculate", "compute"],
                "param1": ["a", "x", "first"],
                "param2": ["b", "y", "second"],
                "local_var": ["result", "value", "temp"],
                "op": ["+", "-", "*", "/"],
                "result_var": ["output", "final", "answer"],
                "arg1": ["5", "10", "42"],
                "arg2": ["2", "7", "13"]
            }
        )
        self.template_engine.add_template(function_template)
        
        # Conditional template
        conditional_template = TestTemplate(
            name="conditional",
            template="""# Conditional test
ι {var1} = {num1}
ι {var2} = {num2}

ι {result_var} = ⊥

{var1} {comp_op} {var2} ∧ {
  {result_var} = ⊤
}

⌽ {result_var}
# CHECKPOINT: conditional_result""",
            variables={
                "var1": ["a", "x", "value1"],
                "var2": ["b", "y", "value2"],
                "result_var": ["result", "is_greater", "condition_met"],
                "num1": ["5", "10", "42", "100"],
                "num2": ["2", "7", "13", "50"],
                "comp_op": ["<", ">", "=", "≠", "≤", "≥"]
            }
        )
        self.template_engine.add_template(conditional_template)
        
        # Error handling template
        error_template = TestTemplate(
            name="error_handling",
            template="""# Error handling test
ι {var1} = {num1}
ι {var2} = {num2}

÷ {
  ι {result_var} = {var1} / {var2}
  ⌽ {result_var}
} {
  ⌽ "Error: Division by zero"
}
# CHECKPOINT: error_handling_result""",
            variables={
                "var1": ["a", "x", "numerator"],
                "var2": ["b", "y", "denominator"],
                "result_var": ["result", "quotient", "output"],
                "num1": ["10", "42", "100"],
                "num2": ["0", "5", "10"]
            }
        )
        self.template_engine.add_template(error_template)
    
    def save_test(self, program: str, name: str = None) -> str:
        """Save a test to a file.
        
        Args:
            program: The program to save
            name: Optional name for the test file
            
        Returns:
            Path to the saved file
        """
        if name is None:
            # Generate a name based on hash
            hash_str = hashlib.md5(program.encode()).hexdigest()[:8]
            name = f"test_{hash_str}"
        
        # Ensure the name has the correct extension
        if not name.endswith(".ai"):
            name += ".ai"
        
        # Save the program
        path = os.path.join(self.output_dir, name)
        with open(path, 'w') as f:
            f.write(program)
        
        return path
    
    def save_tests(self, programs: List[str], prefix: str = "test") -> List[str]:
        """Save multiple tests to files.
        
        Args:
            programs: The programs to save
            prefix: Prefix for the test file names
            
        Returns:
            List of paths to the saved files
        """
        paths = []
        
        for i, program in enumerate(programs):
            # Generate a name
            hash_str = hashlib.md5(program.encode()).hexdigest()[:8]
            name = f"{prefix}_{i}_{hash_str}.ai"
            
            # Save the program
            path = self.save_test(program, name)
            paths.append(path)
        
        return paths
    
    def generate_and_save_tests(self, count: int, method: str = "all", prefix: str = "test") -> List[str]:
        """Generate and save test cases.
        
        Args:
            count: Number of tests to generate
            method: Method to use (template, fuzzing, mutation, or all)
            prefix: Prefix for the test file names
            
        Returns:
            List of paths to the saved files
        """
        # Generate tests
        programs = self.generate_tests(count, method)
        
        # Save tests
        return self.save_tests(programs, prefix)
