use winnow::{ascii::digit1, PResult, Parser};

fn parse_digits(input: &mut &str) -> PResult<usize> {
    digit1.parse_to().parse_next(input)
}

fn main() {
    let mut input = "1024 Hello";

    let output = parse_digits.parse_next(&mut input).unwrap();
    assert_eq!(input, " Hello");
    assert_eq!(output, 1024);

    assert!(parse_digits(&mut "Z").is_err());
}
