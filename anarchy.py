#!/usr/bin/env python3
"""
Anarchy Inference Interpreter - Simple Prototype
This is a basic interpreter for the Anarchy Inference language that supports
the optimized syntax developed for token efficiency.
"""

import sys
import re
import json
import os
import requests
from enum import Enum

class TokenType(Enum):
    MODULE = 1
    FUNCTION = 2
    IDENTIFIER = 3
    NUMBER = 4
    STRING = 5
    OPERATOR = 6
    PUNCTUATION = 7
    KEYWORD = 8
    COMMENT = 9
    WHITESPACE = 10
    EOF = 11

class Token:
    def __init__(self, type, value, line, column):
        self.type = type
        self.value = value
        self.line = line
        self.column = column
    
    def __str__(self):
        return f"Token({self.type}, '{self.value}', line={self.line}, col={self.column})"

class Lexer:
    def __init__(self, source):
        self.source = source
        self.position = 0
        self.line = 1
        self.column = 1
        self.current_char = self.source[0] if self.source else None
    
    def advance(self):
        """Move to the next character in the source code."""
        self.position += 1
        if self.position >= len(self.source):
            self.current_char = None
        else:
            self.current_char = self.source[self.position]
            if self.current_char == '\n':
                self.line += 1
                self.column = 1
            else:
                self.column += 1
    
    def peek(self, n=1):
        """Look ahead n characters without advancing."""
        peek_pos = self.position + n
        if peek_pos >= len(self.source):
            return None
        return self.source[peek_pos]
    
    def skip_whitespace(self):
        """Skip whitespace characters."""
        while self.current_char is not None and self.current_char.isspace():
            self.advance()
    
    def skip_comment(self):
        """Skip comments."""
        # Line comment
        if self.current_char == '/' and self.peek() == '/':
            self.advance()  # Skip first /
            self.advance()  # Skip second /
            while self.current_char is not None and self.current_char != '\n':
                self.advance()
            if self.current_char == '\n':
                self.advance()
        # Block comment
        elif self.current_char == '/' and self.peek() == '*':
            self.advance()  # Skip /
            self.advance()  # Skip *
            while self.current_char is not None:
                if self.current_char == '*' and self.peek() == '/':
                    self.advance()  # Skip *
                    self.advance()  # Skip /
                    break
                self.advance()
    
    def number(self):
        """Parse a number token."""
        result = ''
        start_column = self.column
        
        # Handle decimal numbers
        while self.current_char is not None and (self.current_char.isdigit() or self.current_char == '.'):
            result += self.current_char
            self.advance()
        
        # Check for scientific notation
        if self.current_char in ['e', 'E']:
            result += self.current_char
            self.advance()
            if self.current_char in ['+', '-']:
                result += self.current_char
                self.advance()
            while self.current_char is not None and self.current_char.isdigit():
                result += self.current_char
                self.advance()
        
        if '.' in result:
            return Token(TokenType.NUMBER, float(result), self.line, start_column)
        else:
            return Token(TokenType.NUMBER, int(result), self.line, start_column)
    
    def string(self):
        """Parse a string token."""
        result = ''
        start_column = self.column
        quote_char = self.current_char  # Either ' or "
        self.advance()  # Skip the opening quote
        
        while self.current_char is not None and self.current_char != quote_char:
            if self.current_char == '\\':
                self.advance()  # Skip the backslash
                if self.current_char == 'n':
                    result += '\n'
                elif self.current_char == 't':
                    result += '\t'
                elif self.current_char == 'r':
                    result += '\r'
                elif self.current_char == '\\':
                    result += '\\'
                elif self.current_char == quote_char:
                    result += quote_char
                else:
                    result += '\\' + self.current_char
            else:
                result += self.current_char
            self.advance()
        
        self.advance()  # Skip the closing quote
        return Token(TokenType.STRING, result, self.line, start_column)
    
    def identifier(self):
        """Parse an identifier or keyword token."""
        result = ''
        start_column = self.column
        
        while self.current_char is not None and (self.current_char.isalnum() or self.current_char == '_'):
            result += self.current_char
            self.advance()
        
        # Check if it's a keyword
        keywords = ['if', 'else', 'for', 'while', 'return', 'try', 'catch', 'function', 'var', 'let', 'const', 'true', 'false', 'null', 'undefined', 'in', 'of']
        if result in keywords:
            return Token(TokenType.KEYWORD, result, self.line, start_column)
        
        return Token(TokenType.IDENTIFIER, result, self.line, start_column)
    
    def get_next_token(self):
        """Get the next token from the source code."""
        while self.current_char is not None:
            # Skip whitespace
            if self.current_char.isspace():
                self.skip_whitespace()
                continue
            
            # Skip comments
            if self.current_char == '/' and (self.peek() == '/' or self.peek() == '*'):
                self.skip_comment()
                continue
            
            # Module declaration
            if self.current_char == 'm' and self.peek() == '{':
                start_column = self.column
                self.advance()  # Skip 'm'
                return Token(TokenType.MODULE, 'm', self.line, start_column)
            
            # Numbers
            if self.current_char.isdigit() or (self.current_char == '.' and self.peek().isdigit()):
                return self.number()
            
            # Strings
            if self.current_char in ['"', "'"]:
                return self.string()
            
            # Identifiers and keywords
            if self.current_char.isalpha() or self.current_char == '_':
                return self.identifier()
            
            # Operators
            operators = ['+', '-', '*', '/', '%', '=', '!', '<', '>', '&', '|', '^', '~', '.']
            if self.current_char in operators:
                start_column = self.column
                operator = self.current_char
                self.advance()
                
                # Handle multi-character operators
                if operator == '=' and self.current_char == '=':
                    operator += self.current_char
                    self.advance()
                    if self.current_char == '=':
                        operator += self.current_char
                        self.advance()
                elif operator == '!' and self.current_char == '=':
                    operator += self.current_char
                    self.advance()
                    if self.current_char == '=':
                        operator += self.current_char
                        self.advance()
                elif operator == '<' and self.current_char == '=':
                    operator += self.current_char
                    self.advance()
                elif operator == '>' and self.current_char == '=':
                    operator += self.current_char
                    self.advance()
                elif operator == '&' and self.current_char == '&':
                    operator += self.current_char
                    self.advance()
                elif operator == '|' and self.current_char == '|':
                    operator += self.current_char
                    self.advance()
                elif operator == '+' and self.current_char == '+':
                    operator += self.current_char
                    self.advance()
                elif operator == '-' and self.current_char == '-':
                    operator += self.current_char
                    self.advance()
                elif operator == '+' and self.current_char == '=':
                    operator += self.current_char
                    self.advance()
                elif operator == '-' and self.current_char == '=':
                    operator += self.current_char
                    self.advance()
                elif operator == '*' and self.current_char == '=':
                    operator += self.current_char
                    self.advance()
                elif operator == '/' and self.current_char == '=':
                    operator += self.current_char
                    self.advance()
                
                return Token(TokenType.OPERATOR, operator, self.line, start_column)
            
            # Punctuation
            punctuation = ['{', '}', '(', ')', '[', ']', ',', ';', ':']
            if self.current_char in punctuation:
                token = Token(TokenType.PUNCTUATION, self.current_char, self.line, self.column)
                self.advance()
                return token
            
            # If we get here, we have an unrecognized character
            raise SyntaxError(f"Unexpected character: '{self.current_char}' at line {self.line}, column {self.column}")
        
        # End of file
        return Token(TokenType.EOF, None, self.line, self.column)

