module.exports = grammar({
  name: 'sillyfmt',
  rules: {
    source_file: $ => repeat($._expression),

    _expression: $ => prec.left(5, choice(
      $._nonseq_expr,
      $.comma_delimited_sequence,
    )),

    _nonseq_expr: $ => choice(
      $.container,
      $.time,
      $.nonsymbol,
      $.binary_op,
      $.symbol,
      $.conflicting_symbol,
      $.text,
    ),

    binary_op: $ => choice(
      prec.left(10, seq($.symbol, alias($.binary_op, 'subbinary_op'))),
      prec.left(10, seq($.symbol, $._nonseq_expr)),
      prec.left(5, seq($.conflicting_symbol, alias($.binary_op, 'subbinary_op'))),
      prec.left(5, seq($.conflicting_symbol, $._nonseq_expr)),
    ),

    nonsymbol: $ => choice(
      '::',
    ),

    symbol: $ => choice(
      '===',
      '<=>',
      '=>',
      '->',
      '<=',
      '>=',
      '==',
      '=',
      ':',
      '-',
      '+',
    ),
    conflicting_symbol: $ => choice(
      '<',
      '>',
    ),

    container: $ => choice(
      seq(
        field('open', '('),
        field('contents', repeat($._expression)),
        field('close', ')')
      ),
      seq(
        field('open', '['),
        field('contents', repeat($._expression)),
        field('close', ']')
      ),
      seq(
        field('open', '{'),
        field('contents', repeat($._expression)),
        field('close', '}')
      ),
      seq(
        field('open', '<'),
        field('contents', repeat($._expression)),
        field('close', '>')
      ),
    ),

    comma_delimited_sequence: $ => prec.right(20, seq(
      repeat1(prec.right($._nonseq_expr)),
      repeat1(prec.right(seq(
        ',',
        prec.right(repeat1(prec.right($._nonseq_expr)))))),
    )),

    text: $ => prec.left(-50, /[^()\[\]{},:=<>\s][^()\[\]{},:=<>]*/),
    time: $ => /([0-1]?[0-9]|[2][0-3]):([0-5][0-9])(:[0-5][0-9])?/,
  },
  conflicts: $ => [
    [$.conflicting_symbol, $.container]
  ]
});
