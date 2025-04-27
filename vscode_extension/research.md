# VS Code Extension Development Research

## Overview of VS Code Extension Development

Visual Studio Code extensions allow you to add languages, debuggers, and tools to your VS Code installation to support your development workflow. Creating a syntax highlighting extension for Anarchy Inference will significantly improve the developer experience by providing:

1. Syntax highlighting for `.ai` files
2. Code snippets for common Anarchy Inference patterns
3. Basic language intelligence features

## Extension Types

For Anarchy Inference, we need to create a **Language Extension** that provides:
- Syntax highlighting (TextMate grammar)
- Snippets for common code patterns
- Basic language configuration (comments, brackets, etc.)

## Development Requirements

- Node.js and npm
- Yeoman and VS Code Extension Generator
- Basic understanding of TextMate grammars
- JSON configuration files

## Development Steps

1. **Setup Development Environment**
   - Install Node.js and npm
   - Install Yeoman and VS Code Extension Generator
   ```
   npm install -g yo generator-code
   ```

2. **Generate Extension Scaffold**
   - Use Yeoman to generate a new extension
   ```
   yo code
   ```
   - Select "New Language Support" option
   - Provide language details (Anarchy Inference)
   - Specify file extension (`.ai`)

3. **Define TextMate Grammar**
   - Create syntax highlighting rules in `syntaxes/anarchy-inference.tmLanguage.json`
   - Define token types (keywords, functions, variables, etc.)
   - Map tokens to VS Code's theming system

4. **Configure Language Features**
   - Define language configuration in `language-configuration.json`
   - Set up comment tokens, brackets, auto-closing pairs, etc.

5. **Create Code Snippets**
   - Define snippets in `snippets/anarchy-inference.json`
   - Create snippets for common patterns (functions, loops, conditionals)

6. **Package and Test**
   - Package extension using `vsce`
   ```
   npm install -g vsce
   vsce package
   ```
   - Test in VS Code by installing the `.vsix` file

7. **Publish (Optional)**
   - Publish to VS Code Marketplace if desired

## TextMate Grammar Structure

TextMate grammars use a structured format to define syntax highlighting:

```json
{
  "scopeName": "source.ai",
  "patterns": [
    {
      "match": "\\b(keyword)\\b",
      "name": "keyword.control.ai"
    }
  ],
  "repository": {
    "strings": {
      "patterns": [...]
    }
  }
}
```

## VS Code Theming Scopes

Common scopes used in VS Code themes:

- `comment` - For comments
- `constant` - For constants and enum values
- `entity.name.function` - For function names
- `entity.name.type` - For types
- `keyword` - For language keywords
- `string` - For strings
- `variable` - For variables and parameters

## Resources

- [VS Code Extension API](https://code.visualstudio.com/api)
- [Language Extension Overview](https://code.visualstudio.com/api/language-extensions/overview)
- [Syntax Highlight Guide](https://code.visualstudio.com/api/language-extensions/syntax-highlight-guide)
- [TextMate Grammar Reference](https://macromates.com/manual/en/language_grammars)
