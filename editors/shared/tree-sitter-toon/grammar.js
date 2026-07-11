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
    /[ \t\r\n]+/,
    $.comment,
  ],

  conflicts: $ => [
    [$.pair],
  ],

  rules: {
    document: $ => repeat($._line),

    _line: $ => seq(
      choice($.pair, $.array_item),
      optional($.newline),
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

    key: $ => token(prec(-1, /[\w][\w-]*/)),

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

    double_quoted_string: $ => token(prec(2, /"([^"\\]|\\(["'\\bfnrt]|u[0-9A-Fa-f]{4}))*"/)),

    single_quoted_string: $ => token(prec(2, /'([^'\\]|\\(["'\\bfnrt]|u[0-9A-Fa-f]{4}))*'/)),

    number: $ => {
      const decimal = /[0-9]+/;
      const signed_integer = seq(optional('-'), decimal);
      const exponent = seq(/[eE]/, optional(/[+-]/), decimal);
      const decimal_literal = choice(
        seq(signed_integer, '.', decimal, optional(exponent)),
        seq(signed_integer, exponent),
        signed_integer,
      );
      return token(prec(2, decimal_literal));
    },

    boolean: $ => token(prec(2, choice('true', 'false'))),

    null: $ => token(prec(2, 'null')),

    unquoted_string: $ => token(prec(1, /[^\s#\[\]|,][^#\[\]|,\n]*/)),

    comment: $ => /#.*/,
  },
});
