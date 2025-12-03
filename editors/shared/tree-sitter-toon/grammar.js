/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: 'toon',

  extras: $ => [
    /\s/,
    $.comment,
  ],

  rules: {
    document: $ => repeat($._statement),

    _statement: $ => choice(
      $.pair,
      $.array_item,
      $.table_row,
    ),

    pair: $ => seq(
      field('key', $.key),
      ':',
      optional(field('value', $._value)),
    ),

    key: $ => /[\w][\w-]*/,

    _value: $ => choice(
      $.object,
      $.array,
      $.inline_array,
      $.string,
      $.number,
      $.boolean,
      $.null,
      $.unquoted_string,
    ),

    object: $ => prec.right(seq(
      repeat1($.pair),
    )),

    array: $ => prec.right(repeat1($.array_item)),

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

    table_row: $ => seq(
      '|',
      repeat(seq($.table_cell, '|')),
      optional($.table_cell),
    ),

    table_cell: $ => /(?:[^|\n\\]|\\.)+/,

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
