use std::str::FromStr;

use winnow::combinator::seq;
use winnow::prelude::*;
use winnow::token::take_while;
use winnow::PResult;

#[allow(unused)]
#[derive(Debug)]
struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

fn main() {
    let color = Color::from_str("#ff00ff").unwrap();
    println!("{:#?}", color);
}

impl FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hex_color.parse(s).map_err(|e| e.to_string())
    }
}

fn hex_color(input: &mut &str) -> PResult<Color> {
    seq!(Color {
        _: '#',
        red: hex_primary,
        green: hex_primary,
        blue: hex_primary
    })
    .parse_next(input)
}

fn hex_primary(input: &mut &str) -> PResult<u8> {
    take_while(2, |c: char| c.is_ascii_hexdigit())
        .try_map(|input| u8::from_str_radix(input, 16))
        .parse_next(input)
}
