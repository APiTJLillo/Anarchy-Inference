# Minimal LLM Language Support for VS Code

This extension provides language support for the Minimal LLM Language (.a.i files) including:

- Syntax highlighting
- Code completion
- Error diagnostics
- Go to definition
- Find references
- Document symbols
- Hover information

## Installation

### 1. Build the Language Server

First, build the language server:

```bash
cd /path/to/minimal_llm_language
cargo build --release
```

### 2. Install the VS Code Extension

1. Copy this directory to your VS Code extensions folder:
   - Linux: `~/.vscode/extensions/`
   - Windows: `%USERPROFILE%\.vscode\extensions\`
   - macOS: `~/.vscode/extensions/`

2. Install dependencies:
```bash
cd ~/.vscode/extensions/minimal-llm-language
npm install
npm run compile
```

3. Restart VS Code

### 3. Verify Installation

1. Open a `.a.i` file
2. You should see:
   - Syntax highlighting
   - Code completion when typing
   - Error diagnostics
   - Hover information for symbols

## Configuration

You can configure the extension in VS Code settings:

```json
{
    "minimal-llm.maxNumberOfProblems": 100,
    "minimal-llm.trace.server": "off"
}
```

## Troubleshooting

If the language server isn't working:

1. Check the VS Code output panel (View -> Output) and select "Minimal LLM Language" from the dropdown
2. Verify the language server executable path in extension.js matches your installation
3. Try running the language server manually: `minimal_llm_language --lsp`

## Development

To work on the extension:

1. Open this directory in VS Code
2. Run `npm install`
3. Make changes to the extension
4. Press F5 to launch a new VS Code window with the extension loaded
5. Run the "Developer: Reload Window" command to load changes 