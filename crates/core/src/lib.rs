//! epher-core — the single source of truth for epher's logic.
//!
//! Compiles to both `wasm32-unknown-unknown` (web/PWA/desktop) and native targets
//! (CLI/TUI). Stays pure: no I/O, no threads; the one platform read is the
//! clock behind `now()` (ADR-0037). Numerics per ADR-0005.

pub mod astro;
pub mod graph;
pub mod graph_svg;

use std::collections::HashMap;

use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use num_complex::Complex;
use num_rational::BigRational;
use num_traits::{ToPrimitive, Zero};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// The shared session snapshot persisted in the store (ADR-0010
/// amendment): the environment's variable bindings — user assignments
/// and `ans` — saved by whichever interactive frontend ran last.
pub type ValueBindings = HashMap<String, Value>;

/// The result of evaluating an Expression — the project's single number
/// representation (ADR-0005). `Float` is the default fast path; the other
/// variants are opt-in exactness layers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Float(f64),
    Rational(BigRational),
    Decimal(Decimal),
    Big(BigDecimal),
    Complex(Complex<f64>),
    Bool(bool),
    /// A display string — produced by the base-conversion builtins
    /// (`bin`, `oct`, `hex`; ADR-0022) and good for nothing else: the
    /// language has no string literals or string operations.
    Str(String),
}

impl Value {
    /// Wrap a plain number as the default `Float` variant.
    pub fn float(n: f64) -> Self {
        Value::Float(n)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Float(n) => write!(f, "{n}"),
            Value::Rational(r) => {
                if r.denom() == &num_bigint::BigInt::from(1) {
                    write!(f, "{}", r.numer())
                } else {
                    write!(f, "{}/{}", r.numer(), r.denom())
                }
            }
            Value::Decimal(d) => write!(f, "{d}"),
            Value::Big(b) => write!(f, "{b}"),
            Value::Complex(c) => write!(f, "{}", complex_display(*c)),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Str(s) => write!(f, "{s}"),
        }
    }
}

/// The shortest clean `a+bi` spelling (ADR-0043): zero parts are
/// dropped (`3`, `i`, `-2i`), the unit imaginary keeps no coefficient
/// (`1+i` not `1+1i`), and the real sign separates the terms.
fn complex_display(c: Complex<f64>) -> String {
    let (re, im) = (c.re, c.im);
    if im == 0.0 {
        return format!("{re}");
    }
    let im_abs = im.abs();
    let im_part = if im_abs == 1.0 {
        "i".to_string()
    } else {
        format!("{im_abs}i")
    };
    if re == 0.0 {
        if im < 0.0 {
            format!("-{im_part}")
        } else {
            im_part
        }
    } else {
        let sign = if im < 0.0 { "-" } else { "+" };
        format!("{re}{sign}{im_part}")
    }
}

/// Variable bindings available while evaluating an [`Expression`].
#[derive(Debug, Clone, Default)]
pub struct Env {
    bindings: HashMap<String, Value>,
    constants: HashMap<String, Value>,
    functions: HashMap<String, Function>,
}

impl Env {
    /// Look up a name.
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.bindings.get(name)
    }

    /// Bind a name to a value.
    pub fn set(&mut self, name: impl Into<String>, value: Value) {
        self.bindings.insert(name.into(), value);
    }

    /// The session's variable bindings (user assignments plus `ans`), for
    /// the shared-store snapshot every interactive frontend persists
    /// (ADR-0010 amendment): CLI/REPL, TUI, and desktop GUI.
    pub fn bindings(&self) -> &HashMap<String, Value> {
        &self.bindings
    }

    /// Look up a user-defined constant (ADR-0012).
    pub fn constant(&self, name: &str) -> Option<&Value> {
        self.constants.get(name)
    }

    /// Define a user-defined constant. The guards (no redefinition, no
    /// taking a variable's name) live in the script runner, not here.
    pub fn set_constant(&mut self, name: impl Into<String>, value: Value) {
        self.constants.insert(name.into(), value);
    }

    /// Look up a user-defined function.
    pub fn function(&self, name: &str) -> Option<&Function> {
        self.functions.get(name)
    }

    /// Define a user-defined function.
    pub fn set_function(&mut self, name: impl Into<String>, function: Function) {
        self.functions.insert(name.into(), function);
    }

    /// A child environment for a function call: the function table and the
    /// constants are visible (so recursion works and constants act like the
    /// built-in `pi`); the caller's bindings are not.
    fn new_child(&self) -> Env {
        Env {
            bindings: HashMap::new(),
            constants: self.constants.clone(),
            functions: self.functions.clone(),
        }
    }
}

/// A parsed piece of mathematics that can be evaluated to a [`Value`] — a domain
/// noun (see `CONTEXT.md`). Public so it can be produced by multiple input
/// forms (plain text, LaTeX) and consumed by both [`eval`] and the graphing
/// Sampler; treated opaquely by tests.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(f64),
    Var(String),
    Call(String, Vec<Expression>),
    Neg(Box<Expression>),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Pow(Box<Expression>, Box<Expression>),
    Factorial(Box<Expression>),
    Compare(CmpOp, Box<Expression>, Box<Expression>),
    If(Box<Expression>, Box<Expression>, Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Not(Box<Expression>),
}

/// A comparison operator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CmpOp {
    Gt,
    Lt,
    Ge,
    Le,
    Eq,
    Ne,
}

/// One statement of a [`Script`] — the unit of the script seam (CONTEXT.md).
/// Assignment mutates the [`Env`]; a constant definition binds an immutable
/// name (ADR-0012); plain expressions just evaluate.
#[derive(Debug, Clone)]
pub enum Statement {
    Assign(String, Expression),
    Const(String, Expression),
    FunctionDef(String, Vec<String>, Expression),
    While(Expression, Box<Statement>),
    /// `solve lhs == rhs` (ADR-0043): numeric equation solving, no CAS.
    Solve(Expression),
    Expr(Expression),
}

/// A user-defined function: parameter names and a body expression.
#[derive(Debug, Clone)]
pub struct Function {
    params: Vec<String>,
    body: Expression,
}

/// Errors crossing the epher-core seams.
#[derive(Debug, thiserror::Error)]
pub enum EpherError {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("type error: {0}")]
    Type(String),
    #[error("unknown name: {0}")]
    UnknownName(String),
    #[error("domain error: {0}")]
    Domain(String),
    #[error("division by zero")]
    ZeroDivision,
    #[error("step limit exceeded")]
    StepLimit,
    #[error("cannot assign to constant {0}")]
    AssignToConstant(String),
    #[error("constant already defined: {0}")]
    ConstantAlreadyDefined(String),
    #[error("cannot define constant {0}: the name is already a variable")]
    ConstantNameTaken(String),
    #[error("io error: {0}")]
    Io(String),
}

/// Parse plain text into an [`Expression`] (the plain-text input seam).
///
/// Tokenizer + recursive-descent parser with precedence (additive below
/// multiplicative) and left-associative operator folding.
pub fn parse(text: &str) -> Result<Expression, EpherError> {
    let tokens = tokenize(text)?;
    let mut parser = Parser { tokens, pos: 0 };
    let expr = parser.parse_expression()?;
    if parser.peek().is_some() {
        return Err(EpherError::Parse("unexpected trailing input".into()));
    }
    Ok(expr)
}

/// Parse LaTeX math into an [`Expression`] — the LaTeX input form (Q5). A
/// translation layer rewrites LaTeX constructs into plain epher text, then the
/// same grammar parses it: one grammar, two input forms.
pub fn parse_latex(text: &str) -> Result<Expression, EpherError> {
    parse(&translate_latex(text)?)
}

fn translate_latex(text: &str) -> Result<String, EpherError> {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            let mut cmd = String::new();
            while let Some(&c2) = chars.peek() {
                if c2.is_ascii_alphabetic() {
                    cmd.push(c2);
                    chars.next();
                } else {
                    break;
                }
            }
            match cmd.as_str() {
                "frac" => {
                    let num = translate_latex(&take_braced(&mut chars)?)?;
                    let den = translate_latex(&take_braced(&mut chars)?)?;
                    out.push_str(&format!("({num})/({den})"));
                }
                "sqrt" => {
                    let inner = translate_latex(&take_braced(&mut chars)?)?;
                    out.push_str(&format!("sqrt({inner})"));
                }
                "cdot" | "times" => out.push('*'),
                "div" => out.push('/'),
                "pi" => out.push_str("pi"),
                "left" | "right" => {
                    // \( \left( ... \right) \) — keep the delimiter char
                    if let Some(&c2) = chars.peek() {
                        out.push(c2);
                        chars.next();
                    }
                }
                _ => {
                    return Err(EpherError::Parse(format!(
                        "unsupported LaTeX command: \\{cmd}"
                    )));
                }
            }
        } else if c == '{' {
            // bare grouping → parentheses
            let inner = translate_latex(&take_braced(&mut chars)?)?;
            out.push_str(&format!("({inner})"));
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

/// Take the contents of the next `{...}` group, tracking nested braces.
fn take_braced(chars: &mut impl Iterator<Item = char>) -> Result<String, EpherError> {
    match chars.next() {
        Some('{') => {}
        Some(other) => {
            return Err(EpherError::Parse(format!("expected '{{', found {other}")));
        }
        None => return Err(EpherError::Parse("expected '{'".into())),
    }
    let mut depth = 1;
    let mut inner = String::new();
    while let Some(c) = chars.next() {
        match c {
            '{' => {
                depth += 1;
                inner.push(c);
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(inner);
                }
                inner.push(c);
            }
            _ => inner.push(c),
        }
    }
    Err(EpherError::Parse("unbalanced braces in LaTeX".into()))
}

/// Parse a sequence of statements separated by `;` (the script seam).
pub fn parse_script(text: &str) -> Result<Vec<Statement>, EpherError> {
    let tokens = tokenize(text)?;
    let mut parser = Parser { tokens, pos: 0 };
    let mut statements = Vec::new();
    loop {
        // Newlines and `;` are the same separator; redundant ones (blank
        // lines, `;;`) are skipped, like empty lines at the input layer.
        while matches!(parser.peek(), Some(Token::Semicolon)) {
            parser.next();
        }
        if parser.peek().is_none() {
            break;
        }
        let stmt = parser.parse_statement()?;
        statements.push(stmt);
        match parser.peek() {
            Some(Token::Semicolon) => {
                parser.next();
                // trailing ';' is fine
            }
            None => break,
            Some(_) => {
                return Err(EpherError::Parse(
                    "expected ';' or a newline between statements".into(),
                ));
            }
        }
    }
    Ok(statements)
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Comma,
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    EqualEqual,
    NotEqual,
    Equals,
    Semicolon,
    Bang,
    Percent,
    LParen,
    RParen,
    /// A number with an imaginary suffix (ADR-0043): `4i`, `2.5i`,
    /// `0xFFi`. The tokenizer folds the suffix in so `3 + 4i` parses as
    /// one literal; the parser spells it `4 * i`.
    Imaginary(f64),
}

