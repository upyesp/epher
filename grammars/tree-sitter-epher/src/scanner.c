// The epher external scanner: one job, faithfully.
//
// The language's unit suffix is adjacency-sensitive: `2 m` and `2m`
// are the quantity 2 metres, but `2\nm` is two statements, because a
// newline is a statement separator (reference sections 4 and 11).
// tree-sitter treats newlines as whitespace extras, so a pure grammar
// cannot tell `2 m` from `2\nm` - this scanner can.
//
// The scanner is consulted only where the grammar expects a unit
// (after a number, or as the target of `in` / `->`). It accepts the
// name only if it sits on the same line as the number, separated by
// nothing or by spaces and tabs, and it takes letters and underscores
// only: the real parser rejects digit-bearing suffixes (`2 m3` is a
// parse error). When no unit is there, it returns false and the
// parser falls back to the number-only reading.

#include "tree_sitter/parser.h"

#include <stdbool.h>
#include <stdint.h>

enum TokenType {
  UNIT_SUFFIX,
};

void *tree_sitter_epher_external_scanner_create(void) { return NULL; }
void tree_sitter_epher_external_scanner_destroy(void *payload) { (void)payload; }
unsigned tree_sitter_epher_external_scanner_serialize(void *payload, char *buffer) {
  (void)payload;
  (void)buffer;
  return 0;
}
void tree_sitter_epher_external_scanner_deserialize(void *payload, const char *buffer, unsigned length) {
  (void)payload;
  (void)buffer;
  (void)length;
}

static bool is_unit_start(int32_t c) {
  return (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z') || c == '_';
}

static bool is_unit_char(int32_t c) {
  return is_unit_start(c);
}

// The reserved words (crates/core KEYWORDS, frozen) can never be
// units: `1 to 5` is a range, `2 in cm` is a conversion. The scanner
// reads the whole word and compares before committing.
static const char *KEYWORDS[] = {
  "and", "break", "const", "continue", "def", "do", "else", "end",
  "for", "if", "in", "not", "or", "return", "solve", "step", "then",
  "to", "while", "xor",
};

bool tree_sitter_epher_external_scanner_scan(void *payload, TSLexer *lexer, const bool *valid_symbols) {
  (void)payload;
  if (!valid_symbols[UNIT_SUFFIX]) {
    return false;
  }

  // Same line only: spaces and tabs bridge the number and its unit,
  // a newline would be a statement separator.
  while (lexer->lookahead == ' ' || lexer->lookahead == '\t') {
    lexer->advance(lexer, true);
  }
  if (!is_unit_start(lexer->lookahead)) {
    return false;
  }

  char word[32];
  unsigned length = 0;
  while (is_unit_char(lexer->lookahead) && length < sizeof(word) - 1) {
    word[length++] = (char)lexer->lookahead;
    lexer->advance(lexer, false);
  }
  word[length] = '\0';

  for (unsigned k = 0; k < sizeof(KEYWORDS) / sizeof(KEYWORDS[0]); k++) {
    const char *kw = KEYWORDS[k];
    unsigned i = 0;
    while (kw[i] != '\0' && word[i] == kw[i]) {
      i++;
    }
    if (kw[i] == '\0' && i == length) {
      // A reserved word: not a unit. Returning false resets the
      // lexer, so `1 to 5` still reads as a range.
      return false;
    }
  }

  lexer->result_symbol = UNIT_SUFFIX;
  return true;
}
