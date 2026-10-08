//! The epher-lsp binary: stdio transport around the library.

fn main() {
    // The language server runs scripts (ADR-0069), and script evaluation
    // recurses per expression node; collection searches legitimately reach
    // hundreds of thousands of frames. Run the server on a dedicated
    // fat-stack thread: the reservation is virtual only, pages commit as
    // frames actually nest.
    let child = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(run)
        .expect("spawn the main thread with a fat stack");
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
