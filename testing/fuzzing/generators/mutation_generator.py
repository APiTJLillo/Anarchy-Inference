#!/usr/bin/env python3
"""
Mutation Generator for Anarchy Inference Fuzzing.

This module provides functionality for generating inputs by mutating existing inputs
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
logger = logging.getLogger("mutation_generator")

class MutationGenerator:
    """Generates inputs by mutating existing inputs for fuzzing."""
    
    def __init__(self, config: Optional[Dict[str, Any]] = None):
        """Initialize the mutation generator.
        
        Args:
            config: Optional configuration for the generator
        """
        self.config = config or {}
        
        # Default configuration values
        self.mutation_count_min = self.config.get("mutation_count_min", 1)
        self.mutation_count_max = self.config.get("mutation_count_max", 10)
        self.mutation_types = self.config.get("mutation_types", [
            "bit_flip", "byte_flip", "byte_increment", "byte_decrement",
            "insert_random", "delete_random", "replace_random",
            "swap_adjacent", "duplicate_block", "delete_block",
            "replace_block", "insert_known_value", "replace_with_known_value"
        ])
        
        # Known values that often trigger bugs
        self.known_values = [
            # Integer boundaries
            "0", "1", "-1", "2147483647", "-2147483648", "4294967295", "4294967296",
            # Floating point special values
            "0.0", "-0.0", "1.0", "-1.0", "0.1", "-0.1", "1e100", "-1e100", "1e-100", "-1e-100",
            # Special characters
            "\"", "'", "\\", "/", "\n", "\r", "\t", "\0",
            # SQL injection
            "' OR '1'='1", "'; DROP TABLE users; --",
            # Format string
            "%s", "%d", "%n", "%x", "%%",
            # Buffer overflow
            "A" * 100, "A" * 1000, "A" * 10000,
            # Anarchy Inference specific
            "λ", "⟨", "⟩", "←", "→", "⟼", "⌽", "⊤", "⊥", "ι", "÷", "ƒ", "⟑", "⊢"
        ]
        
        # Anarchy Inference specific tokens
        self.anarchy_tokens = [
            "λ", "⟨", "⟩", "←", "→", "⟼", "⌽", "⊤", "⊥", "ι", "÷", "ƒ", "⟑", "⊢",
            "if", "else", "for", "while", "return", "break", "continue",
            "function", "true", "false", "null", "undefined"
        ]
    
    def generate(self, parent: Optional[TestCase] = None) -> TestCase:
        """Generate a test case by mutation.
        
        Args:
            parent: Optional parent test case to mutate
            
        Returns:
            Generated test case
        """
        # Generate a random ID
        test_id = f"mutation_{int(time.time())}_{random.randint(0, 1000000)}"
        
        # Generate content by mutation
        if parent:
            content, mutations = self._mutate_content(parent.content)
        else:
            # No parent, generate a simple program
            content = "// No parent to mutate\nx ← 0\nreturn x"
            mutations = []
        
        # Create metadata
        metadata = {
            "generator": "mutation",
            "parent_id": parent.id if parent else None,
            "mutations": mutations,
            "generation_time": time.time()
        }
        
        # Create test case
        return TestCase(
            id=test_id,
            content=content,
            generator_type=GeneratorType.MUTATION,
            metadata=metadata,
            parent_id=parent.id if parent else None
        )
    
    def _mutate_content(self, content: str) -> Tuple[str, List[Dict[str, Any]]]:
        """Mutate content.
        
        Args:
            content: Content to mutate
            
        Returns:
            Tuple of (mutated content, list of mutations applied)
        """
        # Convert to list for easier mutation
        chars = list(content)
        
        # Determine number of mutations to apply
        num_mutations = random.randint(self.mutation_count_min, self.mutation_count_max)
        
        # Track applied mutations
        applied_mutations = []
        
        # Apply mutations
        for _ in range(num_mutations):
            # Skip if content is empty
            if not chars:
                break
            
            # Select a mutation type
            mutation_type = random.choice(self.mutation_types)
            
            # Apply the mutation
            if mutation_type == "bit_flip" and chars:
                # Flip a random bit in a random byte
                pos = random.randint(0, len(chars) - 1)
                char = chars[pos]
                bit_pos = random.randint(0, 7)
                byte_val = ord(char)
                byte_val ^= (1 << bit_pos)  # Flip the bit
                chars[pos] = chr(byte_val & 0xFF)  # Ensure it's a valid byte
                
                applied_mutations.append({
                    "type": "bit_flip",
                    "position": pos,
                    "bit_position": bit_pos,
                    "original": char,
                    "result": chars[pos]
                })
            
            elif mutation_type == "byte_flip" and chars:
                # Flip all bits in a random byte
                pos = random.randint(0, len(chars) - 1)
                char = chars[pos]
                byte_val = ord(char)
                byte_val = ~byte_val & 0xFF  # Flip all bits
                chars[pos] = chr(byte_val)
                
                applied_mutations.append({
                    "type": "byte_flip",
                    "position": pos,
                    "original": char,
                    "result": chars[pos]
                })
            
            elif mutation_type == "byte_increment" and chars:
                # Increment a random byte
                pos = random.randint(0, len(chars) - 1)
                char = chars[pos]
                byte_val = ord(char)
                byte_val = (byte_val + 1) & 0xFF  # Increment and wrap
                chars[pos] = chr(byte_val)
                
                applied_mutations.append({
                    "type": "byte_increment",
                    "position": pos,
                    "original": char,
                    "result": chars[pos]
                })
            
            elif mutation_type == "byte_decrement" and chars:
                # Decrement a random byte
                pos = random.randint(0, len(chars) - 1)
                char = chars[pos]
                byte_val = ord(char)
                byte_val = (byte_val - 1) & 0xFF  # Decrement and wrap
                chars[pos] = chr(byte_val)
                
                applied_mutations.append({
                    "type": "byte_decrement",
                    "position": pos,
                    "original": char,
                    "result": chars[pos]
                })
            
            elif mutation_type == "insert_random" and chars:
                # Insert a random character
                pos = random.randint(0, len(chars))
                char = chr(random.randint(32, 126))  # Printable ASCII
                chars.insert(pos, char)
                
                applied_mutations.append({
                    "type": "insert_random",
                    "position": pos,
                    "value": char
                })
            
            elif mutation_type == "delete_random" and chars:
                # Delete a random character
                pos = random.randint(0, len(chars) - 1)
                char = chars[pos]
                chars.pop(pos)
                
                applied_mutations.append({
                    "type": "delete_random",
                    "position": pos,
                    "original": char
                })
            
            elif mutation_type == "replace_random" and chars:
                # Replace a random character
                pos = random.randint(0, len(chars) - 1)
                original = chars[pos]
                char = chr(random.randint(32, 126))  # Printable ASCII
                chars[pos] = char
                
                applied_mutations.append({
                    "type": "replace_random",
                    "position": pos,
                    "original": original,
                    "result": char
                })
            
            elif mutation_type == "swap_adjacent" and len(chars) >= 2:
                # Swap adjacent characters
                pos = random.randint(0, len(chars) - 2)
                chars[pos], chars[pos + 1] = chars[pos + 1], chars[pos]
                
                applied_mutations.append({
                    "type": "swap_adjacent",
                    "position": pos,
                    "original": f"{chars[pos+1]}{chars[pos]}",
                    "result": f"{chars[pos]}{chars[pos+1]}"
                })
            
            elif mutation_type == "duplicate_block" and chars:
                # Duplicate a block of characters
                if len(chars) >= 2:
                    block_size = random.randint(1, min(8, len(chars)))
                    start_pos = random.randint(0, len(chars) - block_size)
                    block = chars[start_pos:start_pos + block_size]
                    insert_pos = random.randint(0, len(chars))
                    for i, char in enumerate(block):
                        chars.insert(insert_pos + i, char)
                    
                    applied_mutations.append({
                        "type": "duplicate_block",
                        "block_start": start_pos,
                        "block_size": block_size,
                        "insert_position": insert_pos,
                        "block": ''.join(block)
                    })
            
            elif mutation_type == "delete_block" and chars:
                # Delete a block of characters
                if len(chars) >= 2:
                    block_size = random.randint(1, min(8, len(chars)))
                    start_pos = random.randint(0, len(chars) - block_size)
                    block = chars[start_pos:start_pos + block_size]
                    del chars[start_pos:start_pos + block_size]
                    
                    applied_mutations.append({
                        "type": "delete_block",
                        "block_start": start_pos,
                        "block_size": block_size,
                        "block": ''.join(block)
                    })
            
            elif mutation_type == "replace_block" and chars:
                # Replace a block of characters
                if len(chars) >= 2:
                    block_size = random.randint(1, min(8, len(chars)))
                    start_pos = random.randint(0, len(chars) - block_size)
                    original_block = chars[start_pos:start_pos + block_size]
                    new_block = [chr(random.randint(32, 126)) for _ in range(block_size)]
                    chars[start_pos:start_pos + block_size] = new_block
                    
                    applied_mutations.append({
                        "type": "replace_block",
                        "block_start": start_pos,
                        "block_size": block_size,
                        "original_block": ''.join(original_block),
                        "new_block": ''.join(new_block)
                    })
            
            elif mutation_type == "insert_known_value" and chars:
                # Insert a known value
                pos = random.randint(0, len(chars))
                value = random.choice(self.known_values)
                for i, char in enumerate(value):
                    chars.insert(pos + i, char)
                
                applied_mutations.append({
                    "type": "insert_known_value",
                    "position": pos,
                    "value": value
                })
            
            elif mutation_type == "replace_with_known_value" and chars:
                # Replace with a known value
                if len(chars) >= 1:
                    value = random.choice(self.known_values)
                    if len(chars) >= len(value):
                        start_pos = random.randint(0, len(chars) - len(value))
                        original = ''.join(chars[start_pos:start_pos + len(value)])
                        chars[start_pos:start_pos + len(value)] = list(value)
                        
                        applied_mutations.append({
                            "type": "replace_with_known_value",
                            "position": start_pos,
                            "original": original,
                            "value": value
                        })
            
            # Add more mutation types as needed
        
        # Convert back to string
        return ''.join(chars), applied_mutations
    
    def _find_token_boundaries(self, content: str) -> List[Tuple[int, int]]:
        """Find token boundaries in the content.
        
        Args:
            content: Content to analyze
            
        Returns:
            List of (start, end) positions for tokens
        """
        # This is a simplified implementation; a real implementation would use a lexer
        boundaries = []
        
        # Simple whitespace-based tokenization
        in_token = False
        token_start = 0
        
        for i, char in enumerate(content):
            if char.isspace():
                if in_token:
                    boundaries.append((token_start, i))
                    in_token = False
            else:
                if not in_token:
                    token_start = i
                    in_token = True
        
        # Handle the last token
        if in_token:
            boundaries.append((token_start, len(content)))
        
        return boundaries


def main():
    """Main entry point for testing the mutation generator."""
    # Create a mutation generator
    generator = MutationGenerator()
    
    # Create a simple test case
    test_content = """
    // Simple Anarchy Inference program
    x ← 10
    y ← 20
    z ← x + y
    if z > 25 {
        return "Greater than 25"
    } else {
        return "Less than or equal to 25"
    }
    """
    
    test_case = TestCase(
        id="test_case_1",
        content=test_content,
        generator_type=GeneratorType.RANDOM,
        metadata={}
    )
    
    # Generate a mutated test case
    mutated_test_case = generator.generate(parent=test_case)
    
    # Print the original and mutated test cases
    print(f"Original Test Case ID: {test_case.id}")
    print(f"Original Content:\n{test_case.content}")
    print("\n" + "-" * 50 + "\n")
    print(f"Mutated Test Case ID: {mutated_test_case.id}")
    print(f"Generator Type: {mutated_test_case.generator_type.value}")
    print(f"Mutations Applied: {mutated_test_case.metadata.get('mutations', [])}")
    print(f"Mutated Content:\n{mutated_test_case.content}")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
