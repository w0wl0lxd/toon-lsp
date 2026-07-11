#include <tree_sitter/parser.h>
#include <stdbool.h>
#include <stdint.h>
#include <string.h>
#include <stdlib.h>

enum TokenType {
  INDENT,
  DEDENT,
  NEWLINE,
};

typedef struct {
  uint16_t indent_lengths[256];
  uint16_t indent_count;
} Scanner;

void *tree_sitter_toon_external_scanner_create() {
  Scanner *s = (Scanner *)calloc(1, sizeof(Scanner));
  s->indent_lengths[0] = 0;
  s->indent_count = 1;
  return s;
}

void tree_sitter_toon_external_scanner_destroy(void *payload) {
  free(payload);
}

unsigned tree_sitter_toon_external_scanner_serialize(void *payload, char *buffer) {
  Scanner *s = (Scanner *)payload;
  unsigned size = sizeof(uint16_t) * s->indent_count;
  if (size > 512) size = 512;
  memcpy(buffer, s->indent_lengths, size);
  return size;
}

void tree_sitter_toon_external_scanner_deserialize(void *payload, const char *buffer, unsigned length) {
  Scanner *s = (Scanner *)payload;
  if (length == 0) {
    s->indent_count = 1;
    s->indent_lengths[0] = 0;
    return;
  }
  unsigned data_len = length > 511 ? 512 : length;
  s->indent_count = data_len / sizeof(uint16_t);
  if (s->indent_count > 256) s->indent_count = 256;
  memcpy(s->indent_lengths, buffer, data_len);
}

static void skip_whitespace(TSLexer *lexer) {
  while (lexer->lookahead == ' ' || lexer->lookahead == '\t') {
    lexer->advance(lexer, true);
  }
}

static bool scan_newline(TSLexer *lexer, Scanner *s) {
  skip_whitespace(lexer);

  if (lexer->lookahead == '#') {
    while (lexer->lookahead != '\n' && lexer->lookahead != '\0') {
      lexer->advance(lexer, true);
    }
    skip_whitespace(lexer);
  }

  if (lexer->lookahead == '\r') {
    lexer->advance(lexer, true);
    if (lexer->lookahead == '\n') {
      lexer->advance(lexer, true);
    }
    lexer->result_symbol = NEWLINE;
    return true;
  }

  if (lexer->lookahead == '\n') {
    lexer->advance(lexer, true);
    lexer->result_symbol = NEWLINE;
    return true;
  }

  return false;
}

static bool scan_indent_or_dedent(TSLexer *lexer, Scanner *s) {
  skip_whitespace(lexer);

  while (lexer->lookahead == '#') {
    while (lexer->lookahead != '\n' && lexer->lookahead != '\0') {
      lexer->advance(lexer, false);
    }
    skip_whitespace(lexer);
  }

  while (lexer->lookahead == '\r' || lexer->lookahead == '\n') {
    if (lexer->lookahead == '\r') {
      lexer->advance(lexer, true);
    }
    if (lexer->lookahead == '\n') {
      lexer->advance(lexer, true);
    }
    skip_whitespace(lexer);
    while (lexer->lookahead == '#') {
      while (lexer->lookahead != '\n' && lexer->lookahead != '\0') {
        lexer->advance(lexer, false);
      }
      skip_whitespace(lexer);
    }
  }

  if (lexer->lookahead == '\0') {
    while (s->indent_count > 1) {
      s->indent_count--;
      lexer->result_symbol = DEDENT;
      return true;
    }
    return false;
  }

  uint16_t indent = lexer->get_column(lexer);
  uint16_t current = s->indent_lengths[s->indent_count - 1];

  if (indent < current) {
    s->indent_count--;
    lexer->result_symbol = DEDENT;
    return true;
  }

  if (indent > current) {
    if (s->indent_count < 256) {
      s->indent_lengths[s->indent_count++] = indent;
    }
    lexer->result_symbol = INDENT;
    return true;
  }

  return false;
}

bool tree_sitter_toon_external_scanner_scan(void *payload, TSLexer *lexer, const bool *valid_symbols) {
  Scanner *s = (Scanner *)payload;

  if (valid_symbols[NEWLINE]) {
    return scan_newline(lexer, s);
  }

  if (valid_symbols[INDENT] || valid_symbols[DEDENT]) {
    return scan_indent_or_dedent(lexer, s);
  }

  return false;
}
