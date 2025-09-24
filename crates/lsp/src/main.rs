//! The epher-lsp binary: stdio transport around the library.

fn main() {
    if std::env::args().any(|a| a == "-V" || a == "--version") {
        println!("epher-lsp {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let (connection, io_threads) = lsp_server::Connection::stdio();
    if let Err(e) = epher_lsp::run(connection) {
        eprintln!("epher-lsp: {e}");
        std::process::exit(1);
    }
    io_threads.join().expect("io threads");
}
