// This is a minimal index.js file for the VS Code extension
// It's not actually needed for functionality but is required by the packaging process

// This extension is primarily declarative and doesn't require JavaScript code
// All functionality is provided through the package.json contributions

// Export an empty activate function to satisfy VS Code extension requirements
function activate(context) {
    // Extension is activated when a file with .ai extension is opened
    console.log('Anarchy Inference extension is now active');
}

// Export an empty deactivate function
function deactivate() {}

module.exports = {
    activate,
    deactivate
};
