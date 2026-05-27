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
  bool has_newline;
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
  if (size < 512) buffer[size] = s->has_newline ? 1 : 0;
  return size < 512 ? size + 1 : size;
}

void tree_sitter_toon_external_scanner_deserialize(void *payload, const char *buffer, unsigned length) {
  Scanner *s = (Scanner *)payload;
  if (length == 0) {
    s->indent_count = 1;
    s->indent_lengths[0] = 0;
    s->has_newline = true;
    return;
  }
  unsigned data_len = length > 0 && length <= 513 ? length - 1 : 0;
  if (data_len > 512) data_len = 512;
  s->indent_count = data_len / sizeof(uint16_t);
  if (s->indent_count > 256) s->indent_count = 256;
  memcpy(s->indent_lengths, buffer, data_len);
  if (length > data_len) {
    s->has_newline = buffer[data_len] != 0;
  }
}

static void skip_comment(TSLexer *lexer) {
  while (lexer->lookahead != '\n' && lexer->lookahead != '\0') {
    lexer->advance(lexer, false);
  }
}

static void skip_whitespace(TSLexer *lexer) {
  while (lexer->lookahead == ' ' || lexer->lookahead == '\t') {
    lexer->advance(lexer, true);
  }
}

bool tree_sitter_toon_external_scanner_scan(void *payload, TSLexer *lexer, const bool *valid_symbols) {
  Scanner *s = (Scanner *)payload;

  if (valid_symbols[NEWLINE] && s->has_newline) {
    s->has_newline = false;
    lexer->result_symbol = NEWLINE;
    return true;
  }

  if (valid_symbols[INDENT] || valid_symbols[DEDENT]) {
    if (s->has_newline) {
      s->has_newline = false;
    } else if (!valid_symbols[NEWLINE]) {
      return false;
    }

    skip_whitespace(lexer);

    if (lexer->lookahead == '#') {
      skip_comment(lexer);
      skip_whitespace(lexer);
    }

    if (lexer->lookahead == '\n' || lexer->lookahead == '\r') {
      lexer->advance(lexer, true);
      s->has_newline = true;
      if (valid_symbols[NEWLINE]) {
        lexer->result_symbol = NEWLINE;
        return true;
      }
      return false;
    }

    if (lexer->lookahead == '\0') {
      if (s->indent_count > 1) {
        s->indent_count--;
        lexer->result_symbol = DEDENT;
        return true;
      }
      s->has_newline = false;
      return false;
    }

    uint16_t indent = lexer->get_column(lexer);

    if (indent < s->indent_lengths[s->indent_count - 1]) {
      s->indent_count--;
      lexer->result_symbol = DEDENT;
      return true;
    }

    if (indent == s->indent_lengths[s->indent_count - 1]) {
      lexer->result_symbol = NEWLINE;
      return true;
    }

    if (indent > s->indent_lengths[s->indent_count - 1]) {
      if (s->indent_count < 256) {
        s->indent_lengths[s->indent_count++] = indent;
      }
      lexer->result_symbol = INDENT;
      return true;
    }
  }

  if (valid_symbols[NEWLINE]) {
    if (lexer->lookahead == '\n' || lexer->lookahead == '\r') {
      lexer->advance(lexer, true);
      lexer->result_symbol = NEWLINE;
      return true;
    }
    if (lexer->lookahead == '\0') {
      return false;
    }
  }

  return false;
}