class Parser:
    def __init__(self, lexer):
        self.lexer = lexer
        self.current_token = self.lexer.get_next_token()
    
    def eat(self, token_type):
        """Consume the current token if it matches the expected type."""
        if self.current_token.type == token_type:
            token = self.current_token
            self.current_token = self.lexer.get_next_token()
            return token
        else:
            raise SyntaxError(f"Expected {token_type}, got {self.current_token.type} at line {self.current_token.line}, column {self.current_token.column}")
    
    def parse(self):
        """Parse the source code and return an AST."""
        return self.module()
    
    def module(self):
        """Parse a module declaration."""
        self.eat(TokenType.MODULE)
        self.eat(TokenType.PUNCTUATION)  # {
        
        functions = {}
        while self.current_token.type != TokenType.PUNCTUATION or self.current_token.value != '}':
            function = self.function()
            functions[function['name']] = function
        
        self.eat(TokenType.PUNCTUATION)  # }
        
        return {
            'type': 'module',
            'functions': functions
        }
    
    def function(self):
        """Parse a function declaration."""
        name_token = self.eat(TokenType.IDENTIFIER)
        self.eat(TokenType.PUNCTUATION)  # (
        
        params = []
        if self.current_token.type != TokenType.PUNCTUATION or self.current_token.value != ')':
            params = self.parameter_list()
        
        self.eat(TokenType.PUNCTUATION)  # )
        self.eat(TokenType.PUNCTUATION)  # {
        
        body = self.block()
        
        return {
            'type': 'function',
            'name': name_token.value,
            'params': params,
            'body': body
        }
    
    def parameter_list(self):
        """Parse a parameter list."""
        params = [self.eat(TokenType.IDENTIFIER).value]
        
        while self.current_token.type == TokenType.PUNCTUATION and self.current_token.value == ',':
            self.eat(TokenType.PUNCTUATION)  # ,
            params.append(self.eat(TokenType.IDENTIFIER).value)
        
        return params
    
    def block(self):
        """Parse a block of statements."""
        statements = []
        
        while self.current_token.type != TokenType.PUNCTUATION or self.current_token.value != '}':
            statements.append(self.statement())
        
        self.eat(TokenType.PUNCTUATION)  # }
        
        return statements
    
    def statement(self):
        """Parse a statement."""
        if self.current_token.type == TokenType.KEYWORD:
            if self.current_token.value == 'if':
                return self.if_statement()
            elif self.current_token.value == 'for':
                return self.for_statement()
            elif self.current_token.value == 'while':
                return self.while_statement()
            elif self.current_token.value == 'return':
                return self.return_statement()
            elif self.current_token.value == 'try':
                return self.try_statement()
        
        # Variable declaration or assignment
        if self.current_token.type == TokenType.IDENTIFIER:
            return self.assignment_or_call()
        
        raise SyntaxError(f"Unexpected token in statement: {self.current_token}")
    
    def if_statement(self):
        """Parse an if statement."""
        self.eat(TokenType.KEYWORD)  # if
        self.eat(TokenType.PUNCTUATION)  # (
        condition = self.expression()
        self.eat(TokenType.PUNCTUATION)  # )
        self.eat(TokenType.PUNCTUATION)  # {
        
        if_body = self.block()
        
        else_body = None
        if self.current_token.type == TokenType.KEYWORD and self.current_token.value == 'else':
            self.eat(TokenType.KEYWORD)  # else
            
            if self.current_token.type == TokenType.KEYWORD and self.current_token.value == 'if':
                else_body = [self.if_statement()]
            else:
                self.eat(TokenType.PUNCTUATION)  # {
                else_body = self.block()
        
        return {
            'type': 'if',
            'condition': condition,
            'if_body': if_body,
            'else_body': else_body
        }
    
    def for_statement(self):
        """Parse a for statement."""
        self.eat(TokenType.KEYWORD)  # for
        self.eat(TokenType.PUNCTUATION)  # (
        
        # Check for different types of for loops
        if self.current_token.type == TokenType.IDENTIFIER:
            init_token = self.current_token
            self.eat(TokenType.IDENTIFIER)
            
            # for-in loop: for(key in object)
            if self.current_token.type == TokenType.KEYWORD and self.current_token.value == 'in':
                self.eat(TokenType.KEYWORD)  # in
                object_expr = self.expression()
                self.eat(TokenType.PUNCTUATION)  # )
                self.eat(TokenType.PUNCTUATION)  # {
                body = self.block()
                
                return {
                    'type': 'for_in',
                    'key': init_token.value,
                    'object': object_expr,
                    'body': body
                }
            
            # for-of loop: for(item of array)
            elif self.current_token.type == TokenType.KEYWORD and self.current_token.value == 'of':
                self.eat(TokenType.KEYWORD)  # of
                array_expr = self.expression()
                self.eat(TokenType.PUNCTUATION)  # )
                self.eat(TokenType.PUNCTUATION)  # {
                body = self.block()
                
                return {
                    'type': 'for_of',
                    'item': init_token.value,
                    'array': array_expr,
                    'body': body
                }
            
            # Standard for loop: for(i=0; i<10; i++)
            else:
                # Put back the identifier token
                self.lexer.position -= len(init_token.value)
                self.current_token = self.lexer.get_next_token()
        
        # Standard for loop
        init = self.expression()
        self.eat(TokenType.PUNCTUATION)  # ;
        condition = self.expression()
        self.eat(TokenType.PUNCTUATION)  # ;
        update = self.expression()
        self.eat(TokenType.PUNCTUATION)  # )
        self.eat(TokenType.PUNCTUATION)  # {
        
        body = self.block()
        
        return {
            'type': 'for',
            'init': init,
            'condition': condition,
            'update': update,
            'body': body
        }
    
    def while_statement(self):
        """Parse a while statement."""
        self.eat(TokenType.KEYWORD)  # while
        self.eat(TokenType.PUNCTUATION)  # (
        condition = self.expression()
        self.eat(TokenType.PUNCTUATION)  # )
        self.eat(TokenType.PUNCTUATION)  # {
        
        body = self.block()
        
        return {
            'type': 'while',
            'condition': condition,
            'body': body
        }
    
    def return_statement(self):
        """Parse a return statement."""
        self.eat(TokenType.KEYWORD)  # return
        
        value = None
        if self.current_token.type != TokenType.PUNCTUATION or self.current_token.value != ';':
            value = self.expression()
        
        self.eat(TokenType.PUNCTUATION)  # ;
        
        return {
            'type': 'return',
            'value': value
        }
    
    def try_statement(self):
        """Parse a try-catch statement."""
        self.eat(TokenType.KEYWORD)  # try
        self.eat(TokenType.PUNCTUATION)  # {
        
        try_body = self.block()
        
        self.eat(TokenType.KEYWORD)  # catch
        
        catch_param = None
        if self.current_token.type == TokenType.PUNCTUATION and self.current_token.value == '(':
            self.eat(TokenType.PUNCTUATION)  # (
            catch_param = self.eat(TokenType.IDENTIFIER).value
            self.eat(TokenType.PUNCTUATION)  # )
        
        self.eat(TokenType.PUNCTUATION)  # {
        catch_body = self.block()
        
        return {
            'type': 'try_catch',
            'try_body': try_body,
            'catch_param': catch_param,
            'catch_body': catch_body
        }
    
    def assignment_or_call(self):
        """Parse an assignment or function call."""
        identifier = self.eat(TokenType.IDENTIFIER).value
        
        # Function call
        if self.current_token.type == TokenType.PUNCTUATION and self.current_token.value == '(':
            self.eat(TokenType.PUNCTUATION)  # (
            
            args = []
            if self.current_token.type != TokenType.PUNCTUATION or self.current_token.value != ')':
                args = self.argument_list()
            
            self.eat(TokenType.PUNCTUATION)  # )
            self.eat(TokenType.PUNCTUATION)  # ;
            
            return {
                'type': 'call',
                'function': identifier,
                'args': args
            }
        
        # Assignment
        elif self.current_token.type == TokenType.OPERATOR and self.current_token.value == '=':
            self.eat(TokenType.OPERATOR)  # =
            value = self.expression()
            self.eat(TokenType.PUNCTUATION)  # ;
            
            return {
                'type': 'assignment',
                'variable': identifier,
                'value': value
            }
        
        # Compound assignment
        elif self.current_token.type == TokenType.OPERATOR and self.current_token.value in ['+=', '-=', '*=', '/=']:
            operator = self.eat(TokenType.OPERATOR).value
            value = self.expression()
            self.eat(TokenType.PUNCTUATION)  # ;
            
            return {
                'type': 'compound_assignment',
                'variable': identifier,
                'operator': operator,
                'value': value
            }
        
        # Property access or method call
        elif self.current_token.type == TokenType.OPERATOR and self.current_token.value == '.':
            return self.property_access(identifier)
        
        raise SyntaxError(f"Unexpected token after identifier: {self.current_token}")
    
    def property_access(self, object_name):
        """Parse property access or method call."""
        self.eat(TokenType.OPERATOR)  # .
        property_name = self.eat(TokenType.IDENTIFIER).value
        
        # Method call
        if self.current_token.type == TokenType.PUNCTUATION and self.current_token.value == '(':
            self.eat(TokenType.PUNCTUATION)  # (
            
            args = []
            if self.current_token.type != TokenType.PUNCTUATION or self.current_token.value != ')':
                args = self.argument_list()
            
            self.eat(TokenType.PUNCTUATION)  # )
            self.eat(TokenType.PUNCTUATION)  # ;
            
            return {
                'type': 'method_call',
                'object': object_name,
                'method': property_name,
                'args': args
            }
        
        # Property access
        else:
            self.eat(TokenType.PUNCTUATION)  # ;
            
            return {
                'type': 'property_access',
                'object': object_name,
                'property': property_name
            }
    
    def argument_list(self):
        """Parse an argument list."""
        args = [self.expression()]
        
        while self.current_token.type == TokenType.PUNCTUATION and self.current_token.value == ',':
            self.eat(TokenType.PUNCTUATION)  # ,
            args.append(self.expression())
        
        return args
    
    def expression(self):
        """Parse an expression."""
        return self.logical_or()
    
    def logical_or(self):
        """Parse a logical OR expression."""
        node = self.logical_and()
        
        while self.current_token.type == TokenType.OPERATOR and self.current_token.value == '||':
            operator = self.eat(TokenType.OPERATOR).value
            right = self.logical_and()
            
            node = {
                'type': 'binary_op',
                'operator': operator,
                'left': node,
                'right': right
            }
        
        return node
    
    def logical_and(self):
        """Parse a logical AND expression."""
        node = self.equality()
        
        while self.current_token.type == TokenType.OPERATOR and self.current_token.value == '&&':
            operator = self.eat(TokenType.OPERATOR).value
            right = self.equality()
            
            node = {
                'type': 'binary_op',
                'operator': operator,
                'left': node,
                'right': right
            }
        
        return node
    
    def equality(self):
        """Parse an equality expression."""
        node = self.relational()
        
        while self.current_token.type == TokenType.OPERATOR and self.current_token.value in ['==', '!=', '===', '!==']:
            operator = self.eat(TokenType.OPERATOR).value
            right = self.relational()
            
            node = {
                'type': 'binary_op',
                'operator': operator,
                'left': node,
                'right': right
            }
        
        return node
    
    def relational(self):
        """Parse a relational expression."""
        node = self.additive()
        
        while self.current_token.type == TokenType.OPERATOR and self.current_token.value in ['<', '>', '<=', '>=']:
            operator = self.eat(TokenType.OPERATOR).value
            right = self.additive()
            
            node = {
                'type': 'binary_op',
                'operator': operator,
                'left': node,
                'right': right
            }
        
        return node
    
    def additive(self):
        """Parse an additive expression."""
        node = self.multiplicative()
        
        while self.current_token.type == TokenType.OPERATOR and self.current_token.value in ['+', '-']:
            operator = self.eat(TokenType.OPERATOR).value
            right = self.multiplicative()
            
            node = {
                'type': 'binary_op',
                'operator': operator,
                'left': node,
                'right': right
            }
        
        return node
    
    def multiplicative(self):
        """Parse a multiplicative expression."""
        node = self.unary()
        
        while self.current_token.type == TokenType.OPERATOR and self.current_token.value in ['*', '/', '%']:
            operator = self.eat(TokenType.OPERATOR).value
            right = self.unary()
            
            node = {
                'type': 'binary_op',
                'operator': operator,
                'left': node,
                'right': right
            }
        
        return node
    
    def unary(self):
        """Parse a unary expression."""
        if self.current_token.type == TokenType.OPERATOR and self.current_token.value in ['+', '-', '!']:
            operator = self.eat(TokenType.OPERATOR).value
            operand = self.unary()
            
            return {
                'type': 'unary_op',
                'operator': operator,
                'operand': operand
            }
        
        return self.primary()
    
    def primary(self):
        """Parse a primary expression."""
        if self.current_token.type == TokenType.NUMBER:
            return {
                'type': 'number',
                'value': self.eat(TokenType.NUMBER).value
            }
        
        elif self.current_token.type == TokenType.STRING:
            return {
                'type': 'string',
                'value': self.eat(TokenType.STRING).value
            }
        
        elif self.current_token.type == TokenType.KEYWORD and self.current_token.value in ['true', 'false']:
            return {
                'type': 'boolean',
                'value': self.eat(TokenType.KEYWORD).value == 'true'
            }
        
        elif self.current_token.type == TokenType.KEYWORD and self.current_token.value == 'null':
            self.eat(TokenType.KEYWORD)
            return {
                'type': 'null',
                'value': None
            }
        
        elif self.current_token.type == TokenType.IDENTIFIER:
            return {
                'type': 'variable',
                'name': self.eat(TokenType.IDENTIFIER).value
            }
        
        elif self.current_token.type == TokenType.PUNCTUATION and self.current_token.value == '(':
            self.eat(TokenType.PUNCTUATION)  # (
            expr = self.expression()
            self.eat(TokenType.PUNCTUATION)  # )
            return expr
        
        elif self.current_token.type == TokenType.PUNCTUATION and self.current_token.value == '[':
            return self.array_literal()
        
        elif self.current_token.type == TokenType.PUNCTUATION and self.current_token.value == '{':
            return self.object_literal()
        
        raise SyntaxError(f"Unexpected token in expression: {self.current_token}")
    
    def array_literal(self):
        """Parse an array literal."""
        self.eat(TokenType.PUNCTUATION)  # [
        
        elements = []
        if self.current_token.type != TokenType.PUNCTUATION or self.current_token.value != ']':
            elements = self.argument_list()
        
        self.eat(TokenType.PUNCTUATION)  # ]
        
        return {
            'type': 'array',
            'elements': elements
        }
    
    def object_literal(self):
        """Parse an object literal."""
        self.eat(TokenType.PUNCTUATION)  # {
        
        properties = {}
        if self.current_token.type != TokenType.PUNCTUATION or self.current_token.value != '}':
            while True:
                key = None
                
                # Property key can be an identifier or a string
                if self.current_token.type == TokenType.IDENTIFIER:
                    key = self.eat(TokenType.IDENTIFIER).value
                elif self.current_token.type == TokenType.STRING:
                    key = self.eat(TokenType.STRING).value
                else:
                    raise SyntaxError(f"Expected property name, got {self.current_token}")
                
                self.eat(TokenType.PUNCTUATION)  # :
                value = self.expression()
                
                properties[key] = value
                
                if self.current_token.type != TokenType.PUNCTUATION or self.current_token.value != ',':
                    break
                
                self.eat(TokenType.PUNCTUATION)  # ,
        
        self.eat(TokenType.PUNCTUATION)  # }
        
        return {
            'type': 'object',
            'properties': properties
        }

