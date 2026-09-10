//! Token classes (ADR-0066): the source classified token by token,
//! each with its span and text. This is what editor coloring and
//! hover lookups ride on, taken from the real lexer so the classes
//! cannot drift from what actually parses.

use epher_core::{token_classes, TokenClass};

fn rendered(text: &str) -> Vec<(TokenClass, String)> {
    token_classes(text)
        .expect("tokenizes")
        .into_iter()
        .map(|t| (t.class, text[t.span.start..t.span.end].to_string()))
        .collect()
}

#[test]
fn tokens_carry_their_source_spans() {
    let text = "x = 1 + 2i";
    let toks = token_classes(text).expect("tokenizes");
    let sliced: Vec<&str> = toks
        .iter()
        .map(|t| &text[t.span.start..t.span.end])
        .collect();
    assert_eq!(sliced, vec!["x", "=", "1", "+", "2i"]);
}

#[test]
fn names_numbers_operators_and_strings_classify() {
    let classes = rendered("name = \"hi\" + 2.5 - 4i");
    use TokenClass::*;
    assert_eq!(
        classes,
        vec![
            (Name, "name".into()),
            (Operator, "=".into()),
            (String, "\"hi\"".into()),
            (Operator, "+".into()),
            (Number, "2.5".into()),
            (Operator, "-".into()),
            (Number, "4i".into()),
        ]
    );
}

#[test]
fn keywords_classify_as_keywords() {
    for kw in ["def", "const", "if", "then", "else", "while", "do", "end", "for", "in", "return", "break", "continue", "solve", "not", "and", "or", "xor", "to", "step"] {
        let classes = rendered(kw);
        assert_eq!(classes[0].0, TokenClass::Keyword, "keyword {kw}");
    }
}

#[test]
fn ordinary_names_are_not_keywords() {
    // `endor` and ` ifdef`-style names stay names, exactly as they
    // still parse as names.
    assert_eq!(rendered("endor")[0].0, TokenClass::Name);
    assert_eq!(rendered("notable")[0].0, TokenClass::Name);
}

#[test]
fn comments_produce_no_tokens() {
    let text = "1 # tail\n/* block */ 2 // line";
    let classes = rendered(text);
    assert_eq!(
        classes,
        vec![(TokenClass::Number, "1".into()), (TokenClass::Number, "2".into())]
    );
}
