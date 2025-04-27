# Anarchy Inference LLM Template Usage Guidelines

## Overview

This document provides comprehensive guidelines for using the Anarchy Inference code generation templates with various Large Language Models (LLMs). These templates are designed to help you generate token-efficient Anarchy Inference code that maintains readability and functionality.

## Available Templates

We provide optimized templates for the following LLM platforms:

1. **OpenAI GPT-4** - `/llm_templates/openai_gpt4_template.md`
2. **Anthropic Claude** - `/llm_templates/anthropic_claude_template.md`
3. **Google Gemini** - `/llm_templates/google_gemini_template.md`
4. **Open Source LLMs** (Llama, Mistral) - `/llm_templates/open_source_llm_template.md`

## General Usage Instructions

### 1. Select the Appropriate Template

Choose the template that matches the LLM platform you're using. Each template is optimized for the specific characteristics and capabilities of its target platform.

### 2. Prepare Your Task Description

Write a clear, detailed description of the programming task you want to accomplish. Be specific about:
- Input and output requirements
- Any constraints or edge cases to handle
- Performance considerations
- Specific Anarchy Inference features you want to utilize

### 3. Customize the Template

Each template contains placeholders (usually `[TASK DESCRIPTION]`) where you should insert your specific programming task. Some templates may have additional placeholders for other customizations.

### 4. Set Recommended Parameters

Each template includes recommended parameter settings (temperature, top_p, etc.) for optimal results. Use these settings when making API calls to the LLM.

### 5. Review and Refine

After receiving the generated code:
- Verify that it correctly implements the requested functionality
- Check that it follows Anarchy Inference syntax conventions
- Evaluate the token efficiency claims
- Make any necessary adjustments

## Platform-Specific Guidelines

### OpenAI GPT-4

- **API Parameters**: Use temperature=0.2, top_p=0.95
- **Key Features**: Uses system message for context, supports few-shot learning
- **Best For**: Complex algorithms, detailed implementations
- **Usage Tips**: 
  - The system message contains the core instructions and should not be modified
  - The user message can be customized with your specific task
  - Include specific requirements about token efficiency in your task description

### Anthropic Claude

- **API Parameters**: Use temperature=0.3
- **Key Features**: Uses XML-style tags for structure, excellent at following detailed instructions
- **Best For**: Text processing, data manipulation, API interactions
- **Usage Tips**:
  - Maintain the XML tag structure (`<anarchy_inference_syntax>`, `<task>`, `<example_1>`, etc.)
  - Claude performs best when examples closely match your target task
  - Be explicit about token efficiency requirements

### Google Gemini

- **API Parameters**: Use temperature=0.2, top_k=40
- **Key Features**: Works well with markdown formatting, provides detailed explanations
- **Best For**: Mathematical algorithms, file operations, data processing
- **Usage Tips**:
  - Maintain the markdown structure with headings and code blocks
  - Request step-by-step explanations for complex tasks
  - Explicitly ask for token efficiency analysis

### Open Source LLMs (Llama, Mistral)

- **API Parameters**: Use temperature=0.1, top_p=0.9
- **Key Features**: Uses clear section headings, requires more explicit instructions
- **Best For**: Simple to moderate complexity tasks, basic algorithms
- **Usage Tips**:
  - Keep the structured format with clear section headings
  - Provide very explicit instructions and examples
  - Use lower temperature for more deterministic outputs
  - May require multiple attempts for complex tasks

## Example Workflow

1. **Identify Task**: Determine the programming task you need to accomplish
2. **Select Platform**: Choose the LLM platform based on task complexity and your access
3. **Prepare Template**: Copy the appropriate template and insert your task description
4. **Generate Code**: Submit the template to the LLM with recommended parameters
5. **Evaluate Results**: Review the generated code for correctness and efficiency
6. **Iterate if Needed**: Refine your task description and regenerate if necessary

## Troubleshooting

### Common Issues and Solutions

1. **Incorrect Syntax**: If the LLM generates code with incorrect Anarchy Inference syntax:
   - Review the examples in your template to ensure they demonstrate correct syntax
   - Explicitly mention problematic syntax elements in your follow-up prompt

2. **Insufficient Token Efficiency**: If the generated code isn't as token-efficient as expected:
   - Ask the LLM to optimize specific sections of the code
   - Request explicit token reduction techniques
   - Specify a minimum token reduction target (e.g., "achieve at least 25% reduction")

3. **Missing Functionality**: If the generated code doesn't implement all required functionality:
   - Break down complex tasks into smaller, more manageable components
   - Provide more detailed specifications in your task description
   - Include test cases or expected outputs

4. **Inconsistent Results**: If you get varying quality across different attempts:
   - Lower the temperature setting for more consistent outputs
   - Provide more detailed examples in your template
   - Use more structured task descriptions

## Advanced Usage

### Chaining Multiple Generations

For complex projects, consider breaking down your task into multiple generations:

1. First generation: Core functionality and structure
2. Second generation: Optimization for token efficiency
3. Third generation: Error handling and edge cases
4. Fourth generation: Documentation and examples

### Custom Templates

You can create custom templates for specific types of tasks by:

1. Starting with the platform-specific template
2. Adding domain-specific examples relevant to your use case
3. Including specialized instructions for your particular requirements
4. Testing and refining with multiple iterations

## Measuring Success

Evaluate the success of your code generation by measuring:

1. **Token Efficiency**: Compare token count with equivalent Python code
2. **Functional Correctness**: Test with various inputs and edge cases
3. **Readability**: Ensure the code remains understandable despite optimization
4. **Maintainability**: Check that the code follows good practices and is extensible

## Conclusion

These templates provide a powerful starting point for generating Anarchy Inference code with various LLM platforms. By following these guidelines and adapting the templates to your specific needs, you can efficiently create token-optimized code while maintaining readability and functionality.

For further assistance or to report issues with the templates, please contribute to the Anarchy Inference GitHub repository.
