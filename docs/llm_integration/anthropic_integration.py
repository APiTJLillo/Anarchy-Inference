#!/usr/bin/env python3
"""
Anthropic Claude Integration Example for Anarchy Inference

This module demonstrates how to integrate Anarchy Inference with Anthropic's Claude API,
allowing for code generation, execution, and token efficiency analysis.
"""

import os
import sys
import json
import time
import argparse
from typing import Dict, Any, List, Optional, Union

# Add parent directory to path to import anarchy module
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

try:
    import anarchy
except ImportError:
    print("Error: Could not import anarchy module. Make sure it's in the parent directory.")
    sys.exit(1)

try:
    import anthropic
except ImportError:
    print("Installing Anthropic Python package...")
    import subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "anthropic"])
    import anthropic

class AnarchyClaudeIntegration:
    """Integration between Anarchy Inference and Anthropic's Claude API."""
    
    def __init__(self, api_key: Optional[str] = None, model: str = "claude-3-opus-20240229"):
        """
        Initialize the integration with Anthropic Claude API.
        
        Args:
            api_key: Anthropic API key (defaults to ANTHROPIC_API_KEY environment variable)
            model: Anthropic model to use (defaults to claude-3-opus-20240229)
        """
        self.api_key = api_key or os.environ.get("ANTHROPIC_API_KEY")
        if not self.api_key:
            raise ValueError("Anthropic API key is required. Set ANTHROPIC_API_KEY environment variable or pass api_key parameter.")
        
        self.model = model
        self.client = anthropic.Anthropic(api_key=self.api_key)
        self.interpreter = anarchy.Interpreter()
        
        # Load the Anarchy Inference template for Claude
        template_path = os.path.join(
            os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
            "llm_templates",
            "anthropic_claude_template.md"
        )
        
        try:
            with open(template_path, 'r') as f:
                template_content = f.read()
                
            # Extract system message and user template from the Claude template
            # Claude uses XML-style tags for structure
            self.system_message = self._extract_between_tags(template_content, "system_prompt")
            self.user_template = self._extract_between_tags(template_content, "user_prompt")
            
        except (FileNotFoundError, ValueError) as e:
            print(f"Warning: Could not load template: {e}")
            # Fallback template
            self.system_message = """You are an expert Anarchy Inference programmer. Anarchy Inference is a token-minimal programming language designed specifically for LLMs, achieving approximately 24% token reduction compared to Python."""
            self.user_template = """<task>
Write Anarchy Inference code for the following task:

[TASK_DESCRIPTION]
</task>

Please provide:
<anarchy_inference_code>
# Your code here
</anarchy_inference_code>

<explanation>
Brief explanation of key parts and token efficiency choices
</explanation>

<token_analysis>
Token count comparison with equivalent Python
</token_analysis>"""
    
    def generate_anarchy_code(self, task_description: str, temperature: float = 0.3) -> Dict[str, Any]:
        """
        Generate Anarchy Inference code for a given task using Claude.
        
        Args:
            task_description: Description of the programming task
            temperature: Creativity parameter (0.0-1.0)
            
        Returns:
            Dictionary containing the generated code and metadata
        """
        user_message = self.user_template.replace("[TASK_DESCRIPTION]", task_description)
        
        start_time = time.time()
        response = self.client.messages.create(
            model=self.model,
            system=self.system_message,
            messages=[
                {"role": "user", "content": user_message}
            ],
            temperature=temperature,
            max_tokens=2000
        )
        generation_time = time.time() - start_time
        
        generated_content = response.content[0].text
        
        # Extract code blocks using XML tags
        anarchy_code = self._extract_between_tags(generated_content, "anarchy_inference_code")
        if not anarchy_code:
            # Fallback to markdown code blocks if XML tags aren't used
            code_blocks = self._extract_code_blocks(generated_content)
            anarchy_code = code_blocks[0] if code_blocks else generated_content
        
        # Count tokens
        anarchy_tokens = self.interpreter.count_tokens(anarchy_code)
        
        # Try to extract Python equivalent if mentioned
        python_code = self._extract_between_tags(generated_content, "python_equivalent")
        if not python_code:
            # Try to find Python code block
            python_blocks = [block for block in self._extract_code_blocks(generated_content) 
                            if block.strip().startswith("def ") or block.strip().startswith("import ")]
            python_code = python_blocks[0] if python_blocks else None
        
        python_tokens = self.interpreter.count_tokens(python_code) if python_code else None
        
        # Calculate token efficiency if possible
        token_efficiency = None
        if python_tokens:
            token_efficiency = (python_tokens - anarchy_tokens) / python_tokens
        
        # Extract token analysis if available
        token_analysis = self._extract_between_tags(generated_content, "token_analysis")
        
        return {
            "anarchy_code": anarchy_code,
            "python_code": python_code,
            "anarchy_tokens": anarchy_tokens,
            "python_tokens": python_tokens,
            "token_efficiency": token_efficiency,
            "token_analysis": token_analysis,
            "generation_time": generation_time,
            "model": self.model,
            "full_response": generated_content
        }
    
    def execute_generated_code(self, code: str) -> Dict[str, Any]:
        """
        Execute generated Anarchy Inference code and return results.
        
        Args:
            code: Anarchy Inference code to execute
            
        Returns:
            Dictionary containing execution results and metadata
        """
        start_time = time.time()
        try:
            result = self.interpreter.execute(code)
            execution_time = time.time() - start_time
            
            return {
                "success": True,
                "result": result,
                "execution_time": execution_time,
                "error": None
            }
        except Exception as e:
            execution_time = time.time() - start_time
            
            return {
                "success": False,
                "result": None,
                "execution_time": execution_time,
                "error": str(e)
            }
    
    def optimize_code(self, code: str, optimization_prompt: Optional[str] = None) -> Dict[str, Any]:
        """
        Optimize Anarchy Inference code for token efficiency.
        
        Args:
            code: Original Anarchy Inference code
            optimization_prompt: Additional instructions for optimization
            
        Returns:
            Dictionary containing optimized code and comparison metrics
        """
        original_tokens = self.interpreter.count_tokens(code)
        
        if not optimization_prompt:
            optimization_prompt = """Optimize this Anarchy Inference code for maximum token efficiency while maintaining functionality. Focus on:
1. Reducing variable name length
2. Eliminating unnecessary whitespace
3. Combining operations where possible
4. Using implicit returns
5. Simplifying control structures"""
        
        prompt = f"""<original_code>
{code}
</original_code>

{optimization_prompt}

Please provide only the optimized code within <optimized_code> tags."""
        
        response = self.client.messages.create(
            model=self.model,
            system=self.system_message,
            messages=[
                {"role": "user", "content": prompt}
            ],
            temperature=0.1,
            max_tokens=1000
        )
        
        generated_content = response.content[0].text
        
        # Extract optimized code
        optimized_code = self._extract_between_tags(generated_content, "optimized_code")
        if not optimized_code:
            # Fallback to code blocks
            code_blocks = self._extract_code_blocks(generated_content)
            optimized_code = code_blocks[0] if code_blocks else generated_content
        
        # Count tokens in optimized code
        optimized_tokens = self.interpreter.count_tokens(optimized_code)
        
        # Calculate improvement
        token_reduction = original_tokens - optimized_tokens
        token_reduction_percent = (token_reduction / original_tokens) * 100 if original_tokens > 0 else 0
        
        return {
            "original_code": code,
            "optimized_code": optimized_code,
            "original_tokens": original_tokens,
            "optimized_tokens": optimized_tokens,
            "token_reduction": token_reduction,
            "token_reduction_percent": token_reduction_percent
        }
    
    def generate_from_python(self, python_code: str) -> Dict[str, Any]:
        """
        Convert Python code to Anarchy Inference code.
        
        Args:
            python_code: Python code to convert
            
        Returns:
            Dictionary containing generated Anarchy Inference code and comparison metrics
        """
        python_tokens = self.interpreter.count_tokens(python_code)
        
        prompt = f"""<python_code>
{python_code}
</python_code>

Convert this Python code to Anarchy Inference, focusing on maximizing token efficiency while maintaining the same functionality.

Please provide the Anarchy Inference code within <anarchy_inference_code> tags."""
        
        response = self.client.messages.create(
            model=self.model,
            system=self.system_message,
            messages=[
                {"role": "user", "content": prompt}
            ],
            temperature=0.1,
            max_tokens=1000
        )
        
        generated_content = response.content[0].text
        
        # Extract Anarchy code
        anarchy_code = self._extract_between_tags(generated_content, "anarchy_inference_code")
        if not anarchy_code:
            # Fallback to code blocks
            code_blocks = self._extract_code_blocks(generated_content)
            anarchy_code = code_blocks[0] if code_blocks else generated_content
        
        # Count tokens in generated code
        anarchy_tokens = self.interpreter.count_tokens(anarchy_code)
        
        # Calculate efficiency
        token_reduction = python_tokens - anarchy_tokens
        token_efficiency = (token_reduction / python_tokens) * 100 if python_tokens > 0 else 0
        
        return {
            "python_code": python_code,
            "anarchy_code": anarchy_code,
            "python_tokens": python_tokens,
            "anarchy_tokens": anarchy_tokens,
            "token_reduction": token_reduction,
            "token_efficiency": token_efficiency
        }
    
    def _extract_between_tags(self, text: str, tag_name: str) -> Optional[str]:
        """Extract content between XML-style tags."""
        start_tag = f"<{tag_name}>"
        end_tag = f"</{tag_name}>"
        
        start_pos = text.find(start_tag)
        if start_pos == -1:
            return None
            
        start_pos += len(start_tag)
        end_pos = text.find(end_tag, start_pos)
        
        if end_pos == -1:
            return None
            
        return text[start_pos:end_pos].strip()
    
    def _extract_code_blocks(self, text: str) -> List[str]:
        """Extract code blocks from markdown text."""
        code_blocks = []
        lines = text.split('\n')
        in_code_block = False
        current_block = []
        
        for line in lines:
            if line.strip().startswith('```'):
                if in_code_block:
                    # End of code block
                    code_blocks.append('\n'.join(current_block))
                    current_block = []
                    in_code_block = False
                else:
                    # Start of code block
                    in_code_block = True
                    # Skip the line with ```
            elif in_code_block:
                current_block.append(line)
        
        return code_blocks


