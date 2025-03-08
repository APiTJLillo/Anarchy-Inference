const path = require('path');
const { workspace, ExtensionContext } = require('vscode');
const {
    LanguageClient,
    TransportKind
} = require('vscode-languageclient/node');

let client;

function activate(context) {
    // Find the LSP server executable
    const serverPath = context.asAbsolutePath(
        path.join('..', '..', 'target', 'release', 'minimal_llm_language')
    );

    // Server options
    const serverOptions = {
        run: {
            command: serverPath,
            args: ['--lsp'],
            transport: TransportKind.stdio
        },
        debug: {
            command: serverPath,
            args: ['--lsp', '--debug'],
            transport: TransportKind.stdio
        }
    };

    // Client options
    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'minimal-llm' }],
        synchronize: {
            fileEvents: workspace.createFileSystemWatcher('**/*.a.i')
        }
    };

    // Create and start the client
    client = new LanguageClient(
        'minimal-llm-language',
        'Minimal LLM Language Server',
        serverOptions,
        clientOptions
    );

    client.start();
}

function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

module.exports = {
    activate,
    deactivate
}; 