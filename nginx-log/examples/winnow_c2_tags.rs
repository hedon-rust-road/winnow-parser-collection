use winnow::{PResult, Parser};

fn parse_prefix<'a>(input: &mut &'a str) -> PResult<&'a str> {
    // let expected = "0x";
    // ①
    // if input.len() < expected.len() {
    //     return Err(ErrMode::from_error_kind(input, ErrorKind::Slice));
    // }
    // let actual = input.next_slice(expected.len());
    // if actual != expected {
    //     return Err(ErrMode::from_error_kind(input, ErrorKind::Verify));
    // }

    // ②
    // let actual = literal(expected).parse_next(input)?;

    // ③
    let actual = "0x".parse_next(input)?;
    Ok(actual)
}

fn main() {
    let mut input = "0x1a2b Hello";
    let output = parse_prefix.parse_next(&mut input).unwrap();
    assert_eq!(input, "1a2b Hello");
    assert_eq!(output, "0x");

    assert!(parse_prefix.parse_next(&mut "0o123").is_err())
}
