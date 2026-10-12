//! The epher-lsp binary: stdio transport around the library.

// The server thread's stack. Script evaluation recurses per expression
// node and collection searches legitimately reach hundreds of thousands
// of frames (ADR-0069). On native targets the fat reservation is virtual
// only: pages commit as frames actually nest. Under wasm the stack is
// carved out of the shared linear memory, and the VS Code and VSCodium
// hosts cap that memory at 128 MiB (wasmServer.ts declares maximum:
// 2048 pages), so a 512 MiB stack cannot exist there: the spawn fails
// and the server dies at startup. The wasm build therefore runs on a
// 64 MiB stack that fits, proven against the shipped extension.
#[cfg(not(target_arch = "wasm32"))]
const SERVER_STACK: usize = 512 * 1024 * 1024;
#[cfg(target_arch = "wasm32")]
const SERVER_STACK: usize = 64 * 1024 * 1024;

fn main() {
    let child = std::thread::Builder::new()
        .stack_size(SERVER_STACK)
        .spawn(run)
        .expect("spawn the server thread with a fat stack");
    if let Err(_) = child.join().expect("main thread panicked") {
        std::process::exit(1);
    }
}

fn run() -> Result<(), ()> {
    if std::env::args().any(|a| a == "-V" || a == "--version") {
        println!("epher-lsp {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let (connection, io_threads) = lsp_server::Connection::stdio();
    if let Err(e) = epher_lsp::run(connection) {
        eprintln!("epher-lsp: {e}");
        return Err(());
    }
    io_threads.join().expect("io threads");
    Ok(())
}
