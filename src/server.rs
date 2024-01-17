use std::path::PathBuf;
use tokio::process::Command;
use tower_lsp::{jsonrpc::Result, lsp_types::*, Client, LanguageServer};

struct Blame {
    hash: String,
    author: String,
    summary: String,
}

async fn get_blame(file: PathBuf, line: u32) -> Option<Blame> {
    let file_name = file.file_name()?.to_string_lossy().to_string();
    let path = file.parent().map(|p| p.to_string_lossy().to_string());
    let line = format!("{},{}", line, line);
    let mut cmd = Command::new("git");
    cmd.args(["blame", "-p", "-L", line.as_str(), file_name.as_str()]);
    if let Some(path) = path {
        cmd.current_dir(path);
    }
    let output = cmd.output().await.ok()?;
    if !output.status.success() {
        return None;
    }
    // TODO: date
    let mut summary = None;
    let mut author = None;
    let mut hash = None;
    for (index, line) in output.stdout.split(|&b| b == b'\n').enumerate() {
        match (index, line) {
            (0, a) => hash = Some(String::from_utf8_lossy(&a[0..7]).to_string()),
            (_, a) if a.starts_with(b"summary ") => {
                summary = Some(String::from_utf8_lossy(&a[8..]).to_string())
            }
            (_, a) if a.starts_with(b"author-mail ") => {
                author = Some(String::from_utf8_lossy(&a[12..]).to_string())
            }
            _ => {}
        }
    }
    match (summary, author, hash) {
        (Some(summary), Some(author), Some(hash)) => Some(Blame {
            summary,
            author,
            hash,
        }),
        _ => None,
    }
}

pub struct Server {
    _client: Client,
}

impl Server {
    pub fn new(client: Client) -> Self {
        Self { _client: client }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Server {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "helis".to_owned(),
                version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            }),
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
        })
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        if params.text_document_position_params.position.character == 0 {
            let line = params.text_document_position_params.position.line + 1;
            let file = params
                .text_document_position_params
                .text_document
                .uri
                .to_file_path()
                .unwrap();
            let blame = match get_blame(file, line).await {
                Some(a) => a,
                None => {
                    return Ok(None);
                }
            };
            // On an array of marked string, helix only display the last one :/
            Ok(Some(Hover {
                contents: HoverContents::Scalar(MarkedString::String(format!(
                    "{} {} {}",
                    blame.hash, blame.author, blame.summary
                ))),
                range: None,
            }))
        } else {
            Ok(None)
        }
    }
}
