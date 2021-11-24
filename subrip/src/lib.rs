use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, line_ending, not_line_ending, one_of, u64 as decimal},
    combinator::{eof, verify},
    multi::{many0, many1},
    sequence::{separated_pair, terminated, tuple},
    IResult,
};
use parsing_utils::*;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Entry {
    index: u32,
    start: Duration,
    end: Duration,
    text: String,
}

//  hours:minutes:seconds,milliseconds
fn duration(input: &str) -> IResult<&str, Duration> {
    let (input, (hours, mins, secs, millis)) = tuple((
        terminated(decimal, char(':')),
        terminated(decimal, char(':')),
        terminated(decimal, one_of(",")),
        decimal,
    ))(input)?;
    Ok((
        input,
        Duration::from_millis(hours * 3_600_000 + mins * 60_000 + secs * 1_000 + millis),
    ))
}

pub fn entry(input: &str) -> IResult<&str, Entry> {
    let (input, index) = terminated(decimal, line_ending)(input.trim_start())?;
    let (input, (start, end)) = terminated(
        separated_pair(duration, ws(tag("-->")), duration),
        line_ending,
    )(input)?;
    let (input, line) = many1(terminated(
        verify(not_line_ending, |s: &str| !s.is_empty()),
        alt((eof, line_ending)),
    ))(input)?;

    Ok((
        input,
        Entry {
            index: index as u32,
            start,
            end,
            text: line.join("\n"),
        },
    ))
}

pub fn entries(input: &str) -> IResult<&str, Vec<Entry>> {
    many0(entry)(input)
}
