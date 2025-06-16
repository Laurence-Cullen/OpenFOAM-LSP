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
mod parser;
mod parser_utils;

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

impl Backend {
    
    async fn on_change(&self, params: TextDocumentItem) {
        let text = params.text;

        let (_, (tokens, spans)) = parser::scan(&text).unwrap();

        let errors = parser::get_errors(&tokens, &spans);

        // let diagnostics =  errors
        //     .into_iter()
        //     .map(|error| {
        //         Diagnostic {
        //             range: Range {
        //                 start: Position {
        //                     line: error.line as u32,
        //                     character: error.col as u32,
        //                 },
        //                 end: Position {
        //                     line: error.line as u32,
        //                     character: error.col as u32 + 1,
        //                 },
        //             },
        //             severity: Some(DiagnosticSeverity::ERROR),
        //             code: None,
        //             source: Some("Foam Language Server".to_string()),
        //             message: error.message,
        //             ..Diagnostic::default()
        //         }
        //     })
        //     .collect::<Vec<_>>();

    }
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

    async fn initialized(&self, params: InitializedParams) {
        self.client.log_message(MessageType::INFO, "...").await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let pos = params.text_document_position_params;
        let file = pos.text_document.uri.path();
        self.client.log_message(MessageType::INFO, file).await;


        let buffer =  std::fs::read_to_string(&PathBuf::from(file)).ok().unwrap();

        let (_, (tokens, spans)) = parser::scan(&buffer).unwrap();

        let chars_per_line = parser::count_characters_per_line(&buffer);
        let index = parser::index_from_line_and_col(chars_per_line.clone(), pos.position.line as usize, pos.position.character as usize);

        let mut span_index = 0;

        // iterate through spans until index sits between start and end
        for (i, span) in spans.iter().enumerate() {
            if span.start <= index && index < span.end {
                span_index = i;
                break;
            }
        }

        let hover_text = parser::get_foam_definition(tokens[span_index]);

        self.client.log_message(MessageType::INFO, pos.position.line).await;
        self.client.log_message(MessageType::INFO, hover_text.clone()).await;

        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::LanguageString(LanguageString {
                language: "".to_string(),
                value: hover_text.to_string(),
            })),
            range: None
        }))
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
