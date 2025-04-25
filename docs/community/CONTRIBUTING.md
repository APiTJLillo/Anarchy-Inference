# Contributing to Anarchy Inference

Thank you for your interest in contributing to Anarchy Inference! This document provides guidelines and instructions for contributing to the project. We welcome contributions of all kinds, from code improvements to documentation updates, bug reports, and feature suggestions.

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [How to Contribute](#how-to-contribute)
   - [Reporting Bugs](#reporting-bugs)
   - [Suggesting Enhancements](#suggesting-enhancements)
   - [Code Contributions](#code-contributions)
   - [Documentation](#documentation)
4. [Development Process](#development-process)
   - [Setting Up Your Environment](#setting-up-your-environment)
   - [Coding Standards](#coding-standards)
   - [Testing Guidelines](#testing-guidelines)
   - [Pull Request Process](#pull-request-process)
5. [Community Roles](#community-roles)
6. [Recognition and Attribution](#recognition-and-attribution)
7. [Communication Channels](#communication-channels)
8. [Resources](#resources)

## Code of Conduct

The Anarchy Inference project is committed to providing a welcoming and inclusive environment for all contributors. We expect all participants to adhere to our Code of Conduct, which promotes respect, open-mindedness, and constructive collaboration.

Key principles:
- Be respectful and inclusive
- Focus on constructive criticism
- Maintain a harassment-free environment
- Support and mentor newcomers
- Prioritize community well-being

For the full Code of Conduct, please see [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md).

## Getting Started

### Prerequisites

To contribute to Anarchy Inference, you'll need:

- Basic understanding of programming language design concepts
- Familiarity with Python (for the interpreter implementation)
- Understanding of LLMs and tokenization (for language optimization)
- Git for version control

### First Steps

1. **Explore the project**: Familiarize yourself with the [language reference](../language_reference.md) and [interpreter guide](../interpreter_guide.md)
2. **Try the language**: Experiment with the [web playground](../web_playground/index.html) or run examples locally
3. **Identify areas of interest**: Review the [todo list](../updated_todo_list.md) and [open issues](https://github.com/APiTJLillo/Anarchy-Inference/issues)
4. **Join the community**: Introduce yourself in our community channels

## How to Contribute

### Reporting Bugs

Bug reports help us improve the project. When reporting bugs:

1. Check if the bug has already been reported
2. Use the bug report template
3. Include detailed steps to reproduce the issue
4. Provide your environment details (OS, Python version, etc.)
5. Include any relevant error messages or screenshots
6. If possible, suggest a fix or workaround

### Suggesting Enhancements

We welcome ideas for improvements! When suggesting enhancements:

1. Check if the enhancement has already been suggested
2. Use the feature request template
3. Clearly describe the problem your enhancement would solve
4. Outline the proposed solution
5. Discuss alternatives you've considered
6. Explain how this enhancement benefits the project's goals of token efficiency

### Code Contributions

#### For First-Time Contributors

1. Find an issue labeled "good first issue" or "help wanted"
2. Comment on the issue to express your interest
3. Fork the repository and create a branch for your work
4. Make your changes following our coding standards
5. Submit a pull request with a clear description of your changes

#### For Experienced Contributors

1. Identify areas where you can add value based on your expertise
2. Discuss major changes in an issue before implementation
3. Follow the development process outlined below
4. Help review other contributors' pull requests

### Documentation

Documentation is crucial for the project's success. You can contribute by:

1. Improving existing documentation for clarity
2. Adding examples and tutorials
3. Creating diagrams or visualizations
4. Translating documentation to other languages
5. Writing blog posts or articles about Anarchy Inference

## Development Process

### Setting Up Your Environment

1. Fork the repository
2. Clone your fork locally
3. Set up the development environment:

```bash
# Clone your fork
git clone https://github.com/YOUR-USERNAME/Anarchy-Inference.git
cd Anarchy-Inference

# Set up the original repository as upstream
git remote add upstream https://github.com/APiTJLillo/Anarchy-Inference.git

# Create a virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt
```

### Coding Standards

#### General Guidelines

- Write clear, readable code with meaningful variable names
- Include comments for complex logic
- Follow the DRY (Don't Repeat Yourself) principle
- Write modular, testable code

#### Python Code Style

- Follow PEP 8 style guidelines
- Use type hints where appropriate
- Document functions and classes with docstrings
- Keep functions focused on a single responsibility

#### Anarchy Inference Code Style

- Prioritize token efficiency while maintaining readability
- Follow the syntax conventions in the language reference
- Include token efficiency comments explaining optimization choices
- Provide Python equivalent code for comparison when appropriate

### Testing Guidelines

- Write tests for new features and bug fixes
- Ensure all tests pass before submitting pull requests
- Include both unit tests and integration tests where appropriate
- For language features, include token efficiency benchmarks

### Pull Request Process

1. **Create a branch**: Create a branch from `main` with a descriptive name
2. **Make changes**: Implement your changes following our coding standards
3. **Write tests**: Add tests for your changes
4. **Update documentation**: Update relevant documentation
5. **Run tests locally**: Ensure all tests pass
6. **Submit PR**: Create a pull request with a clear description
7. **Code review**: Address feedback from maintainers
8. **Merge**: Once approved, your PR will be merged

## Community Roles

The Anarchy Inference project has several community roles:

- **Users**: People who use Anarchy Inference
- **Contributors**: People who contribute to the project
- **Reviewers**: Experienced contributors who review pull requests
- **Maintainers**: Core team members responsible for the project's direction
- **Community Managers**: People who help grow and support the community

As you contribute consistently, you can progress through these roles based on your involvement and expertise.

## Recognition and Attribution

We believe in recognizing all contributions:

- All contributors are listed in our [CONTRIBUTORS.md](./CONTRIBUTORS.md) file
- Significant contributions are highlighted in release notes
- Major contributors may be invited to join the core team
- We follow the [all-contributors](https://allcontributors.org/) specification

## Communication Channels

- **GitHub Issues**: For bug reports, feature requests, and project discussions
- **Discord Server**: For real-time communication and community support
- **Mailing List**: For announcements and longer discussions
- **Community Calls**: Regular video calls to discuss project direction

## Resources

- [Language Reference](../language_reference.md)
- [Interpreter Guide](../interpreter_guide.md)
- [Benchmark Framework](../benchmark_framework.py)
- [Token Efficiency Analysis](../benchmark_results/token_efficiency_analysis.md)
- [Project Roadmap](../roadmap.md)

## Thank You!

Your contributions help make Anarchy Inference better for everyone. We appreciate your time and effort in supporting this project!
