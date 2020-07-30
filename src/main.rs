use anyhow::Result;
use log::info;

use dbml_language_server::file::open_and_parse;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::{
    lsp_types::{
        CompletionItem, CompletionOptions, CompletionParams, CompletionResponse,
        DidChangeTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
        InitializeParams, InitializeResult, InitializedParams, MessageType, RenameParams,
        RenameProviderCapability, ServerCapabilities, TextDocumentSyncCapability,
        TextDocumentSyncKind, WorkspaceEdit,
    },
    Client, LanguageServer, LspService, Server,
};

#[derive(Debug, Default)]
struct Backend {
    last_complete_text: Arc<Mutex<String>>,
}

impl Backend {
    async fn update_last_text(&self, text_to_update: String) {
        let mut inner_last_text = self.last_complete_text.lock().await;
        *inner_last_text = text_to_update;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(
        &self,
        _: &Client,
        _: InitializeParams,
    ) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        // Request full content each change
        let text_sync_kind = TextDocumentSyncKind::Full;
        // Trigger completion automatically on dot
        let completion_characters = vec![".".to_string()];

        let initialize = InitializeResult {
            capabilities: ServerCapabilities {
                rename_provider: Some(RenameProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: None,
                    trigger_characters: Some(completion_characters),
                    work_done_progress_options: Default::default(),
                }),
                definition_provider: Some(false),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(text_sync_kind)),
                ..Default::default()
            },
            ..Default::default()
        };
        Ok(initialize)
    }

    async fn initialized(&self, client: &Client, _: InitializedParams) {
        client.log_message(MessageType::Info, "server initialized!");
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        info!("shutdown request");
        Ok(())
    }

    async fn did_open(&self, client: &Client, params: DidOpenTextDocumentParams) {
        let document = params.text_document;
        let teste = open_and_parse(&document.uri, None).unwrap().unwrap();
        client.log_message(MessageType::Log, teste.root_node().to_sexp());
    }

    async fn did_change(&self, client: &Client, mut params: DidChangeTextDocumentParams) {
        info!("did_change event");

        let document = params.text_document;
        let changes = params.content_changes.remove(0).text;
        self.update_last_text(changes).await;

        client.log_message(MessageType::Log, document.uri);
    }
    async fn did_save(&self, client: &Client, params: DidSaveTextDocumentParams) {
        let document = params.text_document;
        info!("save request");
        client.log_message(MessageType::Log, "basingao");
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<CompletionResponse>> {
        info!("completion parameters: {:#?}", params);

        let test = "test".to_string();
        let test_two = "test_two".to_string();
        let simple = CompletionResponse::from(vec![
            CompletionItem::new_simple(test.clone(), test),
            CompletionItem::new_simple(test_two.clone(), test_two),
        ]);
        Ok(Some(simple))
    }
    async fn rename(
        &self,
        params: RenameParams,
    ) -> tower_lsp::jsonrpc::Result<Option<WorkspaceEdit>> {
        let new_name = params.new_name;
        let position = params.text_document_position.position;
        let uri = params.text_document_position.text_document.uri;

        let current_source_file = self.last_complete_text.lock().await.clone();
        info!("source: {:?}", current_source_file);

        Ok(dbml_language_server::providers::rename(
            current_source_file,
            position,
            new_name,
            uri,
        ))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let mut listener = tokio::net::TcpListener::bind("127.0.0.1:9001").await?;
    let (stream, _) = listener.accept().await?;
    let (read, write) = tokio::io::split(stream);

    info!("Starting generic LSP Server..");

    let (service, messages) = LspService::new(Backend::default());

    Server::new(read, write)
        .interleave(messages)
        .serve(service)
        .await;

    Ok(())
}
