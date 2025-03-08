use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::interpreter::Interpreter;
use crate::error::LangError;
use log::{debug, info};

pub struct LspState {
    client: Client,
    documents: Arc<Mutex<HashMap<Url, String>>>
}

impl LspState {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    fn position_to_index(&self, content: &str, position: Position) -> usize {
        let mut current_line = 0;
        let mut current_char = 0;

        for (i, c) in content.char_indices() {
            if current_line == position.line as usize && current_char == position.character as usize {
                return i;
            }

            if c == '\n' {
                current_line += 1;
                current_char = 0;
            } else {
                current_char += 1;
            }
        }
        content.len()
    }

    fn get_completion_items(&self, _line: &str, _character: usize) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        // Add some basic keywords
        items.push(CompletionItem {
            label: "function".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Define a function".to_string()),
            ..Default::default()
        });

        items.push(CompletionItem {
            label: "let".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Declare a variable".to_string()),
            ..Default::default()
        });

        items
    }

    async fn analyze_and_report_diagnostics(&self, uri: &Url, content: &str) {
        let mut diagnostics = Vec::new();

        // Parse and check for syntax errors
        let lexer = Lexer::new(content);
        match Parser::new(lexer).parse() {
            Ok(_) => {
                // Parsing successful - could add semantic analysis here
            }
            Err(err) => {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position::new(0, 0),
                        end: Position::new(0, 1),
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("Parse error: {}", err),
                    source: Some("anarchy-inference".to_string()),
                    ..Default::default()
                });
            }
        }

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for LspState {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        info!("Starting language server initialization...");

        let capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::INCREMENTAL
            )),
            completion_provider: Some(CompletionOptions {
                resolve_provider: Some(false),
                trigger_characters: Some(vec![".".to_string()]),
                all_commit_characters: None,
                work_done_progress_options: Default::default(),
                completion_item: None,
            }),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            definition_provider: Some(OneOf::Left(true)),
            references_provider: Some(OneOf::Left(true)),
            document_symbol_provider: Some(OneOf::Left(true)),
            workspace_symbol_provider: Some(OneOf::Left(true)),
            ..ServerCapabilities::default()
        };

        Ok(InitializeResult {
            capabilities,
            server_info: Some(ServerInfo {
                name: String::from("Anarchy Inference Language Server"),
                version: Some(String::from("0.1.0")),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Anarchy Inference Language Server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(MessageType::INFO, "Shutting down Anarchy Inference Language Server...")
            .await;
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        self.documents.lock().insert(uri.clone(), text.clone());
        self.analyze_and_report_diagnostics(&uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        
        if let Some(content) = self.documents.lock().get_mut(&uri) {
            for change in params.content_changes {
                if let Some(range) = change.range {
                    let start_pos = self.position_to_index(content, range.start);
                    let end_pos = self.position_to_index(content, range.end);
                    content.replace_range(start_pos..end_pos, &change.text);
                } else {
                    *content = change.text;
                }
            }
            self.analyze_and_report_diagnostics(&uri, content).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if let Some(content) = self.documents.lock().get(&params.text_document.uri) {
            self.analyze_and_report_diagnostics(&params.text_document.uri, content).await;
        }
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        if let Some(doc) = self.documents.lock().get(&params.text_document_position.text_document.uri) {
            let items = self.get_completion_items(doc, params.text_document_position.position.character as usize);
            if !items.is_empty() {
                return Ok(Some(CompletionResponse::Array(items)));
            }
        }
        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        if let Some(_) = self.documents.lock().get(&params.text_document_position_params.text_document.uri) {
            Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Anarchy Inference Language".to_string(),
                }),
                range: None,
            }))
        } else {
            Ok(None)
        }
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        Ok(None)
    }

    async fn document_symbol(&self, params: DocumentSymbolParams) -> Result<Option<DocumentSymbolResponse>> {
        Ok(None)
    }
}

pub async fn start_lsp() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| LspState::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
