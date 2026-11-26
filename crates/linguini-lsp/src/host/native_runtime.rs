//! Tokio scheduling and stdio transport for the native host.
use super::*;
use std::io;
use tower_lsp_server::Server;

#[derive(Debug, Default)]
pub struct NativeScheduler;
impl LspScheduler for NativeScheduler {
    fn spawn_after(&self, delay: Duration, task: LspTask) {
        tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            task.await;
        });
    }
}

pub async fn run_stdio() {
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    let (service, socket) = crate::server::native_service();
    Server::new(stdin, stdout, socket).serve(service).await;
}

pub fn run_stdio_blocking() {
    if let Err(error) = try_run_stdio_blocking() {
        eprintln!("failed to start Linguini language server: {error}");
    }
}

pub fn try_run_stdio_blocking() -> io::Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(run_stdio());
    Ok(())
}
