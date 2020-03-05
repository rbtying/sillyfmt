module.exports = grammar({
  name: 'sillyfmt',
  rules: {
    source_file: $ => repeat($._expression),

    _expression: $ => prec(-10, choice(
      $._nonseq_expr,
      $.comma_delimited_sequence,
    )),

    _nonseq_expr: $ => choice(
      $.container,
      $.time,
      $.binary_op,
      $.symbol,
      $.text,
    ),

    binary_op: $ => prec.left(10, seq($._nonseq_expr, $.symbol, $._nonseq_expr)),

    symbol: $ => prec(-100, choice(
      '===',
      '<=>',
      '=>',
      '->',
      '<=',
      '>=',
      '==',
      '=',
      '<',
      '>',
      ':',
      '-',
      '+',
    )),

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
      repeat1($._nonseq_expr),
      repeat1(seq(
        ',',
        prec.right(repeat1($._nonseq_expr)))),
    )),

    text: $ => prec.left(-50, /[^()\[\]{},:=<>\s]+/),
    time: $ => /([0-1]?[0-9]|[2][0-3]):([0-5][0-9])(:[0-5][0-9])?/,
  },
  conflicts: $ => [
    [$.binary_op],
  ]
});