class Interpreter:
    def __init__(self, ast):
        self.ast = ast
        self.global_scope = {}
        self.current_scope = self.global_scope
        
        # Built-in functions
        self.builtins = {
            'print': self.builtin_print,
            'read': self.builtin_read,
            'write': self.builtin_write,
            'append': self.builtin_append,
            'exists': self.builtin_exists,
            'readdir': self.builtin_readdir,
            'get': self.builtin_get,
            'regex': self.builtin_regex,
            'Number': self.builtin_number,
            'String': self.builtin_string,
            'JSON': {
                'parse': self.builtin_json_parse,
                'stringify': self.builtin_json_stringify
            }
        }
        
        # Add builtins to global scope
        for name, func in self.builtins.items():
            self.global_scope[name] = func
    
    def interpret(self):
        """Interpret the AST and execute the program."""
        if self.ast['type'] != 'module':
            raise ValueError("Expected a module AST")
        
        # Register all functions in the global scope
        for name, func in self.ast['functions'].items():
            self.global_scope[name] = func
        
        # Check if main function exists
        if 'main' not in self.global_scope:
            raise ValueError("No main function found")
        
        # Call the main function
        return self.call_function('main', [])
    
    def call_function(self, name, args):
        """Call a function by name with arguments."""
        if name not in self.current_scope:
            raise ValueError(f"Function {name} not found")
        
        func = self.current_scope[name]
        
        # Handle built-in functions
        if callable(func):
            return func(*args)
        
        # Handle user-defined functions
        if func['type'] != 'function':
            raise ValueError(f"{name} is not a function")
        
        # Create a new scope for the function
        old_scope = self.current_scope
        self.current_scope = {**old_scope}  # Copy the parent scope
        
        # Bind parameters to arguments
        for i, param in enumerate(func['params']):
            if i < len(args):
                self.current_scope[param] = args[i]
            else:
                self.current_scope[param] = None  # Default value for missing arguments
        
        # Execute the function body
        result = None
        try:
            for statement in func['body']:
                result = self.execute_statement(statement)
                if isinstance(result, dict) and result.get('type') == 'return':
                    result = result.get('value')
                    break
        except Exception as e:
            # Restore the scope
            self.current_scope = old_scope
            raise e
        
        # Restore the scope
        self.current_scope = old_scope
        
        return result
    
    def execute_statement(self, statement):
        """Execute a statement."""
        if statement['type'] == 'assignment':
            return self.execute_assignment(statement)
        elif statement['type'] == 'compound_assignment':
            return self.execute_compound_assignment(statement)
        elif statement['type'] == 'call':
            return self.execute_call(statement)
        elif statement['type'] == 'method_call':
            return self.execute_method_call(statement)
        elif statement['type'] == 'if':
            return self.execute_if(statement)
        elif statement['type'] == 'for':
            return self.execute_for(statement)
        elif statement['type'] == 'for_in':
            return self.execute_for_in(statement)
        elif statement['type'] == 'for_of':
            return self.execute_for_of(statement)
        elif statement['type'] == 'while':
            return self.execute_while(statement)
        elif statement['type'] == 'return':
            return self.execute_return(statement)
        elif statement['type'] == 'try_catch':
            return self.execute_try_catch(statement)
        else:
            raise ValueError(f"Unknown statement type: {statement['type']}")
    
    def execute_assignment(self, statement):
        """Execute an assignment statement."""
        value = self.evaluate_expression(statement['value'])
        self.current_scope[statement['variable']] = value
        return value
    
    def execute_compound_assignment(self, statement):
        """Execute a compound assignment statement."""
        variable = statement['variable']
        operator = statement['operator']
        value = self.evaluate_expression(statement['value'])
        
        if variable not in self.current_scope:
            raise ValueError(f"Variable {variable} not defined")
        
        current_value = self.current_scope[variable]
        
        if operator == '+=':
            result = current_value + value
        elif operator == '-=':
            result = current_value - value
        elif operator == '*=':
            result = current_value * value
        elif operator == '/=':
            result = current_value / value
        else:
            raise ValueError(f"Unknown compound assignment operator: {operator}")
        
        self.current_scope[variable] = result
        return result
    
    def execute_call(self, statement):
        """Execute a function call statement."""
        args = [self.evaluate_expression(arg) for arg in statement['args']]
        return self.call_function(statement['function'], args)
    
    def execute_method_call(self, statement):
        """Execute a method call statement."""
        obj_name = statement['object']
        method_name = statement['method']
        args = [self.evaluate_expression(arg) for arg in statement['args']]
        
        if obj_name not in self.current_scope:
            raise ValueError(f"Object {obj_name} not defined")
        
        obj = self.current_scope[obj_name]
        
        if not isinstance(obj, dict) or method_name not in obj:
            raise ValueError(f"Method {method_name} not found on object {obj_name}")
        
        method = obj[method_name]
        
        if not callable(method):
            raise ValueError(f"{obj_name}.{method_name} is not a function")
        
        return method(*args)
    
    def execute_if(self, statement):
        """Execute an if statement."""
        condition = self.evaluate_expression(statement['condition'])
        
        if condition:
            for stmt in statement['if_body']:
                result = self.execute_statement(stmt)
                if isinstance(result, dict) and result.get('type') == 'return':
                    return result
        elif statement['else_body']:
            for stmt in statement['else_body']:
                result = self.execute_statement(stmt)
                if isinstance(result, dict) and result.get('type') == 'return':
                    return result
        
        return None
    
    def execute_for(self, statement):
        """Execute a for statement."""
        # Initialize
        self.evaluate_expression(statement['init'])
        
        # Loop
        while self.evaluate_expression(statement['condition']):
            # Execute body
            for stmt in statement['body']:
                result = self.execute_statement(stmt)
                if isinstance(result, dict) and result.get('type') == 'return':
                    return result
            
            # Update
            self.evaluate_expression(statement['update'])
        
        return None
    
    def execute_for_in(self, statement):
        """Execute a for-in statement."""
        key = statement['key']
        obj = self.evaluate_expression(statement['object'])
        
        if not isinstance(obj, dict):
            raise ValueError("for-in requires an object")
        
        for k in obj:
            self.current_scope[key] = k
            
            for stmt in statement['body']:
                result = self.execute_statement(stmt)
                if isinstance(result, dict) and result.get('type') == 'return':
                    return result
        
        return None
    
    def execute_for_of(self, statement):
        """Execute a for-of statement."""
        item = statement['item']
        array = self.evaluate_expression(statement['array'])
        
        if not isinstance(array, list):
            raise ValueError("for-of requires an array")
        
        for value in array:
            self.current_scope[item] = value
            
            for stmt in statement['body']:
                result = self.execute_statement(stmt)
                if isinstance(result, dict) and result.get('type') == 'return':
                    return result
        
        return None
    
    def execute_while(self, statement):
        """Execute a while statement."""
        while self.evaluate_expression(statement['condition']):
            for stmt in statement['body']:
                result = self.execute_statement(stmt)
                if isinstance(result, dict) and result.get('type') == 'return':
                    return result
        
        return None
    
    def execute_return(self, statement):
        """Execute a return statement."""
        value = None
        if statement['value']:
            value = self.evaluate_expression(statement['value'])
        
        return {
            'type': 'return',
            'value': value
        }
    
    def execute_try_catch(self, statement):
        """Execute a try-catch statement."""
        try:
            for stmt in statement['try_body']:
                result = self.execute_statement(stmt)
                if isinstance(result, dict) and result.get('type') == 'return':
                    return result
        except Exception as e:
            # Bind the exception to the catch parameter if provided
            if statement['catch_param']:
                self.current_scope[statement['catch_param']] = str(e)
            
            for stmt in statement['catch_body']:
                result = self.execute_statement(stmt)
                if isinstance(result, dict) and result.get('type') == 'return':
                    return result
        
        return None
    
    def evaluate_expression(self, expr):
        """Evaluate an expression."""
        if expr['type'] == 'number':
            return expr['value']
        elif expr['type'] == 'string':
            return expr['value']
        elif expr['type'] == 'boolean':
            return expr['value']
        elif expr['type'] == 'null':
            return None
        elif expr['type'] == 'variable':
            if expr['name'] not in self.current_scope:
                raise ValueError(f"Variable {expr['name']} not defined")
            return self.current_scope[expr['name']]
        elif expr['type'] == 'binary_op':
            return self.evaluate_binary_op(expr)
        elif expr['type'] == 'unary_op':
            return self.evaluate_unary_op(expr)
        elif expr['type'] == 'array':
            return [self.evaluate_expression(element) for element in expr['elements']]
        elif expr['type'] == 'object':
            return {key: self.evaluate_expression(value) for key, value in expr['properties'].items()}
        elif expr['type'] == 'call':
            return self.execute_call(expr)
        elif expr['type'] == 'method_call':
            return self.execute_method_call(expr)
        elif expr['type'] == 'property_access':
            return self.evaluate_property_access(expr)
        else:
            raise ValueError(f"Unknown expression type: {expr['type']}")
    
    def evaluate_binary_op(self, expr):
        """Evaluate a binary operation."""
        left = self.evaluate_expression(expr['left'])
        right = self.evaluate_expression(expr['right'])
        
        if expr['operator'] == '+':
            return left + right
        elif expr['operator'] == '-':
            return left - right
        elif expr['operator'] == '*':
            return left * right
        elif expr['operator'] == '/':
            return left / right
        elif expr['operator'] == '%':
            return left % right
        elif expr['operator'] == '==':
            return left == right
        elif expr['operator'] == '!=':
            return left != right
        elif expr['operator'] == '===':
            return left == right and type(left) == type(right)
        elif expr['operator'] == '!==':
            return left != right or type(left) != type(right)
        elif expr['operator'] == '<':
            return left < right
        elif expr['operator'] == '>':
            return left > right
        elif expr['operator'] == '<=':
            return left <= right
        elif expr['operator'] == '>=':
            return left >= right
        elif expr['operator'] == '&&':
            return left and right
        elif expr['operator'] == '||':
            return left or right
        else:
            raise ValueError(f"Unknown binary operator: {expr['operator']}")
    
    def evaluate_unary_op(self, expr):
        """Evaluate a unary operation."""
        operand = self.evaluate_expression(expr['operand'])
        
        if expr['operator'] == '+':
            return +operand
        elif expr['operator'] == '-':
            return -operand
        elif expr['operator'] == '!':
            return not operand
        else:
            raise ValueError(f"Unknown unary operator: {expr['operator']}")
    
    def evaluate_property_access(self, expr):
        """Evaluate property access."""
        obj_name = expr['object']
        property_name = expr['property']
        
        if obj_name not in self.current_scope:
            raise ValueError(f"Object {obj_name} not defined")
        
        obj = self.current_scope[obj_name]
        
        if not isinstance(obj, dict) or property_name not in obj:
            raise ValueError(f"Property {property_name} not found on object {obj_name}")
        
        return obj[property_name]
    
    # Built-in functions
    def builtin_print(self, *args):
        """Print values to the console."""
        print(*args)
        return None
    
    def builtin_read(self, path):
        """Read a file."""
        try:
            with open(path, 'r') as f:
                return f.read()
        except Exception as e:
            raise ValueError(f"Error reading file {path}: {e}")
    
    def builtin_write(self, path, content):
        """Write to a file."""
        try:
            with open(path, 'w') as f:
                f.write(content)
            return True
        except Exception as e:
            raise ValueError(f"Error writing to file {path}: {e}")
    
    def builtin_append(self, path, content):
        """Append to a file."""
        try:
            with open(path, 'a') as f:
                f.write(content)
            return True
        except Exception as e:
            raise ValueError(f"Error appending to file {path}: {e}")
    
    def builtin_exists(self, path):
        """Check if a file exists."""
        return os.path.exists(path)
    
    def builtin_readdir(self, path):
        """List files in a directory."""
        try:
            return os.listdir(path)
        except Exception as e:
            raise ValueError(f"Error reading directory {path}: {e}")
    
    def builtin_get(self, url, options=None):
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
    
    def builtin_regex(self, pattern, flags=0):
        """Create a regular expression."""
        try:
            matches = re.findall(pattern, flags)
            return matches
        except Exception as e:
            raise ValueError(f"Error in regex: {e}")
    
    def builtin_number(self, value):
        """Convert a value to a number."""
        try:
            return float(value)
        except:
            return 0
    
    def builtin_string(self, value):
        """Convert a value to a string."""
        return str(value)
    
    def builtin_json_parse(self, text):
        """Parse JSON."""
        try:
            return json.loads(text)
        except Exception as e:
            raise ValueError(f"Error parsing JSON: {e}")
    
    def builtin_json_stringify(self, obj, indent=None):
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
        
        lexer = Lexer(source)
        parser = Parser(lexer)
        ast = parser.parse()
        
        interpreter = Interpreter(ast)
        result = interpreter.interpret()
        
        return result
    except Exception as e:
        print(f"Error: {e}")
        return None

def main():
    """Main function."""
    if len(sys.argv) < 2:
        print("Usage: python anarchy.py <filename>")
        return
    
    filename = sys.argv[1]
    result = run_file(filename)
    
    if result is not None:
        print(f"Program returned: {result}")

if __name__ == "__main__":
    main()
