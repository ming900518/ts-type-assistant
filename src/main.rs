#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::perf)]

mod processor;
mod type_parser;
mod types;

use lsp_types::{
    Hover, HoverContents, HoverParams, HoverProviderCapability, MarkupContent, MarkupKind,
    ServerCapabilities, ServerInfo,
};
use swc_common::BytePos;
use swc_ecma_ast::{EsVersion, Module};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};
use tower_lsp::{
    lsp_types::{InitializeParams, InitializeResult, InitializedParams, MessageType},
    Client, LanguageServer, LspService, Server,
};

use crate::types::Statement;

struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(
        &self,
        _: InitializeParams,
    ) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "TypeScript Type Assistant".to_string(),
                version: None,
            }),
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::ERROR, "TypeScript Type Assistant initialized.")
            .await;
        self.client
            .log_message(MessageType::ERROR, "Ming Chang <mail@mingchang.tw> 2023")
            .await;
    }

    async fn hover(&self, param: HoverParams) -> tower_lsp::jsonrpc::Result<Option<Hover>> {
        self.client
            .log_message(MessageType::ERROR, format!("Hover:\n{param:#?}"))
            .await;
        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::PlainText,
                value: String::new(),
            }),
            range: None,
        }))
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}

fn parser(input: &str) -> Module {
    let lexer = Lexer::new(
        Syntax::Typescript(TsConfig::default()),
        EsVersion::latest(),
        StringInput::new(
            input,
            BytePos(0),
            BytePos(u32::try_from(input.len()).expect("Unable to convert usize to u32")),
        ),
        None,
    );
    let mut parser = Parser::new_from(lexer);
    parser
        .parse_typescript_module()
        .expect("failed to parse module")
}
