# VS Code Extension for Anarchy Inference

This extension provides language support for Anarchy Inference, a token-minimal programming language designed specifically for LLMs.

## Features

- Syntax highlighting for `.ai` files
- Code snippets for common Anarchy Inference patterns
- Basic language configuration (comments, brackets, etc.)

## Installation

### From VSIX File

1. Download the `anarchy-inference-0.1.0.vsix` file
2. Open VS Code
3. Go to Extensions view (Ctrl+Shift+X)
4. Click on the "..." menu in the top-right of the Extensions view
5. Select "Install from VSIX..."
6. Choose the downloaded VSIX file

### Manual Installation

1. Copy the `anarchy-inference` folder to your VS Code extensions folder:
   - Windows: `%USERPROFILE%\.vscode\extensions`
   - macOS/Linux: `~/.vscode/extensions`
2. Restart VS Code

## Usage

### Syntax Highlighting

The extension automatically provides syntax highlighting for files with the `.ai` extension.

### Code Snippets

The following snippets are available:

- `var` - Variable declaration
- `func` - Function definition
- `if` - Conditional statement
- `loop` - Loop statement
- `print` - Print statement
- `str` - String manipulation example
- `file` - File operations example
- `web` - Web request example
- `data` - Data processing example

To use a snippet, start typing its prefix and press Tab when it appears in the suggestions.

## Development

### Building the Extension

1. Install Node.js and npm
2. Install vsce: `npm install -g vsce`
3. Navigate to the extension directory
4. Run `vsce package`

This will create a `.vsix` file that can be installed in VS Code.

## License

MIT
