use std::sync::Arc;

use anyhow::Result;
use log::info;
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
use tree_sitter::Tree;

use dbml_language_server::{
    file::{parse_file, read_file},
    populate_identifiers, IdentifiersMap,
};

#[derive(Debug)]
struct Backend {
    client: Client,
    raw_source_code: Arc<Mutex<String>>,
    parsed_source_code: Arc<Mutex<Option<Tree>>>,
    identifier_list: Arc<Mutex<IdentifiersMap>>,
}

impl Backend {
    async fn update_source_code_and_parse(&self, text_to_update: String) -> Result<()> {
        let mut inner_last_text = self.raw_source_code.lock().await;
        *inner_last_text = text_to_update.clone();

        let mut inner_parsed_code = self.parsed_source_code.lock().await;
        let tree = parse_file(text_to_update.as_bytes(), None);
        *inner_parsed_code = tree;

        Ok(())
    }

    async fn populate_identifier_map(&self) -> Result<()> {
        let source = self.raw_source_code.lock().await;
        let parsed_code = self
            .parsed_source_code
            .lock()
            .await
            .as_ref()
            .map(|tree| populate_identifiers(source.as_bytes(), tree.root_node()));

        let mut afds = self.identifier_list.lock().await;
        *afds = parsed_code.unwrap();

        Ok(())
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(
        &self,
        _: InitializeParams,
    ) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        // Request full content each change
        let text_sync_kind = TextDocumentSyncKind::Full;
        // Trigger completion automatically on dot
        let completion_characters = vec![".".to_string(), "[".to_string()];

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

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::Info, "server initialized!");
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        info!("shutdown request");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let document = params.text_document;
        self.update_source_code_and_parse(document.text).await.ok();

        self.client
            .log_message(MessageType::Log, "Opened file sucessfully.");
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        info!("did_change event");

        let document = params.text_document;
        let changes = params.content_changes.remove(0).text;
        self.update_source_code_and_parse(changes).await;
        self.populate_identifier_map().await;

        self.client.log_message(MessageType::Log, document.uri);
    }
    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let document = params.text_document;
        info!("save request");
        self.client.log_message(MessageType::Log, "basingao");
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> tower_lsp::jsonrpc::Result<Option<CompletionResponse>> {
        info!("completion parameters: {:#?}", params);

        let current_pos = params.text_document_position.position;
        let context = params.context.unwrap();

        let current_source_file = self.raw_source_code.lock().await.clone();
        let current_tree = self.parsed_source_code.lock().await.clone();
        let identifiers = self.identifier_list.lock().await;
        info!("{:?}", identifiers);

        if current_tree.is_none() {
            return Ok(None);
        }

        let completions_available = dbml_language_server::providers::complete_at_point(
            current_source_file,
            current_tree.unwrap(), // Safe to unwrap after the early return
            &identifiers,
            current_pos,
            context,
        );

        // if completions are available, we map them into a proper response
        let completion_response = completions_available.map(|all_completions| {
            let to_completion_items = all_completions
                .iter()
                .map(|each| CompletionItem::new_simple(each.to_string(), each.to_string()))
                .collect::<Vec<CompletionItem>>();
            CompletionResponse::from(to_completion_items)
        });

        Ok(completion_response)
    }
    async fn rename(
        &self,
        params: RenameParams,
    ) -> tower_lsp::jsonrpc::Result<Option<WorkspaceEdit>> {
        let new_name = params.new_name;
        let position = params.text_document_position.position;
        let uri = params.text_document_position.text_document.uri;

        let current_source_file = self.raw_source_code.lock().await.clone();
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

    let (service, messages) = LspService::new(|client| Backend {
        client,
        raw_source_code: Arc::new(Default::default()),
        parsed_source_code: Arc::new(Default::default()),
        identifier_list: Default::default(),
    });

    Server::new(read, write)
        .interleave(messages)
        .serve(service)
        .await;

    Ok(())
}
