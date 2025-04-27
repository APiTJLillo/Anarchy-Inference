#!/usr/bin/env python3
"""
Anarchy Inference Interpreter - Simple Prototype (Fixed Version)
This is a basic interpreter for the Anarchy Inference language that supports
the optimized syntax developed for token efficiency.
"""

import sys
import re
import json
import os
import requests
from datetime import datetime

class Interpreter:
    def __init__(self, source_code):
        self.source_code = source_code
        self.variables = {}
        self.functions = {}
        self.current_line = 0
        self.lines = source_code.strip().split('\n')
        
        # Built-in functions
        self.builtins = {
            'print': self._print,
            'read': self._read_file,
            'write': self._write_file,
            'append': self._append_file,
            'exists': self._file_exists,
            'readdir': self._read_dir,
            'get': self._http_get,
            'Date': {
                'now': self._date_now
            },
            'JSON': {
                'parse': self._json_parse,
                'stringify': self._json_stringify
            }
        }
        
        # Add builtins to global scope
        for name, func in self.builtins.items():
            self.variables[name] = func
    
    def run(self):
        """Run the Anarchy Inference program."""
        # Parse the program
        self._parse_program()
        
        # Check if main function exists
        if 'main' not in self.functions:
            raise ValueError("No main function found")
        
        # Call the main function
        return self._call_function('main', [])
    
    def _parse_program(self):
        """Parse the Anarchy Inference program."""
        # Skip comments and empty lines
        i = 0
        while i < len(self.lines):
            line = self.lines[i].strip()
            if line.startswith('//') or not line:
                i += 1
                continue
            
            # Check for module declaration
            if line.startswith('m{'):
                i = self._parse_module(i + 1)
            else:
                i += 1
    
    def _parse_module(self, start_line):
        """Parse a module declaration."""
        i = start_line
        while i < len(self.lines):
            line = self.lines[i].strip()
            if line.startswith('//') or not line:
                i += 1
                continue
            
            # Check for end of module
            if line == '}':
                return i + 1
            
            # Check for function declaration
            match = re.match(r'(\w+)\((.*?)\)\s*{', line)
            if match:
                func_name = match.group(1)
                params_str = match.group(2)
                params = [p.strip() for p in params_str.split(',')] if params_str else []
                
                i, body = self._parse_function_body(i + 1)
                self.functions[func_name] = {
                    'params': params,
                    'body': body
                }
            else:
                i += 1
        
        return i
    
    def _parse_function_body(self, start_line):
        """Parse a function body."""
        body = []
        i = start_line
        brace_count = 1  # We've already seen one opening brace
        
        while i < len(self.lines) and brace_count > 0:
            line = self.lines[i].strip()
            
            # Skip comments
            if line.startswith('//'):
                i += 1
                continue
            
            # Track braces to find the end of the function
            brace_count += line.count('{')
            brace_count -= line.count('}')
            
            # Add line to function body
            if brace_count > 0:  # Don't include the closing brace
                body.append(line)
            
            i += 1
        
        return i, body
    
    def _call_function(self, name, args):
        """Call a function by name with arguments."""
        # Check for built-in functions
        if name in self.builtins:
            return self.builtins[name](*args)
        
        # Check for user-defined functions
        if name not in self.functions:
            raise ValueError(f"Function {name} not found")
        
        func = self.functions[name]
        
        # Create a new scope for the function
        old_vars = self.variables.copy()
        
        # Bind parameters to arguments
        for i, param in enumerate(func['params']):
            if i < len(args):
                self.variables[param] = args[i]
            else:
                self.variables[param] = None  # Default value for missing arguments
        
        # Execute the function body
        result = None
        for line in func['body']:
            result = self._execute_line(line)
            if isinstance(result, dict) and result.get('__return__'):
                result = result.get('value')
                break
        
        # Restore the previous scope
        self.variables = old_vars
        
        return result
    
    def _execute_line(self, line):
        """Execute a single line of code."""
        line = line.strip()
        
        # Skip empty lines and comments
        if not line or line.startswith('//'):
            return None
        
        # Return statement
        if line.startswith('return '):
            value_str = line[7:].strip()
            if value_str.endswith(';'):
                value_str = value_str[:-1]
            
            value = self._evaluate_expression(value_str)
            return {'__return__': True, 'value': value}
        
        # If statement
        if line.startswith('if('):
            return self._execute_if_statement(line)
        
        # For loop
        if line.startswith('for('):
            return self._execute_for_loop(line)
        
        # While loop
        if line.startswith('while('):
            return self._execute_while_loop(line)
        
        # Try-catch block
        if line.startswith('try{'):
            return self._execute_try_catch(line)
        
        # Variable assignment or function call
        if '=' in line and not any(op in line for op in ['==', '!=', '<=', '>=', '=>']):
            return self._execute_assignment(line)
        
        # Function call
        if line.endswith(');'):
            return self._execute_function_call(line)
        
        return None
    
    def _execute_if_statement(self, line):
        """Execute an if statement."""
        # Extract condition
        match = re.match(r'if\((.*?)\)\s*{', line)
        if not match:
            raise ValueError(f"Invalid if statement: {line}")
        
        condition_str = match.group(1)
        condition = self._evaluate_expression(condition_str)
        
        # Find the body of the if statement
        # This is a simplified implementation that assumes proper formatting
        # A real interpreter would need to handle nested blocks properly
        if condition:
            # Execute the if block
            # For simplicity, we'll just return a placeholder
            return {'__if__': True, 'condition': condition}
        else:
            # Check for else block
            # For simplicity, we'll just return a placeholder
            return {'__if__': False, 'condition': condition}
    
    def _execute_for_loop(self, line):
        """Execute a for loop."""
        # Extract loop components
        match = re.match(r'for\((.*?);(.*?);(.*?)\)\s*{', line)
        if not match:
            raise ValueError(f"Invalid for loop: {line}")
        
        init_str = match.group(1)
        condition_str = match.group(2)
        update_str = match.group(3)
        
        # Initialize loop variable
        self._execute_assignment(init_str + ';')
        
        # Check condition
        condition = self._evaluate_expression(condition_str)
        
        # For simplicity, we'll just return a placeholder
        return {'__for__': True, 'condition': condition}
    
    def _execute_while_loop(self, line):
        """Execute a while loop."""
        # Extract condition
        match = re.match(r'while\((.*?)\)\s*{', line)
        if not match:
            raise ValueError(f"Invalid while loop: {line}")
        
        condition_str = match.group(1)
        condition = self._evaluate_expression(condition_str)
        
        # For simplicity, we'll just return a placeholder
        return {'__while__': True, 'condition': condition}
    
    def _execute_try_catch(self, line):
        """Execute a try-catch block."""
        # For simplicity, we'll just return a placeholder
        return {'__try__': True}
    
    def _execute_assignment(self, line):
        """Execute a variable assignment."""
        # Remove trailing semicolon
        if line.endswith(';'):
            line = line[:-1]
        
        # Split by assignment operator
        parts = line.split('=', 1)
        var_name = parts[0].strip()
        value_str = parts[1].strip()
        
        # Evaluate the expression
        value = self._evaluate_expression(value_str)
        
        # Assign to variable
        self.variables[var_name] = value
        
        return value
    
    def _execute_function_call(self, line):
        """Execute a function call."""
        # Remove trailing semicolon
        if line.endswith(';'):
            line = line[:-1]
        
        # Extract function name and arguments
        match = re.match(r'(\w+)\((.*?)\)', line)
        if not match:
            raise ValueError(f"Invalid function call: {line}")
        
        func_name = match.group(1)
        args_str = match.group(2)
        
        # Parse arguments
        args = []
        if args_str:
            # This is a simplified argument parser
            # A real interpreter would need to handle nested expressions properly
            args = [self._evaluate_expression(arg.strip()) for arg in args_str.split(',')]
        
        # Call the function
        return self._call_function(func_name, args)
    
    def _evaluate_expression(self, expr):
        """Evaluate an expression."""
        expr = expr.strip()
        
        # Null
        if expr == 'null':
            return None
        
        # Boolean
        if expr == 'true':
            return True
        if expr == 'false':
            return False
        
        # Number
        try:
            if '.' in expr:
                return float(expr)
            else:
                return int(expr)
        except ValueError:
            pass
        
        # String
        if (expr.startswith('"') and expr.endswith('"')) or (expr.startswith("'") and expr.endswith("'")):
            return expr[1:-1]
        
        # Array
        if expr.startswith('[') and expr.endswith(']'):
            items_str = expr[1:-1].strip()
            if not items_str:
                return []
            
            # This is a simplified array parser
            # A real interpreter would need to handle nested expressions properly
            items = []
            for item in items_str.split(','):
                items.append(self._evaluate_expression(item.strip()))
            return items
        
        # Object
        if expr.startswith('{') and expr.endswith('}'):
            props_str = expr[1:-1].strip()
            if not props_str:
                return {}
            
            # This is a simplified object parser
            # A real interpreter would need to handle nested expressions properly
            obj = {}
            props = props_str.split(',')
            for prop in props:
                key_value = prop.split(':', 1)
                if len(key_value) == 2:
                    key = key_value[0].strip()
                    value = self._evaluate_expression(key_value[1].strip())
                    
                    # Remove quotes from key if present
                    if (key.startswith('"') and key.endswith('"')) or (key.startswith("'") and key.endswith("'")):
                        key = key[1:-1]
                    
                    obj[key] = value
            
            return obj
        
        # Variable
        if expr in self.variables:
            return self.variables[expr]
        
        # Function call
        if '(' in expr and ')' in expr and not '+' in expr and not '-' in expr:
            return self._execute_function_call(expr)
        
        # String concatenation
        if '+' in expr and ('"' in expr or "'" in expr):
            parts = expr.split('+')
            result = ""
            for part in parts:
                part_value = self._evaluate_expression(part.strip())
                result += str(part_value)
            return result
        
        # Binary operation
        for op in ['+', '-', '*', '/', '%', '==', '!=', '<', '>', '<=', '>=', '&&', '||']:
            if op in expr:
                # Skip if this is part of a string
                if ('"' in expr or "'" in expr) and op == '+':
                    continue
                
                left_str, right_str = expr.split(op, 1)
                left = self._evaluate_expression(left_str.strip())
                right = self._evaluate_expression(right_str.strip())
                
                # Convert to same type for comparison
                if op in ['<', '>', '<=', '>='] and (isinstance(left, (int, float)) and isinstance(right, (int, float))):
                    left = float(left)
                    right = float(right)
                
                if op == '+':
                    return left + right
                elif op == '-':
                    return left - right
                elif op == '*':
                    return left * right
                elif op == '/':
                    return left / right
                elif op == '%':
                    return left % right
                elif op == '==':
                    return left == right
                elif op == '!=':
                    return left != right
                elif op == '<':
                    return left < right
                elif op == '>':
                    return left > right
                elif op == '<=':
                    return left <= right
                elif op == '>=':
                    return left >= right
                elif op == '&&':
                    return left and right
                elif op == '||':
                    return left or right
        
        # Property access
        if '.' in expr:
            parts = expr.split('.', 1)
            obj_name = parts[0].strip()
            prop_name = parts[1].strip()
            
            if obj_name in self.variables:
                obj = self.variables[obj_name]
                if isinstance(obj, dict) and prop_name in obj:
                    return obj[prop_name]
                elif hasattr(obj, prop_name):
                    return getattr(obj, prop_name)
                elif isinstance(obj, list) and prop_name == 'length':
                    return len(obj)
        
        # If we can't evaluate the expression, return it as is
        return expr
    
    # Built-in functions
    def _print(self, *args):
        """Print values to the console."""
        print(*args)
        return None
    
    def _read_file(self, path):
        """Read a file."""
        try:
            with open(path, 'r') as f:
                return f.read()
        except Exception as e:
            raise ValueError(f"Error reading file {path}: {e}")
    
    def _write_file(self, path, content):
        """Write to a file."""
        try:
            with open(path, 'w') as f:
                f.write(content)
            return True
        except Exception as e:
            raise ValueError(f"Error writing to file {path}: {e}")
    
    def _append_file(self, path, content):
        """Append to a file."""
        try:
            with open(path, 'a') as f:
                f.write(content)
            return True
        except Exception as e:
            raise ValueError(f"Error appending to file {path}: {e}")
    
    def _file_exists(self, path):
        """Check if a file exists."""
        return os.path.exists(path)
    
    def _read_dir(self, path):
        """List files in a directory."""
        try:
            return os.listdir(path)
        except Exception as e:
            raise ValueError(f"Error reading directory {path}: {e}")
    
    def _http_get(self, url, options=None):
        """Perform an HTTP GET request."""
        try:
            headers = {}
            if options and 'headers' in options:
                headers = options['headers']
            
            response = requests.get(url, headers=headers)
            
            return {
                'code': response.status_code,
                'body': response.text,
                'headers': dict(response.headers)
            }
        except Exception as e:
            raise ValueError(f"Error making GET request to {url}: {e}")
    
    def _date_now(self):
        """Get current timestamp."""
        return datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    
    def _json_parse(self, text):
        """Parse JSON."""
        try:
            return json.loads(text)
        except Exception as e:
            raise ValueError(f"Error parsing JSON: {e}")
    
    def _json_stringify(self, obj, indent=None):
        """Stringify JSON."""
        try:
            return json.dumps(obj, indent=indent)
        except Exception as e:
            raise ValueError(f"Error stringifying JSON: {e}")

def run_file(filename):
    """Run an Anarchy Inference file."""
    try:
        with open(filename, 'r') as f:
            source = f.read()
        
        interpreter = Interpreter(source)
        result = interpreter.run()
        
        return result
    except Exception as e:
        print(f"Error: {e}")
        return None

def main():
    """Main function."""
    if len(sys.argv) < 2:
        print("Usage: python anarchy_simple.py <filename>")
        return
    
    filename = sys.argv[1]
    result = run_file(filename)
    
    if result is not None:
        print(f"Program returned: {result}")

if __name__ == "__main__":
    main()
