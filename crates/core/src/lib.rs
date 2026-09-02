//! epher-core — the single source of truth for epher's logic.
//!
//! Compiles to both `wasm32-unknown-unknown` (web/PWA/desktop) and native targets
//! (CLI/TUI). Stays pure: no I/O, no threads; the one platform read is the
//! clock behind `now()` (ADR-0037). Numerics per ADR-0005.

pub mod astro;
pub mod graph;
pub mod graph_svg;

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

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
    /// A list of numbers — a data column (ADR-0044): `{1, 2, 3}`.
    /// Elements are floats; complex values are rejected with a type
    /// error at list construction.
    List(Vec<Value>),
    /// A matrix (ADR-0049): a rows × cols grid of floats, row-major,
    /// from the `[[1, 2], [3, 4]]` literal. Floats only, like lists.
    Matrix {
        rows: usize,
        cols: usize,
        data: Vec<f64>,
    },
    /// A quantity (ADR-0046): an SI value plus the seven base
    /// dimensions, and an optional display unit (the typed spelling
    /// with its SI factor) so the value can be shown back in the unit
    /// it was typed or converted to. `3.2 AU` stores 4.7871…e11 m with
    /// unit `("AU", 1.4959…e11)` and displays `3.2 AU`.
    Quantity {
        value: f64,
        dims: Dims,
        unit: Option<(String, f64)>,
    },
    /// A display string — produced by the base-conversion builtins
    /// (`bin`, `oct`, `hex`; ADR-0022), the solve statement, the
    /// regression and test/interval functions (ADR-0044), the stats
    /// builtins, and now written directly: string literals, `+`
    /// concatenation, `str`, and `print` (ADR-0054).
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
            Value::List(items) => write!(
                f,
                "{{{}}}",
                items
                    .iter()
                    .map(|v| format!("{v}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            // A quantity displays its SI value in its display unit
            // (ADR-0046) — `3.2 AU`, `96.56064 km/hr`, `15 N`.
            Value::Quantity { value, dims, unit } => {
                write!(f, "{}", quantity_display(*value, *dims, unit.clone()))
            }
            Value::Matrix { rows, cols, data } => {
                let mut out = String::from("[");
                for r in 0..*rows {
                    if r > 0 {
                        out.push_str(", ");
                    }
                    out.push('[');
                    for c in 0..*cols {
                        if c > 0 {
                            out.push_str(", ");
                        }
                        out.push_str(&data[r * cols + c].to_string());
                    }
                    out.push(']');
                }
                out.push(']');
                write!(f, "{out}")
            }
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
        return auto_float(re);
    }
    let im_abs = im.abs();
    let im_part = if im_abs == 1.0 {
        "i".to_string()
    } else {
        format!("{}i", auto_float(im_abs))
    };
    if re == 0.0 {
        if im < 0.0 {
            format!("-{im_part}")
        } else {
            im_part
        }
    } else {
        let sign = if im < 0.0 { "-" } else { "+" };
        format!("{}{sign}{im_part}", auto_float(re))
    }
}

/// Variable bindings available while evaluating an [`Expression`].
#[derive(Debug, Clone)]
pub struct Env {
    bindings: HashMap<String, Value>,
    constants: HashMap<String, Value>,
    functions: HashMap<String, Function>,
    /// The seeded generator state (ADR-0045): an `Rc` so child
    /// environments (user-function bodies) share the counter — draws
    /// inside a function advance the session's sequence. `Env::default()`
    /// pins one seed so `evaluate()` and the tests are deterministic;
    /// interactive sessions re-seed from the clock in `Session::new`.
    rng: Rc<Cell<u64>>,
    /// The bitwise word size in bits (ADR-0047): 8, 16, 32, or 64,
    /// shared through child envs like the generator — `bits(8)` in a
    /// script stays in force for its function calls.
    word_bits: Rc<Cell<u32>>,
}

impl Default for Env {
    fn default() -> Self {
        Self {
            bindings: HashMap::new(),
            constants: HashMap::new(),
            functions: HashMap::new(),
            rng: Rc::new(Cell::new(0x9E37_79B9_7F4A_7C15)),
            word_bits: Rc::new(Cell::new(64)),
        }
    }
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
            rng: self.rng.clone(),
            word_bits: self.word_bits.clone(),
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
    /// A list literal (ADR-0044): `{1, 2, 3}` — the elements are
    /// expressions, evaluated when the list is.
    List(Vec<Expression>),
    /// A matrix literal (ADR-0049): `[[1, 2], [3, 4]]` — rows of
    /// expressions, evaluated when the matrix is.
    Matrix(Vec<Vec<Expression>>),
    /// A postfix element access (ADR-0044): `d[2]` is the second
    /// element, 1-based; the index is any expression.
    Index(Box<Expression>, Box<Expression>),
    /// A unit suffix (ADR-0046): `5 m`, `60 mile/hr`, `2 m^2` — the
    /// inner expression times the SI factor, carrying the dimensions
    /// and the typed display unit.
    Unit(Box<Expression>, f64, Dims, String),
    /// A unit conversion (ADR-0046): `expr in km/hr` or `expr -> km/hr`
    /// rescales a quantity to the named unit and remembers it as the
    /// display unit.
    In(Box<Expression>, f64, Dims, String),
    Compare(CmpOp, Box<Expression>, Box<Expression>),
    /// Bitwise operations (ADR-0047): `&`, `|`, `xor`, `<<`, `>>` —
    /// integer-only, results are exact `Big` whole numbers masked to
    /// the session's word size.
    BitAnd(Box<Expression>, Box<Expression>),
    BitOr(Box<Expression>, Box<Expression>),
    BitXor(Box<Expression>, Box<Expression>),
    ShiftLeft(Box<Expression>, Box<Expression>),
    ShiftRight(Box<Expression>, Box<Expression>),
    BitNot(Box<Expression>),
    If(Box<Expression>, Box<Expression>, Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Not(Box<Expression>),
    /// A string literal (ADR-0054): `"hello"`. Strings concatenate
    /// with `+`, compare with `==`/`!=`, index 1-based like lists, and
    /// feed `print`.
    StrLit(String),
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
    /// `for name in iter do body` (ADR-0054): the iterable is either a
    /// range `start to end [step s]` or a list expression. Each body
    /// value is collected; the loop's value is the list of them (an
    /// empty loop produces an empty list).
    For(String, ForIterable, Box<Statement>),
    /// `solve lhs == rhs` (ADR-0043): numeric equation solving, no CAS.
    Solve(Expression),
    Expr(Expression),
}

/// What a `for` loop iterates (ADR-0054): an inclusive numeric range
/// with an optional step, or the elements of a list.
#[derive(Debug, Clone)]
pub enum ForIterable {
    Range {
        start: Expression,
        end: Expression,
        step: Option<Expression>,
    },
    Items(Expression),
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
    #[error("dimension error: {0}")]
    Dimension(String),
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
    Amp,
    Pipe,
    Tilde,
    ShiftLeft,
    ShiftRight,
    LParen,
    RParen,
    /// List literal delimiters (ADR-0044): `{1, 2, 3}`.
    LBrace,
    RBrace,
    /// Postfix index brackets (ADR-0044): `d[2]`.
    LBracket,
    RBracket,
    /// A number with an imaginary suffix (ADR-0043): `4i`, `2.5i`,
    /// `0xFFi`. The tokenizer folds the suffix in so `3 + 4i` parses as
    /// one literal; the parser spells it `4 * i`.
    Imaginary(f64),
    /// A string literal (ADR-0054): `"hello"`. The tokenizer reads to
    /// the closing quote; there are no escape sequences, so a string
    /// cannot contain a double quote.
    Str(String),
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
                if matches!(chars.peek(), Some('>')) {
                    chars.next();
                    tokens.push(Token::ShiftRight);
                } else if matches!(chars.peek(), Some('=')) {
                    chars.next();
                    tokens.push(Token::GreaterEqual);
                } else {
                    tokens.push(Token::GreaterThan);
                }
            }
            '<' => {
                chars.next();
                if matches!(chars.peek(), Some('<')) {
                    chars.next();
                    tokens.push(Token::ShiftLeft);
                } else if matches!(chars.peek(), Some('=')) {
                    chars.next();
                    tokens.push(Token::LessEqual);
                } else {
                    tokens.push(Token::LessThan);
                }
            }
            '&' => {
                tokens.push(Token::Amp);
                chars.next();
            }
            '|' => {
                tokens.push(Token::Pipe);
                chars.next();
            }
            '~' => {
                tokens.push(Token::Tilde);
                chars.next();
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
            '{' => {
                tokens.push(Token::LBrace);
                chars.next();
            }
            '}' => {
                tokens.push(Token::RBrace);
                chars.next();
            }
            '[' => {
                tokens.push(Token::LBracket);
                chars.next();
            }
            ']' => {
                tokens.push(Token::RBracket);
                chars.next();
            }
            '"' => {
                // A string literal (ADR-0054): read to the closing
                // quote. No escape sequences; a string cannot contain a
                // double quote, which keeps the tokenizer one pass.
                chars.next(); // the opening quote
                let mut s = String::new();
                loop {
                    match chars.next() {
                        Some('"') => break,
                        Some(c2) => s.push(c2),
                        None => {
                            return Err(EpherError::Parse(
                                "unterminated string: a literal needs its closing quote".to_string(),
                            ))
                        }
                    }
                }
                tokens.push(Token::Str(s));
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
        if matches!(self.peek(), Some(Token::Ident(kw)) if kw == "for") {
            // `for i in 1 to 5 [step s] do body` or `for x in d do body`
            // (ADR-0054). The iterable is parsed as an ordinary
            // expression: `to`, `step`, and `do` are ordinary names the
            // expression grammar cannot continue with, so the range
            // forms end cleanly after their expressions.
            self.next(); // consume 'for'
            let var = self.expect_ident("loop variable")?;
            self.expect_keyword("in")?;
            let first = self.parse_expression()?;
            let iterable = if matches!(self.peek(), Some(Token::Ident(kw)) if kw == "to") {
                self.next(); // consume 'to'
                let end = self.parse_expression()?;
                let step = if matches!(self.peek(), Some(Token::Ident(kw)) if kw == "step") {
                    self.next(); // consume 'step'
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                ForIterable::Range { start: first, end, step }
            } else {
                ForIterable::Items(first)
            };
            self.expect_keyword("do")?;
            let body = Box::new(self.parse_statement()?);
            return Ok(Statement::For(var, iterable, body));
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
        let left = self.parse_bitwise()?;
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
            let right = self.parse_bitwise()?;
            Ok(Expression::Compare(op, Box::new(left), Box::new(right)))
        } else {
            Ok(left)
        }
    }

    /// Bitwise levels (ADR-0047), C-style between the comparisons and
    /// the arithmetic: `|`/`xor` bind loosest, then `&`, then the
    /// shifts. `5 & 3 == 1` is `(5 & 3) == 1`, `1 | 2 << 3` is 17.
    fn parse_bitwise(&mut self) -> Result<Expression, EpherError> {
        let mut left = self.parse_bit_and()?;
        loop {
            let is_xor = matches!(self.peek(), Some(Token::Ident(kw)) if kw == "xor");
            let is_pipe = matches!(self.peek(), Some(Token::Pipe));
            if !(is_xor || is_pipe) {
                break;
            }
            self.next();
            let right = self.parse_bit_and()?;
            left = if is_xor {
                Expression::BitXor(Box::new(left), Box::new(right))
            } else {
                Expression::BitOr(Box::new(left), Box::new(right))
            };
        }
        Ok(left)
    }

    /// The `&` level of the bitwise family.
    fn parse_bit_and(&mut self) -> Result<Expression, EpherError> {
        let mut left = self.parse_shift()?;
        while matches!(self.peek(), Some(Token::Amp)) {
            self.next();
            let right = self.parse_shift()?;
            left = Expression::BitAnd(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    /// The shift level: `<<` and `>>`, left-associative.
    fn parse_shift(&mut self) -> Result<Expression, EpherError> {
        let mut left = self.parse_additive()?;
        loop {
            match self.peek() {
                Some(Token::ShiftLeft) => {
                    self.next();
                    let right = self.parse_additive()?;
                    left = Expression::ShiftLeft(Box::new(left), Box::new(right));
                }
                Some(Token::ShiftRight) => {
                    self.next();
                    let right = self.parse_additive()?;
                    left = Expression::ShiftRight(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// The unit power bound to a unit token (ADR-0046): `m^2` scales
    /// the factor and dims and extends the display spelling. Returns
    /// the suffix node over `expr` with the folded unit.
    fn apply_unit_suffix(
        &mut self,
        expr: Expression,
        mut factor: f64,
        mut dims: Dims,
        mut unit: String,
    ) -> Result<Expression, EpherError> {
        if matches!(self.peek(), Some(Token::Caret)) {
            self.next();
            match self.next() {
                Some(Token::Number(e)) if e.fract() == 0.0 && e.abs() <= 127.0 => {
                    let e = e as i8;
                    factor = factor.powf(e as f64);
                    dims = scale_dims(dims, e)?;
                    if e != 1 {
                        unit = format!("{unit}^{e}");
                    }
                }
                other => {
                    return Err(EpherError::Parse(format!(
                        "expected a whole-number power after the unit, found {other:?}"
                    )))
                }
            }
        }
        Ok(Expression::Unit(Box::new(expr), factor, dims, unit))
    }

    /// The unit path of a conversion (ADR-0046): `km/hr`, `m^2`,
    /// `km/hr^2` — unit idents with optional whole-number powers joined
    /// by `/`. Returns the folded (SI factor, dims, display spelling).
    fn parse_unit_path(&mut self) -> Result<(f64, Dims, String), EpherError> {
        let mut factor = 1.0;
        let mut dims = [0i8; 7];
        let mut display = String::new();
        let mut first = true;
        loop {
            let name = match self.next() {
                Some(Token::Ident(n)) => n,
                other => {
                    return Err(EpherError::Parse(format!(
                        "expected a unit after the conversion, found {other:?}"
                    )))
                }
            };
            let Some(UnitDef { factor: f, dims: d }) = unit_def_with_prefix(&name) else {
                return Err(EpherError::Parse(format!("unknown unit '{name}'")));
            };
            let (mut f, mut d, mut u) = (f, d, name);
            if matches!(self.peek(), Some(Token::Caret)) {
                self.next();
                match self.next() {
                    Some(Token::Number(e)) if e.fract() == 0.0 && e.abs() <= 127.0 => {
                        let e = e as i8;
                        f = f.powf(e as f64);
                        d = scale_dims(d, e)?;
                        if e != 1 {
                            u = format!("{u}^{e}");
                        }
                    }
                    other => {
                        return Err(EpherError::Parse(format!(
                            "expected a whole-number power in the unit, found {other:?}"
                        )))
                    }
                }
            }
            if first {
                factor = f;
                dims = d;
                first = false;
            } else {
                factor /= f;
                dims = sub_dims(dims, d)?;
            }
            if !display.is_empty() {
                display.push('/');
            }
            display.push_str(&u);
            if !matches!(self.peek(), Some(Token::Slash)) {
                break;
            }
            self.next();
        }
        Ok((factor, dims, display))
    }

    /// Additive level: `+` and `-`, folded left-associatively, plus
    /// the conversion operator (ADR-0046) which binds loosest of all:
    /// `5 m + 3 m in km` converts the whole sum.
    fn parse_additive(&mut self) -> Result<Expression, EpherError> {
        let mut left = self.parse_term()?;
        loop {
            let is_in = matches!(self.peek(), Some(Token::Ident(kw)) if kw == "in");
            let is_arrow = matches!(self.peek(), Some(Token::Minus))
                && matches!(self.tokens.get(self.pos + 1), Some(Token::GreaterThan));
            if is_in || is_arrow {
                self.next();
                if is_arrow {
                    self.next();
                }
                let (factor, dims, unit) = self.parse_unit_path()?;
                left = Expression::In(Box::new(left), factor, dims, unit);
                continue;
            }
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
    /// A `/` directly after a suffixed number continues the unit chain
    /// (ADR-0046): `60 mile/hr` and `5 m/s^2` are single units, while
    /// `x / hr` still divides by the variable `hr`.
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
                    if let Expression::Unit(inner, f, d, u) = &left {
                        if let Some(Token::Ident(unit)) = self.peek().cloned() {
                            if let Some(UnitDef { factor, dims }) = unit_def_with_prefix(&unit) {
                                if !matches!(self.tokens.get(self.pos + 1), Some(Token::LParen)) {
                                    self.next();
                                    let mut u2 = unit;
                                    let mut f2 = factor;
                                    let mut d2 = dims;
                                    if matches!(self.peek(), Some(Token::Caret)) {
                                        self.next();
                                        match self.next() {
                                            Some(Token::Number(e))
                                                if e.fract() == 0.0 && e.abs() <= 127.0 =>
                                            {
                                                let e = e as i8;
                                                f2 = f2.powf(e as f64);
                                                d2 = scale_dims(d2, e)?;
                                                if e != 1 {
                                                    u2 = format!("{u2}^{e}");
                                                }
                                            }
                                            other => {
                                                return Err(EpherError::Parse(format!(
                                                    "expected a whole-number power after the unit, found {other:?}"
                                                )))
                                            }
                                        }
                                    }
                                    left = Expression::Unit(
                                        inner.clone(),
                                        *f / f2,
                                        sub_dims(*d, d2)?,
                                        format!("{u}/{u2}"),
                                    );
                                    continue;
                                }
                            }
                        }
                    }
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
        match self.peek() {
            Some(Token::Minus) => {
                self.next();
                let inner = self.parse_unary()?;
                Ok(Expression::Neg(Box::new(inner)))
            }
            Some(Token::Tilde) => {
                self.next();
                let inner = self.parse_unary()?;
                Ok(Expression::BitNot(Box::new(inner)))
            }
            _ => self.parse_pow(),
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
                Some(Token::LBracket) => {
                    // Postfix index (ADR-0044): `d[2]` is the second
                    // element, 1-based. Binds tighter than `^`, so
                    // `d[2]^2` is `(d[2])^2`.
                    self.next();
                    let index = self.parse_expression()?;
                    self.expect_token(Token::RBracket, "']' after the index")?;
                    expr = Expression::Index(Box::new(expr), Box::new(index));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, EpherError> {
        match self.next() {
            Some(Token::Number(n)) => {
                // Unit-suffix literal (ADR-0037, extended by ADR-0046):
                // a number immediately followed by a unit token is that
                // number times the unit's SI factor, carrying the
                // dimensions and the typed display unit - baked in at
                // grammar level, so user shadowing cannot change what
                // `2 AU` means. An Ident followed by `(` is always a
                // call, never a suffix, so `30 deg(x)` stays a
                // (trailing-input) parse error and `min(3, 7)` keeps
                // working next to `5 min`. A whole-number power binds
                // to the unit, not the number: `2 m^2` is two square
                // metres, `(2 m)^2` the square of two metres.
                if let Some(Token::Ident(name)) = self.peek().cloned() {
                    if let Some(UnitDef { factor, dims }) = unit_def_with_prefix(&name) {
                        if !matches!(self.tokens.get(self.pos + 1), Some(Token::LParen)) {
                            self.next();
                            return self.apply_unit_suffix(
                                Expression::Literal(n),
                                factor,
                                dims,
                                name,
                            );
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
            Some(Token::Str(s)) => Ok(Expression::StrLit(s)),
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
                                if let Some(UnitDef { factor, dims }) = unit_def_with_prefix(&unit)
                                {
                                    if !matches!(self.tokens.get(self.pos + 1), Some(Token::LParen))
                                    {
                                        self.next();
                                        return self.apply_unit_suffix(expr, factor, dims, unit);
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
            // A matrix literal (ADR-0049): an expression-start `[`
            // begins the row-of-rows spelling `[[1, 2], [3, 4]]`.
            // Postfix `[` stays the index operator — the two positions
            // never collide.
            Some(Token::LBracket) => {
                // (the match already consumed the opening '[')
                let mut rows = Vec::new();
                if matches!(self.peek(), Some(Token::RBracket)) {
                    return Err(EpherError::Parse(
                        "a matrix needs rows: [[1, 2], [3, 4]]".to_string(),
                    ));
                }
                loop {
                    self.expect_token(Token::LBracket, "'[' to start a row")?;
                    let mut row = Vec::new();
                    if matches!(self.peek(), Some(Token::RBracket)) {
                        return Err(EpherError::Parse(
                            "a matrix row needs at least one element".to_string(),
                        ));
                    }
                    loop {
                        row.push(self.parse_expression()?);
                        match self.next() {
                            Some(Token::Comma) => continue,
                            Some(Token::RBracket) => break,
                            Some(other) => {
                                return Err(EpherError::Parse(format!(
                                    "expected ',' or ']' in the row, found {other:?}"
                                )))
                            }
                            None => {
                                return Err(EpherError::Parse("unexpected end of input".into()))
                            }
                        }
                    }
                    rows.push(row);
                    match self.next() {
                        Some(Token::Comma) => continue,
                        Some(Token::RBracket) => break,
                        Some(other) => {
                            return Err(EpherError::Parse(format!(
                                "expected ',' or ']' between rows, found {other:?}"
                            )))
                        }
                        None => return Err(EpherError::Parse("unexpected end of input".into())),
                    }
                }
                Ok(Expression::Matrix(rows))
            }
            Some(Token::LBrace) => {
                // A list literal (ADR-0044): `{1, 2, 3}`, `{}`, elements
                // are expressions. A trailing comma is allowed (`{1, 2,}`
                // — the comma is a separator, not a terminator).
                let mut items = Vec::new();
                if matches!(self.peek(), Some(Token::RBrace)) {
                    self.next();
                    return Ok(Expression::List(items));
                }
                loop {
                    let item = self.parse_expression()?;
                    items.push(item);
                    match self.next() {
                        Some(Token::Comma) => {
                            if matches!(self.peek(), Some(Token::RBrace)) {
                                self.next();
                                break;
                            }
                        }
                        Some(Token::RBrace) => break,
                        Some(other) => {
                            return Err(EpherError::Parse(format!(
                                "expected ',' or '}}' in the list, found {other:?}"
                            )))
                        }
                        None => {
                            return Err(EpherError::Parse(
                                "unexpected end of input in the list".into(),
                            ))
                        }
                    }
                }
                Ok(Expression::List(items))
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
            Value::Quantity { value, dims, unit } => Ok(Value::Quantity {
                value: -value,
                dims,
                unit,
            }),
            Value::Matrix { rows, cols, data } => Ok(Value::Matrix {
                rows,
                cols,
                data: data.iter().map(|x| -x).collect(),
            }),
            Value::List(items) => {
                let mut out = Vec::with_capacity(items.len());
                for v in items {
                    match v {
                        Value::Float(n) => out.push(Value::Float(-n)),
                        other => return Err(EpherError::Type(format!("cannot negate {other:?}"))),
                    }
                }
                Ok(Value::List(out))
            }
            other => Err(EpherError::Type(format!("cannot negate {other:?}"))),
        },
        Expression::Add(lhs, rhs) => binop(eval(lhs, env)?, eval(rhs, env)?, BinOp::Add),
        Expression::Sub(lhs, rhs) => binop(eval(lhs, env)?, eval(rhs, env)?, BinOp::Sub),
        Expression::Mul(lhs, rhs) => binop(eval(lhs, env)?, eval(rhs, env)?, BinOp::Mul),
        Expression::Div(lhs, rhs) => binop(eval(lhs, env)?, eval(rhs, env)?, BinOp::Div),
        Expression::Pow(lhs, rhs) => binop(eval(lhs, env)?, eval(rhs, env)?, BinOp::Pow),
        // Bitwise operations (ADR-0047): exact Big integers masked to
        // the session's word size.
        Expression::BitAnd(lhs, rhs) => {
            bitwise_binop(BitOp::And, eval(lhs, env)?, eval(rhs, env)?, env)
        }
        Expression::BitOr(lhs, rhs) => {
            bitwise_binop(BitOp::Or, eval(lhs, env)?, eval(rhs, env)?, env)
        }
        Expression::BitXor(lhs, rhs) => {
            bitwise_binop(BitOp::Xor, eval(lhs, env)?, eval(rhs, env)?, env)
        }
        Expression::ShiftLeft(lhs, rhs) => {
            bitwise_binop(BitOp::Shl, eval(lhs, env)?, eval(rhs, env)?, env)
        }
        Expression::ShiftRight(lhs, rhs) => {
            bitwise_binop(BitOp::Shr, eval(lhs, env)?, eval(rhs, env)?, env)
        }
        Expression::BitNot(inner) => {
            let v = eval(inner, env)?;
            let x = value_to_bigint("~", &v)?;
            Ok(mask_word(-x - 1, env.word_bits.get()))
        }
        // A unit suffix (ADR-0046): the value times the SI factor,
        // carrying the dimensions and the typed display unit.
        Expression::Unit(inner, factor, dims, unit) => match eval(inner, env)? {
            Value::Float(n) => Ok(Value::Quantity {
                value: n * factor,
                dims: *dims,
                unit: Some((unit.clone(), *factor)),
            }),
            other => Err(EpherError::Type(format!(
                "a unit suffix applies to a number, got {other:?}"
            ))),
        },
        // A unit conversion (ADR-0046): `expr in unit` rescales the
        // SI value to the named unit and remembers it as the display
        // unit; the dimensions must match.
        Expression::In(inner, factor, dims, unit) => match eval(inner, env)? {
            Value::Float(n) => Ok(Value::Quantity {
                value: n * factor,
                dims: *dims,
                unit: Some((unit.clone(), *factor)),
            }),
            Value::Quantity {
                value,
                dims: d,
                unit: _,
            } => {
                if d != *dims {
                    return Err(dimension_error(&format!(
                        "cannot convert {} to {unit}: the dimensions do not match",
                        quantity_display(value, d, None)
                    )));
                }
                // The SI value is unchanged; the display unit only
                // rescales what is shown.
                Ok(Value::Quantity {
                    value,
                    dims: d,
                    unit: Some((unit.clone(), *factor)),
                })
            }
            other => Err(EpherError::Type(format!(
                "cannot convert {other:?} to {unit}"
            ))),
        },
        Expression::List(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                let v = eval(item, env)?;
                match v {
                    Value::Float(_) => out.push(v),
                    other => {
                        return Err(EpherError::Type(format!(
                            "lists hold numbers, got {other:?}"
                        )))
                    }
                }
            }
            Ok(Value::List(out))
        }
        // A matrix literal (ADR-0049): rows of floats, uniform length.
        Expression::Matrix(rows) => {
            let mut data = Vec::new();
            let cols = rows.first().map(|r| r.len()).unwrap_or(0);
            for row in rows {
                if row.len() != cols {
                    return Err(EpherError::Type(format!(
                        "matrix rows must have the same length: {} and {}",
                        cols,
                        row.len()
                    )));
                }
                for item in row {
                    match eval(item, env)? {
                        Value::Float(x) => data.push(x),
                        other => {
                            return Err(EpherError::Type(format!(
                                "matrices hold numbers, got {other:?}"
                            )))
                        }
                    }
                }
            }
            let rows = rows.len();
            if rows == 0 || cols == 0 {
                return Err(EpherError::Type(
                    "a matrix needs at least one row".to_string(),
                ));
            }
            Ok(Value::Matrix { rows, cols, data })
        }
        Expression::Index(list, index) => {
            let list = eval(list, env)?;
            let index = eval(index, env)?;
            let Value::Float(i) = index else {
                return Err(EpherError::Type(format!(
                    "the index must be a whole number, got {index:?}"
                )));
            };
            let i = float_to_int(i).ok_or_else(|| {
                EpherError::Type(format!("the index must be a whole number, got {i}"))
            })?;
            match &list {
                Value::List(items) => {
                    if !(1..=items.len() as i64).contains(&i) {
                        return Err(EpherError::Type(format!(
                            "index {i} is out of range for a list of {} element(s)",
                            items.len()
                        )));
                    }
                    Ok(items[(i - 1) as usize].clone())
                }
                // A matrix row is a list (ADR-0049): `M[2][1]` chains.
                Value::Matrix { rows, cols, data } => {
                    if !(1..=*rows as i64).contains(&i) {
                        return Err(EpherError::Type(format!(
                            "index {i} is out of range for a matrix with {rows} rows"
                        )));
                    }
                    let start = (i - 1) as usize * *cols;
                    Ok(Value::List(
                        data[start..start + *cols]
                            .iter()
                            .map(|x| Value::Float(*x))
                            .collect(),
                    ))
                }
                // A 1-based character (ADR-0054): `"hello"[1]` is "h".
                Value::Str(s) => {
                    let n = s.chars().count();
                    if !(1..=n as i64).contains(&i) {
                        return Err(EpherError::Type(format!(
                            "index {i} is out of range for a string of {n} character(s)"
                        )));
                    }
                    Ok(Value::Str(s.chars().nth((i - 1) as usize).expect("in range").to_string()))
                }
                other => Err(EpherError::Type(format!(
                    "indexing needs a list, matrix, or string, got {other:?}"
                ))),
            }
        }
        Expression::StrLit(s) => Ok(Value::Str(s.clone())),
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
                // Whole-list equality (ADR-0044): `{1, 2} == {1, 2}` is
                // true; ordering comparisons stay a type error.
                (Value::List(a), Value::List(b)) if matches!(op, CmpOp::Eq | CmpOp::Ne) => {
                    Ok(Value::Bool(if matches!(op, CmpOp::Eq) {
                        a == b
                    } else {
                        a != b
                    }))
                }
                // Quantity comparisons (ADR-0046): values compare when
                // the dimensions match (a plain number is dimensionless);
                // a mismatch is a dimension error, not a false answer.
                (Value::Quantity { .. }, _) | (_, Value::Quantity { .. }) => {
                    let (a, da, ua) = as_quantity(l)?;
                    let (b, db, ub) = as_quantity(r)?;
                    if da != db {
                        return Err(dimension_error(&format!(
                            "cannot compare {} and {}",
                            quantity_display(a, da, ua),
                            quantity_display(b, db, ub)
                        )));
                    }
                    let result = match op {
                        CmpOp::Gt => a > b,
                        CmpOp::Lt => a < b,
                        CmpOp::Ge => a >= b,
                        CmpOp::Le => a <= b,
                        CmpOp::Eq => a == b,
                        CmpOp::Ne => a != b,
                    };
                    Ok(Value::Bool(result))
                }
                // Whole-matrix equality (ADR-0049): `A == B` compares
                // elementwise; ordering comparisons stay a type error.
                (Value::Matrix { .. }, Value::Matrix { .. })
                    if matches!(op, CmpOp::Eq | CmpOp::Ne) =>
                {
                    Ok(Value::Bool(if matches!(op, CmpOp::Eq) {
                        l == r
                    } else {
                        l != r
                    }))
                }
                // String equality (ADR-0054): `"a" == "b"` compares
                // whole strings; ordering stays a type error.
                (Value::Str(a), Value::Str(b)) if matches!(op, CmpOp::Eq | CmpOp::Ne) => {
                    Ok(Value::Bool(if matches!(op, CmpOp::Eq) { a == b } else { a != b }))
                }
                // Numeric comparisons across all the numeric types
                // (ADR-0047): same-type exact pairs compare exactly;
                // anything with a float or mixed exact types compares
                // through f64. Bitwise results are Big, so `1 & 3 == 1`
                // must work.
                _ => {
                    let ord = match (&l, &r) {
                        (Value::Float(x), Value::Float(y)) => x.partial_cmp(y),
                        (Value::Big(x), Value::Big(y)) => x.partial_cmp(y),
                        (Value::Rational(x), Value::Rational(y)) => x.partial_cmp(y),
                        (Value::Decimal(x), Value::Decimal(y)) => x.partial_cmp(y),
                        (Value::Float(x), other) => x.partial_cmp(&numeric_f64(other)?),
                        (other, Value::Float(y)) => numeric_f64(other)?.partial_cmp(y),
                        (a, b) => numeric_f64(a)?.partial_cmp(&numeric_f64(b)?),
                    };
                    let Some(ord) = ord else {
                        return Err(EpherError::Type(format!("cannot compare {l:?} and {r:?}")));
                    };
                    let result = match op {
                        CmpOp::Gt => ord == std::cmp::Ordering::Greater,
                        CmpOp::Lt => ord == std::cmp::Ordering::Less,
                        CmpOp::Ge => ord != std::cmp::Ordering::Less,
                        CmpOp::Le => ord != std::cmp::Ordering::Greater,
                        CmpOp::Eq => ord == std::cmp::Ordering::Equal,
                        CmpOp::Ne => ord != std::cmp::Ordering::Equal,
                    };
                    Ok(Value::Bool(result))
                }
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
                // Seeded random numbers (ADR-0045): the generator state
                // lives in the environment, so these need `env` like the
                // calculus forms do.
                "random" | "randint" | "randseed" | "randn" => {
                    let mut values = Vec::with_capacity(args.len());
                    for arg in args {
                        values.push(eval(arg, env)?);
                    }
                    eval_random(name, values, env)
                }
                // The bitwise word size (ADR-0047): `bits()` reports,
                // `bits(8|16|32|64)` sets it and reports it.
                "bits" => {
                    let mut values = Vec::with_capacity(args.len());
                    for arg in args {
                        values.push(eval(arg, env)?);
                    }
                    match values.len() {
                        0 => Ok(Value::Float(env.word_bits.get() as f64)),
                        1 => {
                            let n = integer_arg(name, &values)?;
                            if !matches!(n, 8 | 16 | 32 | 64) {
                                return Err(domain_error(format!(
                                    "bits expects 8, 16, 32, or 64, got {n}"
                                )));
                            }
                            env.word_bits.set(n as u32);
                            Ok(Value::Float(n as f64))
                        }
                        _ => Err(EpherError::Type(format!(
                            "bits takes 0 or 1 arguments, got {}",
                            values.len()
                        ))),
                    }
                }
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
/// `x` when it appears (symbolic, even when the session holds a value
/// for it, like solve), otherwise the single other free variable.
/// Constants (builtin and user) are parameters, never unknowns; bound
/// variables are parameters too when x is not the one differentiated.
/// No candidates means the expression is constant; several is an error.
fn calculus_var(expr: &Expression, env: &Env) -> Result<Option<String>, EpherError> {
    let mut names = std::collections::BTreeSet::new();
    crate::graph::free_names(expr, &mut names);
    names.retain(|n| builtin_const(n).is_none() && env.constant(n).is_none());
    if names.is_empty() {
        return Ok(None);
    }
    let variable = if names.contains("x") {
        "x"
    } else if names.len() == 1 {
        names.iter().next().expect("len checked").as_str()
    } else {
        return Err(EpherError::Type(format!(
            "the expression uses several variables: {}",
            names.iter().cloned().collect::<Vec<_>>().join(", ")
        )));
    };
    // every name besides the differentiated one must be a bound
    // parameter (a value, not an unknown)
    for other in names.iter().filter(|n| n.as_str() != variable) {
        if env.get(other).is_none() {
            return Err(EpherError::Type(format!(
                "the expression uses several variables: {}",
                names.iter().cloned().collect::<Vec<_>>().join(", ")
            )));
        }
    }
    Ok(Some(variable.to_string()))
}

/// The calculus child environment: the caller's bindings (bound names
/// are parameters), the constants, the functions, and the calculus
/// variable bound to a fresh value - shadowing any session value.
fn calculus_child(env: &Env, var: &str, value: f64) -> Env {
    let mut child = Env {
        bindings: env.bindings.clone(),
        constants: env.constants.clone(),
        functions: env.functions.clone(),
        rng: env.rng.clone(),
        word_bits: env.word_bits.clone(),
    };
    child.set(var.to_string(), Value::float(value));
    child
}

/// `derivative(expr, p)` (ADR-0043): the numeric derivative of the
/// expression at p, 5-point central difference with step
/// 1e-4 * (1 + |p|). A constant expression differentiates to 0.
/// SplitMix64: one 64-bit counter, three mixes per draw. Deterministic
/// per seed, wasm-safe, and small enough to inline in the evaluator
/// (ADR-0045). The returned value is the draw; `state` advances.
fn splitmix_next(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The bitwise operations (ADR-0047): operands are integers (whole
/// floats, integral rationals/decimals, or Big), the result is the
/// exact Big integer masked to the session's word size as a signed
/// two's complement word. Shifts by a negative amount reverse, and
/// right shift is arithmetic.
fn bitwise_binop(op: BitOp, lhs: Value, rhs: Value, env: &Env) -> Result<Value, EpherError> {
    let a = value_to_bigint("&", &lhs)?;
    let b = value_to_bigint("&", &rhs)?;
    let n = match op {
        BitOp::And => a & b,
        BitOp::Or => a | b,
        BitOp::Xor => a ^ b,
        BitOp::Shl | BitOp::Shr => {
            let shift = b
                .to_i64()
                .ok_or_else(|| EpherError::Type(format!("shift amount {b} is too large")))?;
            let shift_abs = shift.unsigned_abs();
            // A negative shift reverses the direction; right shift is
            // arithmetic (floor for negatives, like Python).
            if matches!(op, BitOp::Shl) {
                if shift >= 0 {
                    a << shift_abs
                } else {
                    a >> shift_abs
                }
            } else if shift >= 0 {
                a >> shift_abs
            } else {
                a << shift_abs
            }
        }
    };
    Ok(mask_word(n, env.word_bits.get()))
}

/// Mask a mathematical integer to a signed n-bit two's complement word
/// (ADR-0047): the low n bits, with the top bit deciding the sign.
fn mask_word(v: num_bigint::BigInt, bits: u32) -> Value {
    let mask: num_bigint::BigInt = (num_bigint::BigInt::from(1) << bits as usize) - 1;
    let m: num_bigint::BigInt = v & mask;
    let sign: num_bigint::BigInt = num_bigint::BigInt::from(1) << (bits as usize - 1);
    let word = if m >= sign { m - (sign << 1) } else { m };
    Value::Big(word.to_string().parse().expect("whole big"))
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum BitOp {
    And,
    Or,
    Xor,
    Shl,
    Shr,
}

/// Seeded random numbers (ADR-0045): `random()` is uniform in [0, 1),
/// `random(a, b)` uniform in [a, b), `randint(a, b)` a whole number in
/// the closed range [a, b] (Lemire's rejection method, no modulo bias),
/// and `randseed(n)` re-seeds and reports n. The state lives in the
/// environment, shared with user-function bodies through the child envs.
fn eval_random(name: &str, args: Vec<Value>, env: &Env) -> Result<Value, EpherError> {
    let next = |env: &Env| -> u64 {
        let mut state = env.rng.get();
        let z = splitmix_next(&mut state);
        env.rng.set(state);
        z
    };
    match name {
        "randseed" => {
            let n = integer_arg(name, &args)?;
            env.rng.set(n as u64);
            Ok(Value::Float(n as f64))
        }
        "random" => {
            let u = ((next(env) >> 11) as f64) * (1.0 / 9_007_199_254_740_992.0);
            match args.len() {
                0 => Ok(Value::Float(u)),
                2 => {
                    let (a, b) = two_floats(name, &args)?;
                    if a.partial_cmp(&b) != Some(std::cmp::Ordering::Less) {
                        return Err(domain_error(format!(
                            "random(a, b) needs a < b, got {a} and {b}"
                        )));
                    }
                    Ok(Value::Float(a + (b - a) * u))
                }
                _ => Err(EpherError::Type(format!(
                    "{name} takes no arguments or two, got {}",
                    args.len()
                ))),
            }
        }
        "randn" => {
            // Normal draws (ADR-0054): Box-Muller on the seeded
            // generator, so `randseed` makes every draw reproducible
            // exactly like `random` (Desmos randomNormal, TI randNorm).
            let (mu, sigma) = two_floats(name, &args)?;
            if sigma <= 0.0 {
                return Err(domain_error(format!("{name} needs sigma > 0, got {sigma}")));
            }
            let uniform = || {
                let u = ((next(env) >> 11) as f64) * (1.0 / 9_007_199_254_740_992.0);
                // 0.0 is measure-zero but a ln(0) would poison the draw
                if u == 0.0 { f64::MIN_POSITIVE } else { u }
            };
            let u1 = uniform();
            let u2 = uniform();
            let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            Ok(Value::Float(mu + sigma * z))
        }
        _ => {
            let (a, b) = integer_pair(name, &args)?;
            if a > b {
                return Err(domain_error(format!(
                    "randint(a, b) needs a <= b, got {a} and {b}"
                )));
            }
            let m = (b - a) as u64 + 1;
            let high = loop {
                let x = next(env);
                let prod = (x as u128) * (m as u128);
                let low = prod as u64;
                if low >= m {
                    break (prod >> 64) as u64;
                }
                let threshold = (0u64).wrapping_sub(m) % m;
                if low >= threshold {
                    break (prod >> 64) as u64;
                }
            };
            Ok(Value::Float(a as f64 + high as f64))
        }
    }
}

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
    derivative_at(&args[0], p, env).map(Value::float)
}

/// Numeric derivative of `expr` at `p` (the 5-point stencil,
/// ADR-0043): the shared engine behind `derivative(expr, p)` and the
/// table's derivative column (ADR-0044). A constant expression (no
/// free variable) differentiates to 0.
pub(crate) fn derivative_at(expr: &Expression, p: f64, env: &Env) -> Result<f64, EpherError> {
    let Some(var) = calculus_var(expr, env)? else {
        return Ok(0.0);
    };
    let h = 1e-4 * (1.0 + p.abs());
    let at = |x: f64| -> Result<f64, EpherError> {
        let child = calculus_child(env, &var, x);
        match eval(expr, &child)? {
            Value::Float(y) => Ok(y),
            other => Err(EpherError::Type(format!(
                "derivative expects a real-valued expression, got {other}"
            ))),
        }
    };
    // 5-point stencil: error ~ h^4 in the function, rounding ~ eps/h
    let ym2 = at(p - 2.0 * h)?;
    let ym1 = at(p - h)?;
    let y1 = at(p + h)?;
    let y2 = at(p + 2.0 * h)?;
    Ok((ym2 - 8.0 * ym1 + 8.0 * y1 - y2) / (12.0 * h))
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
/// The seven SI base dimensions (ADR-0046): length, mass, time,
/// electric current, thermodynamic temperature, amount of substance,
/// and luminous intensity — the exponent of each in a quantity.
pub type Dims = [i8; 7];

pub const DIMS_L: Dims = [1, 0, 0, 0, 0, 0, 0];
pub const DIMS_M: Dims = [0, 1, 0, 0, 0, 0, 0];
pub const DIMS_T: Dims = [0, 0, 1, 0, 0, 0, 0];

/// A unit table entry: the SI factor and the base dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitDef {
    pub factor: f64,
    pub dims: Dims,
}

/// The unit table (ADR-0037, extended by ADR-0046): exact token →
/// (SI factor, dimensions). Angle units are dimensionless; `h` is
/// absent (Planck's constant keeps its name — hours are `hr`), and so
/// is `in` (the conversion operator; inches are `inch`).
fn unit_def(token: &str) -> Option<UnitDef> {
    let d = |f: f64, dims: Dims| Some(UnitDef { factor: f, dims });
    match token {
        // SI base units
        "m" => d(1.0, DIMS_L),
        "s" => d(1.0, DIMS_T),
        "g" => d(1e-3, DIMS_M),
        "kg" => d(1.0, DIMS_M),
        "A" => d(1.0, [0, 0, 0, 1, 0, 0, 0]),
        "K" => d(1.0, [0, 0, 0, 0, 1, 0, 0]),
        "mol" => d(1.0, [0, 0, 0, 0, 0, 1, 0]),
        "cd" => d(1.0, [0, 0, 0, 0, 0, 0, 1]),
        // SI derived units (exact display names for the same dims)
        "Hz" => d(1.0, [0, 0, -1, 0, 0, 0, 0]),
        "N" => d(1.0, [1, 1, -2, 0, 0, 0, 0]),
        "Pa" => d(1.0, [-1, 1, -2, 0, 0, 0, 0]),
        "J" => d(1.0, [2, 1, -2, 0, 0, 0, 0]),
        "W" => d(1.0, [2, 1, -3, 0, 0, 0, 0]),
        "C" => d(1.0, [0, 0, 1, 1, 0, 0, 0]),
        "V" => d(1.0, [2, 1, -3, -1, 0, 0, 0]),
        "F" => d(1.0, [-2, 1, 4, 2, 0, 0, 0]),
        "ohm" | "Ohm" => d(1.0, [2, 1, -3, -2, 0, 0, 0]),
        "S" => d(1.0, [-2, 1, 3, 2, 0, 0, 0]),
        "Wb" => d(1.0, [2, 1, -2, -1, 0, 0, 0]),
        "T" => d(1.0, [0, 1, -2, -1, 0, 0, 0]),
        "H" => d(1.0, [2, 1, -2, -2, 0, 0, 0]),
        "lm" => d(1.0, [0, 0, 0, 0, 0, 1, 1]),
        "lx" => d(1.0, [-2, 0, 0, 0, 0, 1, 1]),
        "Bq" => d(1.0, [0, 0, -1, 0, 0, 0, 0]),
        "Gy" | "Sv" => d(1.0, [2, 0, -2, 0, 0, 0, 0]),
        // Common non-SI units
        "L" | "l" => d(1e-3, [3, 0, 0, 0, 0, 0, 0]),
        "t" => d(1e3, DIMS_M),
        "bar" => d(1e5, [-1, 1, -2, 0, 0, 0, 0]),
        "atm" => d(101_325.0, [-1, 1, -2, 0, 0, 0, 0]),
        "torr" => d(133.322_368_421_052_63, [-1, 1, -2, 0, 0, 0, 0]),
        "psi" => d(6_894.757_293_168_361, [-1, 1, -2, 0, 0, 0, 0]),
        "eV" => d(1.602_176_634e-19, [2, 1, -2, 0, 0, 0, 0]),
        // Time (ADR-0037) and angles (dimensionless)
        "min" => d(60.0, DIMS_T),
        "hr" => d(3_600.0, DIMS_T),
        "d" => d(86_400.0, DIMS_T),
        "yr" => d(31_557_600.0, DIMS_T),
        "rad" => d(1.0, [0; 7]),
        "deg" => d(std::f64::consts::PI / 180.0, [0; 7]),
        "arcmin" => d(std::f64::consts::PI / 10_800.0, [0; 7]),
        "arcsec" => d(std::f64::consts::PI / 648_000.0, [0; 7]),
        // Imperial and everyday units
        "mile" => d(1_609.344, DIMS_L),
        "yd" => d(0.9144, DIMS_L),
        "ft" => d(0.3048, DIMS_L),
        "inch" => d(0.0254, DIMS_L),
        "nmi" => d(1_852.0, DIMS_L),
        "lb" => d(0.453_592_37, DIMS_M),
        "oz" => d(0.028_349_523_125, DIMS_M),
        "gal" => d(3.785_411_784e-3, [3, 0, 0, 0, 0, 0, 0]),
        "qt" => d(9.463_529_46e-4, [3, 0, 0, 0, 0, 0, 0]),
        "pt" => d(4.731_764_73e-4, [3, 0, 0, 0, 0, 0, 0]),
        "mph" => d(0.447_04, [1, 0, -1, 0, 0, 0, 0]),
        "knot" => d(0.514_444_444_444_444_5, [1, 0, -1, 0, 0, 0, 0]),
        // Astronomy (ADR-0037): lengths, jansky = W m-2 Hz-1
        "AU" | "au" => d(1.495_978_707e11, DIMS_L),
        "pc" => d(3.085_677_581_491_367_3e16, DIMS_L),
        "ly" => d(9.460_730_472_580_8e15, DIMS_L),
        "Jy" => d(1e-26, [0, 1, -2, 0, 0, 0, 0]),
        _ => None,
    }
}

/// The SI prefixes (longest first, so `da` wins over `d` and `µ` over
/// `u`): token → factor.
const UNIT_PREFIXES: &[(&str, f64)] = &[
    ("da", 1e1),
    ("Y", 1e24),
    ("Z", 1e21),
    ("E", 1e18),
    ("P", 1e15),
    ("T", 1e12),
    ("G", 1e9),
    ("M", 1e6),
    ("k", 1e3),
    ("h", 1e2),
    ("d", 1e-1),
    ("c", 1e-2),
    ("m", 1e-3),
    ("µ", 1e-6),
    ("u", 1e-6),
    ("n", 1e-9),
    ("p", 1e-12),
    ("f", 1e-15),
    ("a", 1e-18),
    ("z", 1e-21),
    ("y", 1e-24),
];

/// A unit token with its prefix resolved: exact table match first
/// (`kg` is the kilogram, `Pa` the pascal, `cd` the candela), then a
/// prefix plus a table unit (`km`, `ms`, `µm`, `MPa`, `dam`).
fn unit_def_with_prefix(token: &str) -> Option<UnitDef> {
    if let Some(def) = unit_def(token) {
        return Some(def);
    }
    for (prefix, p) in UNIT_PREFIXES {
        if let Some(rest) = token.strip_prefix(prefix) {
            if let Some(UnitDef { factor, dims }) = unit_def(rest) {
                return Some(UnitDef {
                    factor: p * factor,
                    dims,
                });
            }
        }
    }
    None
}

/// The SI spelling of a dims vector (ADR-0046): the exact derived
/// name when the dims match one (`N`, `W`, `Pa`, …), else the composed
/// base form (`m/s^2`, `kg m/s^2`, `1/s`).
pub fn si_unit_str(dims: Dims) -> String {
    const DERIVED: &[(Dims, &str)] = &[
        ([0, 0, -1, 0, 0, 0, 0], "Hz"),
        ([1, 1, -2, 0, 0, 0, 0], "N"),
        ([-1, 1, -2, 0, 0, 0, 0], "Pa"),
        ([2, 1, -2, 0, 0, 0, 0], "J"),
        ([2, 1, -3, 0, 0, 0, 0], "W"),
        ([0, 0, 1, 1, 0, 0, 0], "C"),
        ([2, 1, -3, -1, 0, 0, 0], "V"),
        ([-2, 1, 4, 2, 0, 0, 0], "F"),
        ([2, 1, -3, -2, 0, 0, 0], "ohm"),
        ([-2, 1, 3, 2, 0, 0, 0], "S"),
        ([2, 1, -2, -1, 0, 0, 0], "Wb"),
        ([0, 1, -2, -1, 0, 0, 0], "T"),
        ([2, 1, -2, -2, 0, 0, 0], "H"),
        ([0, 0, 0, 0, 0, 1, 1], "lm"),
        ([-2, 0, 0, 0, 0, 1, 1], "lx"),
        ([2, 0, -2, 0, 0, 0, 0], "Gy"),
    ];
    if dims == [0; 7] {
        return String::new();
    }
    for (d, name) in DERIVED {
        if *d == dims {
            return (*name).to_string();
        }
    }
    let base = [
        ("m", 0),
        ("kg", 1),
        ("s", 2),
        ("A", 3),
        ("K", 4),
        ("mol", 5),
        ("cd", 6),
    ];
    let mut pos = Vec::new();
    let mut neg = Vec::new();
    for (name, i) in base {
        let e = dims[i];
        if e > 0 {
            pos.push(if e == 1 {
                name.to_string()
            } else {
                format!("{name}^{e}")
            });
        } else if e < 0 {
            neg.push(if e == -1 {
                name.to_string()
            } else {
                format!("{name}^{}", -e)
            });
        }
    }
    let head = if pos.is_empty() {
        "1".to_string()
    } else {
        pos.join(" ")
    };
    if neg.is_empty() {
        head
    } else {
        format!("{head}/{}", neg.join(" "))
    }
}

/// Is the token a unit (with its prefix resolved)? The frontends use
/// this so the autocomplete never hijacks a unit-ending entry
/// (ADR-0046): `5 m` plus Enter evaluates instead of completing `m`
/// to `m_P(`.
pub fn is_unit_token(token: &str) -> bool {
    unit_def_with_prefix(token).is_some()
}

/// The old unit_suffix spelling: keep `unit_factor` for the handful of
/// callers that only need the SI factor (none today — see ADR-0046).
#[allow(dead_code)]
fn unit_factor(token: &str) -> Option<f64> {
    unit_def_with_prefix(token).map(|u| u.factor)
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
        // Lunar mass/radius (IAU, ADR-0045): SI like the solar ones.
        "m_moon" => Some(Value::float(7.342e22)),
        "r_moon" => Some(Value::float(1.737_4e6)),
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
        // The rest of the standard CODATA 2022 set (ADR-0045), SI units
        // like their peers: Planck mass/length/time, the classical
        // electron radius, the Compton wavelength, the nuclear magneton.
        "m_P" => Some(Value::float(2.176_434e-8)),
        "l_P" => Some(Value::float(1.616_255e-35)),
        "t_P" => Some(Value::float(5.391_247e-44)),
        "r_e" => Some(Value::float(2.817_940_320_5e-15)),
        "lambda_c" => Some(Value::float(2.426_310_238_67e-12)),
        "mu_n" => Some(Value::float(5.050_783_699e-27)),
        _ => None,
    }
}

/// The group a builtin constant belongs to, mirroring the guide's
/// tables — what the constants browsers (ADR-0045) use to organize
/// their lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstGroup {
    Math,
    Astronomy,
    Physics,
    Chemistry,
}

/// The SI value of a builtin constant, for the constants browsers
/// (ADR-0045): the same resolution the evaluator uses, so a browser can
/// never show a value the language would not evaluate. Constants with
/// non-float values (the imaginary unit `i`) return `None`.
pub fn builtin_constant_value(name: &str) -> Option<f64> {
    match builtin_const(name) {
        Some(Value::Float(x)) => Some(x),
        _ => None,
    }
}

/// Every builtin constant with its group, the single source of truth
/// for the frontends' browsers: name and group, sorted by name. The
/// values come from [`builtin_const`] via the evaluator's name
/// resolution, so the browser can never drift from what evaluates.
pub fn builtin_constant_groups() -> &'static [(&'static str, ConstGroup)] {
    &[
        ("G", ConstGroup::Physics),
        ("a_0", ConstGroup::Physics),
        ("alpha", ConstGroup::Physics),
        ("atm", ConstGroup::Chemistry),
        ("au", ConstGroup::Astronomy),
        ("c", ConstGroup::Astronomy),
        ("e", ConstGroup::Math),
        ("eps_0", ConstGroup::Physics),
        ("ev", ConstGroup::Physics),
        ("faraday", ConstGroup::Chemistry),
        ("g", ConstGroup::Astronomy),
        ("gamma", ConstGroup::Math),
        ("h", ConstGroup::Astronomy),
        ("h_bar", ConstGroup::Astronomy),
        ("i", ConstGroup::Math),
        ("k_b", ConstGroup::Astronomy),
        ("l_P", ConstGroup::Physics),
        ("l_sun", ConstGroup::Astronomy),
        ("lambda_c", ConstGroup::Physics),
        ("ly", ConstGroup::Astronomy),
        ("m_P", ConstGroup::Physics),
        ("m_e", ConstGroup::Physics),
        ("m_earth", ConstGroup::Astronomy),
        ("m_moon", ConstGroup::Astronomy),
        ("m_n", ConstGroup::Physics),
        ("m_p", ConstGroup::Physics),
        ("m_sun", ConstGroup::Astronomy),
        ("m_u", ConstGroup::Physics),
        ("mu_0", ConstGroup::Physics),
        ("mu_b", ConstGroup::Physics),
        ("mu_n", ConstGroup::Physics),
        ("n_a", ConstGroup::Chemistry),
        ("pc", ConstGroup::Astronomy),
        ("phi", ConstGroup::Math),
        ("phi_0", ConstGroup::Physics),
        ("pi", ConstGroup::Math),
        ("q_e", ConstGroup::Physics),
        ("r_e", ConstGroup::Physics),
        ("r_earth", ConstGroup::Astronomy),
        ("r_gas", ConstGroup::Chemistry),
        ("r_inf", ConstGroup::Physics),
        ("r_moon", ConstGroup::Astronomy),
        ("r_sun", ConstGroup::Astronomy),
        ("sigma_sb", ConstGroup::Astronomy),
        ("t_P", ConstGroup::Physics),
        ("tau", ConstGroup::Math),
        ("wien", ConstGroup::Physics),
        ("z_0", ConstGroup::Physics),
    ]
}

/// What kind of thing a builtin catalog entry names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogKind {
    Function,
    Constant,
}

// --- data platform: statistics, distributions, and tests (ADR-0044) --

/// The median of a sorted slice (the `mean`-style middle for the
/// five-number summary and quartiles).
pub(crate) fn median_sorted(xs: &[f64]) -> f64 {
    let n = xs.len();
    if n % 2 == 1 {
        xs[n / 2]
    } else {
        (xs[n / 2 - 1] + xs[n / 2]) / 2.0
    }
}

/// The k-th quartile, TI-style median-of-halves (ADR-0044): Q1 is the
/// median of the lower half, Q3 of the upper; the halves include the
/// middle element when the count is odd.
pub(crate) fn quartile_sorted(xs: &[f64], k: u32) -> f64 {
    let n = xs.len();
    let (lo, hi) = if n % 2 == 1 {
        (n / 2 + 1, n / 2)
    } else {
        (n / 2, n / 2)
    };
    match k {
        1 => median_sorted(&xs[..lo]),
        3 => median_sorted(&xs[hi..]),
        _ => median_sorted(xs),
    }
}

/// The least-squares fit of `ys ~ a*x + b` plus Pearson's r (ADR-0044).
/// Two or more points, at least two distinct x values. Returns (a, b, r).
pub(crate) fn linear_fit(xs: &[f64], ys: &[f64]) -> Result<(f64, f64, f64), EpherError> {
    let n = xs.len();
    if n < 2 {
        return Err(domain_error("linear fit needs at least 2 points"));
    }
    let mx = xs.iter().sum::<f64>() / n as f64;
    let my = ys.iter().sum::<f64>() / n as f64;
    let mut sxx = 0.0;
    let mut sxy = 0.0;
    let mut syy = 0.0;
    for (x, y) in xs.iter().zip(ys.iter()) {
        let dx = x - mx;
        let dy = y - my;
        sxx += dx * dx;
        sxy += dx * dy;
        syy += dy * dy;
    }
    if sxx == 0.0 {
        return Err(domain_error(
            "linear fit needs at least 2 distinct x values",
        ));
    }
    let a = sxy / sxx;
    let b = my - a * mx;
    let r = if syy == 0.0 {
        // every y equal: a horizontal line fits perfectly
        1.0
    } else {
        (sxy / (sxx * syy).sqrt()).clamp(-1.0, 1.0)
    };
    Ok((a, b, r))
}

/// The regression family (ADR-0054): the models real calculators fit
/// on a list pair. `Exponential` fits `y = a·e^(bx)` (y > 0), `Power`
/// fits `y = a·x^b` (x, y > 0), `Logarithmic` fits `y = a + b·ln(x)`
/// (x > 0), each through a linear fit on the transformed pair, and r
/// is the correlation of that linearized fit, which is what TI and
/// NumWorks report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitKind {
    Linear,
    Quadratic,
    Exponential,
    Power,
    Logarithmic,
}

/// A fitted model: the coefficients (`c` only for the quadratic) and
/// the reported r. `Fit::eval` draws the overlay; `Fit::caption` is
/// the display string the `*reg` builtins return.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fit {
    pub kind: FitKind,
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub r: f64,
}

impl Fit {
    /// The model's value at x. Outside the domain of a transformed
    /// model (`Power`/`Logarithmic` at x <= 0) the value is NaN, which
    /// renderers skip like any other hole.
    pub fn eval(&self, x: f64) -> f64 {
        match self.kind {
            FitKind::Linear => self.a * x + self.b,
            FitKind::Quadratic => (self.a * x + self.b) * x + self.c,
            FitKind::Exponential => self.a * (self.b * x).exp(),
            FitKind::Power => {
                if x > 0.0 {
                    self.a * x.powf(self.b)
                } else {
                    f64::NAN
                }
            }
            FitKind::Logarithmic => {
                if x > 0.0 {
                    self.a + self.b * x.ln()
                } else {
                    f64::NAN
                }
            }
        }
    }

    /// The display string: the same `y = a*x + b (r = r)` spelling
    /// `linreg` has always returned, extended per model.
    pub fn caption(&self) -> String {
        match self.kind {
            FitKind::Linear => {
                format!("y = {}*x + {} (r = {})", stat_str(self.a), stat_str(self.b), stat_str(self.r))
            }
            FitKind::Quadratic => format!(
                "y = {}*x^2 + {}*x + {} (r = {})",
                stat_str(self.a),
                stat_str(self.b),
                stat_str(self.c),
                stat_str(self.r)
            ),
            FitKind::Exponential => format!(
                "y = {}*e^({}*x) (r = {})",
                stat_str(self.a),
                stat_str(self.b),
                stat_str(self.r)
            ),
            FitKind::Power => {
                format!("y = {}*x^{} (r = {})", stat_str(self.a), stat_str(self.b), stat_str(self.r))
            }
            FitKind::Logarithmic => format!(
                "y = {} + {}*ln(x) (r = {})",
                stat_str(self.a),
                stat_str(self.b),
                stat_str(self.r)
            ),
        }
    }
}

/// Fit one of the regression models to a list pair (ADR-0054).
pub fn fit_regression(kind: FitKind, xs: &[f64], ys: &[f64]) -> Result<Fit, EpherError> {
    if xs.len() != ys.len() {
        return Err(EpherError::Type(format!(
            "the fit needs two same-length lists, got {} and {}",
            xs.len(),
            ys.len()
        )));
    }
    match kind {
        FitKind::Linear => {
            let (a, b, r) = linear_fit(xs, ys)?;
            Ok(Fit { kind, a, b, c: 0.0, r })
        }
        FitKind::Quadratic => {
            if xs.len() < 3 {
                return Err(domain_error("quadratic fit needs at least 3 points"));
            }
            // The normal equations of the Vandermonde design [x² x 1]:
            // solve the 3×3 Gram system for [a, b, c].
            let (mut s0, mut s1, mut s2, mut s3, mut s4) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
            let (mut t0, mut t1, mut t2) = (0.0f64, 0.0f64, 0.0f64);
            for (x, y) in xs.iter().zip(ys.iter()) {
                let x2 = x * x;
                s0 += 1.0;
                s1 += x;
                s2 += x2;
                s3 += x2 * x;
                s4 += x2 * x2;
                t0 += y;
                t1 += x * y;
                t2 += x2 * y;
            }
            let Some([a, b, c]) = solve3(
                [[s4, s3, s2], [s3, s2, s1], [s2, s1, s0]],
                [t2, t1, t0],
            ) else {
                return Err(domain_error(
                    "quadratic fit needs at least 3 distinct x values",
                ));
            };
            let fitted: Vec<f64> = xs.iter().map(|&x| (a * x + b) * x + c).collect();
            let r = pearson(ys, &fitted);
            Ok(Fit { kind, a, b, c, r })
        }
        FitKind::Exponential => {
            if xs.len() < 2 {
                return Err(domain_error("exponential fit needs at least 2 points"));
            }
            if ys.iter().any(|&y| y <= 0.0) {
                return Err(domain_error(
                    "exponential fit needs y > 0 (the model is y = a*e^(b*x))",
                ));
            }
            let ly: Vec<f64> = ys.iter().map(|y| y.ln()).collect();
            let (b, ln_a, r) = linear_fit(xs, &ly)?;
            if ln_a > 700.0 {
                return Err(domain_error("the exponential fit overflows"));
            }
            Ok(Fit { kind, a: ln_a.exp(), b, c: 0.0, r })
        }
        FitKind::Power => {
            if xs.len() < 2 {
                return Err(domain_error("power fit needs at least 2 points"));
            }
            if xs.iter().any(|&x| x <= 0.0) || ys.iter().any(|&y| y <= 0.0) {
                return Err(domain_error(
                    "power fit needs x > 0 and y > 0 (the model is y = a*x^b)",
                ));
            }
            let lx: Vec<f64> = xs.iter().map(|x| x.ln()).collect();
            let ly: Vec<f64> = ys.iter().map(|y| y.ln()).collect();
            let (b, ln_a, r) = linear_fit(&lx, &ly)?;
            if ln_a > 700.0 {
                return Err(domain_error("the power fit overflows"));
            }
            Ok(Fit { kind, a: ln_a.exp(), b, c: 0.0, r })
        }
        FitKind::Logarithmic => {
            if xs.len() < 2 {
                return Err(domain_error("logarithmic fit needs at least 2 points"));
            }
            if xs.iter().any(|&x| x <= 0.0) {
                return Err(domain_error(
                    "logarithmic fit needs x > 0 (the model is y = a + b*ln(x))",
                ));
            }
            let lx: Vec<f64> = xs.iter().map(|x| x.ln()).collect();
            let (b, a, r) = linear_fit(&lx, ys)?;
            Ok(Fit { kind, a, b, c: 0.0, r })
        }
    }
}

/// The Pearson correlation of an already-transformed pair, with the
/// degenerate conventions `linear_fit` uses: a constant side fits
/// perfectly.
fn pearson(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len() as f64;
    let mx = xs.iter().sum::<f64>() / n;
    let my = ys.iter().sum::<f64>() / n;
    let (mut sxx, mut sxy, mut syy) = (0.0f64, 0.0f64, 0.0f64);
    for (x, y) in xs.iter().zip(ys.iter()) {
        let dx = x - mx;
        let dy = y - my;
        sxx += dx * dx;
        sxy += dx * dy;
        syy += dy * dy;
    }
    if sxx == 0.0 || syy == 0.0 {
        return 1.0;
    }
    (sxy / (sxx * syy).sqrt()).clamp(-1.0, 1.0)
}

/// Solve a 3×3 system by Gaussian elimination with partial pivoting.
/// Own code by the reuse ladder's last resort (ADR-0054): a plain-float
/// 3×3 solve is ~20 lines, and pulling in a linear-algebra crate for it
/// (nalgebra, statrs) would out-weigh the feature.
fn solve3(m: [[f64; 3]; 3], v: [f64; 3]) -> Option<[f64; 3]> {
    let mut a = m;
    let mut b = v;
    for col in 0..3 {
        let pivot = (col..3).max_by(|&i, &j| a[i][col].total_cmp(&a[j][col]))?;
        if a[pivot][col] == 0.0 {
            return None; // singular: duplicate x values
        }
        a.swap(col, pivot);
        b.swap(col, pivot);
        for row in (col + 1)..3 {
            let f = a[row][col] / a[col][col];
            // The pivot row is copied first: rows below the pivot never
            // overlap it, and a plain double index would fight the borrow
            // checker on the way to the iterator form.
            let pivot = a[col];
            for (target, &p) in a[row].iter_mut().zip(pivot.iter()).skip(col) {
                *target -= f * p;
            }
            b[row] -= f * b[col];
        }
    }
    let mut x = [0.0f64; 3];
    for row in (0..3).rev() {
        let sum: f64 = (row + 1..3).map(|k| a[row][k] * x[k]).sum();
        x[row] = (b[row] - sum) / a[row][row];
    }
    Some(x)
}

/// The F distribution CDF through the incomplete beta (ADR-0054):
/// P(F <= f; d1, d2) = I_x(d1/2, d2/2) at x = d1·f / (d1·f + d2).
fn f_cdf(f: f64, d1: f64, d2: f64) -> f64 {
    if f <= 0.0 {
        return 0.0;
    }
    let x = d1 * f / (d1 * f + d2);
    regularized_beta(d1 / 2.0, d2 / 2.0, x)
}

/// Extract the `(xs, ys)` pair the regression builtins fit: two
/// same-length lists of numbers.
fn reg_pair(name: &str, args: &[Value]) -> Result<(Vec<f64>, Vec<f64>), EpherError> {
    let to = |items: &Value| -> Result<Vec<f64>, EpherError> {
        let Value::List(items) = items else {
            return Err(EpherError::Type(format!(
                "{name} expects two same-length lists, got {} argument(s)",
                args.len()
            )));
        };
        let mut out = Vec::with_capacity(items.len());
        for item in items {
            match item {
                Value::Float(x) => out.push(*x),
                other => {
                    return Err(EpherError::Type(format!(
                        "{name} expects numbers, got {other:?}"
                    )))
                }
            }
        }
        Ok(out)
    };
    match args {
        [a, b] if matches!(a, Value::List(_)) && matches!(b, Value::List(_)) => {
            let xs = to(a)?;
            let ys = to(b)?;
            if xs.len() != ys.len() {
                return Err(EpherError::Type(format!(
                    "{name} lists have different lengths: {} and {}",
                    xs.len(),
                    ys.len()
                )));
            }
            Ok((xs, ys))
        }
        _ => Err(EpherError::Type(format!(
            "{name} expects two same-length lists, got {} argument(s)",
            args.len()
        ))),
    }
}

/// The regularized upper incomplete gamma Q(a, x) = 1 - P(a, x), by
/// puruspe's NR-style
/// `gammq` (continued fraction below the switch, Gauss-Legendre
/// quadrature for large a), direct in the tail so extreme survivors
/// keep their digits (ADR-0052). Pure f64, deterministic, wasm-safe.
fn regularized_gamma_q(a: f64, x: f64) -> f64 {
    puruspe::gammq(a, x)
}

/// Lanczos ln(gamma) — puruspe's double-precision implementation
/// (Fukushima-class, ~eps relative).
fn ln_gamma(x: f64) -> f64 {
    puruspe::ln_gamma(x)
}

/// The regularized incomplete beta I_x(a, b) — puruspe's NR-style
/// `betai` — for a, b > 0 and x clamped into [0, 1] (the crate's
/// betai asserts on its domain, so the clamp lives here).
fn regularized_beta(a: f64, b: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    puruspe::betai(a, b, x)
}

/// The standard normal CDF through the complementary error function:
/// norm_cdf(x) = 0.5*erfc(-x/sqrt(2)), puruspe's erfcx-based erfc,
/// which keeps relative accuracy deep into the tails (down to the
/// underflow floor) instead of cancelling 1 - tiny. Exact at 0. The
/// inverse CDF is Acklam's rational approximation (1.15e-9) polished
/// by Newton steps against the tail-space survivor.
fn norm_cdf(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    if x == 0.0 {
        return 0.5;
    }
    // 0.5*erfc(-x/sqrt(2)): erfc is accurate on both sides, no
    // cancellation against 1 - tiny.
    0.5 * puruspe::erfc(-x / std::f64::consts::SQRT_2)
}

fn inv_norm(p: f64) -> f64 {
    const A: [f64; 6] = [
        -3.969683028665376e1,
        2.209460984245205e2,
        -2.759285104469687e2,
        1.383577518672690e2,
        -3.066479806614716e1,
        2.506628277459239,
    ];
    const B: [f64; 5] = [
        -5.447609879822406e1,
        1.615858368580409e2,
        -1.556989798598866e2,
        6.680131188771972e1,
        -1.328068155288572e1,
    ];
    const C: [f64; 6] = [
        -7.784894002430293e-3,
        -3.223964580411365e-1,
        -2.400758277161838,
        -2.549732539343734,
        4.374664141464968,
        2.938163982698783,
    ];
    const D: [f64; 4] = [
        7.784695709041462e-3,
        3.224671290700398e-1,
        2.445134137142996,
        3.754408661907416,
    ];
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    // Acklam's rational approximation (1.15e-9 class), polished by
    // Newton against the gamma-based tail below. The polish runs in
    // tail space (ADR-0052): g(x) = 0.5*q(0.5, x^2/2) is the survivor
    // for both signs, so the target is min(p, 1-p). Polishing against
    // norm_cdf(x) = 1 - 0.5*q would lose the tail's digits once the
    // CDF saturates toward 1, which stalled extreme quantiles (p ~
    // 1 - 1e-10 and beyond) at 1e-8 relative error.
    let x = if p < 0.02425 {
        let q = (-2.0 * p.ln()).sqrt();
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    } else if p > 0.97575 {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    } else {
        let q = p - 0.5;
        let r = q * q;
        q * (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5])
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    };
    if p == 0.5 {
        return 0.0;
    }
    let target = if p > 0.5 { 1.0 - p } else { p };
    let mut x = x;
    for _ in 0..6 {
        // g(x) = 0.5*q(0.5, x^2/2) is even, so the derivative is
        // -sign(x)*pdf(x) and the Newton step is
        // x += (g - target) / (sign(x) * pdf(x)).
        let g = 0.5 * puruspe::erfc(x.abs() / std::f64::consts::SQRT_2);
        let denom = norm_pdf(x) * x.signum();
        if denom == 0.0 {
            break;
        }
        let step = (g - target) / denom;
        if step.abs() <= 5e-15 * x.abs().max(1.0) {
            break;
        }
        x += step;
    }
    x
}

fn norm_pdf(x: f64) -> f64 {
    (-0.5 * x * x).exp() / std::f64::consts::TAU.sqrt()
}

/// The Student t CDF (regularized incomplete beta: t_cdf = 1 - 0.5 *
/// I_{df/(df+t^2)}(df/2, 1/2)) and PDF; `invt` inverts by Newton on
/// the CDF with the PDF as the derivative.
fn t_cdf(t: f64, df: f64) -> f64 {
    if t.is_nan() {
        return f64::NAN;
    }
    let x = df / (df + t * t);
    let ib = regularized_beta(df / 2.0, 0.5, x);
    if t >= 0.0 {
        1.0 - 0.5 * ib
    } else {
        0.5 * ib
    }
}

fn t_pdf(t: f64, df: f64) -> f64 {
    (ln_gamma((df + 1.0) / 2.0)
        - ln_gamma(df / 2.0)
        - 0.5 * (df * std::f64::consts::PI).ln()
        - ((df + 1.0) / 2.0) * (1.0 + t * t / df).ln())
    .exp()
}

/// 1 - t_cdf(t) computed without cancellation: the upper tail is
/// 0.5*I for positive t, and 1 - 0.5*I for negative t (where the CDF
/// itself is the tiny 0.5*I). The survivor is what extreme upper
/// quantiles invert against (ADR-0052).
fn t_survivor(t: f64, df: f64) -> f64 {
    let ib = regularized_beta(df / 2.0, 0.5, df / (df + t * t));
    if t >= 0.0 {
        0.5 * ib
    } else {
        1.0 - 0.5 * ib
    }
}

/// The chi-squared CDF, the lower probability P(X <= x) — the
/// regularized incomplete gamma P(df/2, x/2); `invchi2` inverts by
/// Newton with the PDF as the derivative.
fn chi2_cdf(x: f64, df: f64) -> f64 {
    1.0 - regularized_gamma_q(df / 2.0, x / 2.0)
}

fn chi2_pdf(x: f64, df: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    ((df / 2.0 - 1.0) * x.ln() - x / 2.0 - ln_gamma(df / 2.0) - (df / 2.0) * 2.0f64.ln()).exp()
}

/// Newton inversion of a monotone CDF with a PDF derivative, bracketed
/// by doubling (so p values near 0 or 1 still converge). The `survivor`
/// is 1 - cdf computed without cancellation (the tail), used for p >
/// 0.5: inverting the CDF directly loses the last digits once it
/// saturates toward 1, which stalled extreme quantiles at 1e-8..1e-12
/// relative error and clamped invt at the caller's bracket (ADR-0052).
/// Convergence is measured on the Newton step relative to |x|, not on
/// the CDF residual, whose size varies by orders of magnitude across
/// the tails. Degenerate probabilities (0 or 1) return the bracket
/// edge, exactly as the old bisection floor did.
fn invert_cdf(
    cdf: impl Fn(f64) -> f64,
    survivor: impl Fn(f64) -> f64,
    pdf: impl Fn(f64) -> f64,
    p: f64,
    lo: f64,
    hi: f64,
) -> f64 {
    if p <= 0.0 {
        return lo;
    }
    if p >= 1.0 {
        return hi;
    }
    let mut lo = lo;
    let mut hi = hi;
    // Straddle the root before iterating: the caller's bracket (e.g.
    // [-100, 100] for t) may lie entirely below an extreme quantile.
    if p <= 0.5 {
        for _ in 0..64 {
            if cdf(lo) >= p {
                lo = 2.0 * lo - hi;
            } else {
                break;
            }
        }
        for _ in 0..64 {
            if cdf(hi) <= p {
                hi = 2.0 * hi - lo;
            } else {
                break;
            }
        }
    } else {
        let q = 1.0 - p;
        for _ in 0..64 {
            if survivor(lo) <= q {
                lo = 2.0 * lo - hi;
            } else {
                break;
            }
        }
        for _ in 0..64 {
            if survivor(hi) >= q {
                hi = 2.0 * hi - lo;
            } else {
                break;
            }
        }
    }
    let mut x = 0.5 * (lo + hi);
    for _ in 0..200 {
        let target = if p <= 0.5 { p } else { 1.0 - p };
        let g = if p <= 0.5 { cdf(x) } else { survivor(x) };
        // f is the residual of the increasing-equivalent form: zero at
        // the root, positive below it (the survivor path flips sign).
        let f = if p <= 0.5 { g - target } else { target - g };
        if f == 0.0 {
            break;
        }
        if f < 0.0 {
            lo = x;
        } else {
            hi = x;
        }
        let d = pdf(x);
        let step = if d > 1e-300 { f / d } else { 0.0 };
        if step.abs() <= 5e-14 * x.abs().max(1.0) {
            break;
        }
        let nx = x - step;
        x = if nx > lo && nx < hi {
            nx
        } else {
            0.5 * (lo + hi)
        };
    }
    x
}

/// The binomial PDF by the numerically stable recurrence (p^i q^(n-i)
/// built up from the peak) and the Poisson PDF by the log-gamma form.
fn binom_pdf(k: i64, n: i64, p: f64) -> f64 {
    if k < 0 || k > n {
        return 0.0;
    }
    if p == 0.0 {
        return if k == 0 { 1.0 } else { 0.0 };
    }
    if p == 1.0 {
        return if k == n { 1.0 } else { 0.0 };
    }
    let log = ln_gamma(n as f64 + 1.0) - ln_gamma(k as f64 + 1.0) - ln_gamma((n - k) as f64 + 1.0)
        + k as f64 * p.ln()
        + (n - k) as f64 * (1.0 - p).ln();
    log.exp()
}

fn poisson_pdf(k: i64, lambda: f64) -> f64 {
    if k < 0 {
        return 0.0;
    }
    if lambda == 0.0 {
        return if k == 0 { 1.0 } else { 0.0 };
    }
    (k as f64 * lambda.ln() - lambda - ln_gamma(k as f64 + 1.0)).exp()
}

/// The count-typed argument of the discrete distributions.
fn count_arg(name: &str, v: f64) -> Result<i64, EpherError> {
    float_to_int(v)
        .ok_or_else(|| EpherError::Type(format!("{name} expects a whole number, got {v}")))
}

/// The probability argument of a CDF inverse or a test level.
fn prob_arg(name: &str, v: f64) -> Result<f64, EpherError> {
    if !(0.0..=1.0).contains(&v) {
        return Err(domain_error(format!(
            "{name} expects a probability in 0..1, got {v}"
        )));
    }
    Ok(v)
}

/// A data list as floats, with its length — the tests and intervals
/// take a named column.
fn data_list(name: &str, args: &[Value], arg: usize) -> Result<Vec<f64>, EpherError> {
    let v = args.get(arg).ok_or_else(|| {
        EpherError::Type(format!(
            "{name} expects a data list, got {} argument(s)",
            args.len()
        ))
    })?;
    let Value::List(items) = v else {
        return Err(EpherError::Type(format!(
            "{name} expects a data list, got {v}"
        )));
    };
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        match item {
            Value::Float(x) => out.push(*x),
            other => {
                return Err(EpherError::Type(format!(
                    "{name} expects numbers in the list, got {other:?}"
                )))
            }
        }
    }
    Ok(out)
}

/// The sample mean and the sample (n-1) standard deviation — the
/// building blocks of the tests and intervals.
fn sample_mean_std(data: &[f64]) -> (f64, f64) {
    let n = data.len();
    let mean = data.iter().sum::<f64>() / n as f64;
    let var = data.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / (n - 1) as f64;
    (mean, var.sqrt())
}

/// Two-sided p of a z statistic and a t statistic.
fn z_two_sided(z: f64) -> f64 {
    2.0 * (1.0 - norm_cdf(z.abs()))
}

fn t_two_sided(t: f64, df: f64) -> f64 {
    2.0 * (1.0 - t_cdf(t.abs(), df))
}

/// One rounded stat, for the test result strings (ADR-0044).
fn stat_str(x: f64) -> String {
    format!("{x:.4}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

/// The `(lo, hi)` spelling of a confidence interval.
fn interval_str(lo: f64, hi: f64) -> String {
    format!("({}, {})", stat_str(lo), stat_str(hi))
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
        name: "amort",
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
        name: "binomcdf",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "binompdf",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "bits",
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
        name: "chi2cdf",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "chi2pdf",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "chisq_gof",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "compound_interest",
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
        name: "det",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "dim",
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
        name: "inv",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "invchi2",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "invnorm",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "invt",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "irr",
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
        name: "l_P",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "l_sun",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "lambda_c",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "lcm",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "len",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "linreg",
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
        name: "m_P",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "m_e",
        kind: CatalogKind::Constant,
    },
    CatalogEntry {
        name: "m_moon",
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
        name: "mode",
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
        name: "mu_n",
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
        name: "normcdf",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "normpdf",
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
        name: "npv",
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
        name: "poissoncdf",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "poissonpdf",
        kind: CatalogKind::Function,
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
        name: "quartile",
        kind: CatalogKind::Function,
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
        name: "r_moon",
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
        name: "randint",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "random",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "randseed",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "range",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "re",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "ref",
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
        name: "rref",
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
        name: "simple_interest",
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
        name: "sort",
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
        name: "str",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "sum",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "t_P",
        kind: CatalogKind::Constant,
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
        name: "tcdf",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "tinterval",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "totient",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "tpdf",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "trace",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "transpose",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "trunc",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "ttest",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "tvm_fv",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "tvm_i",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "tvm_n",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "tvm_pmt",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "tvm_pv",
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
    CatalogEntry {
        name: "zinterval",
        kind: CatalogKind::Function,
    },
    CatalogEntry {
        name: "ztest",
        kind: CatalogKind::Function,
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
        // A quantity's SI value (ADR-0046): the numeric builtins
        // consume the base-unit value, SpeedCrunch-style.
        [Value::Quantity { value, .. }] => Ok(*value),
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
        // A quantity computes with its SI value (ADR-0046).
        Value::Quantity { value, .. } => match real(*value) {
            Ok(y) => Ok(Value::Float(y)),
            Err(EpherError::Domain(_)) => Ok(Value::Complex(complex(Complex::new(*value, 0.0)))),
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

/// Take exactly four Float arguments (quantities unwrap).
fn four_floats(name: &str, args: &[Value]) -> Result<(f64, f64, f64, f64), EpherError> {
    match args {
        [a, b, c, d] => Ok((
            one_float(name, &[a.clone()])?,
            one_float(name, &[b.clone()])?,
            one_float(name, &[c.clone()])?,
            one_float(name, &[d.clone()])?,
        )),
        _ => Err(EpherError::Type(format!(
            "{name} expects 4 numbers, got {} argument(s)",
            args.len()
        ))),
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
        // Quantities unwrap to their SI values (ADR-0046).
        [a, b] => Ok((
            one_float(name, &[a.clone()])?,
            one_float(name, &[b.clone()])?,
        )),
        _ => Err(EpherError::Type(format!(
            "{name} expects 2 numbers, got {} argument(s)",
            args.len()
        ))),
    }
}

/// One or more Float arguments, or a single list of them (ADR-0044):
/// `mean({1, 2, 3})` and `mean(1, 2, 3)` are the same call, and
/// `linreg({..}, {..})` feeds the stats machinery. An empty list is a
/// domain error for statistics that divide by the count.
fn any_floats(name: &str, args: &[Value]) -> Result<Vec<f64>, EpherError> {
    if args.is_empty() {
        return Err(EpherError::Type(format!(
            "{name} expects at least 1 number, got 0"
        )));
    }
    if args.len() == 1 {
        if let Value::List(items) = &args[0] {
            if items.is_empty() {
                return Err(domain_error(format!("{name} of an empty list")));
            }
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                match item {
                    Value::Float(x) => out.push(*x),
                    other => {
                        return Err(EpherError::Type(format!(
                            "{name} expects numbers, got {other:?}"
                        )))
                    }
                }
            }
            return Ok(out);
        }
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
            // Relative tolerance: an absolute test would call every
            // tiny value (m_e, h, epsilon_0) a perfect "0" within tol.
            if (guess - a).abs() <= tol * a.abs() {
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

/// Result notation (ADR-0043/0051): Auto is the float rounded to
/// twelve significant digits (exact integers keep every digit); the
/// other two force their exponent shape.
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

/// The Auto-mode float spelling (ADR-0051): the shortest round-trip
/// decimal, rounded to twelve significant digits when that shortens
/// it — the reference-calculator convention that turns the float
/// 0.30000000000000004 into "0.3". Exact integers never round: their
/// rounded spelling is no shorter, so the guard keeps the original.
/// The rounding works on the decimal spelling, not the float: scaling
/// a double and back can land on a value whose shortest spelling
/// still needs more digits (5.551115123125783e-17 comes back as
/// 5.551115123130001e-17, not a clean twelve).
pub fn auto_float(x: f64) -> String {
    let s = format!("{x}");
    if significant_digits(&s) <= 12 {
        return s;
    }
    let (sign, body) = match s.strip_prefix('-') {
        Some(b) => ("-", b),
        None => ("", s.as_str()),
    };
    let (int_part, frac_part) = match body.split_once('.') {
        Some((i, f)) => (i, f),
        None => (body, ""),
    };
    let int_len = int_part.len();
    let mut digits: Vec<char> = int_part.chars().chain(frac_part.chars()).collect();
    // The exponent of the first significant digit; leading zeros then
    // drop off so the vector holds exactly the significant digits.
    let dpos = digits
        .iter()
        .position(|c| *c != '0')
        .unwrap_or(digits.len());
    let mut k = int_len as i64 - 1 - dpos as i64;
    digits.drain(..dpos);
    if digits.len() > 12 {
        let rest = digits.split_off(12);
        if rest[0] >= '5' {
            // Round the twelfth digit up, propagating the carry; a
            // carry past the front shifts the first digit's exponent.
            let mut i = digits.len();
            loop {
                if i == 0 {
                    digits.insert(0, '1');
                    break;
                }
                i -= 1;
                if digits[i] == '9' {
                    digits[i] = '0';
                } else {
                    digits[i] = (digits[i] as u8 + 1) as char;
                    break;
                }
            }
            if digits.len() > 12 {
                // 999… carried to 1000…: one more leading digit, so
                // the first significant digit moved one place up.
                digits.pop();
                k += 1;
            }
        }
    }
    // Trailing zeros after the decimal point are spelling noise.
    while digits.len() > 1 && k - digits.len() as i64 + 1 < 0 && *digits.last().unwrap() == '0' {
        digits.pop();
    }
    render_rounded(sign, &digits, k, &s)
}

/// Re-emit the rounded significant digits as a plain decimal and keep
/// whichever spelling is shorter (the guard that protects exact
/// integers: rounding 1234567890123456 gives the same length).
fn render_rounded(sign: &str, digits: &[char], k: i64, s: &str) -> String {
    let digits: String = digits.iter().collect();
    let out = if k >= 0 {
        if (digits.len() as i64) > k + 1 {
            let at = (k + 1) as usize;
            format!("{}.{}", &digits[..at], &digits[at..])
        } else {
            format!(
                "{digits}{}",
                "0".repeat((k + 1 - digits.len() as i64) as usize)
            )
        }
    } else {
        format!("0.{}{}", "0".repeat((-k - 1) as usize), digits)
    };
    let out = format!("{sign}{out}");
    if out.len() < s.len() {
        out
    } else {
        s.to_string()
    }
}

/// Count the significant digits of a plain-decimal float spelling
/// ("-0.30000000000000004" has 17, "1234567890123" has 13, "0.0001"
/// has 1). Rust's Display always spells floats without an exponent.
fn significant_digits(s: &str) -> usize {
    let mut seen = false;
    let mut count = 0;
    for ch in s.chars() {
        if ch.is_ascii_digit() && (seen || ch != '0') {
            seen = true;
            count += 1;
        }
    }
    count
}

/// Whether the reduced fraction has a finite decimal expansion — the
/// denominator holds only the factors 2 and 5. Reconstructed
/// denominators are at most 1000, so the divisibility loop is short.
/// A terminating fraction displays as a decimal (0.3, 0.125); only a
/// repeating value keeps the fraction spelling (ADR-0051).
pub fn terminating_decimal(r: &BigRational) -> bool {
    let mut d = r.denom().clone();
    if d.sign() == num_bigint::Sign::Minus {
        d = -d;
    }
    let two = BigInt::from(2);
    let five = BigInt::from(5);
    loop {
        if d == BigInt::from(1) {
            return true;
        }
        if (&d % &two).is_zero() {
            d /= &two;
        } else if (&d % &five).is_zero() {
            d /= &five;
        } else {
            return false;
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
                        // Half a display unit (5e-13 relative): a
                        // fraction shows only when it agrees with the
                        // value through all twelve displayed digits.
                        // The old 1e-9 tolerance let large decimals
                        // with a coincidental convergent show as
                        // fractions (123456.789 became 13456790/109).
                        if let Some(r) = reconstruct_fraction(*x, 1000, 5e-13) {
                            // A terminating decimal shows as a decimal
                            // (0.1 + 0.2 is 0.3, not 3/10); only a
                            // repeating value keeps the fraction.
                            if !terminating_decimal(&r) {
                                return format!("{r}");
                            }
                        }
                    }
                    auto_float(*x)
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
        other => match other {
            Value::List(items) => {
                let inner = items
                    .iter()
                    .map(|v| format_value(v, prefs))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{{inner}}}")
            }
            Value::Matrix { rows, cols, data } => {
                let mut out = String::from("[");
                for r in 0..*rows {
                    if r > 0 {
                        out.push_str(", ");
                    }
                    out.push('[');
                    for c in 0..*cols {
                        if c > 0 {
                            out.push_str(", ");
                        }
                        out.push_str(&format_value(
                            &Value::Float(data[r * cols + c] + 0.0),
                            prefs,
                        ));
                    }
                    out.push(']');
                }
                out.push(']');
                out
            }
            // A quantity (ADR-0046): the SI value converted to its
            // display unit, formatted with the session's prefs. The
            // unit text itself is not reformatted. The value goes
            // through the same twelve-digit rounding as every other
            // result line (ADR-0052): `30 deg in rad` shows
            // 0.523598775598, not the raw 16-digit spelling.
            Value::Quantity { value, dims, unit } => {
                let shown = if *dims == [0; 7] {
                    // Dimensionless (angles, plain conversions): the SI
                    // value, no unit text.
                    auto_float(*value)
                } else {
                    match unit {
                        Some((name, factor)) => format!("{} {name}", auto_float(value / factor)),
                        None => format!("{} {}", auto_float(*value), si_unit_str(*dims)),
                    }
                };
                let shown = if prefs.separators {
                    let (head, tail) = match shown.split_once(' ') {
                        Some((h, t)) => (h, Some(t)),
                        None => (shown.as_str(), None),
                    };
                    let mut out = grouped_str(head);
                    if let Some(t) = tail {
                        out.push(' ');
                        out.push_str(t);
                    }
                    out
                } else {
                    shown
                };
                shown
            }
            _ => format!("{other}"),
        },
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
        // A string literal is not a polynomial (ADR-0054).
        Expression::StrLit(_) => None,
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
        Expression::Unit(inner, factor, _, _) => {
            poly_coeffs(inner, variable, env).map(|c| c.into_iter().map(|x| x * factor).collect())
        }
        Expression::In(inner, factor, _, _) => {
            poly_coeffs(inner, variable, env).map(|c| c.into_iter().map(|x| x / factor).collect())
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
        Expression::Matrix(rows) => {
            for row in rows {
                for item in row {
                    poly_coeffs(item, variable, env)?;
                }
            }
            None
        }
        Expression::Factorial(_)
        | Expression::Compare(_, _, _)
        | Expression::BitAnd(_, _)
        | Expression::BitOr(_, _)
        | Expression::BitXor(_, _)
        | Expression::ShiftLeft(_, _)
        | Expression::ShiftRight(_, _)
        | Expression::BitNot(_)
        | Expression::If(_, _, _)
        | Expression::And(_, _)
        | Expression::Or(_, _)
        | Expression::Not(_)
        | Expression::List(_)
        | Expression::Index(_, _) => None,
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

/// The TVM solver (ADR-0050): the five functions solve one field of
/// the time-value equation
/// `pv*(1+i)^n + pmt*(1+i*begin)*((1+i)^n - 1)/i + fv = 0`
/// (TI sign convention: money out negative, in positive; `i` is the
/// per-period fraction, 0.01 = 1%; `begin` = 1 for annuity-due
/// payments). The linear fields have closed forms; n and i bisect the
/// factorized balance, which stays finite where the expanded form
/// overflows.
fn eval_tvm(name: &str, args: &[Value]) -> Result<Value, EpherError> {
    let timed = |n: f64, i: f64, pv: f64, pmt: f64, fv: f64, begin: f64| -> f64 {
        if i == 0.0 {
            pv + pmt * n + fv
        } else {
            let g = (1.0 + i).powf(n);
            let b = pmt * (1.0 + i * begin) / i;
            (pv + b) * g - b + fv
        }
    };
    // parse the fields and the optional timing
    let (begin, rest): (f64, &[Value]) = match args {
        [_, _, _, _] => (0.0, args),
        [_, _, _, _, e] => {
            let begin = one_float(name, &[e.clone()])?;
            if begin != 0.0 && begin != 1.0 {
                return Err(domain_error(format!(
                    "the payment timing is 0 (end) or 1 (beginning), got {begin}"
                )));
            }
            (begin, &args[..4])
        }
        _ => {
            return Err(EpherError::Type(format!(
                "{name} expects 4 arguments (plus an optional 0/1 timing), got {}",
                args.len()
            )))
        }
    };
    let nums = |i: usize| -> f64 { one_float(name, &[rest[i].clone()]).expect("float") };
    let (n, i, pv, pmt, fv) = match name {
        "tvm_n" => {
            // the unknown n: bisect over [0, 1e7]
            let (i, pv, pmt, fv) = (nums(0), nums(1), nums(2), nums(3));
            let f = |n: f64| timed(n, i, pv, pmt, fv, begin);
            (bisect_n(name, f)?, i, pv, pmt, fv)
        }
        "tvm_i" => {
            let (n, pv, pmt, fv) = (nums(0), nums(1), nums(2), nums(3));
            let f = |i: f64| timed(n, i, pv, pmt, fv, begin);
            (n, bisect_rate(name, f, -0.999_999, 1.0)?, pv, pmt, fv)
        }
        "tvm_pv" => {
            let (n, i, pmt, fv) = (nums(0), nums(1), nums(2), nums(3));
            let pv = if i == 0.0 {
                -(pmt * n + fv)
            } else {
                let g = (1.0 + i).powf(n);
                -(pmt * (1.0 + i * begin) * (g - 1.0) / i + fv) / g
            };
            (n, i, pv, pmt, fv)
        }
        "tvm_fv" => {
            let (n, i, pv, pmt) = (nums(0), nums(1), nums(2), nums(3));
            let fv = if i == 0.0 {
                -(pv + pmt * n)
            } else {
                let g = (1.0 + i).powf(n);
                -(pv * g + pmt * (1.0 + i * begin) * (g - 1.0) / i)
            };
            (n, i, pv, pmt, fv)
        }
        _ => {
            let (n, i, pv, fv) = (nums(0), nums(1), nums(2), nums(3));
            let pmt = if i == 0.0 {
                -(pv + fv) / n
            } else {
                let g = (1.0 + i).powf(n);
                -(pv * g + fv) * i / ((1.0 + i * begin) * (g - 1.0))
            };
            (n, i, pv, pmt, fv)
        }
    };
    let value = match name {
        "tvm_n" => n,
        "tvm_i" => i,
        "tvm_pv" => pv,
        "tvm_pmt" => pmt,
        _ => fv,
    };
    Ok(Value::Float(value))
}

/// Bisect `f` over `lo..=hi` for a sign change; none is a domain error
/// naming the searched range.
fn bisect_rate(name: &str, f: impl Fn(f64) -> f64, lo: f64, hi: f64) -> Result<f64, EpherError> {
    let mut lo = lo;
    let mut hi = hi;
    let mut flo = f(lo);
    let fhi0 = f(hi);
    if flo.is_nan() || fhi0.is_nan() || flo * fhi0 > 0.0 {
        return Err(domain_error(format!(
            "{name}: no solution found between {lo:.4} and {hi:.4}"
        )));
    }
    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        let fm = f(mid);
        if flo * fm <= 0.0 {
            hi = mid;
        } else {
            lo = mid;
            flo = fm;
        }
    }
    Ok((lo + hi) / 2.0)
}

/// Bisect the TVM term over `[0, 1e7]` (the factorized balance stays
/// finite; the rate is fixed).
fn bisect_n(name: &str, f: impl Fn(f64) -> f64) -> Result<f64, EpherError> {
    bisect_rate(name, f, 0.0, 1e7)
}

/// The (rate, flows) pair of npv.
fn finance_rate_and_flows(name: &str, args: &[Value]) -> Result<(f64, Vec<f64>), EpherError> {
    match args {
        [rate, Value::List(items)] => {
            let rate = one_float(name, &[rate.clone()])?;
            let mut flows = Vec::with_capacity(items.len());
            for item in items {
                match item {
                    Value::Float(x) => flows.push(*x),
                    other => {
                        return Err(EpherError::Type(format!(
                            "{name} expects cash flows as numbers, got {other:?}"
                        )))
                    }
                }
            }
            if flows.is_empty() {
                return Err(domain_error(format!("{name} needs at least one cash flow")));
            }
            Ok((rate, flows))
        }
        _ => Err(EpherError::Type(format!(
            "{name} expects a rate and a cash-flow list, like npv(0.1, {{-100, 60, 60}})"
        ))),
    }
}

/// The cash-flow list of irr (a single list argument).
fn finance_flows(name: &str, args: &[Value]) -> Result<Vec<f64>, EpherError> {
    let Value::List(items) = one_arg(name, args)? else {
        return Err(EpherError::Type(format!(
            "{name} expects a cash-flow list, like irr({{-100, 60, 60}})"
        )));
    };
    let mut flows = Vec::with_capacity(items.len());
    for item in items {
        match item {
            Value::Float(x) => flows.push(*x),
            other => {
                return Err(EpherError::Type(format!(
                    "{name} expects cash flows as numbers, got {other:?}"
                )))
            }
        }
    }
    if flows.is_empty() {
        return Err(domain_error(format!("{name} needs at least one cash flow")));
    }
    Ok(flows)
}

/// The matrix argument of the matrix functions (ADR-0049).
fn matrix_arg(name: &str, args: &[Value]) -> Result<(usize, usize, Vec<f64>), EpherError> {
    let Value::Matrix { rows, cols, data } = one_arg(name, args)? else {
        return Err(EpherError::Type(format!("{name} expects a matrix")));
    };
    Ok((*rows, *cols, data.clone()))
}

/// The square-matrix check shared by det, inv, and trace.
fn square_matrix(name: &str, m: &Value) -> Result<(usize, Vec<f64>), EpherError> {
    let Value::Matrix { rows, cols, data } = m else {
        return Err(EpherError::Type(format!("{name} expects a matrix")));
    };
    if rows != cols {
        return Err(domain_error(format!(
            "{name} needs a square matrix, got {rows}x{cols}"
        )));
    }
    Ok((*rows, data.clone()))
}

/// The determinant with partial pivoting (ADR-0049): Gaussian
/// elimination tracking the pivot product and the row-swap sign.
fn det_value(n: usize, mut a: Vec<f64>) -> Result<f64, EpherError> {
    let mut det = 1.0;
    for col in 0..n {
        let pivot = (col..n).max_by(|&i, &j| {
            a[i * n + col]
                .abs()
                .partial_cmp(&a[j * n + col].abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let Some(pivot) = pivot else { break };
        if a[pivot * n + col].abs() < 1e-300 {
            return Ok(0.0);
        }
        if pivot != col {
            for k in 0..n {
                a.swap(pivot * n + k, col * n + k);
            }
            det = -det;
        }
        det *= a[col * n + col];
        for row in (col + 1)..n {
            let factor = a[row * n + col] / a[col * n + col];
            for k in col..n {
                a[row * n + k] -= factor * a[col * n + k];
            }
        }
    }
    Ok(det)
}

/// Gauss-Jordan on the augmented matrix: `rref` is the reduced row
/// echelon form, `ref` stops after the forward pass, and `inv` works
/// on [M | I] (a singular pivot is a domain error).
fn gauss_jordan(
    n: usize,
    m: usize,
    mut a: Vec<f64>,
    reduce_above: bool,
    singular_errors: bool,
) -> Result<Vec<f64>, EpherError> {
    let mut row = 0usize;
    for col in 0..m {
        // For an inverse, only the first n columns can host matrix
        // pivots: a pivot in the augmented part would mask a singular
        // matrix (ADR-0049).
        if singular_errors && col >= n {
            break;
        }
        // the pivot: the largest |value| at or below `row` in this column
        let pivot = (row..n)
            .max_by(|&i, &j| {
                a[i * m + col]
                    .abs()
                    .partial_cmp(&a[j * m + col].abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .filter(|&i| a[i * m + col].abs() > 1e-12);
        let Some(pivot) = pivot else {
            continue; // this column is all zeros below; not a pivot
        };
        if pivot != row {
            for k in 0..m {
                a.swap(pivot * m + k, row * m + k);
            }
        }
        let p = a[row * m + col];
        for k in 0..m {
            a[row * m + k] /= p;
        }
        // eliminate every other row (rref) or only the rows below (ref)
        for r in 0..n {
            if r == row {
                continue;
            }
            if !reduce_above && r < row {
                continue;
            }
            let factor = a[r * m + col];
            if factor == 0.0 {
                continue;
            }
            for k in 0..m {
                a[r * m + k] -= factor * a[row * m + k];
            }
        }
        row += 1;
        if row == n {
            break;
        }
    }
    if singular_errors && row < n.min(m) {
        // a free column with no pivot means the augmented system or the
        // matrix is singular
        return Err(domain_error("the matrix is singular"));
    }
    Ok(a)
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
        // half a display unit of relative tolerance) - exact(0.3333333333333333)
        // is 1/3. Irrationals pass through unchanged: no convergent is
        // good enough, so pi stays decimal.
        "exact" => {
            let v = one_arg(name, &args)?;
            match v {
                Value::Float(x) => Ok(match reconstruct_fraction(*x, 1000, 5e-13) {
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
        "sqrt" => match one_arg(name, &args)? {
            Value::Quantity { value, dims, .. } => {
                if *value < 0.0 {
                    return Err(domain_error(format!("sqrt of negative number {value}")));
                }
                if dims.iter().any(|e| e % 2 != 0) {
                    return Err(dimension_error(&format!(
                        "cannot take the square root of {}: the dimensions do not divide evenly",
                        quantity_display(*value, *dims, None)
                    )));
                }
                let half = dims.map(|e| e / 2);
                Ok(finish_quantity(value.sqrt(), half, None))
            }
            _ => real_or_complex(
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
        },
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
        // List shape and ordering (ADR-0044).
        "len" => {
            match one_arg(name, &args)? {
                Value::List(items) => Ok(Value::Float(items.len() as f64)),
                // String length (ADR-0054): characters, not bytes.
                Value::Str(s) => Ok(Value::Float(s.chars().count() as f64)),
                other => Err(EpherError::Type(format!(
                    "len expects a list or string, got {other:?}"
                ))),
            }
        }
        // Strings in, strings out (ADR-0054): `str` spells one value
        // the way the answer panel would; `print` joins its arguments
        // with spaces; the line a loop collects.
        "str" => {
            let [v] = args.as_slice() else {
                return Err(EpherError::Type(format!(
                    "str expects 1 argument, got {}",
                    args.len()
                )));
            };
            Ok(Value::Str(format_value(v, &DisplayPrefs::default())))
        }
        "print" => Ok(Value::Str(
            args.iter()
                .map(|v| format_value(v, &DisplayPrefs::default()))
                .collect::<Vec<_>>()
                .join(" "),
        )),
        "sort" => {
            let mut xs = any_floats(name, &args)?;
            xs.sort_by(|a, b| a.partial_cmp(b).expect("floats are comparable"));
            Ok(Value::List(xs.into_iter().map(Value::Float).collect()))
        }
        "mode" => {
            let xs = any_floats(name, &args)?;
            if xs.is_empty() {
                return Err(domain_error("mode of an empty list"));
            }
            let mut counts = std::collections::HashMap::<u64, (f64, usize)>::new();
            for x in &xs {
                let bits = x.to_bits();
                let entry = counts.entry(bits).or_insert((*x, 0));
                entry.1 += 1;
            }
            // Most frequent; the smallest value wins ties.
            let best = counts
                .values()
                .max_by(|a, b| {
                    a.1.cmp(&b.1)
                        .then_with(|| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal))
                })
                .map(|(v, _)| *v)
                .expect("counts is non-empty");
            Ok(Value::Float(best))
        }
        "range" => {
            let xs = any_floats(name, &args)?;
            if xs.is_empty() {
                return Err(domain_error("range of an empty list"));
            }
            let (lo, hi) = xs
                .iter()
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), x| {
                    (lo.min(*x), hi.max(*x))
                });
            Ok(Value::Float(hi - lo))
        }
        "quartile" => {
            let (list, k) = match args.as_slice() {
                [Value::List(items), Value::Float(k)] => (items.clone(), *k),
                _ => {
                    return Err(EpherError::Type(format!(
                        "{name} expects a list and a whole number 1..3, got {} argument(s)",
                        args.len()
                    )))
                }
            };
            let k = float_to_int(k).ok_or_else(|| {
                EpherError::Type(format!("{name} expects a whole number 1..3, got {k}"))
            })?;
            if !(1..=3).contains(&k) || list.is_empty() {
                return Err(domain_error(format!(
                    "{name} needs a non-empty list and k in 1..3"
                )));
            }
            let mut xs: Vec<f64> = Vec::with_capacity(list.len());
            for item in &list {
                match item {
                    Value::Float(x) => xs.push(*x),
                    other => {
                        return Err(EpherError::Type(format!(
                            "{name} expects numbers, got {other:?}"
                        )))
                    }
                }
            }
            xs.sort_by(|a, b| a.partial_cmp(b).expect("floats are comparable"));
            Ok(Value::Float(quartile_sorted(&xs, k as u32)))
        }
        "linreg" => {
            let (xs, ys) = match args.as_slice() {
                [Value::List(a), Value::List(b)] if a.len() == b.len() => {
                    let to = |items: &[Value]| -> Result<Vec<f64>, EpherError> {
                        let mut out = Vec::with_capacity(items.len());
                        for item in items {
                            match item {
                                Value::Float(x) => out.push(*x),
                                other => {
                                    return Err(EpherError::Type(format!(
                                        "{name} expects numbers, got {other:?}"
                                    )))
                                }
                            }
                        }
                        Ok(out)
                    };
                    (to(a)?, to(b)?)
                }
                _ => {
                    return Err(EpherError::Type(format!(
                        "{name} expects two same-length lists, got {} argument(s)",
                        args.len()
                    )))
                }
            };
            if xs.is_empty() {
                return Err(domain_error("linear fit needs at least 2 points"));
            }
            let (a, b, r) = linear_fit(&xs, &ys)?;
            Ok(Value::Str(format!(
                "y = {}*x + {} (r = {})",
                stat_str(a),
                stat_str(b),
                stat_str(r)
            )))
        }
        // The regression family (ADR-0054): the models TI and NumWorks
        // fit on a list pair, each reporting its r and drawing an
        // overlay through `scatter xs, ys, <model>`.
        "quadreg" | "expreg" | "powreg" | "logreg" => {
            let (xs, ys) = reg_pair(name, &args)?;
            let kind = match name {
                "quadreg" => FitKind::Quadratic,
                "expreg" => FitKind::Exponential,
                "powreg" => FitKind::Power,
                _ => FitKind::Logarithmic,
            };
            Ok(Value::Str(fit_regression(kind, &xs, &ys)?.caption()))
        }
        // Probability distributions (ADR-0044): the normal family takes
        // one or three arguments (1-argument forms are the standard
        // normal); the others take their textbook parameters.
        "normpdf" | "normcdf" | "invnorm" => {
            let (x, mu, sigma) = match args.as_slice() {
                [Value::Float(x)] => (*x, 0.0, 1.0),
                [Value::Float(x), Value::Float(mu), Value::Float(sigma)] => (*x, *mu, *sigma),
                _ => {
                    return Err(EpherError::Type(format!(
                        "{name} expects 1 or 3 numbers, got {} argument(s)",
                        args.len()
                    )))
                }
            };
            if sigma <= 0.0 {
                return Err(domain_error(format!("{name} needs sigma > 0, got {sigma}")));
            }
            match name {
                "normpdf" => Ok(Value::Float(norm_pdf((x - mu) / sigma) / sigma)),
                "normcdf" => Ok(Value::Float(norm_cdf((x - mu) / sigma))),
                _ => {
                    let p = prob_arg(name, x)?;
                    Ok(Value::Float(mu + sigma * inv_norm(p)))
                }
            }
        }
        "tpdf" | "tcdf" | "invt" => {
            let (x, df) = two_floats(name, &args)?;
            if df <= 0.0 {
                return Err(domain_error(format!("{name} needs df > 0, got {df}")));
            }
            match name {
                "tpdf" => Ok(Value::Float(t_pdf(x, df))),
                "tcdf" => Ok(Value::Float(t_cdf(x, df))),
                _ => {
                    let p = prob_arg(name, x)?;
                    Ok(Value::Float(invert_cdf(
                        |t| t_cdf(t, df),
                        |t| t_survivor(t, df),
                        |t| t_pdf(t, df),
                        p,
                        -100.0,
                        100.0,
                    )))
                }
            }
        }
        "chi2pdf" | "chi2cdf" | "invchi2" => {
            let (x, df) = two_floats(name, &args)?;
            if df <= 0.0 {
                return Err(domain_error(format!("{name} needs df > 0, got {df}")));
            }
            match name {
                "chi2pdf" => {
                    if x < 0.0 {
                        return Err(domain_error(format!("{name} of a negative x")));
                    }
                    Ok(Value::Float(chi2_pdf(x, df)))
                }
                "chi2cdf" => {
                    if x < 0.0 {
                        return Err(domain_error(format!("{name} of a negative x")));
                    }
                    Ok(Value::Float(chi2_cdf(x, df)))
                }
                _ => {
                    let p = prob_arg(name, x)?;
                    Ok(Value::Float(invert_cdf(
                        |v| chi2_cdf(v, df),
                        |v| regularized_gamma_q(df / 2.0, v / 2.0),
                        |v| chi2_pdf(v, df),
                        p,
                        0.0,
                        (df + 40.0 * (2.0 * df).sqrt()).max(16.0),
                    )))
                }
            }
        }
        "binompdf" | "binomcdf" => {
            let (k, n, p) = three_floats(name, &args)?;
            let k = count_arg(name, k)?;
            let n = count_arg(name, n)?;
            let p = prob_arg(name, p)?;
            if n < 0 {
                return Err(domain_error(format!("{name} needs n >= 0, got {n}")));
            }
            if name == "binompdf" {
                Ok(Value::Float(binom_pdf(k, n, p)))
            } else {
                let mut acc = 0.0;
                for i in 0..=k {
                    acc += binom_pdf(i, n, p);
                }
                Ok(Value::Float(acc.min(1.0)))
            }
        }
        "poissonpdf" | "poissoncdf" => {
            let (k, lambda) = two_floats(name, &args)?;
            let k = count_arg(name, k)?;
            if lambda < 0.0 {
                return Err(domain_error(format!(
                    "{name} needs lambda >= 0, got {lambda}"
                )));
            }
            if name == "poissonpdf" {
                Ok(Value::Float(poisson_pdf(k, lambda)))
            } else {
                let mut acc = 0.0;
                for i in 0..=k {
                    acc += poisson_pdf(i, lambda);
                }
                Ok(Value::Float(acc.min(1.0)))
            }
        }
        // Hypothesis tests and confidence intervals (ADR-0044): data
        // lists in, display strings out.
        // Matrices (ADR-0049): the NumWorks floor — det, inv, transpose,
        // trace, dim, ref, rref.
        "det" => {
            let (n, data) = square_matrix(name, one_arg(name, &args)?)?;
            Ok(Value::Float(det_value(n, data)?))
        }
        "trace" => {
            let (n, data) = square_matrix(name, one_arg(name, &args)?)?;
            Ok(Value::Float((0..n).map(|i| data[i * n + i]).sum()))
        }
        "transpose" => {
            let (rows, cols, data) = matrix_arg(name, &args)?;
            let mut out = vec![0.0; rows * cols];
            for r in 0..rows {
                for c in 0..cols {
                    out[c * rows + r] = data[r * cols + c];
                }
            }
            Ok(Value::Matrix {
                rows: cols,
                cols: rows,
                data: out,
            })
        }
        "dim" => {
            let (rows, cols, _) = matrix_arg(name, &args)?;
            Ok(Value::List(vec![
                Value::Float(rows as f64),
                Value::Float(cols as f64),
            ]))
        }
        "ref" | "rref" => {
            let (rows, cols, data) = matrix_arg(name, &args)?;
            // rref eliminates above and below every pivot; ref stops
            // after the forward pass (upper echelon).
            Ok(Value::Matrix {
                rows,
                cols,
                data: gauss_jordan(rows, cols, data, matches!(name, "rref"), false)?,
            })
        }
        "inv" => {
            let (n, data) = square_matrix(name, one_arg(name, &args)?)?;
            let mut aug = vec![0.0; n * 2 * n];
            for r in 0..n {
                for c in 0..n {
                    aug[r * (2 * n) + c] = data[r * n + c];
                }
                aug[r * (2 * n) + n + r] = 1.0;
            }
            let out = gauss_jordan(n, 2 * n, aug, true, true)?;
            let mut inv = vec![0.0; n * n];
            for r in 0..n {
                for c in 0..n {
                    inv[r * n + c] = out[r * (2 * n) + n + c];
                }
            }
            Ok(Value::Matrix {
                rows: n,
                cols: n,
                data: inv,
            })
        }
        // Finance (ADR-0050): the TVM solver, NPV/IRR, amortization.
        "tvm_n" | "tvm_i" | "tvm_pv" | "tvm_pmt" | "tvm_fv" => eval_tvm(name, &args),
        "npv" => {
            let (rate, flows) = finance_rate_and_flows(name, &args)?;
            let mut total = 0.0;
            for (k, c) in flows.iter().enumerate() {
                total += c / (1.0 + rate).powi(k as i32);
            }
            Ok(Value::Float(total))
        }
        "irr" => {
            let flows = finance_flows(name, &args)?;
            // The rate where npv is zero, by bisection over the same
            // range the TVM rate solver uses.
            let f = |r: f64| -> f64 {
                let mut total = 0.0;
                for (k, c) in flows.iter().enumerate() {
                    total += c / (1.0 + r).powi(k as i32);
                }
                total
            };
            let (lo, hi) = (-0.999_999, 1.0);
            Ok(Value::Float(bisect_rate(name, f, lo, hi)?))
        }
        "amort" => {
            let (p, r, n, k) = four_floats(name, &args)?;
            if n <= 0.0 || n.fract() != 0.0 {
                return Err(domain_error(format!(
                    "amort needs a whole number of periods, got {n}"
                )));
            }
            if k < 0.0 || k > n {
                return Err(domain_error(format!(
                    "amort's period k must be between 0 and {n}, got {k}"
                )));
            }
            let balance = if r == 0.0 {
                p * (1.0 - k / n)
            } else {
                let g = (1.0 + r).powf(n);
                let pmt = -p * r * g / (g - 1.0);
                let gk = (1.0 + r).powf(k);
                p * gk + pmt * (gk - 1.0) / r
            };
            Ok(Value::Float(balance))
        }
        "simple_interest" => {
            let (p, r, t) = three_floats(name, &args)?;
            Ok(Value::Float(p * r * t))
        }
        "compound_interest" => {
            let (p, r, n) = three_floats(name, &args)?;
            Ok(Value::Float(p * (1.0 + r).powf(n) - p))
        }
        "ztest" => {
            let (data, mu0, sigma) = match args.as_slice() {
                [_d, Value::Float(mu0), Value::Float(sigma)] => {
                    (data_list(name, &args, 0)?, *mu0, *sigma)
                }
                _ => {
                    return Err(EpherError::Type(format!(
                        "{name} expects a data list, a mean, and sigma, got {} argument(s)",
                        args.len()
                    )))
                }
            };
            if data.len() < 2 {
                return Err(domain_error("ztest needs at least 2 data points"));
            }
            if sigma <= 0.0 {
                return Err(domain_error(format!("ztest needs sigma > 0, got {sigma}")));
            }
            let (mean, _) = sample_mean_std(&data);
            let z = (mean - mu0) / (sigma / (data.len() as f64).sqrt());
            Ok(Value::Str(format!(
                "z = {}, p = {}",
                stat_str(z),
                stat_str(z_two_sided(z))
            )))
        }
        "ttest" => {
            let (data, mu0) = match args.as_slice() {
                [_d, Value::Float(mu0)] => (data_list(name, &args, 0)?, *mu0),
                _ => {
                    return Err(EpherError::Type(format!(
                        "{name} expects a data list and a mean, got {} argument(s)",
                        args.len()
                    )))
                }
            };
            if data.len() < 2 {
                return Err(domain_error("ttest needs at least 2 data points"));
            }
            let n = data.len() as f64;
            let (mean, sd) = sample_mean_std(&data);
            let t = (mean - mu0) / (sd / n.sqrt());
            let p = if sd == 0.0 {
                // a degenerate sample: every value equal — the statistic
                // is 0 (when mu0 matches) or infinite
                if mean == mu0 {
                    1.0
                } else {
                    0.0
                }
            } else {
                t_two_sided(t, n - 1.0)
            };
            Ok(Value::Str(format!(
                "t = {}, p = {}",
                stat_str(t),
                stat_str(p)
            )))
        }
        "ttestpaired" => {
            // Paired t (ADR-0054): the one-sample t of the differences,
            // tested against 0, the classroom "before and after" test:
            let (a, b) = reg_pair(name, &args)?;
            if a.len() < 2 {
                return Err(domain_error("ttestpaired needs at least 2 pairs"));
            }
            let diffs: Vec<f64> = a.iter().zip(b.iter()).map(|(x, y)| x - y).collect();
            let n = diffs.len() as f64;
            let (mean, sd) = sample_mean_std(&diffs);
            let t = (mean) / (sd / n.sqrt());
            let p = if sd == 0.0 {
                if mean == 0.0 { 1.0 } else { 0.0 }
            } else {
                t_two_sided(t, n - 1.0)
            };
            Ok(Value::Str(format!(
                "t = {}, p = {}",
                stat_str(t),
                stat_str(p)
            )))
        }
        "anova" => {
            // One-way ANOVA (ADR-0054): F and its p over 2+ groups,
            // unequal group lengths welcome. The F CDF runs on the
            // incomplete beta the t family's tail work already relies
            // on (puruspe's betai behind the clamping wrapper).
            if args.len() < 2 {
                return Err(EpherError::Type(format!(
                    "anova needs at least two lists, got {}",
                    args.len()
                )));
            }
            let mut groups = Vec::with_capacity(args.len());
            for i in 0..args.len() {
                groups.push(data_list(name, &args, i)?);
            }
            if groups.iter().any(|g| g.is_empty()) {
                return Err(domain_error("anova needs non-empty groups"));
            }
            let k = groups.len() as f64;
            let n_total: f64 = groups.iter().map(|g| g.len()).sum::<usize>() as f64;
            if n_total <= k {
                return Err(domain_error(
                    "anova needs more data points than groups",
                ));
            }
            let grand = groups
                .iter()
                .flat_map(|g| g.iter().copied())
                .sum::<f64>()
                / n_total;
            let ssb: f64 = groups
                .iter()
                .map(|g| {
                    let m = g.iter().sum::<f64>() / g.len() as f64;
                    g.len() as f64 * (m - grand) * (m - grand)
                })
                .sum();
            let ssw: f64 = groups
                .iter()
                .map(|g| {
                    let m = g.iter().sum::<f64>() / g.len() as f64;
                    g.iter().map(|x| (x - m) * (x - m)).sum::<f64>()
                })
                .sum();
            let df1 = k - 1.0;
            let df2 = n_total - k;
            let (f_stat, p) = if ssw == 0.0 {
                // every value in every group identical: no within-group
                // variance, so the F statistic is degenerate
                if ssb == 0.0 { (0.0, 1.0) } else { (f64::INFINITY, 0.0) }
            } else {
                let f = (ssb / df1) / (ssw / df2);
                (f, (1.0 - f_cdf(f, df1, df2)).max(0.0))
            };
            Ok(Value::Str(format!(
                "F = {}, p = {}",
                stat_str(f_stat),
                stat_str(p)
            )))
        }
        "zinterval" => {
            let (data, sigma, level) = match args.as_slice() {
                [_d, Value::Float(sigma), Value::Float(level)] => {
                    (data_list(name, &args, 0)?, *sigma, *level)
                }
                _ => {
                    return Err(EpherError::Type(format!(
                        "{name} expects a data list, sigma, and a level, got {} argument(s)",
                        args.len()
                    )))
                }
            };
            if data.is_empty() {
                return Err(domain_error("zinterval needs data"));
            }
            if sigma <= 0.0 {
                return Err(domain_error(format!("{name} needs sigma > 0, got {sigma}")));
            }
            let level = prob_arg(name, level)?;
            let mean = data.iter().sum::<f64>() / data.len() as f64;
            let z = inv_norm(0.5 + level / 2.0);
            let hw = z * sigma / (data.len() as f64).sqrt();
            Ok(Value::Str(interval_str(mean - hw, mean + hw)))
        }
        "tinterval" => {
            let (data, level) = match args.as_slice() {
                [_d, Value::Float(level)] => (data_list(name, &args, 0)?, *level),
                _ => {
                    return Err(EpherError::Type(format!(
                        "{name} expects a data list and a level, got {} argument(s)",
                        args.len()
                    )))
                }
            };
            if data.len() < 2 {
                return Err(domain_error("tinterval needs at least 2 data points"));
            }
            let level = prob_arg(name, level)?;
            let n = data.len() as f64;
            let (mean, sd) = sample_mean_std(&data);
            let t = invert_cdf(
                |v| t_cdf(v, n - 1.0),
                |v| t_survivor(v, n - 1.0),
                |v| t_pdf(v, n - 1.0),
                0.5 + level / 2.0,
                -100.0,
                100.0,
            );
            let hw = t * sd / n.sqrt();
            Ok(Value::Str(interval_str(mean - hw, mean + hw)))
        }
        "chisq_gof" => {
            let (observed, expected) = match args.as_slice() {
                [Value::List(a), Value::List(b)] if a.len() == b.len() && !a.is_empty() => {
                    (data_list(name, &args, 0)?, data_list(name, &args, 1)?)
                }
                _ => {
                    return Err(EpherError::Type(format!(
                        "{name} expects two same-length non-empty lists, got {} argument(s)",
                        args.len()
                    )))
                }
            };
            let mut chi2 = 0.0;
            for (o, e) in observed.iter().zip(expected.iter()) {
                if *e <= 0.0 {
                    return Err(domain_error(format!(
                        "expected counts must be positive, got {e}"
                    )));
                }
                chi2 += (o - e) * (o - e) / e;
            }
            let df = observed.len() as f64 - 1.0;
            Ok(Value::Str(format!(
                "chi2 = {}, p = {}",
                stat_str(chi2),
                stat_str((1.0 - chi2_cdf(chi2, df)).max(0.0))
            )))
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
        Statement::For(var, iterable, body) => Some(run_for(var, iterable, body, env, steps)?),
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

/// The most iterations a `for` loop may run (ADR-0054): the same
/// runaway guard spirit as the table's 1000 points, sized larger
/// because each iteration does real work the step budget also counts.
const MAX_FOR_ITERATIONS: i64 = 100_000;

/// Run a `for` loop (ADR-0054): bind the loop variable for each
/// element, collect the body's values (statements with no value, such
/// as a definition or a nested `while`, contribute nothing), and return the
/// collected list. The loop variable keeps its last value afterwards,
/// like TI's `For`.
fn run_for(
    var: &str,
    iterable: &ForIterable,
    body: &Statement,
    env: &mut Env,
    steps: &mut u64,
) -> Result<Value, EpherError> {
    let items: Vec<Value> = match iterable {
        ForIterable::Items(expr) => match eval(expr, env)? {
            Value::List(items) => items,
            other => {
                return Err(EpherError::Type(format!(
                    "for needs a list or a range, got {other:?}; \
                     try `for i in 1 to 5 do ...` or `for x in d do ...`"
                )))
            }
        },
        ForIterable::Range { start, end, step } => {
            let Value::Float(start) = eval(start, env)? else {
                return Err(EpherError::Type(
                    "the range start must be a number".to_string(),
                ));
            };
            let Value::Float(end) = eval(end, env)? else {
                return Err(EpherError::Type("the range end must be a number".to_string()));
            };
            let step = match step {
                Some(expr) => match eval(expr, env)? {
                    Value::Float(s) => s,
                    other => {
                        return Err(EpherError::Type(format!(
                            "the step must be a number, got {other:?}"
                        )))
                    }
                },
                None => 1.0,
            };
            if step == 0.0 || !step.is_finite() {
                return Err(domain_error(format!(
                    "the step must be a nonzero number, got {step}"
                )));
            }
            // Index-based so values never accumulate float drift: the
            // k-th value is computed directly from k. A range that runs
            // backwards against its step is simply empty.
            let count = (((end - start) / step) + 1e-9).floor() + 1.0;
            let count = count.max(0.0);
            if count > MAX_FOR_ITERATIONS as f64 {
                return Err(domain_error(format!(
                    "for runs at most {MAX_FOR_ITERATIONS} iterations, got {count:.0}"
                )));
            }
            (0..count as i64)
                .map(|k| Value::Float(start + k as f64 * step))
                .collect()
        }
    };
    let mut collected = Vec::with_capacity(items.len());
    for item in items {
        env.set(var.to_string(), item);
        if let Some(value) = stmt_value(body, env, steps)? {
            collected.push(value);
        }
    }
    Ok(Value::List(collected))
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
        let session = Self::default();
        // Interactive sessions seed the generator from the clock
        // (ADR-0045); `Session::default()` and `evaluate()` keep the
        // fixed seed so tests and scripted runs are deterministic. The
        // clock read is the same wasm-safe one `now()` uses (ADR-0037).
        let nanos = (astro::now_unix_seconds() * 1e9) as u64;
        session.env.rng.set(nanos ^ 0x9E37_79B9_7F4A_7C15);
        session
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

    /// Read the session's result display preferences — the exact-fraction
    /// toggle, notation, and separators (the table's exact cells follow
    /// the same preference, ADR-0044).
    pub fn display(&self) -> DisplayPrefs {
        self.display
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

    /// Submit a script line and return every answer it produced, in
    /// order, one per line (`= 10\n= 15\n= 25`) — a script's whole
    /// transcript, not only its final value (ADR-0052). The line is
    /// recorded in history exactly like [`Session::submit`] (the line
    /// with its last answer appended). Statements that produce no value
    /// (`def`, `while`, `graph`) contribute nothing, and an error stops
    /// the run, exactly as one-shot scripts do.
    pub fn submit_all(&mut self, line: &str) -> String {
        let line = line.trim().to_string();
        if line.is_empty() {
            return String::new();
        }
        let outputs = match parse_script(&line) {
            Ok(script) => match run_all(&script, &mut self.env) {
                Ok(values) => values
                    .iter()
                    .map(|v| format!("= {}", format_value(v, &self.display)))
                    .collect::<Vec<_>>(),
                Err(e) => vec![format!("error: {e}")],
            },
            Err(e) => vec![format!("error: {e}")],
        };
        let joined = outputs.join("\n");
        if joined.is_empty() {
            self.history.push(line.clone());
        } else {
            self.history.push(format!(
                "{line}  {}",
                outputs.last().map(String::as_str).unwrap_or_default()
            ));
        }
        self.last_line = Some(line.clone());
        if let Some(name) = def_name(&line) {
            self.defs.insert(name, line.clone());
        }
        if let Some(name) = const_name(&line) {
            self.consts.insert(name, line);
        }
        joined
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
    // Quantity arithmetic (ADR-0046): dimensions compose for `*` and
    // `/`, must match for `+` and `-`, and a plain number pairs with a
    // quantity as a dimensionless value.
    if matches!(&lhs, Value::Quantity { .. }) || matches!(&rhs, Value::Quantity { .. }) {
        return quantity_binop(lhs, rhs, op);
    }
    // Matrix arithmetic (ADR-0049): elementwise + and -, the matrix
    // product for *, and scaling by plain numbers.
    if matches!(&lhs, Value::Matrix { .. }) || matches!(&rhs, Value::Matrix { .. }) {
        return matrix_binop(lhs, rhs, op);
    }
    // Elementwise list arithmetic (ADR-0044): `{1,2,3} * 2`, `2 /
    // {1,2,3}`, `{1,2} + {3,4}`. Lists of different lengths are a
    // type error; a scalar is any plain number.
    if matches!(&lhs, Value::List(_)) || matches!(&rhs, Value::List(_)) {
        return list_binop(lhs, rhs, op);
    }
    // String arithmetic (ADR-0054): `+` concatenates, and that is the
    // whole of it; there is no string subtraction, and mixing a string
    // with a number is a type error (use `str` or `print`).
    if matches!(&lhs, Value::Str(_)) || matches!(&rhs, Value::Str(_)) {
        if let (Value::Str(a), Value::Str(b), BinOp::Add) = (&lhs, &rhs, op) {
            return Ok(Value::Str(format!("{a}{b}")));
        }
        return Err(EpherError::Type(
            "strings only support + (concatenation)".to_string(),
        ));
    }
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

/// A numeric value as f64 for cross-type comparisons (ADR-0047):
/// rationals, decimals, and big integers convert lossily, which is the
/// honest float-vs-exact comparison.
fn numeric_f64(v: &Value) -> Result<f64, EpherError> {
    use num_traits::ToPrimitive;
    match v {
        Value::Float(x) => Ok(*x),
        Value::Rational(r) => r
            .to_f64()
            .ok_or_else(|| EpherError::Type(format!("cannot compare {v}"))),
        Value::Decimal(d) => d
            .to_f64()
            .ok_or_else(|| EpherError::Type(format!("cannot compare {v}"))),
        Value::Big(b) => b
            .to_f64()
            .ok_or_else(|| EpherError::Type(format!("cannot compare {v}"))),
        other => Err(EpherError::Type(format!("cannot compare {other:?}"))),
    }
}

/// Matrix arithmetic (ADR-0049): `+`/`-` are elementwise with matching
/// shapes, `*` is the matrix product, a plain number scales (or, for
/// division, divides) elementwise, and `^` is the whole-number matrix
/// power (n = 0 gives the identity, so powers need square matrices).
fn matrix_binop(lhs: Value, rhs: Value, op: BinOp) -> Result<Value, EpherError> {
    let as_matrix = |v: Value| -> Result<(usize, usize, Vec<f64>), EpherError> {
        match v {
            Value::Matrix { rows, cols, data } => Ok((rows, cols, data)),
            other => Err(EpherError::Type(format!(
                "cannot combine a matrix with {other:?}"
            ))),
        }
    };
    let as_scalar = |v: &Value| -> Result<f64, EpherError> {
        match v {
            Value::Float(x) => Ok(*x),
            other => Err(EpherError::Type(format!(
                "a matrix only scales by a number, got {other:?}"
            ))),
        }
    };
    match (&lhs, &rhs) {
        (Value::Matrix { .. }, Value::Matrix { .. }) => {
            let (ra, ca, a) = as_matrix(lhs)?;
            let (rb, cb, b) = as_matrix(rhs)?;
            match op {
                BinOp::Add | BinOp::Sub => {
                    if ra != rb || ca != cb {
                        return Err(EpherError::Type(format!(
                            "matrix shapes must match: {ra}x{ca} and {rb}x{cb}"
                        )));
                    }
                    let data = a
                        .iter()
                        .zip(&b)
                        .map(|(x, y)| {
                            if matches!(op, BinOp::Add) {
                                x + y
                            } else {
                                x - y
                            }
                        })
                        .collect();
                    Ok(Value::Matrix {
                        rows: ra,
                        cols: ca,
                        data,
                    })
                }
                BinOp::Mul => {
                    if ca != rb {
                        return Err(EpherError::Type(format!(
                            "matrix product needs {ca} columns in A and {rb} rows in B"
                        )));
                    }
                    let mut data = vec![0.0; ra * cb];
                    for i in 0..ra {
                        for k in 0..ca {
                            let aik = a[i * ca + k];
                            for j in 0..cb {
                                data[i * cb + j] += aik * b[k * cb + j];
                            }
                        }
                    }
                    Ok(Value::Matrix {
                        rows: ra,
                        cols: cb,
                        data,
                    })
                }
                BinOp::Div => Err(EpherError::Type(
                    "cannot divide by a matrix; multiply by inv(M) instead".to_string(),
                )),
                BinOp::Pow => Err(EpherError::Type(
                    "a matrix power needs one matrix and a whole number".to_string(),
                )),
            }
        }
        (Value::Matrix { rows, cols, data }, Value::Float(_)) => {
            let s = as_scalar(&rhs)?;
            match op {
                BinOp::Mul => Ok(Value::Matrix {
                    rows: *rows,
                    cols: *cols,
                    data: data.iter().map(|x| x * s).collect(),
                }),
                BinOp::Div => Ok(Value::Matrix {
                    rows: *rows,
                    cols: *cols,
                    data: data.iter().map(|x| x / s).collect(),
                }),
                BinOp::Pow => {
                    // The whole-number matrix power (ADR-0049): n = 0 is
                    // the identity, so powers need square matrices.
                    if rows != cols {
                        return Err(EpherError::Type(format!(
                            "the matrix power needs a square matrix, got {rows}x{cols}"
                        )));
                    }
                    if s.fract() != 0.0 || !s.is_finite() || s < 0.0 || s > 1024.0 {
                        return Err(EpherError::Type(format!(
                            "the matrix power needs a whole number 0..=1024, got {s}"
                        )));
                    }
                    let n = s as usize;
                    let mut result = identity_matrix(*rows);
                    let mut base = data.clone();
                    let mut e = n;
                    while e > 0 {
                        if e % 2 == 1 {
                            result = matrix_product(&result, &base, *rows);
                        }
                        e /= 2;
                        if e > 0 {
                            base = matrix_product(&base, &base, *rows);
                        }
                    }
                    Ok(Value::Matrix {
                        rows: *rows,
                        cols: *cols,
                        data: result,
                    })
                }
                BinOp::Add | BinOp::Sub => Err(EpherError::Type(format!(
                    "a matrix and a number only multiply, divide, or power, not {op:?}"
                ))),
            }
        }
        (Value::Float(_), Value::Matrix { rows, cols, data }) => {
            let s = as_scalar(&lhs)?;
            match op {
                BinOp::Mul => Ok(Value::Matrix {
                    rows: *rows,
                    cols: *cols,
                    data: data.iter().map(|x| s * x).collect(),
                }),
                BinOp::Add | BinOp::Sub | BinOp::Div | BinOp::Pow => Err(EpherError::Type(
                    "a matrix only scales by a number on its left for *".to_string(),
                )),
            }
        }
        _ => Err(EpherError::Type(
            "cannot combine a matrix with that".to_string(),
        )),
    }
}

/// The n×n identity matrix (ADR-0049), for the matrix power's n = 0.
fn identity_matrix(n: usize) -> Vec<f64> {
    let mut out = vec![0.0; n * n];
    for i in 0..n {
        out[i * n + i] = 1.0;
    }
    out
}

/// The product of two square matrices of size n (ADR-0049).
fn matrix_product(a: &[f64], b: &[f64], n: usize) -> Vec<f64> {
    let mut out = vec![0.0; n * n];
    for i in 0..n {
        for k in 0..n {
            let aik = a[i * n + k];
            for j in 0..n {
                out[i * n + j] += aik * b[k * n + j];
            }
        }
    }
    out
}

/// A float or a quantity unified for the ADR-0046 arithmetic rules:
/// a plain number is a dimensionless quantity.
fn as_quantity(v: Value) -> Result<(f64, Dims, Option<(String, f64)>), EpherError> {
    match v {
        Value::Float(x) => Ok((x, [0; 7], None)),
        Value::Quantity { value, dims, unit } => Ok((value, dims, unit)),
        other => Err(EpherError::Type(format!(
            "cannot combine quantities with {other:?}"
        ))),
    }
}

/// Dimension-aware arithmetic (ADR-0046): `+`/`-` need matching dims
/// (a dimensionless side folds into the other), `*`/`/` compose them,
/// and a power raises the value and scales the dims — a non-whole
/// exponent on a dimensioned quantity is an error. Dimensionless
/// results collapse back to plain floats.
fn quantity_binop(lhs: Value, rhs: Value, op: BinOp) -> Result<Value, EpherError> {
    let (a, da, ua) = as_quantity(lhs)?;
    let (b, db, ub) = as_quantity(rhs)?;
    let zero = [0i8; 7];
    match op {
        BinOp::Add | BinOp::Sub => {
            // The dims must match exactly: a plain number is
            // dimensionless, so adding one to a dimensioned quantity
            // is a dimension error too (ADR-0046).
            if da != db {
                return Err(dimension_error(&format!(
                    "cannot {} {} and {}",
                    if matches!(op, BinOp::Add) {
                        "add"
                    } else {
                        "subtract"
                    },
                    quantity_display(a, da, ua),
                    quantity_display(b, db, ub)
                )));
            }
            let v = if matches!(op, BinOp::Add) {
                a + b
            } else {
                a - b
            };
            Ok(finish_quantity(v, da, merge_unit(ua, ub)))
        }
        BinOp::Mul => Ok(finish_quantity(a * b, add_dims(da, db)?, None)),
        BinOp::Div => {
            if b == 0.0 {
                return Err(EpherError::ZeroDivision);
            }
            Ok(finish_quantity(a / b, sub_dims(da, db)?, None))
        }
        BinOp::Pow => {
            if db != zero {
                return Err(EpherError::Type(format!(
                    "cannot raise to a quantity ({})",
                    quantity_display(b, db, ub)
                )));
            }
            if da != zero && (b.fract() != 0.0 || b.abs() > 127.0 || !b.is_finite()) {
                return Err(EpherError::Type(format!(
                    "a quantity can only be raised to a whole-number power, got {b}"
                )));
            }
            let dims = if da == zero {
                zero
            } else {
                scale_dims(da, b as i8)?
            };
            Ok(finish_quantity(a.powf(b), dims, None))
        }
    }
}

fn add_dims(a: Dims, b: Dims) -> Result<Dims, EpherError> {
    let mut out = [0i8; 7];
    for i in 0..7 {
        let v = a[i] as i16 + b[i] as i16;
        if v < -127 || v > 127 {
            return Err(dimension_error("the dimensions overflow"));
        }
        out[i] = v as i8;
    }
    Ok(out)
}

fn sub_dims(a: Dims, b: Dims) -> Result<Dims, EpherError> {
    let mut out = [0i8; 7];
    for i in 0..7 {
        let v = a[i] as i16 - b[i] as i16;
        if v < -127 || v > 127 {
            return Err(dimension_error("the dimensions overflow"));
        }
        out[i] = v as i8;
    }
    Ok(out)
}

fn scale_dims(a: Dims, e: i8) -> Result<Dims, EpherError> {
    let mut out = [0i8; 7];
    for i in 0..7 {
        let v = a[i] as i16 * e as i16;
        if v < -127 || v > 127 {
            return Err(dimension_error("the dimensions overflow"));
        }
        out[i] = v as i8;
    }
    Ok(out)
}

/// Two display units survive `+`/`-` only when they are identical.
fn merge_unit(a: Option<(String, f64)>, b: Option<(String, f64)>) -> Option<(String, f64)> {
    match (a, b) {
        (Some((sa, fa)), Some((sb, fb))) if sa == sb && fa == fb => Some((sa, fa)),
        _ => None,
    }
}

/// A dimensionless result is a plain float again; otherwise keep the
/// quantity (and its display unit).
fn finish_quantity(v: f64, dims: Dims, unit: Option<(String, f64)>) -> Value {
    if dims == [0; 7] {
        Value::Float(v)
    } else {
        Value::Quantity {
            value: v,
            dims,
            unit,
        }
    }
}

fn dimension_error(msg: &str) -> EpherError {
    EpherError::Dimension(msg.to_string())
}

/// The plain display of a quantity: the value in its display unit (or
/// the SI composition), without the result-formatting preferences —
/// used in dimension errors.
fn quantity_display(value: f64, dims: Dims, unit: Option<(String, f64)>) -> String {
    if dims == [0; 7] {
        return format!("{value}");
    }
    match unit {
        Some((name, factor)) => format!("{} {name}", value / factor),
        None => format!("{value} {}", si_unit_str(dims)),
    }
}

/// Elementwise list arithmetic (ADR-0044): a list paired with a scalar
/// applies the scalar to every element; two lists must have the same
/// length. The elementwise results stay floats.
fn list_binop(lhs: Value, rhs: Value, op: BinOp) -> Result<Value, EpherError> {
    let scalar = |v: &Value| -> Result<f64, EpherError> {
        match v {
            Value::Float(x) => Ok(*x),
            Value::Rational(r) => r
                .to_f64()
                .ok_or_else(|| EpherError::Type(format!("cannot use {v} as a list scalar"))),
            Value::Decimal(d) => d
                .to_f64()
                .ok_or_else(|| EpherError::Type(format!("cannot use {v} as a list scalar"))),
            Value::Big(b) => b
                .to_f64()
                .ok_or_else(|| EpherError::Type(format!("cannot use {v} as a list scalar"))),
            other => Err(EpherError::Type(format!(
                "cannot combine a list with {other:?}"
            ))),
        }
    };
    let apply = |a: f64, b: f64| float_binop(op, a, b);
    match (&lhs, &rhs) {
        (Value::List(a), Value::List(b)) => {
            if a.len() != b.len() {
                return Err(EpherError::Type(format!(
                    "lists have different lengths: {} and {}",
                    a.len(),
                    b.len()
                )));
            }
            let mut out = Vec::with_capacity(a.len());
            for (x, y) in a.iter().zip(b.iter()) {
                let (Value::Float(x), Value::Float(y)) = (x, y) else {
                    return Err(EpherError::Type(format!(
                        "cannot combine {x:?} and {y:?} elementwise"
                    )));
                };
                out.push(Value::Float(apply(*x, *y)?));
            }
            Ok(Value::List(out))
        }
        (Value::List(a), other) => {
            let s = scalar(other)?;
            let mut out = Vec::with_capacity(a.len());
            for v in a {
                let Value::Float(x) = v else {
                    return Err(EpherError::Type(format!(
                        "cannot combine {v:?} elementwise"
                    )));
                };
                out.push(Value::Float(apply(*x, s)?));
            }
            Ok(Value::List(out))
        }
        (other, Value::List(b)) => {
            let s = scalar(other)?;
            let mut out = Vec::with_capacity(b.len());
            for v in b {
                let Value::Float(x) = v else {
                    return Err(EpherError::Type(format!(
                        "cannot combine {v:?} elementwise"
                    )));
                };
                out.push(Value::Float(apply(s, *x)?));
            }
            Ok(Value::List(out))
        }
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
