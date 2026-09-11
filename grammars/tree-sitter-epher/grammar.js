// tree-sitter-epher: the epher calculator language.
//
// Written from site/reference.md (the normative definition): section
// 6 for expressions and precedence, section 7 for statements. Units
// are a postfix on numbers (reference section 11); adjacency is not
// enforced here because tree-sitter cannot see token boundaries
// through whitespace extras - the real lexer's adjacency rule lives
// in crates/core, and the language server's semantic tokens are the
// exact rule (ADR-0066). Note the one deliberate looseness: the
// reference says comparisons do not chain, and this grammar accepts
// chained ones; a highlighting grammar parses a superset, and the
// server remains the truth.

// Precedence, tightest to loosest, straight from reference 6.1.
const PREC = {
  RANGE: 0,
  OR: 1,
  AND: 2,
  NOT: 3,
  COMPARISON: 4,
  BIT_OR: 5,
  BIT_AND: 6,
  ADD: 7,
  CONVERT: 8,
  MULTIPLY: 9,
  UNARY: 10,
  POWER: 11,
  POSTFIX: 12,
  CALL: 13,
  UNIT: 14,
};

module.exports = grammar({
  name: 'epher',

  extras: $ => [
    /\s/,
    $.line_comment,
    $.block_comment,
  ],

  word: $ => $.identifier,

  externals: $ => [
    $.unit,
  ],

  conflicts: $ => [
    [$.destructuring, $._primary],
  ],

  rules: {
    source: $ => repeat($._statement),

    _statement: $ => choice(
      $.assignment,
      $.constant_definition,
      $.destructuring,
      $.function_definition,
      $.return_statement,
      $.break_statement,
      $.continue_statement,
      $.if_statement,
      $.while_statement,
      $.for_statement,
      $.solve_statement,
      $.expression_statement,
    ),

    assignment: $ => seq(
      field('name', $.identifier),
      '=',
      field('value', $._expression),
    ),

    constant_definition: $ => seq(
      'const',
      field('name', $.identifier),
      '=',
      field('value', $._expression),
    ),

    // The reference recognizes the pattern only when the statement
    // begins with '{names} =' and the contents are plain names; the
    // conflict declaration below keeps the list-literal reading alive
    // so the '=' (or a non-name element) decides, not a precedence.
    destructuring: $ => seq(
      '{',
      commaSep1(field('name', choice($.identifier, '_'))),
      '}',
      '=',
      field('value', $._expression),
    ),

    function_definition: $ => choice(
      seq(
        'def',
        field('name', $.identifier),
        '(',
        optional(field('parameters', commaSep1($.identifier))),
        ')',
        '=',
        field('body', $._expression),
      ),
      seq(
        'def',
        field('name', $.identifier),
        '(',
        optional(field('parameters', commaSep1($.identifier))),
        ')',
        'do',
        repeat($._statement),
        'end',
      ),
    ),

    // prec.right: 'return {1, 2}' returns the list; a bare 'return'
    // only reduces when no expression can follow.
    return_statement: $ => prec.right(seq('return', optional($._expression))),
    break_statement: $ => 'break',
    continue_statement: $ => 'continue',

    // prec: in statement position, `if c then x` is the statement
    // form; the conditional expression lives inside expressions. And
    // prec.right is the dangling else: it binds to the innermost
    // unterminated if.
    if_statement: $ => prec.right(1, seq(
      'if',
      field('condition', $._expression),
      'then',
      field('consequence', $._statement),
      optional(seq('else', field('alternative', $._statement))),
    )),

    while_statement: $ => seq(
      'while',
      field('condition', $._expression),
      'do',
      field('body', $._statement),
    ),

    for_statement: $ => seq(
      'for',
      field('name', $.identifier),
      'in',
      field('iterable', $._expression),
      'do',
      field('body', $._statement),
    ),

    solve_statement: $ => seq('solve', field('equation', $._expression)),

    expression_statement: $ => $._expression,

    // Expressions, tightest to loosest (reference section 6.1). Every
    // level participates in one flat choice; the prec numbers decide
    // which binding wins, in the order the table spells out.
    _expression: $ => choice(
      $._conditional,
      $.or_expression,
      $.and_expression,
      $.not_expression,
      $.comparison_expression,
      $.bit_or_expression,
      $.bit_and_expression,
      $.additive_expression,
      $.conversion_expression,
      $.multiplicative_expression,
      $.unary_expression,
      $.power_expression,
      $.postfix_expression,
      $.call_expression,
      $.range_expression,
      $._primary,
    ),

    // The conditional EXPRESSION form always carries an else (the
    // reference's operator table spells it `if c then a else b`); the
    // statement form's else is optional. This split is what keeps
    // statement position unambiguous, and the prec makes a dangling
    // else bind to the inner conditional, the reading everyone
    // expects.
    _conditional: $ => prec.right(1, seq(
      'if',
      field('condition', $._expression),
      'then',
      field('consequence', $._expression),
      'else',
      field('alternative', $._expression),
    )),

    or_expression: $ => prec.left(PREC.OR, seq(
      field('left', $._expression),
      'or',
      field('right', $._expression),
    )),

    and_expression: $ => prec.left(PREC.AND, seq(
      field('left', $._expression),
      'and',
      field('right', $._expression),
    )),

    not_expression: $ => prec(PREC.NOT, seq(
      'not',
      field('operand', $._expression),
    )),

    comparison_expression: $ => prec.left(PREC.COMPARISON, seq(
      field('left', $._expression),
      field('operator', choice('>', '<', '>=', '<=', '==', '!=')),
      field('right', $._expression),
    )),

    bit_or_expression: $ => prec.left(PREC.BIT_OR, seq(
      field('left', $._expression),
      field('operator', choice('|', 'xor')),
      field('right', $._expression),
    )),

    bit_and_expression: $ => prec.left(PREC.BIT_AND, seq(
      field('left', $._expression),
      '&',
      field('right', $._expression),
    )),

    additive_expression: $ => prec.left(PREC.ADD, seq(
      field('left', $._expression),
      field('operator', choice('+', '-')),
      field('right', $._expression),
    )),

    conversion_expression: $ => prec.left(PREC.CONVERT, seq(
      field('value', $._expression),
      field('operator', choice('in', '->')),
      field('unit', $.unit),
    )),

    multiplicative_expression: $ => prec.left(PREC.MULTIPLY, seq(
      field('left', $._expression),
      field('operator', choice('*', '/')),
      field('right', $._expression),
    )),

    unary_expression: $ => prec(PREC.UNARY, seq(
      field('operator', choice('-', '~')),
      field('operand', $._expression),
    )),

    power_expression: $ => prec.right(PREC.POWER, seq(
      field('left', $._expression),
      '^',
      field('right', $._expression),
    )),

    postfix_expression: $ => prec.left(PREC.POSTFIX, seq(
      field('operand', $._expression),
      choice('!', '%', $.index_suffix),
    )),

    index_suffix: $ => seq('[', field('index', $._expression), ']'),

    quantity: $ => prec(1, seq(
      field('number', $.number),
      field('unit', $.unit),
    )),

    call_expression: $ => prec(PREC.CALL, seq(
      field('function', $._expression),
      '(',
      optional(field('arguments', commaSep1($._expression))),
      ')',
    )),

    range_expression: $ => prec.left(PREC.RANGE, seq(
      field('start', $._expression),
      'to',
      field('end', $._expression),
      optional(seq('step', field('step', $._expression))),
    )),

    _primary: $ => choice(
      $.quantity,
      $.number,
      $.string,
      $.list,
      $.matrix,
      $.identifier,
      seq('(', $._expression, ')'),
    ),

    number: $ => token(choice(
      /0[bB][01]+/,
      /0[oO][0-7]+/,
      /0[xX][0-9a-fA-F]+/,
      /(\d+(\.\d*)?|\.\d+)([eE][+-]?\d+)?i?/,
    )),

    string: $ => seq(
      '"',
      repeat(choice($.escape_sequence, /[^"\\\n]+/)),
      '"',
    ),

    escape_sequence: $ => token(/\\[ntr\\"]/),

    list: $ => seq(
      '{',
      optional(commaSep1($._expression)),
      '}',
    ),

    // Rows in plain brackets: [[1, 2], [3, 4]] reads as a list of
    // rows. The multi-character `[[`/`]]` tokens would corrupt the
    // lexing of the closing corner.
    matrix: $ => seq(
      '[',
      commaSep1($.row),
      ']',
    ),

    row: $ => seq(
      '[',
      optional(commaSep1($._expression)),
      ']',
    ),

    identifier: $ => /[A-Za-z_][A-Za-z0-9_]*/,


    line_comment: $ => token(choice(seq('#', /.*/), seq('//', /.*/))),
    block_comment: $ => token(seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/')),
  },
});

function commaSep1(rule) {
  return seq(rule, repeat(seq(',', rule)));
}
