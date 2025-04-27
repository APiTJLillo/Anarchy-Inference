#!/usr/bin/env python3
"""
Random Generator for Anarchy Inference Fuzzing.

This module provides functionality for generating completely random inputs
for fuzzing the Anarchy Inference language implementation.
"""

import os
import sys
import time
import random
import string
import logging
from typing import Dict, List, Any, Optional, Tuple
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
logger = logging.getLogger("random_generator")

class RandomGenerator:
    """Generates completely random inputs for fuzzing."""
    
    def __init__(self, config: Optional[Dict[str, Any]] = None):
        """Initialize the random generator.
        
        Args:
            config: Optional configuration for the generator
        """
        self.config = config or {}
        
        # Default configuration values
        self.min_length = self.config.get("min_length", 10)
        self.max_length = self.config.get("max_length", 1000)
        self.include_unicode = self.config.get("include_unicode", True)
        self.include_special_chars = self.config.get("include_special_chars", True)
        self.include_anarchy_symbols = self.config.get("include_anarchy_symbols", True)
        
        # Character sets
        self.alphanumeric = string.ascii_letters + string.digits
        self.whitespace = " \t\n\r"
        self.special_chars = "!@#$%^&*()-_=+[]{}|;:'\",.<>/?"
        self.anarchy_symbols = "λ⟨⟩←→⟼⌽⊤⊥ι÷ƒ⟑⊢"
        self.unicode_chars = "αβγδεζηθικλμνξοπρστυφχψω∀∃∈∉∋∌∩∪⊂⊃⊆⊇⊕⊖⊗⊘⊙⊚⊛⊜⊝⊞⊟⊠⊡⊢⊣⊤⊥⊦⊧⊨⊩⊪⊫⊬⊭⊮⊯⊰⊱⊲⊳⊴⊵⊶⊷⊸⊹⊺⊻⊼⊽⊾⊿⋀⋁⋂⋃⋄⋅⋆⋇⋈⋉⋊⋋⋌⋍⋎⋏⋐⋑⋒⋓⋔⋕⋖⋗⋘⋙⋚⋛⋜⋝⋞⋟⋠⋡⋢⋣⋤⋥⋦⋧⋨⋩⋪⋫⋬⋭⋮⋯⋰⋱⋲⋳⋴⋵⋶⋷⋸⋹⋺⋻⋼⋽⋾⋿"
        
        # Anarchy Inference keywords and operators
        self.keywords = ["if", "else", "for", "while", "return", "break", "continue", "function", "true", "false", "null"]
        self.operators = ["+", "-", "*", "/", "%", "==", "!=", "<", ">", "<=", ">=", "&&", "||", "!"]
        
        # Anarchy Inference specific constructs
        self.constructs = [
            "λ⟨ function_name ⟩(param) { return param }",
            "if condition { statement }",
            "if condition { statement } else { statement }",
            "for i in range(10) { statement }",
            "while condition { statement }",
            "x ← value",
            "return value",
            "⌽(value)",
            "array ← []",
            "object ← {}"
        ]
    
    def generate(self, parent: Optional[TestCase] = None) -> TestCase:
        """Generate a random test case.
        
        Args:
            parent: Optional parent test case (ignored for random generation)
            
        Returns:
            Generated test case
        """
        # Generate a random ID
        test_id = f"random_{int(time.time())}_{random.randint(0, 1000000)}"
        
        # Generate random content
        content = self._generate_random_content()
        
        # Create metadata
        metadata = {
            "generator": "random",
            "length": len(content),
            "generation_time": time.time()
        }
        
        # Create test case
        return TestCase(
            id=test_id,
            content=content,
            generator_type=GeneratorType.RANDOM,
            metadata=metadata,
            parent_id=parent.id if parent else None
        )
    
    def _generate_random_content(self) -> str:
        """Generate random content.
        
        Returns:
            Random content
        """
        # Determine generation strategy
        strategy = random.choice([
            "pure_random",
            "structured_random",
            "template_based",
            "keyword_heavy",
            "symbol_heavy"
        ])
        
        if strategy == "pure_random":
            return self._generate_pure_random()
        elif strategy == "structured_random":
            return self._generate_structured_random()
        elif strategy == "template_based":
            return self._generate_template_based()
        elif strategy == "keyword_heavy":
            return self._generate_keyword_heavy()
        elif strategy == "symbol_heavy":
            return self._generate_symbol_heavy()
        else:
            return self._generate_pure_random()
    
    def _generate_pure_random(self) -> str:
        """Generate purely random content.
        
        Returns:
            Random content
        """
        # Determine length
        length = random.randint(self.min_length, self.max_length)
        
        # Build character set
        charset = self.alphanumeric + self.whitespace
        
        if self.include_special_chars:
            charset += self.special_chars
        
        if self.include_anarchy_symbols:
            charset += self.anarchy_symbols
        
        if self.include_unicode and random.random() < 0.3:  # 30% chance to include Unicode
            charset += self.unicode_chars
        
        # Generate random string
        return ''.join(random.choice(charset) for _ in range(length))
    
    def _generate_structured_random(self) -> str:
        """Generate structured random content with some valid syntax.
        
        Returns:
            Structured random content
        """
        # Generate a basic program structure
        lines = []
        
        # Add some variable declarations
        num_vars = random.randint(1, 5)
        for i in range(num_vars):
            var_name = random.choice(["x", "y", "z", "a", "b", "c", "i", "j", "k"]) + str(i)
            var_value = self._generate_random_value()
            lines.append(f"{var_name} ← {var_value}")
        
        # Add some operations
        num_ops = random.randint(1, 5)
        for i in range(num_ops):
            var_name = random.choice(["x", "y", "z", "a", "b", "c", "i", "j", "k"]) + str(random.randint(0, num_vars-1))
            op = random.choice(self.operators)
            var_value = self._generate_random_value()
            lines.append(f"{var_name} ← {var_name} {op} {var_value}")
        
        # Add a return statement
        var_name = random.choice(["x", "y", "z", "a", "b", "c", "i", "j", "k"]) + str(random.randint(0, num_vars-1))
        lines.append(f"return {var_name}")
        
        # Join lines
        return '\n'.join(lines)
    
    def _generate_template_based(self) -> str:
        """Generate content based on templates.
        
        Returns:
            Template-based content
        """
        # Select a random template
        template = random.choice(self.constructs)
        
        # Randomly modify the template
        for _ in range(random.randint(1, 5)):
            # Select a random position
            if not template:
                break
            pos = random.randint(0, len(template) - 1)
            
            # Perform a random modification
            mod_type = random.choice(["insert", "delete", "replace"])
            
            if mod_type == "insert":
                char = random.choice(self.alphanumeric + self.whitespace + self.special_chars)
                template = template[:pos] + char + template[pos:]
            
            elif mod_type == "delete" and template:
                template = template[:pos] + template[pos+1:]
            
            elif mod_type == "replace" and template:
                char = random.choice(self.alphanumeric + self.whitespace + self.special_chars)
                template = template[:pos] + char + template[pos+1:]
        
        return template
    
    def _generate_keyword_heavy(self) -> str:
        """Generate content with many keywords.
        
        Returns:
            Keyword-heavy content
        """
        lines = []
        
        # Add random keywords and operators
        num_lines = random.randint(5, 20)
        for _ in range(num_lines):
            line = ""
            num_words = random.randint(1, 10)
            for _ in range(num_words):
                if random.random() < 0.7:  # 70% chance for keyword
                    line += random.choice(self.keywords) + " "
                else:
                    line += self._generate_random_word() + " "
            
            # Add some operators
            num_ops = random.randint(0, 3)
            for _ in range(num_ops):
                line += random.choice(self.operators) + " "
            
            lines.append(line)
        
        return '\n'.join(lines)
    
    def _generate_symbol_heavy(self) -> str:
        """Generate content with many Anarchy symbols.
        
        Returns:
            Symbol-heavy content
        """
        lines = []
        
        # Add random symbols
        num_lines = random.randint(5, 20)
        for _ in range(num_lines):
            line = ""
            num_symbols = random.randint(1, 10)
            for _ in range(num_symbols):
                if random.random() < 0.7:  # 70% chance for Anarchy symbol
                    line += random.choice(self.anarchy_symbols) + " "
                else:
                    line += random.choice(self.alphanumeric) + " "
            
            lines.append(line)
        
        return '\n'.join(lines)
    
    def _generate_random_value(self) -> str:
        """Generate a random value.
        
        Returns:
            Random value as a string
        """
        value_type = random.choice(["number", "string", "boolean", "array", "object"])
        
        if value_type == "number":
            return str(random.randint(-1000, 1000))
        
        elif value_type == "string":
            length = random.randint(1, 20)
            chars = string.ascii_letters + string.digits
            return f'"{self._generate_random_word(length)}"'
        
        elif value_type == "boolean":
            return random.choice(["true", "false"])
        
        elif value_type == "array":
            length = random.randint(0, 5)
            elements = [self._generate_random_value() for _ in range(length)]
            return f"[{', '.join(elements)}]"
        
        elif value_type == "object":
            length = random.randint(0, 3)
            keys = [f'"{self._generate_random_word(5)}"' for _ in range(length)]
            values = [self._generate_random_value() for _ in range(length)]
            pairs = [f"{keys[i]}: {values[i]}" for i in range(length)]
            return f"{{{', '.join(pairs)}}}"
        
        return "0"
    
    def _generate_random_word(self, length: Optional[int] = None) -> str:
        """Generate a random word.
        
        Args:
            length: Optional length of the word
            
        Returns:
            Random word
        """
        if length is None:
            length = random.randint(1, 10)
        
        return ''.join(random.choice(string.ascii_letters) for _ in range(length))


def main():
    """Main entry point for testing the random generator."""
    # Create a random generator
    generator = RandomGenerator()
    
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
