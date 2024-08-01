use winnow::{PResult, Parser};

fn parse_prefix(input: &mut &str) -> PResult<char> {
    // let c = any.verify(|c| *c == '0').parse_next(input)?;
    let c = '0'.parse_next(input)?;
    Ok(c)
}

fn main() {
    let mut input = "0x1a2b Hello";

    let output = parse_prefix.parse_next(&mut input).unwrap();

    assert_eq!(input, "x1a2b Hello");
    assert_eq!(output, '0');

    assert!(parse_prefix.parse_next(&mut "d").is_err());
}
