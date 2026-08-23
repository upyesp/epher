//! epher-core — the single source of truth for epher's logic.
//!
//! Compiles to both `wasm32-unknown-unknown` (web/PWA/desktop) and native targets
//! (CLI/TUI). Stays pure: no I/O, no threads, no platform calls. Numerics per
//! ADR-0005.

pub mod graph;
pub mod graph_svg;

use std::collections::HashMap;

use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use num_complex::Complex;
use num_rational::BigRational;
use num_traits::Zero;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

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
            Value::Complex(c) => write!(f, "{c}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Str(s) => write!(f, "{s}"),
        }
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
                return Err(EpherError::Parse("expected ';' or a newline between statements".into()));
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
    LParen,
    RParen,
}

fn tokenize(text: &str) -> Result<Vec<Token>, EpherError> {
    let mut tokens = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            c if c.is_whitespace() => {
                // Newlines are statement separators, exactly like `;`
                // (ADR-0001 seam unification): the language has no
                // strings, comments, or multi-line constructs, so a
                // newline can only ever appear between statements.
                if c == '\n' || c == '\r' {
                    tokens.push(Token::Semicolon);
                }
                chars.next();
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
            '/' => {
                tokens.push(Token::Slash);
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
            '0'
                if matches!(
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
                tokens.push(Token::Number(n));
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
                tokens.push(Token::Number(n));
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
            other => return Err(EpherError::Parse(format!("unexpected character: {other:?}"))),
        }
    }
    Ok(tokens)
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
            Some(other) => Err(EpherError::Parse(format!("expected {what}, found {other:?}"))),
            None => Err(EpherError::Parse("unexpected end of input".into())),
        }
    }

    fn expect_token(&mut self, token: Token, what: &str) -> Result<(), EpherError> {
        match self.next() {
            Some(found) if found == token => Ok(()),
            Some(other) => Err(EpherError::Parse(format!("expected {what}, found {other:?}"))),
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
            Some(other) => Err(EpherError::Parse(format!("expected '{kw}', found {other:?}"))),
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
        // postfix factorial binds tightest: 3! ^ 2 is (3!) ^ 2, and 4!!
        // is (4!)!; `!=` lexes as one token so `5! != 3` still works
        let mut expr = primary;
        while matches!(self.peek(), Some(Token::Bang)) {
            self.next();
            expr = Expression::Factorial(Box::new(expr));
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, EpherError> {
        match self.next() {
            Some(Token::Number(n)) => Ok(Expression::Literal(n)),
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
                } else {
                    Ok(Expression::Var(name))
                }
            }
            Some(Token::LParen) => {
                let expr = self.parse_expression()?;
                match self.next() {
                    Some(Token::RParen) => Ok(expr),
                    Some(other) => {
                        Err(EpherError::Parse(format!("expected ')', found {other:?}")))
                    }
                    None => Err(EpherError::Parse("unexpected end of input".into())),
                }
            }
            Some(other) => Err(EpherError::Parse(format!("expected a number, found {other:?}"))),
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
            let mut values = Vec::with_capacity(args.len());
            for arg in args {
                values.push(eval(arg, env)?);
            }
            if let Some(f) = env.function(name) {
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
            call_builtin(name, values)
        }
    }
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
        let (Ok(Value::Float(x)), Ok(Value::Float(y))) = (eval(x_expr, &child), eval(y_expr, &child))
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

/// Built-in constants (π, e, τ, φ), resolved when a name isn't in the
/// environment.
fn builtin_const(name: &str) -> Option<Value> {
    match name {
        "pi" => Some(Value::float(std::f64::consts::PI)),
        "e" => Some(Value::float(std::f64::consts::E)),
        "tau" => Some(Value::float(std::f64::consts::TAU)),
        "phi" => Some(Value::float(1.618_033_988_749_895)),
        _ => None,
    }
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
    float_to_int(x).ok_or_else(|| {
        EpherError::Type(format!("{name} expects integers, got {x}"))
    })
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

/// Dispatch a builtin function call. User-defined functions are resolved by
/// the caller; everything here is the scientific function library (the
/// calculator's function keys).
fn call_builtin(name: &str, args: Vec<Value>) -> Result<Value, EpherError> {
    match name {
        "sin" => Ok(Value::Float(one_float(name, &args)?.sin())),
        "cos" => Ok(Value::Float(one_float(name, &args)?.cos())),
        "tan" => Ok(Value::Float(one_float(name, &args)?.tan())),
        "asin" => {
            let x = one_float(name, &args)?;
            if x < -1.0 || x > 1.0 {
                return Err(domain_error(format!("asin of {x} outside -1..1")));
            }
            Ok(Value::Float(x.asin()))
        }
        "acos" => {
            let x = one_float(name, &args)?;
            if x < -1.0 || x > 1.0 {
                return Err(domain_error(format!("acos of {x} outside -1..1")));
            }
            Ok(Value::Float(x.acos()))
        }
        "atan" => Ok(Value::Float(one_float(name, &args)?.atan())),
        "sinh" => Ok(Value::Float(one_float(name, &args)?.sinh())),
        "cosh" => Ok(Value::Float(one_float(name, &args)?.cosh())),
        "tanh" => Ok(Value::Float(one_float(name, &args)?.tanh())),
        "asinh" => Ok(Value::Float(one_float(name, &args)?.asinh())),
        "acosh" => {
            let x = one_float(name, &args)?;
            if x < 1.0 {
                return Err(domain_error(format!("acosh of {x} below 1")));
            }
            Ok(Value::Float(x.acosh()))
        }
        "atanh" => {
            let x = one_float(name, &args)?;
            if x <= -1.0 || x >= 1.0 {
                return Err(domain_error(format!("atanh of {x} outside -1..1")));
            }
            Ok(Value::Float(x.atanh()))
        }
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
        "exp" => Ok(Value::Float(one_float(name, &args)?.exp())),
        "ln" => {
            let x = one_float(name, &args)?;
            if x <= 0.0 {
                return Err(domain_error(format!("ln of non-positive number {x}")));
            }
            Ok(Value::Float(x.ln()))
        }
        // calculator convention: log is base 10 (the LOG key), ln is natural
        "log" => {
            let x = one_float(name, &args)?;
            if x <= 0.0 {
                return Err(domain_error(format!("log of non-positive number {x}")));
            }
            Ok(Value::Float(x.log10()))
        }
        "log2" => {
            let x = one_float(name, &args)?;
            if x <= 0.0 {
                return Err(domain_error(format!("log2 of non-positive number {x}")));
            }
            Ok(Value::Float(x.log2()))
        }
        "logb" => {
            let (base, x) = two_floats(name, &args)?;
            if x <= 0.0 {
                return Err(domain_error(format!(
                    "logb of non-positive number {x}"
                )));
            }
            if base <= 0.0 || base == 1.0 {
                return Err(domain_error(format!("logb base {base} must be positive and not 1")));
            }
            Ok(Value::Float(x.log(base)))
        }
        "cbrt" => Ok(Value::Float(one_float(name, &args)?.cbrt())),
        "root" => {
            // root(n, x): the real nth root; odd roots of negatives are negative
            let (n, x) = two_floats(name, &args)?;
            if n == 0.0 || n.fract() != 0.0 {
                return Err(domain_error(format!("root order {n} must be a non-zero integer")));
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
        "abs" => Ok(Value::Float(one_float(name, &args)?.abs())),
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
        "sqrt" => {
            let x = one_float(name, &args)?;
            if x < 0.0 {
                return Err(domain_error(format!("sqrt of negative number {x}")));
            }
            Ok(Value::Float(x.sqrt()))
        }
        "min" => {
            let xs = any_floats(name, &args)?;
            Ok(Value::Float(xs.iter().cloned().fold(f64::INFINITY, f64::min)))
        }
        "max" => {
            let xs = any_floats(name, &args)?;
            Ok(Value::Float(xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max)))
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
                    Ok(Value::Rational(BigRational::new(BigInt::from(n), BigInt::from(d))))
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
                        (xs.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>()
                            / xs.len() as f64)
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
        _ => Err(EpherError::UnknownName(name.to_string())),
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
    };
    if let Some(v) = &value {
        env.set("ans", v.clone());
    }
    Ok(value)
}

fn run_inner(script: &[Statement], env: &mut Env, steps: &mut u64) -> Result<Option<Value>, EpherError> {
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

/// Define a constant, refusing to redefine an existing constant or to take
/// a variable's name — a name is either a variable or a constant, never
/// both (ADR-0012).
fn define_constant(env: &mut Env, name: &str, expr: &Expression) -> Result<Value, EpherError> {
    if env.constant(name).is_some() {
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
                Ok(Some(value)) => format!("= {value}"),
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
                Ok(Some(value)) => format!("= {value}"),
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
}

/// The name defined by a `def name(...) = ...` line, if any.
fn def_name(line: &str) -> Option<String> {
    let rest = line.trim().strip_prefix("def")?.trim_start();
    let name: String = rest.chars().take_while(|c| c.is_alphabetic() || *c == '_').collect();
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
        _ => Err(EpherError::Type(format!("cannot combine {lhs:?} and {rhs:?}"))),
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
        BinOp::Pow => Ok(a.powf(b)),
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
        BinOp::Pow => Err(EpherError::Type(
            "rational exponentiation is not supported yet".into(),
        )),
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
        BinOp::Pow => Err(EpherError::Type(
            "decimal exponentiation is not supported yet".into(),
        )),
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
        BinOp::Pow => Err(EpherError::Type(
            "big exponentiation is not supported yet".into(),
        )),
    }
}