def main():
    """Command-line interface for the Anarchy Claude integration."""
    parser = argparse.ArgumentParser(description="Anarchy Inference Claude Integration")
    subparsers = parser.add_subparsers(dest="command", help="Command to run")
    
    # Generate command
    generate_parser = subparsers.add_parser("generate", help="Generate Anarchy code from task description")
    generate_parser.add_argument("--task", required=True, help="Task description")
    generate_parser.add_argument("--temperature", type=float, default=0.3, help="Temperature parameter (0.0-1.0)")
    generate_parser.add_argument("--output", help="Output file for generated code")
    generate_parser.add_argument("--execute", action="store_true", help="Execute the generated code")
    
    # Optimize command
    optimize_parser = subparsers.add_parser("optimize", help="Optimize existing Anarchy code")
    optimize_parser.add_argument("--file", required=True, help="File containing Anarchy code to optimize")
    optimize_parser.add_argument("--output", help="Output file for optimized code")
    
    # Convert command
    convert_parser = subparsers.add_parser("convert", help="Convert Python code to Anarchy")
    convert_parser.add_argument("--file", required=True, help="File containing Python code to convert")
    convert_parser.add_argument("--output", help="Output file for converted code")
    
    args = parser.parse_args()
    
    # Check for API key
    api_key = os.environ.get("ANTHROPIC_API_KEY")
    if not api_key:
        print("Error: ANTHROPIC_API_KEY environment variable not set")
        print("Set it with: export ANTHROPIC_API_KEY=your_api_key")
        sys.exit(1)
    
    try:
        integration = AnarchyClaudeIntegration(api_key)
        
        if args.command == "generate":
            result = integration.generate_anarchy_code(args.task, args.temperature)
            
            print(f"Generated Anarchy Inference code ({result['anarchy_tokens']} tokens):")
            print("=" * 80)
            print(result['anarchy_code'])
            print("=" * 80)
            
            if result['token_efficiency'] is not None:
                print(f"Token efficiency: {result['token_efficiency'] * 100:.2f}% reduction vs Python")
                
            if args.output:
                with open(args.output, 'w') as f:
                    f.write(result['anarchy_code'])
                print(f"Code saved to {args.output}")
                
            if args.execute:
                print("\nExecuting generated code:")
                print("-" * 80)
                execution = integration.execute_generated_code(result['anarchy_code'])
                
                if execution['success']:
                    print(f"Execution successful ({execution['execution_time']:.2f}s)")
                    print(f"Result: {execution['result']}")
                else:
                    print(f"Execution failed ({execution['execution_time']:.2f}s)")
                    print(f"Error: {execution['error']}")
        
        elif args.command == "optimize":
            with open(args.file, 'r') as f:
                code = f.read()
                
            result = integration.optimize_code(code)
            
            print(f"Original code: {result['original_tokens']} tokens")
            print(f"Optimized code: {result['optimized_tokens']} tokens")
            print(f"Token reduction: {result['token_reduction']} tokens ({result['token_reduction_percent']:.2f}%)")
            print("\nOptimized code:")
            print("=" * 80)
            print(result['optimized_code'])
            print("=" * 80)
            
            if args.output:
                with open(args.output, 'w') as f:
                    f.write(result['optimized_code'])
                print(f"Optimized code saved to {args.output}")
        
        elif args.command == "convert":
            with open(args.file, 'r') as f:
                python_code = f.read()
                
            result = integration.generate_from_python(python_code)
            
            print(f"Python code: {result['python_tokens']} tokens")
            print(f"Anarchy code: {result['anarchy_tokens']} tokens")
            print(f"Token reduction: {result['token_reduction']} tokens ({result['token_efficiency']:.2f}%)")
            print("\nGenerated Anarchy Inference code:")
            print("=" * 80)
            print(result['anarchy_code'])
            print("=" * 80)
            
            if args.output:
                with open(args.output, 'w') as f:
                    f.write(result['anarchy_code'])
                print(f"Converted code saved to {args.output}")
        
        else:
            parser.print_help()
    
    except Exception as e:
        print(f"Error: {str(e)}")
        sys.exit(1)


if __name__ == "__main__":
    main()
