# Introduction to Anarchy Inference

## Overview

Welcome to the first tutorial in our Anarchy Inference learning path! This tutorial introduces you to Anarchy Inference, a token-minimal programming language designed specifically for LLM efficiency.

## Learning Objectives

By the end of this tutorial, you will:
- Understand what Anarchy Inference is and why it was created
- Learn why token efficiency matters in LLM-generated code
- Set up your development environment for Anarchy Inference
- Write and run your first Anarchy Inference program

## What is Anarchy Inference?

Anarchy Inference is a programming language designed from the ground up to be token-efficient while maintaining readability and functionality. It uses symbolic operators and concise syntax to reduce the number of tokens required for common programming tasks.

### Key Features

- **Token Efficiency**: Uses 30-50% fewer tokens than traditional languages
- **Symbolic Operators**: Employs special symbols for common operations
- **Familiar Syntax**: Draws inspiration from JavaScript, Python, and functional languages
- **Full Functionality**: Supports all common programming paradigms and operations
- **LLM Optimization**: Designed specifically for generation by Large Language Models

## Why Token Efficiency Matters

When working with Large Language Models (LLMs) like GPT-4, Claude, or Gemini, every token counts:

1. **Cost**: LLM providers charge based on token usage. Fewer tokens mean lower costs.
2. **Context Windows**: LLMs have limited context windows. Token-efficient code allows more functionality within these limits.
3. **Generation Speed**: Fewer tokens result in faster code generation.
4. **Reduced Hallucinations**: Shorter programs have less room for LLM errors or hallucinations.

### Token Efficiency Comparison

Let's look at a simple example comparing Python and Anarchy Inference:

**Python:**
```python
def calculate_average(numbers):
    if len(numbers) == 0:
        return 0
    total = sum(numbers)
    average = total / len(numbers)
    return average

data = [12, 15, 23, 7, 42]
result = calculate_average(data)
print(f"The average is: {result}")
```
Token count: 89 tokens

**Anarchy Inference:**
```
ƒ calculate_average(numbers) {
  ι numbers.length = 0 { ↵ 0 }
  avg ← numbers.sum() / numbers.length
  ↵ avg
}

data ← [12, 15, 23, 7, 42]
result ← calculate_average(data)
log("The average is: " + result)
```
Token count: 54 tokens

This represents a **39% reduction** in tokens while maintaining readability and functionality.

## Setting Up Your Environment

Let's get your development environment ready for Anarchy Inference.

### Installation

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/APiTJLillo/Anarchy-Inference.git
   cd Anarchy-Inference
   ```

2. **Install Dependencies**:
   ```bash
   npm install
   ```

3. **Verify Installation**:
   ```bash
   node anarchy.js --version
   ```

### Editor Setup

For the best development experience, we recommend using Visual Studio Code with the Anarchy Inference extension:

1. Open VS Code
2. Go to Extensions (Ctrl+Shift+X)
3. Search for "Anarchy Inference"
4. Install the extension

The extension provides syntax highlighting, code completion, and other helpful features for Anarchy Inference development.

## Your First Anarchy Inference Program

Let's write a simple "Hello, World!" program to get started.

### Hello, World!

Create a file named `hello.ai` with the following content:

```
// My first Anarchy Inference program
log("Hello, World!")
```

Run the program:

```bash
node anarchy.js hello.ai
```

You should see `Hello, World!` printed to the console.

### Understanding the Syntax

Let's break down what's happening:

1. `//` starts a comment, just like in JavaScript
2. `log()` is a built-in function for printing to the console (similar to `console.log()` in JavaScript or `print()` in Python)
3. The string `"Hello, World!"` is passed as an argument to the `log` function

### A More Complex Example

Now let's try something a bit more complex:

```
// Define a greeting function
ƒ greet(name) {
  ι name {
    ↵ "Hello, " + name + "!"
  } ε {
    ↵ "Hello, stranger!"
  }
}

// Get user input or use default
user ← args[0] || "World"

// Call the function and log the result
message ← greet(user)
log(message)
```

Save this as `greeting.ai` and run it:

```bash
node anarchy.js greeting.ai Alice
```

You should see `Hello, Alice!` printed to the console.

If you run it without an argument:

```bash
node anarchy.js greeting.ai
```

You'll see `Hello, World!` printed instead.

### Key Syntax Elements

Let's examine the key syntax elements in this example:

- `ƒ` is used to define a function (instead of `function` or `def`)
- `←` is the assignment operator (instead of `=`)
- `ι` is used for "if" conditions (instead of `if`)
- `ε` is used for "else" conditions (instead of `else`)
- `↵` is used for returning values (instead of `return`)

These symbolic operators help reduce token count while maintaining readability once you're familiar with them.

## Token Efficiency in Action

Let's compare the token count of our greeting program with an equivalent JavaScript version:

**JavaScript:**
```javascript
// Define a greeting function
function greet(name) {
  if (name) {
    return "Hello, " + name + "!";
  } else {
    return "Hello, stranger!";
  }
}

// Get user input or use default
const user = process.argv[2] || "World";

// Call the function and log the result
const message = greet(user);
console.log(message);
```
Token count: 78 tokens

**Anarchy Inference:**
```
// Define a greeting function
ƒ greet(name) {
  ι name {
    ↵ "Hello, " + name + "!"
  } ε {
    ↵ "Hello, stranger!"
  }
}

// Get user input or use default
user ← args[0] || "World"

// Call the function and log the result
message ← greet(user)
log(message)
```
Token count: 56 tokens

This represents a **28% reduction** in tokens. As programs grow in complexity, these savings become even more significant.

## Interactive Exercise

Now it's your turn to write some Anarchy Inference code. Try modifying the greeting program to:

1. Add a time-based greeting (e.g., "Good morning", "Good afternoon", "Good evening")
2. Include the current day of the week in the greeting
3. Make the greeting customizable with additional command-line arguments

Here's a starter template:

```
ƒ getTimeGreeting() {
  hour ← new Date().getHours()
  // Your code here
}

ƒ getDayOfWeek() {
  // Your code here
}

ƒ greet(name, options) {
  // Your code here
}

// Parse command line arguments
name ← args[0] || "World"
options ← {
  // Your code here
}

message ← greet(name, options)
log(message)
```

## Challenge

For an additional challenge, try implementing a simple temperature converter that:

1. Takes a temperature value and unit (C or F) as input
2. Converts it to the other unit
3. Displays the result with appropriate formatting
4. Handles invalid inputs gracefully

Compare your solution with equivalent code in a language you're familiar with and count the tokens in each.

## Next Steps

Congratulations on completing your first Anarchy Inference tutorial! In the next tutorial, we'll dive deeper into Anarchy Inference syntax and explore variables, data types, and basic operations.

Continue to [Basic Syntax and Operators →](./basic_syntax_and_operators.md)

## Additional Resources

- [Anarchy Inference GitHub Repository](https://github.com/APiTJLillo/Anarchy-Inference)
- [Language Reference](../language_reference.md)
- [Token Efficiency Analysis](../benchmark_results/token_efficiency_analysis.md)
- [Community Forum](https://discord.gg/anarchy-inference)

## Feedback

Did you find this tutorial helpful? Do you have suggestions for improvement? Join our community and share your feedback!
