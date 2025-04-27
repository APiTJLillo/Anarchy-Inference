#!/bin/bash

# Script to package the VS Code extension for Anarchy Inference

echo "Packaging VS Code extension for Anarchy Inference..."

# Check if vsce is installed
if ! command -v vsce &> /dev/null; then
    echo "vsce is not installed. Installing..."
    npm install -g vsce
fi

# Navigate to the extension directory
cd /home/ubuntu/anarchy_inference/vscode_extension/anarchy-inference

# Create a .vscodeignore file to exclude unnecessary files
cat > .vscodeignore << EOL
.vscode/**
.vscode-test/**
.gitignore
vsc-extension-quickstart.md
EOL

# Create a LICENSE file
cat > LICENSE << EOL
MIT License

Copyright (c) 2025 Anarchy Inference

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
EOL

# Initialize npm package
npm init -y

# Package the extension
vsce package

# Move the packaged extension to the parent directory
mv *.vsix ..

echo "Extension packaged successfully!"
echo "The .vsix file is located in /home/ubuntu/anarchy_inference/vscode_extension/"
