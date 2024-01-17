use tower_lsp::{LspService, Server};

mod server;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::build(|client| server::Server::new(client)).finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}
