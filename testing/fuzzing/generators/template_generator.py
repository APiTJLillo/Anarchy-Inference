#!/usr/bin/env python3
"""
Template Generator for Anarchy Inference Fuzzing.

This module provides functionality for generating inputs based on templates with placeholders
for fuzzing the Anarchy Inference language implementation.
"""

import os
import sys
import time
import random
import string
import logging
import re
from typing import Dict, List, Any, Optional, Tuple, Set
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
logger = logging.getLogger("template_generator")

class TemplateGenerator:
    """Generates inputs based on templates with placeholders for fuzzing."""
    
    def __init__(self, config: Optional[Dict[str, Any]] = None):
        """Initialize the template generator.
        
        Args:
            config: Optional configuration for the generator
        """
        self.config = config or {}
        
        # Default configuration values
        self.template_dir = self.config.get("template_dir", "templates")
        self.max_replacements = self.config.get("max_replacements", 10)
        self.error_probability = self.config.get("error_probability", 0.1)
        
        # Load templates
        self.templates = self._load_templates()
        
        # Initialize placeholder generators
        self.placeholder_generators = self._initialize_placeholder_generators()
        
        # Variable tracking
        self.variables: Set[str] = set()
        
        # Function tracking
        self.functions: Set[str] = set()
    
    def generate(self, parent: Optional[TestCase] = None) -> TestCase:
        """Generate a test case based on templates.
        
        Args:
            parent: Optional parent test case (ignored for template generation)
            
        Returns:
            Generated test case
        """
        # Generate a random ID
        test_id = f"template_{int(time.time())}_{random.randint(0, 1000000)}"
        
        # Reset state
        self.variables = set()
        self.functions = set()
        
        # Generate content based on templates
        content, template_info = self._generate_from_template()
        
        # Create metadata
        metadata = {
            "generator": "template",
            "template_info": template_info,
            "generation_time": time.time()
        }
        
        # Create test case
        return TestCase(
            id=test_id,
            content=content,
            generator_type=GeneratorType.TEMPLATE,
            metadata=metadata,
            parent_id=parent.id if parent else None
        )
    
    def _load_templates(self) -> List[Dict[str, Any]]:
        """Load templates from the template directory.
        
        Returns:
            List of templates
        """
        # This is a simplified implementation; a real implementation would load from files
        templates = [
            {
                "name": "simple_variable",
                "description": "Simple variable declaration and return",
                "content": """
// Simple variable declaration and return
{variable_name} ← {value}
return {variable_name}
"""
            },
            {
                "name": "arithmetic_operations",
                "description": "Arithmetic operations with variables",
                "content": """
// Arithmetic operations
{variable_name1} ← {number_value1}
{variable_name2} ← {number_value2}
{variable_name3} ← {variable_name1} {arithmetic_operator} {variable_name2}
return {variable_name3}
"""
            },
            {
                "name": "conditional",
                "description": "Conditional statement",
                "content": """
// Conditional statement
{variable_name1} ← {number_value1}
{variable_name2} ← {number_value2}
if {variable_name1} {comparison_operator} {variable_name2} {
    return {true_value}
} else {
    return {false_value}
}
"""
            },
            {
                "name": "loop",
                "description": "Loop with counter",
                "content": """
// Loop with counter
{variable_name1} ← 0
for {loop_variable} in range({loop_count}) {
    {variable_name1} ← {variable_name1} + 1
}
return {variable_name1}
"""
            },
            {
                "name": "function_definition",
                "description": "Function definition and call",
                "content": """
// Function definition and call
λ⟨{function_name}⟩({parameter_name}) {
    return {parameter_name} {arithmetic_operator} {number_value1}
}

{variable_name1} ← {number_value2}
{variable_name2} ← {function_name}({variable_name1})
return {variable_name2}
"""
            },
            {
                "name": "string_operations",
                "description": "String operations",
                "content": """
// String operations
{variable_name1} ← {string_value1}
{variable_name2} ← {string_value2}
{variable_name3} ← {variable_name1} + {variable_name2}
return {variable_name3}
"""
            },
            {
                "name": "array_operations",
                "description": "Array operations",
                "content": """
// Array operations
{variable_name1} ← {array_value}
{variable_name2} ← {variable_name1}[{array_index}]
return {variable_name2}
"""
            },
            {
                "name": "error_handling",
                "description": "Error handling with try-catch",
                "content": """
// Error handling with try-catch
{variable_name1} ← {number_value1}
{variable_name2} ← {number_value2}
try {
    {variable_name3} ← {variable_name1} ÷ {variable_name2}
    return {variable_name3}
} catch {
    return "Division error"
}
"""
            },
            {
                "name": "complex_program",
                "description": "Complex program with multiple features",
                "content": """
// Complex program with multiple features
λ⟨{function_name1}⟩({parameter_name1}, {parameter_name2}) {
    return {parameter_name1} {arithmetic_operator} {parameter_name2}
}

λ⟨{function_name2}⟩({parameter_name3}) {
    if {parameter_name3} {comparison_operator} 0 {
        return {true_value}
    } else {
        return {false_value}
    }
}

{variable_name1} ← {number_value1}
{variable_name2} ← {number_value2}
{variable_name3} ← {function_name1}({variable_name1}, {variable_name2})

if {function_name2}({variable_name3}) {
    for {loop_variable} in range({loop_count}) {
        {variable_name3} ← {variable_name3} + 1
    }
}

return {variable_name3}
"""
            },
            {
                "name": "print_statements",
                "description": "Program with print statements",
                "content": """
// Program with print statements
{variable_name1} ← {number_value1}
⌽({variable_name1})

{variable_name2} ← {string_value1}
⌽({variable_name2})

{variable_name3} ← {variable_name1} {arithmetic_operator} {number_value2}
⌽({variable_name3})

return {variable_name3}
"""
            }
        ]
        
        return templates
    
    def _initialize_placeholder_generators(self) -> Dict[str, Any]:
        """Initialize placeholder generators.
        
        Returns:
            Dictionary of placeholder generators
        """
        return {
            "variable_name": self._generate_variable_name,
            "variable_name1": self._generate_variable_name,
            "variable_name2": self._generate_variable_name,
            "variable_name3": self._generate_variable_name,
            "function_name": self._generate_function_name,
            "function_name1": self._generate_function_name,
            "function_name2": self._generate_function_name,
            "parameter_name": self._generate_variable_name,
            "parameter_name1": self._generate_variable_name,
            "parameter_name2": self._generate_variable_name,
            "parameter_name3": self._generate_variable_name,
            "loop_variable": self._generate_variable_name,
            "loop_count": self._generate_loop_count,
            "array_index": self._generate_array_index,
            "value": self._generate_value,
            "number_value1": self._generate_number_value,
            "number_value2": self._generate_number_value,
            "string_value1": self._generate_string_value,
            "string_value2": self._generate_string_value,
            "array_value": self._generate_array_value,
            "true_value": self._generate_true_value,
            "false_value": self._generate_false_value,
            "arithmetic_operator": self._generate_arithmetic_operator,
            "comparison_operator": self._generate_comparison_operator
        }
    
    def _generate_from_template(self) -> Tuple[str, Dict[str, Any]]:
        """Generate content from a template.
        
        Returns:
            Tuple of (generated content, template info)
        """
        # Select a random template
        template = random.choice(self.templates)
        
        # Get the template content
        content = template["content"]
        
        # Find all placeholders
        placeholders = re.findall(r'\{([^}]+)\}', content)
        
        # Track replacements
        replacements = {}
        
        # Replace placeholders
        for placeholder in placeholders:
            if placeholder in self.placeholder_generators:
                # Get the generator function
                generator_func = self.placeholder_generators[placeholder]
                
                # Generate a value
                value = generator_func()
                
                # Replace the placeholder
                content = content.replace(f"{{{placeholder}}}", value)
                
                # Track the replacement
                replacements[placeholder] = value
            else:
                # Unknown placeholder, replace with a default value
                content = content.replace(f"{{{placeholder}}}", f"unknown_{placeholder}")
                
                # Track the replacement
                replacements[placeholder] = f"unknown_{placeholder}"
        
        # Maybe introduce errors
        if random.random() < self.error_probability:
            content = self._introduce_error(content)
        
        # Create template info
        template_info = {
            "name": template["name"],
            "description": template["description"],
            "replacements": replacements
        }
        
        return content, template_info
    
    def _introduce_error(self, content: str) -> str:
        """Introduce an error into the content.
        
        Args:
            content: Content to modify
            
        Returns:
            Modified content with an error
        """
        # Choose an error type
        error_type = random.choice([
            "remove_character",
            "insert_character",
            "replace_character",
            "swap_characters",
            "remove_line",
            "duplicate_line",
            "remove_bracket",
            "change_operator"
        ])
        
        # Apply the error
        if error_type == "remove_character" and content:
            # Remove a random character
            pos = random.randint(0, len(content) - 1)
            content = content[:pos] + content[pos+1:]
        
        elif error_type == "insert_character" and content:
            # Insert a random character
            pos = random.randint(0, len(content))
            char = random.choice(string.ascii_letters + string.digits + string.punctuation)
            content = content[:pos] + char + content[pos:]
        
        elif error_type == "replace_character" and content:
            # Replace a random character
            pos = random.randint(0, len(content) - 1)
            char = random.choice(string.ascii_letters + string.digits + string.punctuation)
            content = content[:pos] + char + content[pos+1:]
        
        elif error_type == "swap_characters" and len(content) >= 2:
            # Swap adjacent characters
            pos = random.randint(0, len(content) - 2)
            content = content[:pos] + content[pos+1] + content[pos] + content[pos+2:]
        
        elif error_type == "remove_line" and "\n" in content:
            # Remove a random line
            lines = content.split("\n")
            if len(lines) > 1:
                line_index = random.randint(0, len(lines) - 1)
                lines.pop(line_index)
                content = "\n".join(lines)
        
        elif error_type == "duplicate_line" and "\n" in content:
            # Duplicate a random line
            lines = content.split("\n")
            if lines:
                line_index = random.randint(0, len(lines) - 1)
                lines.insert(line_index, lines[line_index])
                content = "\n".join(lines)
        
        elif error_type == "remove_bracket" and any(c in content for c in "(){}[]"):
            # Remove a random bracket
            brackets = [i for i, c in enumerate(content) if c in "(){}[]"]
            if brackets:
                pos = random.choice(brackets)
                content = content[:pos] + content[pos+1:]
        
        elif error_type == "change_operator" and any(op in content for op in "+-*/=<>!&|"):
            # Change a random operator
            operators = [i for i, c in enumerate(content) if c in "+-*/=<>!&|"]
            if operators:
                pos = random.choice(operators)
                new_op = random.choice("+-*/=<>!&|")
                content = content[:pos] + new_op + content[pos+1:]
        
        return content
    
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
        
        # Generate a name
        name = random.choice(templates)
        
        # Maybe add a suffix
        if random.random() < 0.3:
            name += str(random.randint(1, 100))
        
        # Add to variables
        self.variables.add(name)
        
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
        
        # Generate a name
        name = random.choice(templates)
        
        # Maybe add a suffix
        if random.random() < 0.3:
            name += str(random.randint(1, 100))
        
        # Add to functions
        self.functions.add(name)
        
        return name
    
    def _generate_loop_count(self) -> str:
        """Generate a loop count.
        
        Returns:
            Generated loop count
        """
        return str(random.randint(1, 10))
    
    def _generate_array_index(self) -> str:
        """Generate an array index.
        
        Returns:
            Generated array index
        """
        return str(random.randint(0, 5))
    
    def _generate_value(self) -> str:
        """Generate a value.
        
        Returns:
            Generated value
        """
        # Choose a value type
        value_type = random.choice(["number", "string", "boolean", "array"])
        
        if value_type == "number":
            return self._generate_number_value()
        elif value_type == "string":
            return self._generate_string_value()
        elif value_type == "boolean":
            return self._generate_boolean_value()
        elif value_type == "array":
            return self._generate_array_value()
        else:
            return "0"
    
    def _generate_number_value(self) -> str:
        """Generate a number value.
        
        Returns:
            Generated number value
        """
        # Choose a number type
        number_type = random.choice(["integer", "float"])
        
        if number_type == "integer":
            return str(random.randint(-100, 100))
        else:
            return str(round(random.uniform(-100.0, 100.0), 2))
    
    def _generate_string_value(self) -> str:
        """Generate a string value.
        
        Returns:
            Generated string value
        """
        # Choose a string template
        templates = [
            "Hello",
            "World",
            "Test",
            "Fuzzing",
            "Anarchy",
            "Inference",
            "Template",
            "Generator",
            "String",
            "Value"
        ]
        
        # Generate a string
        value = random.choice(templates)
        
        # Maybe add a suffix
        if random.random() < 0.3:
            value += str(random.randint(1, 100))
        
        return f'"{value}"'
    
    def _generate_boolean_value(self) -> str:
        """Generate a boolean value.
        
        Returns:
            Generated boolean value
        """
        return random.choice(["⊤", "⊥"])
    
    def _generate_array_value(self) -> str:
        """Generate an array value.
        
        Returns:
            Generated array value
        """
        # Generate array elements
        num_elements = random.randint(0, 5)
        elements = []
        
        for _ in range(num_elements):
            element_type = random.choice(["number", "string", "boolean"])
            
            if element_type == "number":
                elements.append(self._generate_number_value())
            elif element_type == "string":
                elements.append(self._generate_string_value())
            elif element_type == "boolean":
                elements.append(self._generate_boolean_value())
        
        return f"[{', '.join(elements)}]"
    
    def _generate_true_value(self) -> str:
        """Generate a value for true conditions.
        
        Returns:
            Generated true value
        """
        # Choose a value type
        value_type = random.choice(["string", "number", "boolean"])
        
        if value_type == "string":
            return '"True"'
        elif value_type == "number":
            return "1"
        elif value_type == "boolean":
            return "⊤"
        else:
            return "⊤"
    
    def _generate_false_value(self) -> str:
        """Generate a value for false conditions.
        
        Returns:
            Generated false value
        """
        # Choose a value type
        value_type = random.choice(["string", "number", "boolean"])
        
        if value_type == "string":
            return '"False"'
        elif value_type == "number":
            return "0"
        elif value_type == "boolean":
            return "⊥"
        else:
            return "⊥"
    
    def _generate_arithmetic_operator(self) -> str:
        """Generate an arithmetic operator.
        
        Returns:
            Generated arithmetic operator
        """
        return random.choice(["+", "-", "*", "/", "%"])
    
    def _generate_comparison_operator(self) -> str:
        """Generate a comparison operator.
        
        Returns:
            Generated comparison operator
        """
        return random.choice(["==", "!=", "<", ">", "<=", ">="])


def main():
    """Main entry point for testing the template generator."""
    # Create a template generator
    generator = TemplateGenerator()
    
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