fn tokenize(text: &str) -> Result<Vec<Token>, EpherError> {
    let mut tokens = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            c if c.is_whitespace() => {
                // Newlines are statement separators, exactly like `;`
                // (ADR-0001 seam unification): the language has no
                // strings or multi-line constructs, so a newline can
                // only ever appear between statements (a comment's
                // newlines are consumed by the comment itself).
                if c == '\n' || c == '\r' {
                    tokens.push(Token::Semicolon);
                }
                chars.next();
            }
            '#' => {
                // Line comment, PHP style (ADR-0040): `#` runs to the
                // end of the line. The newline itself is left for the
                // whitespace arm, so it still separates statements.
                while let Some(&c2) = chars.peek() {
                    if c2 == '\n' || c2 == '\r' {
                        break;
                    }
                    chars.next();
                }
            }
            '/' => {
                chars.next();
                match chars.peek() {
                    // Block comment, PHP style: `/* ... */` may span
                    // lines and may sit inline between tokens. Its
                    // newlines belong to the comment, not to the
                    // statement separator.
                    Some('*') => {
                        chars.next();
                        loop {
                            match chars.next() {
                                Some('*') if matches!(chars.peek(), Some('/')) => {
                                    chars.next();
                                    break;
                                }
                                Some(_) => {}
                                None => {
                                    return Err(EpherError::Parse(
                                        "unterminated block comment: expected */".into(),
                                    ))
                                }
                            }
                        }
                    }
                    // Line comment, PHP style: `//` runs to the end of
                    // the line; the newline still separates statements.
                    Some('/') => {
                        chars.next();
                        while let Some(&c2) = chars.peek() {
                            if c2 == '\n' || c2 == '\r' {
                                break;
                            }
                            chars.next();
                        }
                    }
                    _ => tokens.push(Token::Slash),
                }
            }
            '+' => {
                tokens.push(Token::Plus);
                chars.next();
            }
            '-' => {
                tokens.push(Token::Minus);
                chars.next();
            }
            '*' => {
                tokens.push(Token::Star);
                chars.next();
            }
            '^' => {
                tokens.push(Token::Caret);
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            '>' => {
                chars.next();
                if matches!(chars.peek(), Some('=')) {
                    chars.next();
                    tokens.push(Token::GreaterEqual);
                } else {
                    tokens.push(Token::GreaterThan);
                }
            }
            '<' => {
                chars.next();
                if matches!(chars.peek(), Some('=')) {
                    chars.next();
                    tokens.push(Token::LessEqual);
                } else {
                    tokens.push(Token::LessThan);
                }
            }
            '=' => {
                chars.next();
                if matches!(chars.peek(), Some('=')) {
                    chars.next();
                    tokens.push(Token::EqualEqual);
                } else {
                    tokens.push(Token::Equals);
                }
            }
            '!' => {
                chars.next();
                if matches!(chars.peek(), Some('=')) {
                    chars.next();
                    tokens.push(Token::NotEqual);
                } else {
                    tokens.push(Token::Bang);
                }
            }
            '%' => {
                tokens.push(Token::Percent);
                chars.next();
            }
            ';' => {
                tokens.push(Token::Semicolon);
                chars.next();
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            '0' if matches!(
                chars.clone().nth(1),
                Some('b' | 'B' | 'o' | 'O' | 'x' | 'X')
            ) =>
            {
                // Based literals (ADR-0022): 0b/0o/0x with the digits the
                // community expects — 0b101, 0o17, 0xFF. The value is the
                // plain number; a base prefix changes the spelling, never
                // the result. Like decimal literals, the token is an f64
                // (exact up to 2^53), so `0xFF` and `255` are the same.
                chars.next(); // the 0
                let marker = chars.next().expect("peeked above");
                let radix: u32 = match marker.to_ascii_lowercase() {
                    'b' => 2,
                    'o' => 8,
                    _ => 16,
                };
                let mut digits = String::new();
                while let Some(&c2) = chars.peek() {
                    let ok = match radix {
                        2 => matches!(c2, '0' | '1'),
                        8 => matches!(c2, '0'..='7'),
                        _ => c2.is_ascii_hexdigit(),
                    };
                    if ok {
                        digits.push(c2);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if digits.is_empty() {
                    if let Some(ch) = chars.peek().copied() {
                        if ch.is_ascii_alphanumeric() {
                            return Err(EpherError::Parse(format!(
                                "invalid digit {ch} after 0{marker}"
                            )));
                        }
                    }
                    return Err(EpherError::Parse(format!(
                        "expected digits after 0{marker}"
                    )));
                }
                let big = num_bigint::BigInt::parse_bytes(digits.as_bytes(), radix)
                    .expect("only valid digits were collected");
                let n: f64 = big
                    .to_string()
                    .parse()
                    .map_err(|_| EpherError::Parse(format!("invalid number: 0{marker}{digits}")))?;
                tokens.push(imaginary_or_number(n, &mut chars));
            }
            c if c.is_ascii_digit() || c == '.' => {
                let mut num = String::new();
                while let Some(&c2) = chars.peek() {
                    if c2.is_ascii_digit() || c2 == '.' {
                        num.push(c2);
                        chars.next();
                    } else {
                        break;
                    }
                }
                // scientific notation: an e/E exponent with an optional sign
                // and at least one digit (looked ahead so `2e` and `2eggs`
                // still tokenize as 2 followed by a name)
                if matches!(chars.peek(), Some('e') | Some('E')) {
                    let mut rest = chars.clone();
                    rest.next(); // the e
                    let signed = matches!(rest.peek(), Some('+') | Some('-'));
                    if signed {
                        rest.next();
                    }
                    if matches!(rest.peek(), Some(c3) if c3.is_ascii_digit()) {
                        num.push(*chars.peek().expect("checked above"));
                        chars.next();
                        if signed {
                            num.push(*chars.peek().expect("checked above"));
                            chars.next();
                        }
                        while let Some(&c2) = chars.peek() {
                            if c2.is_ascii_digit() {
                                num.push(c2);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                    }
                }
                let n: f64 = num
                    .parse()
                    .map_err(|_| EpherError::Parse(format!("invalid number: {num:?}")))?;
                tokens.push(imaginary_or_number(n, &mut chars));
            }
            c if c.is_alphabetic() => {
                let mut ident = String::new();
                while let Some(&c2) = chars.peek() {
                    // identifiers may contain digits after the first
                    // character (atan2, log10, x2), but must start with a
                    // letter so numbers still tokenize as numbers
                    if c2.is_alphanumeric() || c2 == '_' {
                        ident.push(c2);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Ident(ident));
            }
            other => {
                return Err(EpherError::Parse(format!(
                    "unexpected character: {other:?}"
                )))
            }
        }
    }
    Ok(tokens)
}

/// A number directly followed by an `i` that is not part of a longer
/// identifier becomes an imaginary literal (ADR-0043): `4i` is a token,
/// `4it` stays a number and a name. Based literals share the suffix, so
/// `0xFFi` works too.
fn imaginary_or_number(n: f64, chars: &mut std::iter::Peekable<std::str::Chars>) -> Token {
    if matches!(chars.peek(), Some('i')) {
        let mut rest = chars.clone();
        rest.next();
        if !matches!(rest.peek(), Some(c) if c.is_alphanumeric() || *c == '_') {
            chars.next();
            return Token::Imaginary(n);
        }
    }
    Token::Number(n)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos).cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    /// A statement is `while cond do stmt` (loop), `def name(params) = expr`
    /// (function definition), `const name = expr` (constant definition,
    /// ADR-0012), `name = expr` (assignment), or `expr`.
    fn parse_statement(&mut self) -> Result<Statement, EpherError> {
        if matches!(self.peek(), Some(Token::Ident(kw)) if kw == "while") {
            self.next(); // consume 'while'
            let cond = self.parse_expression()?;
            self.expect_keyword("do")?;
            let body = Box::new(self.parse_statement()?);
            return Ok(Statement::While(cond, body));
        }
        if matches!(self.peek(), Some(Token::Ident(kw)) if kw == "solve") {
            // `solve lhs == rhs` (ADR-0043): the comparison level of the
            // grammar parses the equation; anything else is rejected by
            // the solver, not the parser.
            self.next(); // consume 'solve'
            let equation = self.parse_expression()?;
            return Ok(Statement::Solve(equation));
        }
        if matches!(self.peek(), Some(Token::Ident(kw)) if kw == "const") {
            self.next(); // consume 'const'
            let name = self.expect_ident("constant name")?;
            self.expect_token(Token::Equals, "'='")?;
            let expr = self.parse_expression()?;
            return Ok(Statement::Const(name, expr));
        }
        if matches!(self.peek(), Some(Token::Ident(kw)) if kw == "def") {
            self.next(); // consume 'def'
            let name = self.expect_ident("function name")?;
            self.expect_token(Token::LParen, "'('")?;
            let mut params = Vec::new();
            if !matches!(self.peek(), Some(Token::RParen)) {
                loop {
                    params.push(self.expect_ident("parameter name")?);
                    match self.next() {
                        Some(Token::Comma) => continue,
                        Some(Token::RParen) => break,
                        Some(other) => {
                            return Err(EpherError::Parse(format!(
                                "expected ',' or ')', found {other:?}"
                            )));
                        }
                        None => return Err(EpherError::Parse("unexpected end of input".into())),
                    }
                }
            } else {
                self.next(); // zero-parameter function
            }
            self.expect_token(Token::Equals, "'='")?;
            let body = self.parse_expression()?;
            return Ok(Statement::FunctionDef(name, params, body));
        }
        if let Some(Token::Ident(name)) = self.peek().cloned() {
            if matches!(self.tokens.get(self.pos + 1), Some(Token::Equals)) {
                self.next(); // consume the identifier
                self.next(); // consume '='
                let expr = self.parse_expression()?;
                return Ok(Statement::Assign(name, expr));
            }
        }
        let expr = self.parse_expression()?;
        Ok(Statement::Expr(expr))
    }

    fn expect_ident(&mut self, what: &str) -> Result<String, EpherError> {
        match self.next() {
            Some(Token::Ident(name)) => Ok(name),
            Some(other) => Err(EpherError::Parse(format!(
                "expected {what}, found {other:?}"
            ))),
            None => Err(EpherError::Parse("unexpected end of input".into())),
        }
    }

    fn expect_token(&mut self, token: Token, what: &str) -> Result<(), EpherError> {
        match self.next() {
            Some(found) if found == token => Ok(()),
            Some(other) => Err(EpherError::Parse(format!(
                "expected {what}, found {other:?}"
            ))),
            None => Err(EpherError::Parse("unexpected end of input".into())),
        }
    }

    /// Top level: `if cond then a else b` or a comparison.
    fn parse_expression(&mut self) -> Result<Expression, EpherError> {
        if matches!(self.peek(), Some(Token::Ident(kw)) if kw == "if") {
            self.next(); // consume 'if'
            let cond = self.parse_expression()?;
            self.expect_keyword("then")?;
            let then_expr = self.parse_expression()?;
            self.expect_keyword("else")?;
            let else_expr = self.parse_expression()?;
            Ok(Expression::If(
                Box::new(cond),
                Box::new(then_expr),
                Box::new(else_expr),
            ))
        } else {
            self.parse_or()
        }
    }

    /// Boolean `or` level.
    fn parse_or(&mut self) -> Result<Expression, EpherError> {
        let mut left = self.parse_and()?;
        while matches!(self.peek(), Some(Token::Ident(kw)) if kw == "or") {
            self.next();
            let right = self.parse_and()?;
            left = Expression::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    /// Boolean `and` level.
    fn parse_and(&mut self) -> Result<Expression, EpherError> {
        let mut left = self.parse_not()?;
        while matches!(self.peek(), Some(Token::Ident(kw)) if kw == "and") {
            self.next();
            let right = self.parse_not()?;
            left = Expression::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    /// Boolean `not` level: binds looser than comparison (`not x > 3` is
    /// `not (x > 3)`).
    fn parse_not(&mut self) -> Result<Expression, EpherError> {
        if matches!(self.peek(), Some(Token::Ident(kw)) if kw == "not") {
            self.next();
            let inner = self.parse_not()?;
            Ok(Expression::Not(Box::new(inner)))
        } else {
            self.parse_comparison()
        }
    }

    fn expect_keyword(&mut self, kw: &str) -> Result<(), EpherError> {
        match self.next() {
            Some(Token::Ident(found)) if found == kw => Ok(()),
            Some(other) => Err(EpherError::Parse(format!(
                "expected '{kw}', found {other:?}"
            ))),
            None => Err(EpherError::Parse("unexpected end of input".into())),
        }
    }

    /// Comparison level: `>` `<` `>=` `<=` `==` `!=`, non-chaining, with
    /// arithmetic binding tighter.
    fn parse_comparison(&mut self) -> Result<Expression, EpherError> {
        let left = self.parse_additive()?;
        let op = match self.peek() {
            Some(Token::GreaterThan) => Some(CmpOp::Gt),
            Some(Token::LessThan) => Some(CmpOp::Lt),
            Some(Token::GreaterEqual) => Some(CmpOp::Ge),
            Some(Token::LessEqual) => Some(CmpOp::Le),
            Some(Token::EqualEqual) => Some(CmpOp::Eq),
            Some(Token::NotEqual) => Some(CmpOp::Ne),
            _ => None,
        };
        if let Some(op) = op {
            self.next();
            let right = self.parse_additive()?;
            Ok(Expression::Compare(op, Box::new(left), Box::new(right)))
        } else {
            Ok(left)
        }
    }

    /// Additive level: `+` and `-`, folded left-associatively.
    fn parse_additive(&mut self) -> Result<Expression, EpherError> {
        let mut left = self.parse_term()?;
        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.next();
                    let right = self.parse_term()?;
                    left = Expression::Add(Box::new(left), Box::new(right));
                }
                Some(Token::Minus) => {
                    self.next();
                    let right = self.parse_term()?;
                    left = Expression::Sub(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// Multiplicative level: `*` and `/`, folded left-associatively.
    fn parse_term(&mut self) -> Result<Expression, EpherError> {
        let mut left = self.parse_unary()?;
        loop {
            match self.peek() {
                Some(Token::Star) => {
                    self.next();
                    let right = self.parse_unary()?;
                    left = Expression::Mul(Box::new(left), Box::new(right));
                }
                Some(Token::Slash) => {
                    self.next();
                    let right = self.parse_unary()?;
                    left = Expression::Div(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// Unary level: `-` binds looser than `^` (math convention: `-2 ^ 2 = -4`).
    fn parse_unary(&mut self) -> Result<Expression, EpherError> {
        if matches!(self.peek(), Some(Token::Minus)) {
            self.next();
            let inner = self.parse_unary()?;
            Ok(Expression::Neg(Box::new(inner)))
        } else {
            self.parse_pow()
        }
    }

    /// Power level: `^`, right-associative, binds tighter than `*` and `/`; the
    /// exponent may itself be a unary expression (`2 ^ -2`).
    fn parse_pow(&mut self) -> Result<Expression, EpherError> {
        let base = self.parse_factor()?;
        if matches!(self.peek(), Some(Token::Caret)) {
            self.next();
            let exponent = self.parse_unary()?;
            Ok(Expression::Pow(Box::new(base), Box::new(exponent)))
        } else {
            Ok(base)
        }
    }

    fn parse_factor(&mut self) -> Result<Expression, EpherError> {
        let primary = self.parse_primary()?;
        // postfix factorial and percent bind tightest: 3! ^ 2 is (3!) ^ 2,
        // and 4!! is (4!)!; `!=` lexes as one token so `5! != 3` still
        // works. Percent is a transparent /100 suffix (ADR-0042): 5% is
        // 0.05, 200 + 10% is 200.1. The Casio add-on reading (200 + 10%
        // = 220) is deliberately not a grammar rule - "increase 200 by
        // 10%" is spelled 200 * (1 + 10%), which teaches what the
        // calculation actually is.
        let mut expr = primary;
        loop {
            match self.peek() {
                Some(Token::Bang) => {
                    self.next();
                    expr = Expression::Factorial(Box::new(expr));
                }
                Some(Token::Percent) => {
                    self.next();
                    expr = Expression::Div(Box::new(expr), Box::new(Expression::Literal(100.0)));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, EpherError> {
        match self.next() {
            Some(Token::Number(n)) => {
                // Unit-suffix literal (ADR-0037): a number immediately
                // followed by a unit token is that number times the
                // unit's SI factor, baked in at grammar level - user
                // shadowing cannot change what `2 AU` means. An Ident
                // followed by `(` is always a call, never a suffix, so
                // `30 deg(x)` stays a (trailing-input) parse error and
                // `min(3, 7)` keeps working next to `5 min`.
                if let Some(Token::Ident(name)) = self.peek().cloned() {
                    if let Some(factor) = unit_factor(&name) {
                        if !matches!(self.tokens.get(self.pos + 1), Some(Token::LParen)) {
                            self.next();
                            return Ok(Expression::Literal(n * factor));
                        }
                    }
                }
                Ok(Expression::Literal(n))
            }
            Some(Token::Imaginary(n)) => {
                // `4i` is the literal spelling of `4 * i` (ADR-0043); i
                // resolves as the builtin imaginary unit constant.
                Ok(Expression::Mul(
                    Box::new(Expression::Literal(n)),
                    Box::new(Expression::Var("i".into())),
                ))
            }
            Some(Token::Ident(name)) => {
                if matches!(self.peek(), Some(Token::LParen)) {
                    self.next(); // consume '(' — call syntax
                    let mut args = Vec::new();
                    if matches!(self.peek(), Some(Token::RParen)) {
                        self.next(); // zero-argument call
                    } else {
                        loop {
                            let arg = self.parse_expression()?;
                            args.push(arg);
                            match self.next() {
                                Some(Token::Comma) => continue,
                                Some(Token::RParen) => break,
                                Some(other) => {
                                    return Err(EpherError::Parse(format!(
                                        "expected ',' or ')', found {other:?}"
                                    )));
                                }
                                None => {
                                    return Err(EpherError::Parse(
                                        "unexpected end of input".into(),
                                    ));
                                }
                            }
                        }
                    }
                    Ok(Expression::Call(name, args))
                        // A unit token directly after a call result
                        // converts the count to SI - the ADR-0037 worked
                        // example is `mag2jy(20) Jy`: functions return
                        // counts, suffixes convert. Same call-versus-
                        // suffix disambiguation: the next token may not
                        // be a `(`.
                        .and_then(|expr| {
                            if let Some(Token::Ident(unit)) = self.peek().cloned() {
                                if let Some(factor) = unit_factor(&unit) {
                                    if !matches!(self.tokens.get(self.pos + 1), Some(Token::LParen))
                                    {
                                        self.next();
                                        return Ok(Expression::Mul(
                                            Box::new(expr),
                                            Box::new(Expression::Literal(factor)),
                                        ));
                                    }
                                }
                            }
                            Ok(expr)
                        })
                } else {
                    Ok(Expression::Var(name))
                }
            }
            Some(Token::LParen) => {
                let expr = self.parse_expression()?;
                match self.next() {
                    Some(Token::RParen) => Ok(expr),
                    Some(other) => Err(EpherError::Parse(format!("expected ')', found {other:?}"))),
                    None => Err(EpherError::Parse("unexpected end of input".into())),
                }
            }
            Some(other) => Err(EpherError::Parse(format!(
                "expected a number, found {other:?}"
            ))),
            None => Err(EpherError::Parse("unexpected end of input".into())),
        }
    }
}

/// Evaluate an [`Expression`] to a [`Value`] against an [`Env`] (the evaluation
/// seam).
pub fn eval(expr: &Expression, env: &Env) -> Result<Value, EpherError> {
    match expr {
        Expression::Literal(n) => Ok(Value::float(*n)),
        Expression::Var(name) => env
            .get(name)
            .cloned()
            .or_else(|| env.constant(name).cloned())
            .or_else(|| builtin_const(name))
            .ok_or_else(|| EpherError::UnknownName(name.clone())),
        Expression::Neg(inner) => match eval(inner, env)? {
            Value::Float(n) => Ok(Value::Float(-n)),
            Value::Complex(c) => Ok(Value::Complex(-c)),
            other => Err(EpherError::Type(format!("cannot negate {other:?}"))),
        },
        Expression::Add(lhs, rhs) => binop(eval(lhs, env)?, eval(rhs, env)?, BinOp::Add),
        Expression::Sub(lhs, rhs) => binop(eval(lhs, env)?, eval(rhs, env)?, BinOp::Sub),
        Expression::Mul(lhs, rhs) => binop(eval(lhs, env)?, eval(rhs, env)?, BinOp::Mul),
        Expression::Div(lhs, rhs) => binop(eval(lhs, env)?, eval(rhs, env)?, BinOp::Div),
        Expression::Pow(lhs, rhs) => binop(eval(lhs, env)?, eval(rhs, env)?, BinOp::Pow),
        Expression::Factorial(inner) => {
            let v = eval(inner, env)?;
            let x = one_float("!", &[v])?;
            let n = float_to_int(x)
                .ok_or_else(|| EpherError::Type(format!("! expects integers, got {x}")))?;
            Ok(Value::Float(factorial_value(n)?))
        }
        Expression::Compare(op, lhs, rhs) => {
            let l = eval(lhs, env)?;
            let r = eval(rhs, env)?;
            match (&l, &r) {
                (Value::Float(x), Value::Float(y)) => {
                    let result = match op {
                        CmpOp::Gt => x > y,
                        CmpOp::Lt => x < y,
                        CmpOp::Ge => x >= y,
                        CmpOp::Le => x <= y,
                        CmpOp::Eq => x == y,
                        CmpOp::Ne => x != y,
                    };
                    Ok(Value::Bool(result))
                }
                _ => Err(EpherError::Type(format!("cannot compare {l:?} and {r:?}"))),
            }
        }
        Expression::If(cond, then_expr, else_expr) => match eval(cond, env)? {
            Value::Bool(true) => eval(then_expr, env),
            Value::Bool(false) => eval(else_expr, env),
            other => Err(EpherError::Type(format!(
                "if condition must be a boolean, got {other:?}"
            ))),
        },
        Expression::And(lhs, rhs) => match eval(lhs, env)? {
            Value::Bool(false) => Ok(Value::Bool(false)),
            Value::Bool(true) => match eval(rhs, env)? {
                Value::Bool(b) => Ok(Value::Bool(b)),
                other => Err(EpherError::Type(format!(
                    "and expects booleans, got {other:?}"
                ))),
            },
            other => Err(EpherError::Type(format!(
                "and expects booleans, got {other:?}"
            ))),
        },
        Expression::Or(lhs, rhs) => match eval(lhs, env)? {
            Value::Bool(true) => Ok(Value::Bool(true)),
            Value::Bool(false) => match eval(rhs, env)? {
                Value::Bool(b) => Ok(Value::Bool(b)),
                other => Err(EpherError::Type(format!(
                    "or expects booleans, got {other:?}"
                ))),
            },
            other => Err(EpherError::Type(format!(
                "or expects booleans, got {other:?}"
            ))),
        },
        Expression::Not(inner) => match eval(inner, env)? {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            other => Err(EpherError::Type(format!(
                "not expects a boolean, got {other:?}"
            ))),
        },
        Expression::Call(name, args) => {
            if let Some(f) = env.function(name) {
                let mut values = Vec::with_capacity(args.len());
                for arg in args {
                    values.push(eval(arg, env)?);
                }
                if f.params.len() != values.len() {
                    return Err(EpherError::Type(format!(
                        "{name} expects {} arguments, got {}",
                        f.params.len(),
                        values.len()
                    )));
                }
                let mut child = Env::new_child(env);
                for (param, value) in f.params.iter().zip(values) {
                    child.set(param.clone(), value);
                }
                return eval(&f.body, &child);
            }
            // Numeric calculus (ADR-0043): the first argument stays an
            // expression - derivative(x^2, 3) differentiates, and
            // derivative(x^3, x) plots the derivative because the graph
            // sampler supplies x.
            match name.as_str() {
                "derivative" => eval_derivative(args, env),
                "integral" => eval_integral(args, env),
                _ => {
                    let mut values = Vec::with_capacity(args.len());
                    for arg in args {
                        values.push(eval(arg, env)?);
                    }
                    call_builtin(name, values)
                }
            }
        }
    }
}

/// The differentiation variable of a calculus expression (ADR-0043):
/// the expression's free variable, where constants (builtin and user)
/// and bound variables are parameters, not unknowns. No unbound names
/// means the expression is constant; several is an error.
fn calculus_var(expr: &Expression, env: &Env) -> Result<Option<String>, EpherError> {
    let mut names = std::collections::BTreeSet::new();
    crate::graph::free_names(expr, &mut names);
    names.retain(|n| {
        builtin_const(n).is_none() && env.constant(n).is_none() && env.get(n).is_none()
    });
    if names.is_empty() {
        return Ok(None);
    }
    if names.len() > 1 {
        return Err(EpherError::Type(format!(
            "the expression uses several variables: {}",
            names.iter().cloned().collect::<Vec<_>>().join(", ")
        )));
    }
    Ok(names.iter().next().cloned())
}

/// The calculus child environment: the caller's bindings (bound names
/// are parameters), the constants, the functions, and the calculus
/// variable bound to a fresh value - shadowing any session value.
fn calculus_child(env: &Env, var: &str, value: f64) -> Env {
    let mut child = Env {
        bindings: env.bindings.clone(),
        constants: env.constants.clone(),
        functions: env.functions.clone(),
    };
    child.set(var.to_string(), Value::float(value));
    child
}

/// `derivative(expr, p)` (ADR-0043): the numeric derivative of the
/// expression at p, 5-point central difference with step
/// 1e-4 * (1 + |p|). A constant expression differentiates to 0.
fn eval_derivative(args: &[Expression], env: &Env) -> Result<Value, EpherError> {
    if args.len() != 2 {
        return Err(EpherError::Type(format!(
            "derivative expects 2 arguments, got {}",
            args.len()
        )));
    }
    let p = match eval(&args[1], env)? {
        Value::Float(x) => x,
        other => {
            return Err(EpherError::Type(format!(
                "derivative expects a number at the point, got {other}"
            )))
        }
    };
    let Some(var) = calculus_var(&args[0], env)? else {
        return Ok(Value::float(0.0));
    };
    let h = 1e-4 * (1.0 + p.abs());
    let at = |x: f64| -> Result<f64, EpherError> {
        let child = calculus_child(env, &var, x);
        match eval(&args[0], &child)? {
            Value::Float(y) => Ok(y),
            other => Err(EpherError::Type(format!(
                "derivative expects a real-valued expression, got {other}"
            ))),
        }
    };
    // 5-point stencil: error ~ h^4 in the function, rounding ~ eps/h
    let _ym2 = at(p - 2.0 * h)?;
    let _ym1 = at(p - h)?;
    let _y1 = at(p + h)?;
    let _y2 = at(p + 2.0 * h)?;
    let slope = (_ym2 - 8.0 * _ym1 + 8.0 * _y1 - _y2) / (12.0 * h);
    Ok(Value::float(slope))
}

/// `integral(expr, a, b)` (ADR-0043): adaptive Simpson with a relative
/// tolerance of 1e-9, depth-capped; a == b integrates to 0, a > b gives
/// the signed integral. The expression's free variable is the
/// integration variable.
fn eval_integral(args: &[Expression], env: &Env) -> Result<Value, EpherError> {
    if args.len() != 3 {
        return Err(EpherError::Type(format!(
            "integral expects 3 arguments, got {}",
            args.len()
        )));
    }
    let (a, b) = match (eval(&args[1], env)?, eval(&args[2], env)?) {
        (Value::Float(a), Value::Float(b)) => (a, b),
        (other_a, other_b) => {
            return Err(EpherError::Type(format!(
                "integral expects numbers for the bounds, got {other_a} and {other_b}"
            )))
        }
    };
    let Some(var) = calculus_var(&args[0], env)? else {
        // integrating a constant over [a, b] is (b - a) * c
        let child = calculus_child(env, "__epher_const", a);
        match eval(&args[0], &child)? {
            Value::Float(c) => return Ok(Value::float((b - a) * c)),
            other => {
                return Err(EpherError::Type(format!(
                    "integral expects a real-valued expression, got {other}"
                )))
            }
        }
    };
    if a == b {
        return Ok(Value::float(0.0));
    }
    let f = |t: f64| -> Result<f64, EpherError> {
        let child = calculus_child(env, &var, t);
        match eval(&args[0], &child)? {
            Value::Float(y) => Ok(y),
            other => Err(EpherError::Type(format!(
                "integral expects a real-valued expression, got {other}"
            ))),
        }
    };
    Ok(Value::float(adaptive_simpson(&f, a, b, 1e-9)?))
}

/// Adaptive Simpson quadrature (ADR-0043): halves the interval while
/// the composite rule disagrees with the whole-interval rule, then
/// Richardson-extrapolates the correction; depth-capped so a
/// pathological integrand still returns a value.
fn adaptive_simpson(
    f: &impl Fn(f64) -> Result<f64, EpherError>,
    a: f64,
    b: f64,
    tol: f64,
) -> Result<f64, EpherError> {
    let fa = f(a)?;
    let fm = f((a + b) / 2.0)?;
    let fb = f(b)?;
    let whole = (b - a) / 6.0 * (fa + 4.0 * fm + fb);
    adaptive_step(f, a, b, fa, fm, fb, whole, tol, 0, 20)
}

#[allow(clippy::too_many_arguments)]
fn adaptive_step(
    f: &impl Fn(f64) -> Result<f64, EpherError>,
    a: f64,
    b: f64,
    fa: f64,
    fm: f64,
    fb: f64,
    whole: f64,
    tol: f64,
    depth: u32,
    max_depth: u32,
) -> Result<f64, EpherError> {
    let m = (a + b) / 2.0;
    let lm = (a + m) / 2.0;
    let rm = (m + b) / 2.0;
    let fl = f(lm)?;
    let fr = f(rm)?;
    let left = (m - a) / 6.0 * (fa + 4.0 * fl + fm);
    let right = (b - m) / 6.0 * (fm + 4.0 * fr + fb);
    let delta = left + right - whole;
    if depth >= max_depth || delta.abs() <= 15.0 * tol {
        return Ok(left + right + delta / 15.0);
    }
    let half = tol / 2.0;
    Ok(
        adaptive_step(f, a, m, fa, fl, fm, left, half, depth + 1, max_depth)?
            + adaptive_step(f, m, b, fm, fr, fb, right, half, depth + 1, max_depth)?,
    )
}

/// Evaluate source text as an expression with an empty environment — the CLI
/// one-shot convenience (composition of `parse` + `eval`, not a seam).
pub fn evaluate(text: &str) -> Result<Value, EpherError> {
    let env = Env::default();
    eval(&parse(text)?, &env)
}

/// A point on a sampled graph (ADR-0006: the core computes plot data, each
/// frontend renders it).
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    pub x: f64,
    pub y: f64,
}

/// Sample an [`Expression`] as `y = f(x)` over `x_min..=x_max` at `points`
/// evenly spaced values of `x`, which is bound in a child environment for each
/// point. Points where evaluation errors (domain gaps, division by zero) are
/// skipped so renderers can leave gaps.
pub fn sample(
    expr: &Expression,
    x_min: f64,
    x_max: f64,
    points: usize,
    env: &Env,
) -> Result<Vec<Sample>, EpherError> {
    let mut child = Env::new_child(env);
    let mut out = Vec::new();
    for i in 0..points {
        let t = if points == 1 {
            0.0
        } else {
            i as f64 / (points - 1) as f64
        };
        let x = x_min + t * (x_max - x_min);
        child.set("x", Value::float(x));
        if let Ok(Value::Float(y)) = eval(expr, &child) {
            out.push(Sample { x, y });
        }
    }
    Ok(out)
}

/// Sample a parametric curve `x(t), y(t)` over `t_min..=t_max` (ADR-0006).
/// `t` is bound in a child environment for each point; erroring points are
/// skipped.
pub fn sample_parametric(
    x_expr: &Expression,
    y_expr: &Expression,
    t_min: f64,
    t_max: f64,
    points: usize,
    env: &Env,
) -> Result<Vec<Sample>, EpherError> {
    let mut child = Env::new_child(env);
    let mut out = Vec::new();
    for i in 0..points {
        let t = if points == 1 {
            0.0
        } else {
            i as f64 / (points - 1) as f64
        };
        let t = t_min + t * (t_max - t_min);
        child.set("t", Value::float(t));
        let (Ok(Value::Float(x)), Ok(Value::Float(y))) =
            (eval(x_expr, &child), eval(y_expr, &child))
        else {
            continue;
        };
        out.push(Sample { x, y });
    }
    Ok(out)
}

/// Sample a polar curve `r(θ)` over `θ_min..=θ_max`, converted to x/y
/// (ADR-0006). `theta` is bound for each point; erroring points are skipped.
pub fn sample_polar(
    r_expr: &Expression,
    theta_min: f64,
    theta_max: f64,
    points: usize,
    env: &Env,
) -> Result<Vec<Sample>, EpherError> {
    let mut child = Env::new_child(env);
    let mut out = Vec::new();
    for i in 0..points {
        let t = if points == 1 {
            0.0
        } else {
            i as f64 / (points - 1) as f64
        };
        let theta = theta_min + t * (theta_max - theta_min);
        child.set("theta", Value::float(theta));
        let Ok(Value::Float(r)) = eval(r_expr, &child) else {
            continue;
        };
        out.push(Sample {
            x: r * theta.cos(),
            y: r * theta.sin(),
        });
    }
    Ok(out)
}

/// The unit-suffix table (ADR-0037): exact token → SI factor. Length in
/// metres, angle in radians, time in seconds, flux in watts per square
/// metre hertz. `h` is deliberately absent - Planck's constant owns the
/// single letter, and hours are spelled `hr`.
fn unit_factor(token: &str) -> Option<f64> {
    match token {
        "AU" | "au" => Some(1.495_978_707e11),
        "pc" => Some(3.085_677_581_491_367_3e16),
        "ly" => Some(9.460_730_472_580_8e15),
        "deg" => Some(std::f64::consts::PI / 180.0),
        "arcmin" => Some(std::f64::consts::PI / 10_800.0),
        "arcsec" => Some(std::f64::consts::PI / 648_000.0),
        "min" => Some(60.0),
        "hr" => Some(3_600.0),
        "d" => Some(86_400.0),
        "yr" => Some(31_557_600.0),
        "Jy" => Some(1e-26),
        _ => None,
    }
}

/// Deterministic Miller-Rabin on u64: this witness set decides every
/// n < 2^64 exactly (no probabilistic error), so primality is exact on
/// the whole range f64 can address (ADR-0042).
fn is_prime_u64(n: u64) -> bool {
    const WITNESSES: [u64; 12] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
    if n < 2 {
        return false;
    }
    for p in WITNESSES {
        if n.is_multiple_of(p) {
            return n == p;
        }
    }
    let mut d = n - 1;
    let mut s = 0u32;
    while d.is_multiple_of(2) {
        d /= 2;
        s += 1;
    }
    'witness: for a in WITNESSES {
        let mut x = mod_pow_u64(a, d, n);
        if x == 1 || x == n - 1 {
            continue;
        }
        for _ in 1..s {
            x = mul_mod_u64(x, x, n);
            if x == n - 1 {
                continue 'witness;
            }
        }
        return false;
    }
    true
}

/// (base ^ exp) mod m with u128 intermediates - no overflow for any u64
/// input.
fn mod_pow_u64(mut base: u64, mut exp: u64, m: u64) -> u64 {
    if m == 1 {
        return 0;
    }
    let mut acc = 1u64;
    base %= m;
    while exp > 0 {
        if !exp.is_multiple_of(2) {
            acc = mul_mod_u64(acc, base, m);
        }
        base = mul_mod_u64(base, base, m);
        exp /= 2;
    }
    acc
}

fn mul_mod_u64(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 * b as u128) % m as u128) as u64
}

/// Pollard rho (Floyd-style stepping): a nontrivial factor of composite n.
/// Degenerate runs (the cycle swallowed n itself) retry with the next
/// polynomial x^2 + c.
fn pollard_rho(n: u64) -> u64 {
    let mut c = 1u64;
    loop {
        let step = |x: u64| (mul_mod_u64(x, x, n) + c) % n;
        let mut x = 2u64;
        let mut y = 2u64;
        let mut d = 1u64;
        while d == 1 {
            x = step(x);
            y = step(step(y));
            d = gcd_u64(x.abs_diff(y), n);
        }
        if d != n {
            return d;
        }
        c += 1;
    }
}

fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// The prime factorization of n >= 1 as (prime, exponent) pairs ascending:
/// small primes by trial division, then splits proved prime with the exact
/// Miller-Rabin. Powers stay grouped, so 360 is 2^3 * 3^2 * 5.
fn prime_factorization(mut n: u64) -> Vec<(u64, u32)> {
    let mut out: Vec<(u64, u32)> = Vec::new();
    for p in [
        2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83,
        89, 97,
    ] {
        if n.is_multiple_of(p) {
            let mut k = 0u32;
            while n.is_multiple_of(p) {
                n /= p;
                k += 1;
            }
            out.push((p, k));
        }
    }
    if n == 1 {
        return out;
    }
    let mut stack = vec![n];
    let mut found: Vec<u64> = Vec::new();
    while let Some(m) = stack.pop() {
        if is_prime_u64(m) {
            found.push(m);
            continue;
        }
        let d = pollard_rho(m);
        stack.push(d);
        stack.push(m / d);
    }
    found.sort_unstable();
    for p in found {
        match out.last_mut() {
            Some(last) if last.0 == p => last.1 += 1,
            _ => out.push((p, 1)),
        }
    }
    out
}

/// Built-in constants (pi, e, tau, phi), resolved when a name isn't in the
/// environment.
fn builtin_const(name: &str) -> Option<Value> {
    match name {
        "pi" => Some(Value::float(std::f64::consts::PI)),
        "e" => Some(Value::float(std::f64::consts::E)),
        "tau" => Some(Value::float(std::f64::consts::TAU)),
        "phi" => Some(Value::float(1.618_033_988_749_895)),
        // The imaginary unit (ADR-0043): `i` is a constant like `pi`, and
        // `4i` is its literal spelling. Shadowable like every builtin.
        "i" => Some(Value::Complex(Complex::new(0.0, 1.0))),
        // Astronomy constants (ADR-0037): SI values throughout - metres,
        // seconds, kilograms, watts. Shadowable like `pi` (resolution
        // order: user variable, user constant, builtin).
        "au" => Some(Value::float(1.495_978_707e11)),
        "pc" => Some(Value::float(3.085_677_581_491_367_3e16)),
        "ly" => Some(Value::float(9.460_730_472_580_8e15)),
        "c" => Some(Value::float(2.997_924_58e8)),
        "g" => Some(Value::float(9.806_65)),
        "h" => Some(Value::float(6.626_070_15e-34)),
        "h_bar" => Some(Value::float(
            6.626_070_15e-34 / (2.0 * std::f64::consts::PI),
        )),
        "k_b" => Some(Value::float(1.380_649e-23)),
        "sigma_sb" => Some(Value::float(5.670_374_419e-8)),
        "m_sun" => Some(Value::float(1.988_47e30)),
        "r_sun" => Some(Value::float(6.957e8)),
        "l_sun" => Some(Value::float(3.828e26)),
        "m_earth" => Some(Value::float(5.972_2e24)),
        "r_earth" => Some(Value::float(6.371e6)),
        // Physical constants (ADR-0042): CODATA 2022 values, SI units
        // throughout (eV in joules, atm in pascals). Shadowable like `pi`
        // - a user `const` wins by the resolution order.
        "G" => Some(Value::float(6.6743e-11)),
        "gamma" => Some(Value::float(0.577_215_664_901_532_9)),
        "q_e" => Some(Value::float(1.602_176_634e-19)),
        "ev" => Some(Value::float(1.602_176_634e-19)),
        "eps_0" => Some(Value::float(8.854_187_812_8e-12)),
        "mu_0" => Some(Value::float(1.256_637_062_12e-6)),
        "z_0" => Some(Value::float(376.730_313_668)),
        "m_e" => Some(Value::float(9.109_383_713_9e-31)),
        "m_p" => Some(Value::float(1.672_621_925_95e-27)),
        "m_n" => Some(Value::float(1.674_927_500_56e-27)),
        "m_u" => Some(Value::float(1.660_539_068_92e-27)),
        "a_0" => Some(Value::float(5.291_772_105_44e-11)),
        "alpha" => Some(Value::float(7.297_352_564_3e-3)),
        "r_inf" => Some(Value::float(10_973_731.568_16)),
        "mu_b" => Some(Value::float(9.274_010_078_3e-24)),
        "n_a" => Some(Value::float(6.022_140_76e23)),
        "faraday" => Some(Value::float(96_485.332_12)),
        "r_gas" => Some(Value::float(8.314_462_618_153_24)),
        "atm" => Some(Value::float(101_325.0)),
        "wien" => Some(Value::float(2.897_771_955e-3)),
        "phi_0" => Some(Value::float(2.067_833_848e-15)),
        _ => None,
    }
}

/// What kind of thing a builtin catalog entry names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogKind {
    Function,
    Constant,
}

/// One builtin name - the autocomplete/F1 index (ADR-0042). Descriptive
/// only: a name missing from the catalog still evaluates; it just does not
/// suggest. Frontends merge the session's user functions and constants on
/// top of this table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CatalogEntry {
    pub name: &'static str,
    pub kind: CatalogKind,
}

/// The builtin catalog, sorted by name so suggestions appear in a stable
/// order everywhere.
static BUILTIN_CATALOG: &[CatalogEntry] = &[
    CatalogEntry {
        name: "G",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "a_0",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "abs",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "acos",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "acosh",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "alpha",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "alt",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "arg",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "asin",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "asinh",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "atan",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "atan2",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "atanh",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "atm",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "au",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "az",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "big",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "bin",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "c",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "cbrt",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "ceil",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "conj",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "cos",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "cosh",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "dec",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "decl",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "deg",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "deg2hms",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "delta_t",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "derivative",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "dist",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "e",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "engineering",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "eps_0",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "ev",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "exact",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "exp",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "fact",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "factors",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "faraday",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "floor",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "frac",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "g",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "gamma",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "gcd",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "grouped",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "h_bar",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "hex",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "hms2deg",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "hypot",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "i",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "im",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "integral",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "isprime",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "jd",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "k_b",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "kepler",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "l_sun",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "lcm",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "ln",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "log",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "log2",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "logb",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "lst",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "ly",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "m_e",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "m_n",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "m_p",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "m_sun",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "m_u",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "mag2jy",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "max",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "mean",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "median",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "min",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "mjd",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "mu_0",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "mu_b",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "n_a",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "ncr",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "ndivisors",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "nextprime",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "now",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "npr",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "oct",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "pc",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "phi",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "phi_0",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "pi",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "prevprime",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "product",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "q_e",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "r_earth",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "r_gas",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "r_inf",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "r_sun",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "ra",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "rad",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "re",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "rise",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "root",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "round",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "scientific",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "set",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "sigma_sb",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "sign",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "sin",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "sinh",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "sqrt",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "stdev",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "sum",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "tan",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "tanh",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "tau",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "totient",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "trunc",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "wien",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "z_0",
        kind: CatalogKind::Constant,
    },
];

/// The sorted builtin catalog: every function and constant the language
/// ships, for autocomplete and F1 help (ADR-0042).
pub fn catalog() -> &'static [CatalogEntry] {
    BUILTIN_CATALOG
}

/// Take exactly one Float argument.
fn one_float(name: &str, args: &[Value]) -> Result<f64, EpherError> {
    match args {
        [Value::Float(x)] => Ok(*x),
        _ => Err(EpherError::Type(format!(
            "{name} expects 1 number, got {} argument(s)",
            args.len()
        ))),
    }
}

/// Take exactly one argument of any kind.
fn one_arg<'a>(name: &str, args: &'a [Value]) -> Result<&'a Value, EpherError> {
    match args {
        [v] => Ok(v),
        _ => Err(EpherError::Type(format!(
            "{name} expects 1 argument, got {} argument(s)",
            args.len()
        ))),
    }
}

/// The one-argument transcendental bridge (ADR-0043): a complex
/// argument computes in the complex plane; a real argument computes
/// with `real`, and a domain error there falls back to the principal
/// complex result - `sqrt(-1)` is `i`, `ln(-1)` is `i*pi`, `asin(2)`
/// is complex. Non-domain errors (step limits, division by zero) pass
/// through unchanged.
fn real_or_complex(
    name: &str,
    args: &[Value],
    real: impl Fn(f64) -> Result<f64, EpherError>,
    complex: impl Fn(Complex<f64>) -> Complex<f64>,
) -> Result<Value, EpherError> {
    let v = one_arg(name, args)?;
    match v {
        Value::Float(x) => match real(*x) {
            Ok(y) => Ok(Value::Float(y)),
            Err(EpherError::Domain(_)) => Ok(Value::Complex(complex(Complex::new(*x, 0.0)))),
            Err(e) => Err(e),
        },
        Value::Complex(c) => Ok(Value::Complex(complex(*c))),
        other => Err(EpherError::Type(format!(
            "{name} expects a number, got {other}"
        ))),
    }
}

/// The whole-number reading of a value, for the base-conversion builtins
/// (ADR-0022). Exact values convert exactly (rationals and decimals too);
/// a fractional or non-numeric value is a type error.
fn value_to_bigint(name: &str, v: &Value) -> Result<num_bigint::BigInt, EpherError> {
    let bad = || EpherError::Type(format!("{name} expects an integer, got {v}"));
    match v {
        Value::Big(b) => {
            if !b.is_integer() {
                return Err(bad());
            }
            num_bigint::BigInt::parse_bytes(b.to_string().as_bytes(), 10).ok_or_else(bad)
        }
        Value::Float(n) => {
            if !n.is_finite() || n.fract() != 0.0 {
                return Err(bad());
            }
            // An integral f64 formats exactly — the shortest round-trip
            // representation of an integral float is its exact integer.
            num_bigint::BigInt::parse_bytes(format!("{n:.0}").as_bytes(), 10).ok_or_else(bad)
        }
        Value::Rational(r) => {
            if r.denom() != &num_bigint::BigInt::from(1) {
                return Err(bad());
            }
            Ok(r.numer().clone())
        }
        Value::Decimal(d) => {
            if !d.fract().is_zero() {
                return Err(bad());
            }
            num_bigint::BigInt::parse_bytes(d.trunc().to_string().as_bytes(), 10).ok_or_else(bad)
        }
        _ => Err(bad()),
    }
}

/// Take exactly three Float arguments.
fn three_floats(name: &str, args: &[Value]) -> Result<(f64, f64, f64), EpherError> {
    match args {
        [Value::Float(a), Value::Float(b), Value::Float(c)] => Ok((*a, *b, *c)),
        _ => Err(EpherError::Type(format!(
            "{name} expects 3 numbers, got {} argument(s)",
            args.len()
        ))),
    }
}

/// Take exactly two Float arguments.
fn two_floats(name: &str, args: &[Value]) -> Result<(f64, f64), EpherError> {
    match args {
        [Value::Float(a), Value::Float(b)] => Ok((*a, *b)),
        _ => Err(EpherError::Type(format!(
            "{name} expects 2 numbers, got {} argument(s)",
            args.len()
        ))),
    }
}

/// Take one or more Float arguments (variadic statistics and min/max).
fn any_floats(name: &str, args: &[Value]) -> Result<Vec<f64>, EpherError> {
    if args.is_empty() {
        return Err(EpherError::Type(format!(
            "{name} expects at least 1 number, got 0"
        )));
    }
    let mut out = Vec::with_capacity(args.len());
    for arg in args {
        match arg {
            Value::Float(x) => out.push(*x),
            other => {
                return Err(EpherError::Type(format!(
                    "{name} expects numbers, got {other:?}"
                )))
            }
        }
    }
    Ok(out)
}

/// Reject an out-of-domain argument.
fn domain_error(message: impl std::fmt::Display) -> EpherError {
    EpherError::Domain(message.to_string())
}

// --- exact fractions and display formats (ADR-0043) -------------------

/// The float reading of a numeric value for the display verbs.
fn numeric_as_float(name: &str, v: &Value) -> Result<f64, EpherError> {
    let bad = || EpherError::Type(format!("{name} expects a number, got {v}"));
    match v {
        Value::Float(x) => Ok(*x),
        Value::Rational(r) => r.to_f64().ok_or_else(bad),
        Value::Decimal(d) => d.to_f64().ok_or_else(bad),
        Value::Big(b) => b.to_f64().ok_or_else(bad),
        _ => Err(bad()),
    }
}

/// Continued-fraction rational reconstruction (ADR-0043): the first
/// convergent whose denominator stays within the bound and whose error
/// is below the relative tolerance. `reconstruct_fraction(1.0 / 3.0,
/// 1000, 1e-9)` is 1/3; pi and sqrt(2) return None because their good
/// convergents either exceed the bound or miss the tolerance.
pub fn reconstruct_fraction(x: f64, denom_bound: i64, tol: f64) -> Option<BigRational> {
    if !x.is_finite() || x == 0.0 {
        return None;
    }
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let a = x.abs();
    let bound = BigInt::from(denom_bound);
    let mut h0 = BigInt::from(0);
    let mut h1 = BigInt::from(1);
    let mut k0 = BigInt::from(1);
    let mut k1 = BigInt::from(0);
    let mut r = a;
    for _ in 0..64 {
        let n = r.floor();
        let n_int = BigInt::from(n as i64);
        let h2 = &n_int * &h1 + &h0;
        let k2 = &n_int * &k1 + &k0;
        let within_bound = k2 <= bound;
        let (h, k) = if within_bound { (&h2, &k2) } else { (&h1, &k1) };
        if !k.is_zero() {
            let guess = h.to_f64()? / k.to_f64()?;
            if (guess - a).abs() <= tol * (1.0 + a.abs()) {
                let numer = (sign * h.to_f64()?).round() as i64;
                return Some(BigRational::new(BigInt::from(numer), k.clone()));
            }
        }
        if !within_bound || n == r {
            break;
        }
        let f = r - n;
        if f < 1e-15 {
            break;
        }
        h0 = h1;
        h1 = h2;
        k0 = k1;
        k1 = k2;
        r = 1.0 / f;
    }
    None
}

/// Engineering notation (ADR-0043): the mantissa in [1, 1000) and the
/// exponent a multiple of 3 - `12.345e3`, `500e-3`, `999e0` stays `999`.
fn engineering_str(x: f64) -> String {
    if x == 0.0 {
        return "0".into();
    }
    let sign = if x < 0.0 { "-" } else { "" };
    let a = x.abs();
    let e = a.log10().floor() as i64;
    let e3 = e - e.rem_euclid(3);
    let m = a / 10f64.powi(e3 as i32);
    if e3 == 0 {
        format!("{sign}{m}")
    } else {
        format!("{sign}{m}e{e3}")
    }
}

/// Thin-space thousands grouping (ADR-0043), locale-neutral ISO style:
/// `1 234 567.89`. Only the integer part groups; the exponent and any
/// trailing fraction digits stay put.
fn grouped_str(s: &str) -> String {
    let (head, tail) = match s.split_once('.') {
        Some((i, rest)) => (i, format!(".{rest}")),
        None => (s, String::new()),
    };
    let (neg, digits) = match head.strip_prefix('-') {
        Some(d) => ("-", d),
        None => ("", head),
    };
    let mut out = String::new();
    let len = digits.len();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            out.push('\u{2009}');
        }
        out.push(c);
    }
    format!("{neg}{out}{tail}")
}

/// How the interactive frontends render results (ADR-0043): the exact
/// fraction toggle (default on - 1/3 shows as 1/3), the notation, and
/// the thousands separator. The value itself is untouched (ADR-0005).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayPrefs {
    pub exact_fractions: bool,
    pub notation: Notation,
    pub separators: bool,
}

/// Result notation (ADR-0043): Auto is the shortest float; the other
/// two force their exponent shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notation {
    Auto,
    Scientific,
    Engineering,
}

impl Default for DisplayPrefs {
    fn default() -> Self {
        DisplayPrefs {
            exact_fractions: true,
            notation: Notation::Auto,
            separators: false,
        }
    }
}

/// The result string for a value under the display preferences
/// (ADR-0043). Exact fractions apply in Auto mode; the notation modes
/// always win, and separators group Auto and notation digits alike.
pub fn format_value(v: &Value, prefs: &DisplayPrefs) -> String {
    match v {
        Value::Float(x) => {
            let s = match prefs.notation {
                Notation::Auto => {
                    if prefs.exact_fractions {
                        match reconstruct_fraction(*x, 1000, 1e-9) {
                            Some(r) => return format!("{r}"),
                            None => format!("{x}"),
                        }
                    } else {
                        format!("{x}")
                    }
                }
                Notation::Scientific => format!("{x:e}"),
                Notation::Engineering => engineering_str(*x),
            };
            if prefs.separators {
                grouped_str(&s)
            } else {
                s
            }
        }
        other => format!("{other}"),
    }
}

/// A finite float with an integer value, as i64 (shared by the integer
/// function family: frac, fact, ncr, npr, gcd, lcm, mod).
fn float_to_int(x: f64) -> Option<i64> {
    if x.is_finite() && x.fract() == 0.0 && x.abs() <= i64::MAX as f64 {
        Some(x as i64)
    } else {
        None
    }
}

/// Exactly one integer-valued Float argument, as i64.
fn integer_arg(name: &str, args: &[Value]) -> Result<i64, EpherError> {
    let x = one_float(name, args)?;
    float_to_int(x).ok_or_else(|| EpherError::Type(format!("{name} expects integers, got {x}")))
}

/// Exactly two integer-valued Float arguments, as i64.
fn integer_pair(name: &str, args: &[Value]) -> Result<(i64, i64), EpherError> {
    let (a, b) = two_floats(name, args)?;
    let (Some(a), Some(b)) = (float_to_int(a), float_to_int(b)) else {
        return Err(EpherError::Type(format!(
            "{name} expects integers, got {a} and {b}"
        )));
    };
    Ok((a, b))
}

/// n! as a float, erroring for negatives and beyond 170! (the f64 limit).
fn factorial_value(n: i64) -> Result<f64, EpherError> {
    if n < 0 {
        return Err(domain_error(format!("factorial of negative number {n}")));
    }
    let mut acc = 1.0;
    for i in 2..=n {
        acc *= i as f64;
        if !acc.is_finite() {
            return Err(domain_error(format!("factorial of {n} overflows")));
        }
    }
    Ok(acc)
}

// --- equation solving (ADR-0043) --------------------------------------
// `solve lhs == rhs` finds roots of f = lhs - rhs. Polynomial equations
// get every root, real and complex, through Durand-Kerner iteration on a
// coefficient vector; anything else is scanned numerically over -100..100
// with sign-change brackets, bisection safeguard, and Newton polish. No
// CAS: both paths are pure f64 arithmetic.

/// The display for a solved root: `x = 2` (roots within 1e-5 of an
/// integer print without a decimal point - repeated-root clusters land
/// at 1e-6 to 1e-5 even after refinement), `x = i` for the pure unit,
/// `x = 1+2i` otherwise.
fn root_display(root: Complex<f64>) -> String {
    if root.im.abs() < 1e-5 * (1.0 + root.re.abs()) {
        let re = root.re;
        let re = if (re - re.round()).abs() <= 1e-5 * (1.0 + re.abs()) {
            re.round()
        } else {
            re
        };
        format!("{re}")
    } else {
        complex_display(root)
    }
}

/// Solve a `solve` statement (ADR-0043). The equation must be a `==`
/// comparison; the variable solved for is `x` when it appears, otherwise
/// the single free variable.
pub fn solve_statement(equation: &Expression, env: &Env) -> Result<Value, EpherError> {
    let (lhs, rhs) = match equation {
        Expression::Compare(CmpOp::Eq, l, r) => (l.as_ref(), r.as_ref()),
        _ => {
            return Err(EpherError::Type(
                "solve needs an equation with ==, like solve x^2 == 9".into(),
            ))
        }
    };
    let mut names = std::collections::BTreeSet::new();
    epher_core_graph_free_names_helper(lhs, &mut names);
    epher_core_graph_free_names_helper(rhs, &mut names);
    // constants (builtin or user) are parameters, never unknowns; user
    // variables stay in the running, the solved-for one is symbolic
    // even when the session holds a value for it (like every calculator)
    names.retain(|n| builtin_const(n).is_none() && env.constant(n).is_none());
    let variable = if names.contains("x") {
        "x"
    } else if names.len() == 1 {
        names.iter().next().expect("len checked").as_str()
    } else if names.is_empty() {
        return Err(EpherError::Type(
            "solve found no variable in the equation".into(),
        ));
    } else {
        return Err(EpherError::Type(format!(
            "solve found several variables: {}",
            names.into_iter().collect::<Vec<_>>().join(", ")
        )));
    };
    // every name besides the solved-for one must be a bound parameter
    // (a value, not an unknown): `solve a*x == 6` works once `a = 2`
    for other in names.iter().filter(|n| n.as_str() != variable) {
        if env.get(other).is_none() {
            return Err(EpherError::Type(format!(
                "solve needs an equation in one variable; {other} is not bound, so {variable} is not the only unknown ({})",
                names.iter().cloned().collect::<Vec<_>>().join(", ")
            )));
        }
    }

    // Polynomial path: all roots, real and complex.
    if let Some(coeffs) = poly_coeffs(lhs, variable, env) {
        if let Some(rhs_coeffs) = poly_coeffs(rhs, variable, env) {
            let mut f = subtract_polys(coeffs, rhs_coeffs);
            trim_poly(&mut f);
            if f.len() <= 1 {
                // constant f: 0 == 0 solves trivially, anything else has
                // no roots
                if f.first().map(|c| *c == 0.0).unwrap_or(true) {
                    return Ok(Value::Str("x is any number".into()));
                }
                return Ok(Value::Str("no solution".into()));
            }
            let mut roots = durand_kerner(&f);
            roots.sort_by(|a, b| {
                (a.re, a.im)
                    .partial_cmp(&(b.re, b.im))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let out = roots
                .iter()
                .map(|r| format!("x = {}", root_display(*r)))
                .collect::<Vec<_>>()
                .join(", ");
            return Ok(Value::Str(out));
        }
    }

    // Numeric path: scan -100..100 for sign changes, bisection then
    // Newton; poles are rejected by the residual check.
    let eval_f = |v: f64| -> Result<f64, EpherError> {
        let mut child = env.new_child();
        child.set(variable.to_string(), Value::float(v));
        let a = eval(lhs, &child)?;
        let b = eval(rhs, &child)?;
        let (Value::Float(a), Value::Float(b)) = (a, b) else {
            return Err(EpherError::Type(
                "solve needs a real-valued equation over the domain".into(),
            ));
        };
        Ok(a - b)
    };
    let lo = -100.0;
    let hi = 100.0;
    let n = 2000usize;
    let mut brackets: Vec<(f64, f64)> = Vec::new();
    let mut prev_x = lo;
    let mut prev_y = eval_f(lo)?;
    for i in 1..=n {
        let x = lo + (hi - lo) * (i as f64) / (n as f64);
        let y = eval_f(x)?;
        if prev_y.is_finite() && y.is_finite() && prev_y * y <= 0.0 {
            // a root sitting exactly on a sample point (sin at 0) still
            // brackets: the product is zero, and the bisection below
            // finds it; poles are rejected later by the residual check
            brackets.push((prev_x, x));
        }
        prev_x = x;
        prev_y = y;
    }
    let mut roots: Vec<f64> = Vec::new();
    for (a, b) in brackets {
        // bisection to a tight bracket
        let mut lo_b = a;
        let mut hi_b = b;
        let mut f_lo = eval_f(lo_b)?;
        for _ in 0..60 {
            let mid = (lo_b + hi_b) / 2.0;
            let f_mid = eval_f(mid)?;
            if f_lo * f_mid <= 0.0 {
                hi_b = mid;
            } else {
                lo_b = mid;
                f_lo = f_mid;
            }
        }
        // Newton polish from the bracket's mid
        let mut x = (lo_b + hi_b) / 2.0;
        for _ in 0..30 {
            let h = 1e-7 * (1.0 + x.abs());
            let fx = eval_f(x)?;
            let fp = (eval_f(x + h)? - eval_f(x - h)?) / (2.0 * h);
            if fp == 0.0 {
                break;
            }
            let next = x - fx / fp;
            if (next - x).abs() < 1e-12 * (1.0 + x.abs()) {
                x = next;
                break;
            }
            x = next;
        }
        // residual check rejects poles and escaped iterates
        if eval_f(x)?.abs() < 1e-6 * (1.0 + x.abs())
            && x > lo - 1e-6
            && x < hi + 1e-6
            && roots.iter().all(|r| (r - x).abs() > 1e-6)
        {
            roots.push(x);
        }
    }
    roots.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if roots.is_empty() {
        return Ok(Value::Str("no real roots found in -100..100".into()));
    }
    let out = roots
        .iter()
        .map(|r| format!("x = {}", root_display(Complex::new(*r, 0.0))))
        .collect::<Vec<_>>()
        .join(", ");
    Ok(Value::Str(out))
}

/// The free names of one side of a solve equation (graph.rs owns the
/// shared walk; reuse it through this shim so solve sees the same names).
pub fn epher_core_graph_free_names_helper(
    expr: &Expression,
    out: &mut std::collections::BTreeSet<String>,
) {
    crate::graph::free_names(expr, out);
}

/// Structural polynomial extraction (ADR-0043): coefficients of the
/// expression as a polynomial in `variable`, lowest degree first, or
/// None when the expression is not polynomial (calls on the variable,
/// division by a non-constant, fractional exponents). Constants resolve
/// against the environment, so `solve a*x == 12` works with `a` bound.
fn poly_coeffs(expr: &Expression, variable: &str, env: &Env) -> Option<Vec<f64>> {
    const MAX_DEGREE: usize = 12;
    let poly = |coeffs: Vec<f64>| -> Option<Vec<f64>> {
        if coeffs.len() > MAX_DEGREE + 1 {
            None
        } else {
            Some(coeffs)
        }
    };
    match expr {
        Expression::Literal(n) => Some(vec![*n]),
        Expression::Var(name) => {
            if name == variable {
                Some(vec![0.0, 1.0])
            } else if let Some(v) = env
                .get(name)
                .cloned()
                .or_else(|| env.constant(name).cloned())
                .or_else(|| builtin_const(name))
            {
                match v {
                    Value::Float(x) => Some(vec![x]),
                    Value::Rational(r) => r.to_f64().map(|x| vec![x]),
                    Value::Decimal(d) => d.to_f64().map(|x| vec![x]),
                    Value::Big(b) => b.to_f64().map(|x| vec![x]),
                    _ => None,
                }
            } else {
                None
            }
        }
        Expression::Neg(e) => {
            poly_coeffs(e, variable, env).map(|c| c.into_iter().map(|x| -x).collect())
        }
        Expression::Add(a, b) => Some(add_polys(
            poly_coeffs(a, variable, env)?,
            poly_coeffs(b, variable, env)?,
        )),
        Expression::Sub(a, b) => Some(subtract_polys(
            poly_coeffs(a, variable, env)?,
            poly_coeffs(b, variable, env)?,
        )),
        Expression::Mul(a, b) => Some(mul_polys(
            poly_coeffs(a, variable, env)?,
            poly_coeffs(b, variable, env)?,
        )),
        Expression::Div(a, b) => {
            let denom = poly_coeffs(b, variable, env)?;
            if denom.len() != 1 || denom[0] == 0.0 {
                return None; // division by a non-constant is not polynomial
            }
            poly_coeffs(a, variable, env).map(|c| c.into_iter().map(|x| x / denom[0]).collect())
        }
        Expression::Pow(base, exp) => {
            // An integer-literal (or constant-valued) exponent; a
            // polynomial base raised to it stays polynomial.
            let n = match exp.as_ref() {
                Expression::Literal(k)
                    if *k >= 0.0 && k.fract() == 0.0 && *k <= MAX_DEGREE as f64 =>
                {
                    *k as usize
                }
                _ => {
                    let coeffs = poly_coeffs(exp, variable, env)?;
                    if coeffs.len() != 1 {
                        return None;
                    }
                    let k = coeffs[0];
                    if k < 0.0 || k.fract() != 0.0 || k > MAX_DEGREE as f64 {
                        return None;
                    }
                    k as usize
                }
            };
            let base_coeffs = poly_coeffs(base, variable, env)?;
            let mut acc = vec![1.0];
            for _ in 0..n {
                acc = mul_polys(acc, base_coeffs.clone());
                if acc.len() > MAX_DEGREE + 1 {
                    return None;
                }
            }
            Some(acc)
        }
        Expression::Call(name, args) => {
            // A call that mentions the variable is not polynomial; a
            // call that does not is a constant factor (sin(1), log(10)).
            let mut names = std::collections::BTreeSet::new();
            for a in args {
                crate::graph::free_names(a, &mut names);
            }
            if names.iter().any(|n| n == variable) {
                return None;
            }
            let child = env.new_child();
            let mut values = Vec::with_capacity(args.len());
            for a in args {
                values.push(eval(a, &child).ok()?);
            }
            match call_builtin(name, values).ok()? {
                Value::Float(x) => Some(vec![x]),
                Value::Rational(r) => r.to_f64().map(|x| vec![x]),
                Value::Decimal(d) => d.to_f64().map(|x| vec![x]),
                Value::Big(b) => b.to_f64().map(|x| vec![x]),
                _ => None,
            }
        }
        Expression::Factorial(_)
        | Expression::Compare(_, _, _)
        | Expression::If(_, _, _)
        | Expression::And(_, _)
        | Expression::Or(_, _)
        | Expression::Not(_) => None,
    }
    .and_then(poly)
}

fn add_polys(a: Vec<f64>, b: Vec<f64>) -> Vec<f64> {
    let n = a.len().max(b.len());
    let mut out = vec![0.0; n];
    for (i, x) in a.iter().enumerate() {
        out[i] += x;
    }
    for (i, x) in b.iter().enumerate() {
        out[i] += x;
    }
    out
}

fn subtract_polys(a: Vec<f64>, b: Vec<f64>) -> Vec<f64> {
    let n = a.len().max(b.len());
    let mut out = vec![0.0; n];
    for (i, x) in a.iter().enumerate() {
        out[i] += x;
    }
    for (i, x) in b.iter().enumerate() {
        out[i] -= x;
    }
    out
}

fn mul_polys(a: Vec<f64>, b: Vec<f64>) -> Vec<f64> {
    let mut out = vec![0.0; a.len() + b.len() - 1];
    for (i, x) in a.iter().enumerate() {
        for (j, y) in b.iter().enumerate() {
            out[i + j] += x * y;
        }
    }
    out
}

fn trim_poly(p: &mut Vec<f64>) {
    while p.len() > 1 && p.last().map(|c| *c == 0.0).unwrap_or(false) {
        p.pop();
    }
}

/// All roots of a polynomial by Durand-Kerner (Weierstrass) iteration
/// (ADR-0043): numeric, wasm-safe, no CAS. Simple roots land at machine
/// precision; repeated roots converge linearly, so the roots are
/// clustered and each cluster is refined through deflation: a double
/// root's mirror average is exact, higher multiplicities polish by
/// Newton on the deflated polynomial.
fn durand_kerner(coeffs: &[f64]) -> Vec<Complex<f64>> {
    let n = coeffs.len() - 1; // degree
    let mut roots: Vec<Complex<f64>> = (0..n)
        .map(|j| {
            let angle = 2.0 * std::f64::consts::PI * (j as f64 + 0.5) / (n as f64);
            Complex::new(angle.cos(), angle.sin()) * 0.4
        })
        .collect();
    let lead = coeffs[n];
    let eval_p = |z: Complex<f64>| -> Complex<f64> {
        let mut acc = Complex::new(0.0, 0.0);
        for c in coeffs.iter().rev() {
            acc = acc * z + Complex::new(*c, 0.0);
        }
        acc / lead
    };
    for _ in 0..200 {
        let mut converged = true;
        for j in 0..n {
            let mut denom = Complex::new(1.0, 0.0);
            for (k, rk) in roots.iter().enumerate() {
                if k != j {
                    denom *= roots[j] - rk;
                }
            }
            let delta = if denom.norm() > 1e-300 {
                eval_p(roots[j]) / denom
            } else {
                Complex::new(0.0, 0.0)
            };
            if delta.norm() > 1e-13 {
                converged = false;
            }
            roots[j] -= delta;
        }
        if converged {
            break;
        }
    }
    let scale = coeffs.iter().cloned().fold(0.0_f64, f64::max).max(1e-300);
    // cluster the roots; each cluster is one (possibly repeated) root
    let mut clusters: Vec<Vec<Complex<f64>>> = Vec::new();
    for r in roots {
        let tol = 1e-4 * (1.0 + r.norm());
        if let Some(c) = clusters
            .iter_mut()
            .find(|c| c.iter().any(|x| (x - r).norm() < tol))
        {
            c.push(r);
        } else {
            clusters.push(vec![r]);
        }
    }
    let mut out: Vec<Complex<f64>> = Vec::new();
    for cluster in clusters {
        let centroid =
            cluster.iter().fold(Complex::new(0.0, 0.0), |a, b| a + *b) / cluster.len() as f64;
        // deflate the original polynomial by the centroid: the quotient
        // keeps the other roots, the remainder is p(centroid)
        let (q, rem1) = deflate_poly(coeffs, centroid);
        if rem1.norm() > 1e-6 * scale {
            out.push(centroid); // defensively; DK roots satisfy this
            continue;
        }
        // the remainder of the second deflation is q(centroid), which
        // is p'(centroid): small only when the root is repeated
        let rem2 = eval_complex_poly(&q, centroid);
        if rem2.norm() >= 1e-6 * scale {
            // a simple root: Durand-Kerner already reached machine
            // precision, keep the centroid
            out.push(centroid);
            continue;
        }
        // a repeated root. The cluster size IS the multiplicity: a
        // double root's deflated quotient has the mirror root (simple,
        // quadratic convergence), and the average of the pair is the
        // exact root; higher multiplicities converge linearly on the
        // deflated polynomial to the root itself.
        let polished = newton_root(&q, centroid);
        if cluster.len() == 2 {
            out.push((centroid + polished) / 2.0);
        } else {
            out.push(polished);
        }
    }
    out
}

/// Synthetic division of a polynomial (lowest degree first) by `(x - r)`:
/// the quotient and the remainder p(r).
fn deflate_poly(coeffs: &[f64], r: Complex<f64>) -> (Vec<Complex<f64>>, Complex<f64>) {
    let mut q = deflate_complex(
        &coeffs
            .iter()
            .map(|c| Complex::new(*c, 0.0))
            .collect::<Vec<_>>(),
        r,
    );
    let rem = q.pop().expect("deflation keeps one remainder");
    (q, rem)
}

/// Synthetic division with complex coefficients (the quotient of a
/// first deflation stays complex): returns the quotient plus the
/// remainder as the popped last element.
fn deflate_complex(coeffs: &[Complex<f64>], r: Complex<f64>) -> Vec<Complex<f64>> {
    let n = coeffs.len() - 1;
    let mut q = vec![Complex::new(0.0, 0.0); n];
    q[n - 1] = coeffs[n];
    for k in (1..n).rev() {
        q[k - 1] = coeffs[k] + r * q[k];
    }
    let rem = coeffs[0] + r * q[0];
    q.push(rem);
    q
}

/// Horner evaluation of a complex-coefficient polynomial.
fn eval_complex_poly(coeffs: &[Complex<f64>], z: Complex<f64>) -> Complex<f64> {
    let mut acc = Complex::new(0.0, 0.0);
    for c in coeffs.iter().rev() {
        acc = acc * z + *c;
    }
    acc
}

/// Newton iteration on a polynomial (lowest degree first) from a start
/// point; the step p/p' is evaluated by a one-pass Horner.
fn newton_root(coeffs: &[Complex<f64>], start: Complex<f64>) -> Complex<f64> {
    let mut z = start;
    for _ in 0..40 {
        let mut p = Complex::new(0.0, 0.0);
        let mut dp = Complex::new(0.0, 0.0);
        for c in coeffs.iter().rev() {
            dp = dp * z + p;
            p = p * z + *c;
        }
        let step = if dp.norm() > 1e-300 {
            p / dp
        } else {
            Complex::new(0.0, 0.0)
        };
        if step.norm() < 1e-14 * (1.0 + z.norm()) {
            z -= step;
            break;
        }
        z -= step;
    }
    z
}

/// Dispatch a builtin function call. User-defined functions are resolved by
/// the caller; everything here is the scientific function library (the
/// calculator's function keys).
fn call_builtin(name: &str, args: Vec<Value>) -> Result<Value, EpherError> {
    match name {
        // The transcendental family extends to the complex plane
        // (ADR-0043): a complex argument computes in complex, and a real
        // argument outside the real domain falls back to the principal
        // complex result - sqrt(-1) is i, ln(-1) is i*pi, asin(2) is
        // complex. real_or_complex applies both rules from one helper.
        "sin" => real_or_complex(name, &args, |x| Ok(x.sin()), |z| z.sin()),
        "cos" => real_or_complex(name, &args, |x| Ok(x.cos()), |z| z.cos()),
        "tan" => real_or_complex(name, &args, |x| Ok(x.tan()), |z| z.tan()),
        "asin" => real_or_complex(
            name,
            &args,
            |x| {
                if x < -1.0 || x > 1.0 {
                    Err(domain_error(format!("asin of {x} outside -1..1")))
                } else {
                    Ok(x.asin())
                }
            },
            |z| z.asin(),
        ),
        "acos" => real_or_complex(
            name,
            &args,
            |x| {
                if x < -1.0 || x > 1.0 {
                    Err(domain_error(format!("acos of {x} outside -1..1")))
                } else {
                    Ok(x.acos())
                }
            },
            |z| z.acos(),
        ),
        "atan" => real_or_complex(name, &args, |x| Ok(x.atan()), |z| z.atan()),
        "sinh" => real_or_complex(name, &args, |x| Ok(x.sinh()), |z| z.sinh()),
        "cosh" => real_or_complex(name, &args, |x| Ok(x.cosh()), |z| z.cosh()),
        "tanh" => real_or_complex(name, &args, |x| Ok(x.tanh()), |z| z.tanh()),
        "asinh" => real_or_complex(name, &args, |x| Ok(x.asinh()), |z| z.asinh()),
        "acosh" => real_or_complex(
            name,
            &args,
            |x| {
                if x < 1.0 {
                    Err(domain_error(format!("acosh of {x} below 1")))
                } else {
                    Ok(x.acosh())
                }
            },
            |z| z.acosh(),
        ),
        "atanh" => real_or_complex(
            name,
            &args,
            |x| {
                if x <= -1.0 || x >= 1.0 {
                    Err(domain_error(format!("atanh of {x} outside -1..1")))
                } else {
                    Ok(x.atanh())
                }
            },
            |z| z.atanh(),
        ),
        "deg" => Ok(Value::Float(one_float(name, &args)?.to_degrees())),
        "rad" => Ok(Value::Float(one_float(name, &args)?.to_radians())),
        // Base conversion (ADR-0022): one integer in, a prefixed string
        // out — `bin(10)` is `0b1010`, `oct(10)` is `0o12`, `hex(255)` is
        // `0xff`. Prefixes match the literal syntax, so the answer can be
        // fed straight back in. Only whole numbers convert; negatives keep
        // their sign on the prefix (`-0b101`), like Python's bin().
        "bin" | "oct" | "hex" => {
            let radix: u32 = match name {
                "bin" => 2,
                "oct" => 8,
                _ => 16,
            };
            let prefix = match name {
                "bin" => "0b",
                "oct" => "0o",
                _ => "0x",
            };
            let v = one_arg(name, &args)?;
            let n = value_to_bigint(name, &v)?;
            let spelled = n.to_str_radix(radix);
            let out = match spelled.strip_prefix('-') {
                Some(digits) => format!("-{prefix}{digits}"),
                None => format!("{prefix}{spelled}"),
            };
            Ok(Value::Str(out))
        }
        "atan2" => {
            let (y, x) = two_floats(name, &args)?;
            Ok(Value::Float(y.atan2(x)))
        }
        "exp" => real_or_complex(name, &args, |x| Ok(x.exp()), |z| z.exp()),
        "ln" => real_or_complex(
            name,
            &args,
            |x| {
                if x <= 0.0 {
                    Err(domain_error(format!("ln of non-positive number {x}")))
                } else {
                    Ok(x.ln())
                }
            },
            |z| z.ln(),
        ),
        // calculator convention: log is base 10 (the LOG key), ln is natural
        "log" => real_or_complex(
            name,
            &args,
            |x| {
                if x <= 0.0 {
                    Err(domain_error(format!("log of non-positive number {x}")))
                } else {
                    Ok(x.log10())
                }
            },
            |z| z.ln() / std::f64::consts::LN_10,
        ),
        "log2" => real_or_complex(
            name,
            &args,
            |x| {
                if x <= 0.0 {
                    Err(domain_error(format!("log2 of non-positive number {x}")))
                } else {
                    Ok(x.log2())
                }
            },
            |z| z.ln() / std::f64::consts::LN_2,
        ),
        "logb" => {
            let (base, x) = two_floats(name, &args)?;
            if x <= 0.0 {
                return Err(domain_error(format!("logb of non-positive number {x}")));
            }
            if base <= 0.0 || base == 1.0 {
                return Err(domain_error(format!(
                    "logb base {base} must be positive and not 1"
                )));
            }
            Ok(Value::Float(x.log(base)))
        }
        "cbrt" => real_or_complex(
            name,
            &args,
            |x| Ok(x.cbrt()),
            // num_complex has no cbrt; the principal branch is powc(1/3)
            |z| z.powc(Complex::new(1.0 / 3.0, 0.0)),
        ),
        "root" => {
            // root(n, x): the real nth root; odd roots of negatives are negative
            let (n, x) = two_floats(name, &args)?;
            if n == 0.0 || n.fract() != 0.0 {
                return Err(domain_error(format!(
                    "root order {n} must be a non-zero integer"
                )));
            }
            if x < 0.0 && n % 2.0 == 0.0 {
                return Err(domain_error(format!("even root of negative number {x}")));
            }
            if x < 0.0 {
                Ok(Value::Float(-((-x).powf(1.0 / n))))
            } else {
                Ok(Value::Float(x.powf(1.0 / n)))
            }
        }
        "hypot" => {
            let (a, b) = two_floats(name, &args)?;
            Ok(Value::Float(a.hypot(b)))
        }
        // abs is the magnitude: a complex's distance from the origin
        // (ADR-0043), so it always returns a plain number.
        "abs" => {
            let v = one_arg(name, &args)?;
            match v {
                Value::Float(x) => Ok(Value::Float(x.abs())),
                Value::Complex(c) => Ok(Value::Float(c.norm())),
                _ => Err(EpherError::Type(format!(
                    "{name} expects a number, got {v}"
                ))),
            }
        }
        // Complex parts (ADR-0043): re/im/arg read the rectangular and
        // polar coordinates, conj mirrors across the real axis. On a
        // plain number re/arg/conj pass it through and im is 0.
        "re" | "im" | "arg" => {
            let v = one_arg(name, &args)?;
            let c = match v {
                Value::Float(x) => Complex::new(*x, 0.0),
                Value::Complex(c) => *c,
                _ => {
                    return Err(EpherError::Type(format!(
                        "{name} expects a number, got {v}"
                    )))
                }
            };
            Ok(Value::Float(match name {
                "re" => c.re,
                "im" => c.im,
                _ => c.arg(),
            }))
        }
        "conj" => {
            let v = one_arg(name, &args)?;
            match v {
                Value::Float(x) => Ok(Value::Float(*x)),
                Value::Complex(c) => Ok(Value::Complex(c.conj())),
                _ => Err(EpherError::Type(format!(
                    "{name} expects a number, got {v}"
                ))),
            }
        }
        // Exact fractions (ADR-0043): exact(x) reconstructs the rational
        // behind a float (continued fractions, denominator up to 1000,
        // relative tolerance 1e-9) - exact(0.3333333333333333) is 1/3.
        // Irrationals pass through unchanged: no convergent is good
        // enough, so pi stays decimal.
        "exact" => {
            let v = one_arg(name, &args)?;
            match v {
                Value::Float(x) => Ok(match reconstruct_fraction(*x, 1000, 1e-9) {
                    Some(r) => Value::Rational(r),
                    None => Value::Float(*x),
                }),
                other => Ok(other.clone()),
            }
        }
        // Display verbs (ADR-0043): scientific and engineering notation,
        // and thin-space thousands grouping - 1 234 567.89. All three
        // return display strings, like bin/oct/hex.
        "scientific" => {
            let v = one_arg(name, &args)?;
            let x = numeric_as_float(name, v)?;
            Ok(Value::Str(format!("{x:e}")))
        }
        "engineering" => {
            let v = one_arg(name, &args)?;
            let x = numeric_as_float(name, v)?;
            Ok(Value::Str(engineering_str(x)))
        }
        "grouped" => {
            let v = one_arg(name, &args)?;
            let x = numeric_as_float(name, v)?;
            Ok(Value::Str(grouped_str(&format!("{x}"))))
        }
        "floor" => Ok(Value::Float(one_float(name, &args)?.floor())),
        "ceil" => Ok(Value::Float(one_float(name, &args)?.ceil())),
        "trunc" => Ok(Value::Float(one_float(name, &args)?.trunc())),
        // half away from zero, like a calculator
        "round" => Ok(Value::Float(one_float(name, &args)?.round())),
        "sign" => {
            let x = one_float(name, &args)?;
            Ok(Value::Float(if x > 0.0 {
                1.0
            } else if x < 0.0 {
                -1.0
            } else {
                0.0
            }))
        }
        "sqrt" => real_or_complex(
            name,
            &args,
            |x| {
                if x < 0.0 {
                    Err(domain_error(format!("sqrt of negative number {x}")))
                } else {
                    Ok(x.sqrt())
                }
            },
            |z| z.sqrt(),
        ),
        "min" => {
            let xs = any_floats(name, &args)?;
            Ok(Value::Float(
                xs.iter().cloned().fold(f64::INFINITY, f64::min),
            ))
        }
        "max" => {
            let xs = any_floats(name, &args)?;
            Ok(Value::Float(
                xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            ))
        }
        "frac" => {
            let [n, d] = args.as_slice() else {
                return Err(EpherError::Type(format!(
                    "frac expects 2 arguments, got {}",
                    args.len()
                )));
            };
            match (n, d) {
                (Value::Float(n), Value::Float(d)) => {
                    let (Some(n), Some(d)) = (float_to_int(*n), float_to_int(*d)) else {
                        return Err(EpherError::Type(format!(
                            "frac expects integer arguments, got {n:?} and {d:?}"
                        )));
                    };
                    if d == 0 {
                        return Err(EpherError::ZeroDivision);
                    }
                    Ok(Value::Rational(BigRational::new(
                        BigInt::from(n),
                        BigInt::from(d),
                    )))
                }
                _ => Err(EpherError::Type(format!(
                    "frac expects numbers, got {n:?} and {d:?}"
                ))),
            }
        }
        "fact" => {
            let n = integer_arg(name, &args)?;
            Ok(Value::Float(factorial_value(n)?))
        }
        "ncr" | "npr" => {
            let (n, r) = integer_pair(name, &args)?;
            if n < 0 || r < 0 || r > n {
                return Err(domain_error(format!(
                    "{name} needs 0 <= r <= n, got n = {n}, r = {r}"
                )));
            }
            // keep r small for ncr: C(n, r) == C(n, n-r)
            let r = if name == "ncr" { r.min(n - r) } else { r };
            let mut acc = 1.0;
            for i in 0..r {
                if name == "ncr" {
                    // C(n, i+1) = C(n, i) * (n-i) / (i+1) - integral at
                    // every step, so tiny rounding only for huge results
                    acc = acc * ((n - i) as f64) / ((i + 1) as f64);
                } else {
                    acc *= (n - i) as f64;
                    if !acc.is_finite() {
                        return Err(domain_error(format!("npr of {n} and {r} overflows")));
                    }
                }
            }
            Ok(Value::Float(acc))
        }
        "gcd" | "lcm" => {
            let (a, b) = integer_pair(name, &args)?;
            let (a, b) = (a.abs(), b.abs());
            let mut x = a;
            let mut y = b;
            while y != 0 {
                let t = y;
                y = x % y;
                x = t;
            }
            let gcd = x;
            if name == "gcd" {
                Ok(Value::Float(gcd as f64))
            } else if gcd == 0 {
                Ok(Value::Float(0.0))
            } else {
                Ok(Value::Float(((a / gcd) * b) as f64))
            }
        }
        "mod" => {
            let (a, b) = integer_pair(name, &args)?;
            if b == 0 {
                return Err(EpherError::ZeroDivision);
            }
            // truncated remainder, the sign of the dividend (calculator MOD)
            Ok(Value::Float((a % b) as f64))
        }
        // Number theory (ADR-0042): primes and exact-integer helpers on the
        // integers f64 reaches exactly (|n| < 2^53; i64 throughout, u64
        // Miller-Rabin). `factors` follows the bin/oct/hex precedent: a
        // display Str, good for reading, not for further arithmetic.
        "isprime" => {
            let n = integer_arg(name, &args)?;
            Ok(Value::Bool(n >= 2 && is_prime_u64(n as u64)))
        }
        "nextprime" => {
            let n = integer_arg(name, &args)?;
            let mut candidate = n.max(1) + 1;
            while !is_prime_u64(candidate as u64) {
                candidate += 1;
            }
            Ok(Value::Float(candidate as f64))
        }
        "prevprime" => {
            let n = integer_arg(name, &args)?;
            if n <= 2 {
                return Err(domain_error(format!("no prime below {n}")));
            }
            let mut candidate = n - 1;
            while !is_prime_u64(candidate as u64) {
                candidate -= 1;
            }
            Ok(Value::Float(candidate as f64))
        }
        "modpow" => {
            let [b, e, m] = args.as_slice() else {
                return Err(EpherError::Type(format!(
                    "modpow expects 3 arguments, got {}",
                    args.len()
                )));
            };
            let (b, e, m) = match (b, e, m) {
                (Value::Float(b), Value::Float(e), Value::Float(m)) => {
                    let (Some(b), Some(e), Some(m)) =
                        (float_to_int(*b), float_to_int(*e), float_to_int(*m))
                    else {
                        return Err(EpherError::Type(format!(
                            "modpow expects integers, got {b:?}, {e:?}, {m:?}"
                        )));
                    };
                    (b, e, m)
                }
                _ => {
                    return Err(EpherError::Type(format!(
                        "modpow expects numbers, got {b:?}, {e:?}, {m:?}"
                    )))
                }
            };
            if m == 0 {
                return Err(EpherError::ZeroDivision);
            }
            if e < 0 {
                return Err(domain_error(format!("modpow exponent {e} must be >= 0")));
            }
            // exact via big integers: modpow(2, 100, 1000000007) is exact,
            // not a rounded float - the whole point of modular powers
            let result = BigInt::from(b).modpow(&BigInt::from(e), &BigInt::from(m.abs()));
            Ok(Value::Big(BigDecimal::from(result)))
        }
        "totient" => {
            let n = integer_arg(name, &args)?;
            if n <= 0 {
                return Err(domain_error(format!(
                    "totient of {n} must be a positive integer"
                )));
            }
            let phi = prime_factorization(n as u64)
                .iter()
                .fold(1u64, |acc, (p, k)| acc * (p - 1) * p.pow(k - 1));
            Ok(Value::Float(phi as f64))
        }
        "ndivisors" => {
            let n = integer_arg(name, &args)?;
            if n <= 0 {
                return Err(domain_error(format!(
                    "ndivisors of {n} must be a positive integer"
                )));
            }
            let count = prime_factorization(n as u64)
                .iter()
                .fold(1u64, |acc, (_, k)| acc * u64::from(k + 1));
            Ok(Value::Float(count as f64))
        }
        "factors" => {
            let n = integer_arg(name, &args)?;
            if n <= 0 {
                return Err(domain_error(format!(
                    "factors of {n} must be a positive integer"
                )));
            }
            let spelled = prime_factorization(n as u64)
                .iter()
                .map(|(p, k)| {
                    if *k == 1 {
                        p.to_string()
                    } else {
                        format!("{p}^{k}")
                    }
                })
                .collect::<Vec<_>>()
                .join(" * ");
            Ok(Value::Str(if spelled.is_empty() {
                "1".to_string()
            } else {
                spelled
            }))
        }
        // statistics family: population variance (divide by n), like a
        // calculator's 1-Var Stats
        "sum" | "product" | "mean" | "variance" | "stdev" => {
            let xs = any_floats(name, &args)?;
            match name {
                "sum" => Ok(Value::Float(xs.iter().sum())),
                "product" => Ok(Value::Float(xs.iter().product())),
                "mean" => Ok(Value::Float(xs.iter().sum::<f64>() / xs.len() as f64)),
                "variance" => {
                    let mean = xs.iter().sum::<f64>() / xs.len() as f64;
                    Ok(Value::Float(
                        xs.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / xs.len() as f64,
                    ))
                }
                _ => {
                    let mean = xs.iter().sum::<f64>() / xs.len() as f64;
                    Ok(Value::Float(
                        (xs.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / xs.len() as f64)
                            .sqrt(),
                    ))
                }
            }
        }
        "median" => {
            let mut xs = any_floats(name, &args)?;
            xs.sort_by(|a, b| a.partial_cmp(b).expect("floats are comparable"));
            let n = xs.len();
            let mid = n / 2;
            Ok(Value::Float(if n % 2 == 1 {
                xs[mid]
            } else {
                (xs[mid - 1] + xs[mid]) / 2.0
            }))
        }
        "dec" => {
            let [x] = args.as_slice() else {
                return Err(EpherError::Type(format!(
                    "dec expects 1 argument, got {}",
                    args.len()
                )));
            };
            match x {
                Value::Float(n) => float_to_decimal(*n)
                    .map(Value::Decimal)
                    .ok_or_else(|| EpherError::Type(format!("cannot convert {n} to a decimal"))),
                other => Err(EpherError::Type(format!(
                    "dec expects a number, got {other:?}"
                ))),
            }
        }
        "big" => {
            let [x] = args.as_slice() else {
                return Err(EpherError::Type(format!(
                    "big expects 1 argument, got {}",
                    args.len()
                )));
            };
            match x {
                Value::Float(n) => float_to_big(*n)
                    .map(Value::Big)
                    .ok_or_else(|| EpherError::Type(format!("cannot convert {n} to big"))),
                other => Err(EpherError::Type(format!(
                    "big expects a number, got {other:?}"
                ))),
            }
        }
        _ => match astro::call(name, args) {
            Some(result) => result,
            None => Err(EpherError::UnknownName(name.to_string())),
        },
    }
}

/// Execute a script's statements in order against a mutable [`Env`], returning
/// the last statement's value (the script seam).
pub fn run(script: &[Statement], env: &mut Env) -> Result<Option<Value>, EpherError> {
    let mut steps = STEP_LIMIT;
    run_inner(script, env, &mut steps)
}

/// Run a script and collect every statement's value — the one-shot CLI's
/// view of a script (each result prints on its own line, like piped mode
/// without the `=` prefix). `run` returns only the last value; interactive
/// surfaces keep that display.
pub fn run_all(script: &[Statement], env: &mut Env) -> Result<Vec<Value>, EpherError> {
    let mut steps = STEP_LIMIT;
    let mut values = Vec::new();
    for stmt in script {
        // Every statement sets `ans` as it runs, so the next statement can
        // read the previous answer (one-shot: `epher "2 + 3; ans * 2"`).
        if let Some(v) = stmt_value(stmt, env, &mut steps)? {
            values.push(v);
        }
    }
    Ok(values)
}

/// Maximum statement executions per `run` — protects against runaway loops.
const STEP_LIMIT: u64 = 100_000;

fn consume_step(steps: &mut u64) -> Result<(), EpherError> {
    if *steps == 0 {
        return Err(EpherError::StepLimit);
    }
    *steps -= 1;
    Ok(())
}

/// Execute one statement and return its value. Every value-producing
/// statement records its result as the variable `ans` — the previous
/// answer, like a pocket calculator's `Ans` (the keypads carry an `ans`
/// key). Statements that produce no value (definitions, `while`) leave
/// `ans` untouched, and so do errors. `ans` is an ordinary variable: it
/// lives in the session's environment and is not persisted.
fn stmt_value(
    stmt: &Statement,
    env: &mut Env,
    steps: &mut u64,
) -> Result<Option<Value>, EpherError> {
    consume_step(steps)?;
    let value = match stmt {
        Statement::Expr(expr) => Some(eval(expr, env)?),
        Statement::Assign(name, expr) => Some(assign(env, name, expr)?),
        Statement::Const(name, expr) => Some(define_constant(env, name, expr)?),
        Statement::FunctionDef(name, params, body) => {
            env.set_function(
                name.clone(),
                Function {
                    params: params.clone(),
                    body: body.clone(),
                },
            );
            // a definition produces no value
            None
        }
        Statement::While(cond, body) => {
            run_while(cond, body, env, steps)?;
            None
        }
        Statement::Solve(equation) => Some(solve_statement(equation, env)?),
    };
    if let Some(v) = &value {
        env.set("ans", v.clone());
    }
    Ok(value)
}

fn run_inner(
    script: &[Statement],
    env: &mut Env,
    steps: &mut u64,
) -> Result<Option<Value>, EpherError> {
    let mut result = None;
    for stmt in script {
        let value = stmt_value(stmt, env, steps)?;
        if value.is_some() {
            result = value;
        }
    }
    Ok(result)
}

/// Assign to a variable, refusing to rebind a constant (ADR-0012).
fn assign(env: &mut Env, name: &str, expr: &Expression) -> Result<Value, EpherError> {
    if env.constant(name).is_some() {
        return Err(EpherError::AssignToConstant(name.to_string()));
    }
    let value = eval(expr, env)?;
    env.set(name.to_string(), value.clone());
    Ok(value)
}

/// Define a constant, refusing to take a variable's name — a name is
/// either a variable or a constant, never both (ADR-0012). Re-declaring
/// an existing constant is an error only when the value changes:
/// examples with `const` lines get pasted and re-pasted, and the
/// slider and animation paths rewrite constants the same way.
fn define_constant(env: &mut Env, name: &str, expr: &Expression) -> Result<Value, EpherError> {
    if env.constant(name).is_some() {
        let value = eval(expr, env)?;
        if env.constant(name) == Some(&value) {
            return Ok(value);
        }
        return Err(EpherError::ConstantAlreadyDefined(name.to_string()));
    }
    if env.get(name).is_some() {
        return Err(EpherError::ConstantNameTaken(name.to_string()));
    }
    let value = eval(expr, env)?;
    env.set_constant(name.to_string(), value.clone());
    Ok(value)
}

/// Execute one statement for its effect (used by loop bodies; loops produce no
/// value).
fn execute_stmt(stmt: &Statement, env: &mut Env, steps: &mut u64) -> Result<(), EpherError> {
    // Body statements set `ans` exactly like top-level ones.
    stmt_value(stmt, env, steps).map(|_| ())
}

/// Drive a while loop: evaluate the condition, run the body while it's true.
fn run_while(
    cond: &Expression,
    body: &Statement,
    env: &mut Env,
    steps: &mut u64,
) -> Result<(), EpherError> {
    loop {
        match eval(cond, env)? {
            Value::Bool(true) => execute_stmt(body, env, steps)?,
            Value::Bool(false) => break,
            other => {
                return Err(EpherError::Type(format!(
                    "while condition must be a boolean, got {other:?}"
                )));
            }
        }
    }
    Ok(())
}

/// An interactive session: a persistent [`Env`] plus history — the shared
/// "submit a line" logic for the CLI REPL, TUI, and web frontends, so it
/// exists once. Also records the source of each `def` and `const` line so
/// frontends can save user-defined functions and constants.
#[derive(Debug, Clone, Default)]
pub struct Session {
    env: Env,
    history: Vec<String>,
    defs: HashMap<String, String>,
    consts: HashMap<String, String>,
    last_line: Option<String>,
    /// Result rendering (ADR-0043): exact fractions, notation, and
    /// separators. Interactive frontends set this from their settings;
    /// the CLI keeps the default.
    display: DisplayPrefs,
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    /// A session pre-seeded with history (e.g. loaded from the store).
    pub fn with_history(history: Vec<String>) -> Self {
        Self {
            history,
            ..Self::default()
        }
    }

    /// The session's result display preferences (ADR-0043).
    pub fn set_display(&mut self, prefs: DisplayPrefs) {
        self.display = prefs;
    }

    /// Submit a script line: run it against the environment, record it in
    /// history, and return the display string (`= value`, `error: ...`, or
    /// empty for a line that produced no value, like a bare `def`). An empty
    /// line does nothing.
    pub fn submit(&mut self, line: &str) -> String {
        let line = line.trim().to_string();
        if line.is_empty() {
            return String::new();
        }
        let output = match parse_script(&line) {
            Ok(script) => match run(&script, &mut self.env) {
                Ok(Some(value)) => format!("= {}", format_value(&value, &self.display)),
                Ok(None) => String::new(),
                Err(e) => format!("error: {e}"),
            },
            Err(e) => format!("error: {e}"),
        };
        if output.is_empty() {
            self.history.push(line.clone());
        } else {
            self.history.push(format!("{line}  {output}"));
        }
        self.last_line = Some(line.clone());
        if let Some(name) = def_name(&line) {
            self.defs.insert(name, line.clone());
        }
        if let Some(name) = const_name(&line) {
            self.consts.insert(name, line);
        }
        output
    }

    /// The last interactive line submitted (for `save script`).
    /// data (functions/scripts) into the environment.
    pub fn submit_quiet(&mut self, line: &str) -> String {
        let line = line.trim().to_string();
        if line.is_empty() {
            return String::new();
        }
        let output = match parse_script(&line) {
            Ok(script) => match run(&script, &mut self.env) {
                Ok(Some(value)) => format!("= {}", format_value(&value, &self.display)),
                Ok(None) => String::new(),
                Err(e) => format!("error: {e}"),
            },
            Err(e) => format!("error: {e}"),
        };
        if let Some(name) = def_name(&line) {
            self.defs.insert(name, line.clone());
        }
        if let Some(name) = const_name(&line) {
            self.consts.insert(name, line);
        }
        output
    }

    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// Record a submitted line in the history without evaluating it, for
    /// frontend-dispatched commands (`graph x^2`, `graph3d …`, and their
    /// `clear`) whose output is rendered rather than computed here. The
    /// command belongs in the same history as every other submitted line.
    pub fn record(&mut self, line: &str) {
        let line = line.trim().to_string();
        if !line.is_empty() {
            self.history.push(line);
        }
    }

    /// Mark a multi-statement line as the most recent script line, so
    /// `save script` persists the whole script the user entered (with its
    /// `;` separators), not just the last statement.
    pub fn set_last_line(&mut self, line: &str) {
        self.last_line = Some(line.trim().to_string());
    }

    /// Empty the history list (the clear-history control in every frontend).
    /// The environment keeps its definitions and constants.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// The environment, for frontends that need it (e.g. graphing).
    pub fn env(&self) -> &Env {
        &self.env
    }

    /// Redefine a constant's value and its source text in one step — the
    /// slider seam (ADR-0014): a UI control adjusts `const a = 2` to a new
    /// number, and the updated source is what `save a` persists.
    pub fn set_constant(&mut self, name: impl Into<String>, value: Value, source: String) {
        let name = name.into();
        self.env.set_constant(name.clone(), value);
        self.consts.insert(name, source);
    }

    /// The source text of every `def` line submitted this session, by name.
    pub fn def_sources(&self) -> &HashMap<String, String> {
        &self.defs
    }

    /// The source text of every `const` line submitted this session, by name.
    pub fn const_sources(&self) -> &HashMap<String, String> {
        &self.consts
    }

    /// The last interactive line submitted (used by `save script`).
    pub fn last_line(&self) -> Option<&str> {
        self.last_line.as_deref()
    }

    /// The session's variable bindings (user assignments plus `ans`), for
    /// the shared-store snapshot every interactive frontend persists.
    pub fn bindings(&self) -> &ValueBindings {
        self.env.bindings()
    }

    /// Restore bindings saved by another frontend of the same installation
    /// (ADR-0010 amendment): each name is bound into the environment, so
    /// `ans` and every user assignment survive across CLI/REPL/TUI/GUI.
    pub fn restore_bindings(&mut self, bindings: &ValueBindings) {
        for (name, value) in bindings {
            self.env.set(name.clone(), value.clone());
        }
    }

    /// The environment, mutable — for frontends evaluating statements
    /// directly against a shared session (the CLI one-shot path).
    pub fn env_mut(&mut self) -> &mut Env {
        &mut self.env
    }
}

/// The name defined by a `def name(...) = ...` line, if any.
fn def_name(line: &str) -> Option<String> {
    let rest = line.trim().strip_prefix("def")?.trim_start();
    let name: String = rest
        .chars()
        .take_while(|c| c.is_alphabetic() || *c == '_')
        .collect();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// The name defined by a `const name = ...` line, if any. The `const` must
/// stand alone as a word (so a variable named `const_tax` still parses as an
/// assignment).
fn const_name(line: &str) -> Option<String> {
    let rest = line.trim().strip_prefix("const")?;
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let name: String = rest
        .trim_start()
        .chars()
        .take_while(|c| c.is_alphabetic() || *c == '_')
        .collect();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// The expression part of a recorded history entry (ADR-0031).
/// Evaluations are recorded as `"{line}  {output}"` — the line the user
/// entered, then the answer (`= …`) or the failure (`error: …` /
/// `warning: …`). A history pick loads everything before the last such
/// suffix, so the user can edit the expression and re-run it (amending
/// ADR-0027's verbatim picks: the `  ` separator makes the suffix
/// structurally unambiguous, not a heuristic). Entries without an answer
/// suffix — graph commands, definitions — return unchanged.
pub fn history_expression(entry: &str) -> &str {
    // Multi-line script entries (ADR-0027 amendment) are recorded
    // verbatim — no answer suffix — and the suffix scan must not fire
    // inside one: an assignment line like `x = 2 + 2` contains the
    // marker and would be cut short.
    if entry.contains('\n') {
        return entry;
    }
    for suffix in ["  = ", "  error:", "  warning:"] {
        if let Some(pos) = entry.rfind(suffix) {
            return &entry[..pos];
        }
    }
    entry
}

/// A binary arithmetic operator, dispatched per number layer (ADR-0005).
#[derive(Debug, Clone, Copy, PartialEq)]
enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

/// Apply a binary op to two [`Value`]s, promoting to a common number layer
/// (Float → Rational → Decimal → Big) when operands differ (ADR-0005).
fn binop(lhs: Value, rhs: Value, op: BinOp) -> Result<Value, EpherError> {
    match (&lhs, &rhs) {
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(float_binop(op, *a, *b)?)),
        (Value::Rational(a), Value::Rational(b)) => {
            Ok(Value::Rational(rational_binop(op, a.clone(), b.clone())?))
        }
        (Value::Float(a), Value::Rational(b)) => {
            let a = BigRational::from_float(*a)
                .ok_or_else(|| EpherError::Type(format!("cannot promote {a} to a rational")))?;
            Ok(Value::Rational(rational_binop(op, a, b.clone())?))
        }
        (Value::Rational(a), Value::Float(b)) => {
            let b = BigRational::from_float(*b)
                .ok_or_else(|| EpherError::Type(format!("cannot promote {b} to a rational")))?;
            Ok(Value::Rational(rational_binop(op, a.clone(), b)?))
        }
        (Value::Decimal(a), Value::Decimal(b)) => {
            Ok(Value::Decimal(decimal_binop(op, a.clone(), b.clone())?))
        }
        (Value::Float(a), Value::Decimal(b)) => {
            let a = float_to_decimal(*a)
                .ok_or_else(|| EpherError::Type(format!("cannot promote {a} to a decimal")))?;
            Ok(Value::Decimal(decimal_binop(op, a, b.clone())?))
        }
        (Value::Decimal(a), Value::Float(b)) => {
            let b = float_to_decimal(*b)
                .ok_or_else(|| EpherError::Type(format!("cannot promote {b} to a decimal")))?;
            Ok(Value::Decimal(decimal_binop(op, a.clone(), b)?))
        }
        (Value::Big(a), Value::Big(b)) => Ok(Value::Big(big_binop(op, a.clone(), b.clone())?)),
        (Value::Float(a), Value::Big(b)) => {
            let a = float_to_big(*a)
                .ok_or_else(|| EpherError::Type(format!("cannot promote {a} to big")))?;
            Ok(Value::Big(big_binop(op, a, b.clone())?))
        }
        (Value::Big(a), Value::Float(b)) => {
            let b = float_to_big(*b)
                .ok_or_else(|| EpherError::Type(format!("cannot promote {b} to big")))?;
            Ok(Value::Big(big_binop(op, a.clone(), b)?))
        }
        // Complex arithmetic (ADR-0043): floats promote to complex, the
        // exact variants do not (combining them is a type error).
        (Value::Complex(a), Value::Complex(b)) => Ok(Value::Complex(complex_binop(op, *a, *b)?)),
        (Value::Complex(a), Value::Float(b)) => Ok(Value::Complex(complex_binop(
            op,
            *a,
            Complex::new(*b, 0.0),
        )?)),
        (Value::Float(a), Value::Complex(b)) => Ok(Value::Complex(complex_binop(
            op,
            Complex::new(*a, 0.0),
            *b,
        )?)),
        _ => Err(EpherError::Type(format!(
            "cannot combine {lhs:?} and {rhs:?}"
        ))),
    }
}

fn complex_binop(op: BinOp, a: Complex<f64>, b: Complex<f64>) -> Result<Complex<f64>, EpherError> {
    match op {
        BinOp::Add => Ok(a + b),
        BinOp::Sub => Ok(a - b),
        BinOp::Mul => Ok(a * b),
        BinOp::Div => {
            if b.norm_sqr() == 0.0 {
                Err(EpherError::ZeroDivision)
            } else {
                Ok(a / b)
            }
        }
        BinOp::Pow => {
            // An integer power multiplies out exactly instead of going
            // through exp/ln (ADR-0043): i^2 is exactly -1, and
            // repeated squaring keeps the tiny powc noise away.
            if b.im == 0.0 && b.re.is_finite() && b.re.fract() == 0.0 && b.re.abs() < 1e9 {
                let mut base = a;
                let mut n = b.re as i64;
                if n < 0 {
                    if base.norm_sqr() == 0.0 {
                        return Err(EpherError::ZeroDivision);
                    }
                    base = base.inv();
                    n = -n;
                }
                let mut acc = Complex::new(1.0, 0.0);
                while n > 0 {
                    if n & 1 == 1 {
                        acc *= base;
                    }
                    base *= base;
                    n >>= 1;
                }
                Ok(acc)
            } else {
                Ok(a.powc(b))
            }
        }
    }
}

fn float_binop(op: BinOp, a: f64, b: f64) -> Result<f64, EpherError> {
    match op {
        BinOp::Add => Ok(a + b),
        BinOp::Sub => Ok(a - b),
        BinOp::Mul => Ok(a * b),
        BinOp::Div => {
            if b == 0.0 {
                Err(EpherError::ZeroDivision)
            } else {
                Ok(a / b)
            }
        }
        BinOp::Pow => {
            let r = a.powf(b);
            if r.is_nan() {
                // powf's only NaN case: negative base, non-integer
                // exponent. The real odd root exists - point at it.
                return Err(EpherError::Domain(
                    "power of a negative base with a non-integer exponent; use root(n, x) for real roots".into(),
                ));
            }
            Ok(r)
        }
    }
}

fn rational_binop(op: BinOp, a: BigRational, b: BigRational) -> Result<BigRational, EpherError> {
    match op {
        BinOp::Add => Ok(a + b),
        BinOp::Sub => Ok(a - b),
        BinOp::Mul => Ok(a * b),
        BinOp::Div => {
            if b == BigRational::from_integer(0.into()) {
                Err(EpherError::ZeroDivision)
            } else {
                Ok(a / b)
            }
        }
        BinOp::Pow => {
            // exact for an integer exponent (negative exponents give the
            // reciprocal); a fractional exponent has no exact rational
            // answer in general, so the layer refuses rather than guess
            if !b.is_integer() {
                return Err(EpherError::Type(
                    "rational exponentiation needs an integer exponent; work in floats for fractional powers".into(),
                ));
            }
            let exp = b
                .numer()
                .to_i32()
                .ok_or_else(|| EpherError::Type("rational exponent too large".into()))?;
            Ok(a.pow(exp))
        }
    }
}

fn decimal_binop(op: BinOp, a: Decimal, b: Decimal) -> Result<Decimal, EpherError> {
    match op {
        BinOp::Add => a
            .checked_add(b)
            .ok_or_else(|| EpherError::Type("decimal overflow".into())),
        BinOp::Sub => a
            .checked_sub(b)
            .ok_or_else(|| EpherError::Type("decimal overflow".into())),
        BinOp::Mul => a
            .checked_mul(b)
            .ok_or_else(|| EpherError::Type("decimal overflow".into())),
        BinOp::Div => {
            if b.is_zero() {
                Err(EpherError::ZeroDivision)
            } else {
                a.checked_div(b)
                    .ok_or_else(|| EpherError::Type("decimal division error".into()))
            }
        }
        BinOp::Pow => {
            // exact for an integer exponent; fractional exponents refuse,
            // mirroring the other exact layers (ADR-0005)
            let Some(exp) = b.to_i64() else {
                return Err(EpherError::Type(
                    "decimal exponentiation needs an integer exponent; work in floats for fractional powers".into(),
                ));
            };
            // to_i64 truncates, so integrality is checked separately
            if b != b.trunc() {
                return Err(EpherError::Type(
                    "decimal exponentiation needs an integer exponent; work in floats for fractional powers".into(),
                ));
            }
            if exp.unsigned_abs() > 100_000 {
                return Err(EpherError::Type("decimal exponent too large".into()));
            }
            let (base, times) = if exp < 0 {
                (Decimal::ONE, exp.unsigned_abs())
            } else {
                (a, exp as u64)
            };
            let mut acc = Decimal::ONE;
            for _ in 0..times {
                acc = acc
                    .checked_mul(base)
                    .ok_or_else(|| EpherError::Type("decimal overflow".into()))?;
            }
            if exp < 0 {
                Decimal::ONE
                    .checked_div(acc)
                    .ok_or_else(|| EpherError::Type("decimal division error".into()))
            } else {
                Ok(acc)
            }
        }
    }
}

/// Convert a float to its clean decimal representation (the shortest
/// round-trip string form), rejecting non-finite values.
fn float_to_decimal(n: f64) -> Option<Decimal> {
    n.to_string().parse().ok()
}

/// Convert a float to its clean decimal representation (shortest round-trip
/// string form), rejecting non-finite values.
fn float_to_big(n: f64) -> Option<BigDecimal> {
    n.to_string().parse().ok()
}

fn big_binop(op: BinOp, a: BigDecimal, b: BigDecimal) -> Result<BigDecimal, EpherError> {
    match op {
        BinOp::Add => Ok(a + b),
        BinOp::Sub => Ok(a - b),
        BinOp::Mul => Ok(a * b),
        BinOp::Div => {
            if b.is_zero() {
                Err(EpherError::ZeroDivision)
            } else {
                Ok(a / b)
            }
        }
        BinOp::Pow => {
            // exact for an integer exponent (the crate's powi handles
            // negative exponents through the division context); a
            // fractional exponent refuses, mirroring the other layers
            let Some(exp) = b.to_i64() else {
                return Err(EpherError::Type(
                    "big exponentiation needs an integer exponent; work in floats for fractional powers".into(),
                ));
            };
            // to_i64 truncates, so integrality is checked separately
            if b.fractional_digit_count() > 0 {
                return Err(EpherError::Type(
                    "big exponentiation needs an integer exponent; work in floats for fractional powers".into(),
                ));
            }
            // normalized() strips the context's trailing zeros on negative
            // exponents (2^-10 is exactly 0.0009765625, not 100 digits of it)
            Ok(a.powi(exp).normalized())
        }
    }
}
