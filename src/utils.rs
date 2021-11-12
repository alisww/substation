use nom::{
    branch::alt,
    bytes::complete::take_while_m_n,
    character::complete::{i64 as parsei64, multispace0},
    combinator::{map, map_res},
    error::ParseError,
    number::complete::double,
    sequence::delimited,
    IResult,
};

pub fn decimal_or_float(input: &str) -> IResult<&str, f64> {
    alt((double, map(parsei64, |v: i64| v as f64)))(input)
}

pub fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(input, 16)
}

pub fn is_hex_digit(c: char) -> bool {
    c.is_digit(16)
}

pub fn hex_primary(input: &str) -> IResult<&str, u8> {
    map_res(take_while_m_n(2, 2, is_hex_digit), from_hex)(input)
}

pub fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}
