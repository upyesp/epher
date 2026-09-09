use epher_shell::split_statements;

// ADR-0064: the statement splitter must not cut inside a `def ... do`
// block body — its `;` and newlines belong to the block — and must keep
// `\"` inside a string now that strings carry escapes.
#[test]
fn the_splitter_honors_blocks_and_escapes() {
    let pieces = split_statements(
        "def f(n) do total = 0; for i in 1 to n do total = total + i; return total end; f(3)",
    );
    assert_eq!(pieces.len(), 2);
    assert!(pieces[0].starts_with("def f(n) do"));
    assert_eq!(pieces[1], "f(3)");

    let pieces = split_statements("s = \"a; \\\"b\\\"\"");
    assert_eq!(pieces.len(), 1);

    // while/for bodies keep their one-statement shape: no `do` depth
    // without a `def` in front of it
    let pieces = split_statements("x = 0; while x < 5 do x = x + 1; x");
    assert_eq!(pieces.len(), 3);

    // multi-line block bodies stay one piece
    let src = "def f(x) do\n  y = x * 2\n  y + 1\nend\nf(4)";
    let pieces = split_statements(src);
    assert_eq!(pieces.len(), 2);
}
