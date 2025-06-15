use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::ops::Range;
use std::path::PathBuf;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::notification::Notification;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tower_lsp::{async_trait, lsp_types::*};

mod analyzer;
mod parser_utils;
mod parser;

// an expression node in the AST
#[derive(Debug)]
pub enum Expr {}
impl Expr {}

pub type Span = Range<usize>;
pub type Spanned<T> = (T, Span);
pub type Ast = Vec<Spanned<Expr>>;

#[derive(Debug, Deserialize, Serialize)]
struct NotificationParams {
    title: String,
}

#[allow(unused)]
enum CNotification {}

impl Notification for CNotification {
    type Params = NotificationParams;
    const METHOD: &'static str = "custom/notification";
}

#[derive(Debug)]
struct Backend {
    client: Client,
    ast_map: HashMap<String, Ast>,
}

#[async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            offset_encoding: None,
            capabilities: ServerCapabilities {
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["custom/notifcation".to_string()],
                    work_done_progress_options: Default::default(),
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client.log_message(MessageType::INFO, "...").await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let pos = params.text_document_position_params;
        let file = pos.text_document.uri.path();

        if let Some((detail, location)) = analyzer::Analyzer::hover(
            PathBuf::from(file),
            pos.position.line as usize,
            pos.position.character as usize,
        )
        .await
        {
            Ok(Some(Hover {
                contents: HoverContents::Scalar(MarkedString::LanguageString(LanguageString {
                    language: "".to_string(),
                    value: detail,
                })),
                range: Some(location.range),
            }))
        } else {
            Ok(None)
        }
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<Value>> {
        match params.command.as_str() {
            "custom/notification" => {
                self.client
                    .send_notification::<CNotification>(NotificationParams {
                        title: "notify".to_string(),
                    })
                    .await;
                self.client
                    .log_message(MessageType::INFO, String::new())
                    .await;
            }
            _ => {}
        }
        Ok(None)
    }
}

#[tokio::main]
async fn main() {
    // env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        ast_map: HashMap::new(),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
