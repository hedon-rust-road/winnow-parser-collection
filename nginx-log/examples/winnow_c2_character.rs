use winnow::{ascii::hex_digit1, PResult, Parser};

fn parse_digits<'s>(input: &mut &'s str) -> PResult<&'s str> {
    // ①
    // one_of(('0'..='9', 'a'..='f', 'A'..='F')).parse_next(input)

    // ②
    // take_while(1.., ('0'..='9', 'a'..='f', 'A'..='F')).parse_next(input)

    // ③ Recognizes one or more ASCII hexadecimal numerical characters:
    // `'0'..='9'`, `'A'..='F'`, 'a'..='f'`
    hex_digit1.parse_next(input)
}

fn main() {
    let mut input = "1a2b Hello";

    let output = parse_digits.parse_next(&mut input).unwrap();
    // ①
    // assert_eq!(input, "a2b Hello");
    // assert_eq!(output, '1');

    // ②/③
    assert_eq!(input, " Hello");
    assert_eq!(output, "1a2b");

    assert!(parse_digits.parse_next(&mut "Z").is_err())
}
