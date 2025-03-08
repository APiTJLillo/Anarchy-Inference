import * as path from "path";
import { workspace, ExtensionContext } from "vscode";
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from "vscode-languageclient/node";

let client: LanguageClient;
export function activate(context: ExtensionContext) {
    const serverPath = context.asAbsolutePath(path.join("..", "..", "target", "release", "minimal_llm_language"));

    const serverOptions: ServerOptions = {
        run: { command: serverPath, args: ["--lsp"], transport: TransportKind.stdio },
        debug: { command: serverPath, args: ["--lsp", "--debug"], transport: TransportKind.stdio }
    };
    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: "file", language: "minimal-llm" }],
        synchronize: { fileEvents: workspace.createFileSystemWatcher("**/*.a.i") }
    };

    client = new LanguageClient("minimal-llm-language", "Minimal LLM Language Server", serverOptions, clientOptions);
    client.start();
}
export function deactivate(): Thenable<void> | undefined {
    if (!client) { return undefined; }
    return client.stop();
}
