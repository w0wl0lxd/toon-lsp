/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: 'toon',

  externals: $ => [
    $.indent,
    $.dedent,
    $.newline,
  ],

  extras: $ => [
    /[ \t]+/,
    $.comment,
  ],

  rules: {
    document: $ => repeat($._line),

    _line: $ => choice(
      $.pair,
      $.array_item,
    ),

    pair: $ => seq(
      field('key', $.key),
      ':',
      optional(field('value', $._pair_value)),
    ),

    _pair_value: $ => choice(
      $._value,
      seq($.newline, $.indent, repeat($._line), $.dedent),
    ),

    key: $ => /[\w][\w-]*/,

    _value: $ => choice(
      $.inline_array,
      $.string,
      $.number,
      $.boolean,
      $.null,
      $.unquoted_string,
    ),

    array_item: $ => seq(
      '-',
      optional($._value),
    ),

    inline_array: $ => seq(
      '[',
      optional(seq(
        $._value,
        repeat(seq(',', $._value)),
        optional(','),
      )),
      ']',
    ),

    string: $ => choice(
      $.double_quoted_string,
      $.single_quoted_string,
    ),

    double_quoted_string: $ => seq(
      '"',
      repeat(choice(
        $.escape_sequence,
        /[^"\\]+/,
      )),
      '"',
    ),

    single_quoted_string: $ => seq(
      "'",
      repeat(choice(
        $.escape_sequence,
        /[^'\\]+/,
      )),
      "'",
    ),

    escape_sequence: $ => /\\(?:["'\\bfnrt]|u[0-9A-Fa-f]{4})/,

    number: $ => {
      const decimal = /[0-9]+/;
      const signed_integer = seq(optional('-'), decimal);
      const exponent = seq(/[eE]/, optional(/[+-]/), decimal);
      const decimal_literal = choice(
        seq(signed_integer, '.', decimal, optional(exponent)),
        seq(signed_integer, exponent),
        signed_integer,
      );
      return token(decimal_literal);
    },

    boolean: $ => choice('true', 'false'),

    null: $ => 'null',

    unquoted_string: $ => /[^\s#\[\]|,][^#\[\]|,\n]*/,

    comment: $ => /#.*/,
  },
});
