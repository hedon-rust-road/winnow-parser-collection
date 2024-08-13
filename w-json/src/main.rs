use std::collections::HashMap;

use winnow::{
    ascii::{digit1, multispace0},
    combinator::{alt, delimited, opt, separated, separated_pair, trace},
    error::{ContextError, ErrMode, ParserError},
    stream::{AsChar, Stream, StreamIsPartial},
    token::take_until,
    PResult, Parser,
};

#[derive(Debug, Clone, PartialEq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(Num),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

#[derive(Debug, Clone, PartialEq)]
enum Num {
    Int(i64),
    Float(f64),
}

fn main() -> PResult<()> {
    let s = r#"{
        "name": "John Doe",
        "age": 30,
        "is_student": false,
        "marks": [90.0, -80.0, 85.1],
        "address": {
            "city": "New York",
            "zip": 10001
        }
    }"#;

    let input = &mut (&*s);
    let v = parse_json(input)?;
    println!("{:#?}", v);
    Ok(())
}

fn parse_json(input: &str) -> PResult<JsonValue> {
    let input = &mut (&*input);
    parse_value(input)
}

fn parse_null(input: &mut &str) -> PResult<()> {
    "null".value(()).parse_next(input)
}

fn parse_bool(input: &mut &str) -> PResult<bool> {
    alt(("true", "false")).parse_to().parse_next(input)
}

// 1.sign
// 2.digits
// 3. .digits
fn parse_number(input: &mut &str) -> PResult<Num> {
    let sign = opt("-").map(|s| s.is_some()).parse_next(input)?;
    let int = digit1.parse_to::<i64>().parse_next(input)?;
    let ret: Result<(), ErrMode<ContextError>> = ".".value(()).parse_next(input);
    if ret.is_ok() {
        let digits = digit1.parse_to::<i64>().parse_next(input)?;
        let v = format!("{int}.{digits}").parse::<f64>().unwrap();
        Ok(if sign {
            Num::Float(-v as _)
        } else {
            Num::Float(v as _)
        })
    } else {
        Ok(if sign { Num::Int(-int) } else { Num::Int(int) })
    }
}

fn parse_string(input: &mut &str) -> PResult<String> {
    let ret = delimited('"', take_until(0.., '"'), '"').parse_next(input)?;
    Ok(ret.to_string())
}

fn parse_array(input: &mut &str) -> PResult<Vec<JsonValue>> {
    let sep1 = sep_with_space('[');
    let sep2 = sep_with_space(']');
    let sep_comma = sep_with_space(',');
    let parse_values = separated(0.., parse_value, sep_comma);
    delimited(sep1, parse_values, sep2).parse_next(input)
}

pub fn sep_with_space<Input, Output, Error, ParseNext>(
    mut parser: ParseNext,
) -> impl Parser<Input, (), Error>
where
    Input: Stream + StreamIsPartial,
    <Input as Stream>::Token: AsChar + Clone,
    Error: ParserError<Input>,
    ParseNext: Parser<Input, Output, Error>,
{
    trace("sep_with_space", move |input: &mut Input| {
        multispace0.parse_next(input)?;
        parser.parse_next(input)?;
        multispace0.parse_next(input)?;
        Ok(())
    })
}

fn parse_object(input: &mut &str) -> PResult<HashMap<String, JsonValue>> {
    let sep1 = sep_with_space('{');
    let sep2 = sep_with_space('}');
    let sep_comma = sep_with_space(',');
    let sep_colon = sep_with_space(':');

    let parse_kv_pair = separated_pair(parse_string, sep_colon, parse_value);
    let parse_kv = separated(1.., parse_kv_pair, sep_comma);
    delimited(sep1, parse_kv, sep2).parse_next(input)
}

fn parse_value(input: &mut &str) -> PResult<JsonValue> {
    alt((
        parse_null.value(JsonValue::Null),
        parse_bool.map(JsonValue::Bool),
        parse_number.map(JsonValue::Number),
        parse_string.map(JsonValue::String),
        parse_array.map(JsonValue::Array),
        parse_object.map(JsonValue::Object),
    ))
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null_should_work() -> PResult<()> {
        let mut input = "null";
        parse_null(&mut input)?;
        Ok(())
    }

    #[test]
    fn test_parse_bool_should_work() -> PResult<()> {
        let mut input = "true";
        let ret = parse_bool(&mut input)?;
        assert!(ret);

        let mut input = "false";
        let ret = parse_bool(&mut input)?;
        assert!(!ret);
        Ok(())
    }

    #[test]
    fn test_parse_number_should_work() -> PResult<()> {
        let mut input = "123";
        let ret = parse_number(&mut input)?;
        assert_eq!(ret, Num::Int(123));

        let mut input = "-123";
        let ret = parse_number(&mut input)?;
        assert_eq!(ret, Num::Int(-123));

        let mut input = "123.456";
        let ret = parse_number(&mut input)?;
        assert_eq!(ret, Num::Float(123.456));

        let mut input = "-123.456";
        let ret = parse_number(&mut input)?;
        assert_eq!(ret, Num::Float(-123.456));

        let mut input = "0";
        let ret = parse_number(&mut input)?;
        assert_eq!(ret, Num::Int(0));

        Ok(())
    }

    #[test]
    fn test_parse_string_should_work() -> PResult<()> {
        let mut input = r#""hello""#;
        let ret = parse_string(&mut input)?;
        assert_eq!(ret, "hello");

        let mut input = r#""""#;
        let ret = parse_string(&mut input)?;
        assert_eq!(ret, "");
        Ok(())
    }

    #[test]
    fn test_parse_array_should_work() -> PResult<()> {
        let mut input = r#"[1, 2, 3]"#;
        let ret = parse_array(&mut input)?;
        assert_eq!(
            ret,
            vec![
                JsonValue::Number(Num::Int(1)),
                JsonValue::Number(Num::Int(2)),
                JsonValue::Number(Num::Int(3))
            ]
        );

        let mut input = r#"[1, "hello", null]"#;
        let ret = parse_array(&mut input)?;
        assert_eq!(
            ret,
            vec![
                JsonValue::Number(Num::Int(1)),
                JsonValue::String("hello".to_string()),
                JsonValue::Null,
            ]
        );

        Ok(())
    }

    #[test]
    fn test_parse_object_should_work() -> PResult<()> {
        let mut input = r#"{"a": 1, "b": "hello", "c": null}"#;
        let ret = parse_object(&mut input)?;
        assert_eq!(
            ret,
            vec![
                ("a".to_string(), JsonValue::Number(Num::Int(1))),
                ("b".to_string(), JsonValue::String("hello".to_string())),
                ("c".to_string(), JsonValue::Null),
            ]
            .into_iter()
            .map(|v| (v.0, v.1))
            .collect::<std::collections::HashMap<_, _>>()
        );
        Ok(())
    }
}
