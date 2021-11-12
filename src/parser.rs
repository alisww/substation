use crate::{utils::*, *};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until, take_while},
    character::complete::{char, line_ending, not_line_ending, one_of, space1, u64 as decimal},
    combinator::{map, not, opt, peek},
    multi::{many0, many_m_n, separated_list0},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};
use std::collections::HashMap;
use std::time::Duration;

fn full_color(input: &str) -> IResult<&str, Color> {
    let (input, _) = tag("&H")(input)?;
    let (input, (alpha, blue, green, red)) =
        tuple((hex_primary, hex_primary, hex_primary, hex_primary))(input)?;
    Ok((
        input,
        Color {
            alpha: Some(alpha),
            red,
            green,
            blue,
        },
    ))
}

fn partial_color(input: &str) -> IResult<&str, Color> {
    let (input, (red, green, blue)) = tuple((hex_primary, hex_primary, hex_primary))(input)?;
    Ok((
        input,
        Color {
            alpha: None,
            blue,
            green,
            red,
        },
    ))
}

// Hrs:Mins:Secs:hundredths or Hrs:Mins:Secs.hundredths
fn duration(input: &str) -> IResult<&str, Duration> {
    let (input, (hours, mins, secs, hundredths)) = tuple((
        terminated(decimal, char(':')),
        terminated(decimal, char(':')),
        terminated(decimal, one_of(":.")),
        decimal,
    ))(input)?;
    Ok((
        input,
        Duration::from_millis(hours * 3_600_000 + mins * 60_000 + secs * 1_000 + hundredths * 10),
    ))
}

fn draw_move(input: &str) -> IResult<&str, Vec<DrawingCommand>> {
    use DrawingCommand::*;

    let (input, coords) =
        preceded(tag("m "), separated_list0(space1, decimal_or_float))(input.trim_start())?;
    Ok((
        input,
        coords
            .chunks_exact(2)
            .map(|s| Move { x: s[0], y: s[1] })
            .collect(),
    ))
}

fn draw_move_without_closing(input: &str) -> IResult<&str, Vec<DrawingCommand>> {
    use DrawingCommand::*;

    let (input, coords) =
        preceded(tag("n "), separated_list0(space1, decimal_or_float))(input.trim_start())?;
    Ok((
        input,
        coords
            .chunks_exact(2)
            .map(|s| MoveWithoutClosing { x: s[0], y: s[1] })
            .collect(),
    ))
}

fn draw_line(input: &str) -> IResult<&str, Vec<DrawingCommand>> {
    use DrawingCommand::*;

    let (input, coords) =
        preceded(tag("l "), separated_list0(space1, decimal_or_float))(input.trim_start())?;
    Ok((
        input,
        coords
            .chunks_exact(2)
            .map(|s| Line { x: s[0], y: s[1] })
            .collect(),
    ))
}

fn draw_bezier(input: &str) -> IResult<&str, Vec<DrawingCommand>> {
    use DrawingCommand::*;

    let (input, coords) =
        preceded(tag("b "), separated_list0(space1, decimal_or_float))(input.trim_start())?;
    Ok((
        input,
        coords
            .chunks_exact(6)
            .map(|s| Bezier {
                a_x: s[0],
                a_y: s[1],
                b_x: s[2],
                b_y: s[3],
                c_x: s[4],
                c_y: s[5],
            })
            .collect(),
    ))
}

fn draw_bspline(input: &str) -> IResult<&str, Vec<DrawingCommand>> {
    use DrawingCommand::*;

    let (input, coords) =
        preceded(tag("s "), separated_list0(space1, decimal_or_float))(input.trim_start())?;
    Ok((
        input,
        vec![UniformSpline(
            coords.chunks_exact(2).map(|s| (s[0], s[1])).collect(),
        )],
    ))
}

fn draw_extend_bspline(input: &str) -> IResult<&str, Vec<DrawingCommand>> {
    use DrawingCommand::*;

    let (input, coords) =
        preceded(tag("p "), separated_list0(space1, decimal_or_float))(input.trim_start())?;
    Ok((
        input,
        coords
            .chunks_exact(2)
            .map(|s| ExtendBspline { x: s[0], y: s[1] })
            .collect(),
    ))
}

fn draw_close_bspline(input: &str) -> IResult<&str, Vec<DrawingCommand>> {
    use DrawingCommand::*;
    let (input, _) = char('c')(input.trim_start())?;
    Ok((input, vec![CloseBspline]))
}

fn drawing(input: &str) -> IResult<&str, Vec<DrawingCommand>> {
    map(
        many0(alt((
            draw_move,
            draw_move_without_closing,
            draw_line,
            draw_bezier,
            draw_bspline,
            draw_extend_bspline,
            draw_close_bspline,
        ))),
        |v: Vec<Vec<DrawingCommand>>| v.concat(),
    )(input.trim_start())
}

fn function(input: &str) -> IResult<&str, StyleOverride> {
    let (input, kind) = alt((
        tag("move"),
        tag("pos"),
        tag("fade"),
        tag("clip"),
        tag("fad"),
        tag("org"),
        tag("t"),
    ))(input)?;

    // if kind == "clip" {
    // } else {
    let (input, arg_string) = delimited(char('('), is_not(")"), char(')'))(input)?;

    match kind {
        "clip" => {
            if let (_, Some((a_x, a_y, b_x, b_y))) = opt(tuple((
                decimal_or_float,
                preceded(ws(char(',')), decimal_or_float),
                preceded(ws(char(',')), decimal_or_float),
                preceded(ws(char(',')), decimal_or_float),
            )))(arg_string)?
            {
                Ok((input, StyleOverride::Clip { a_x, a_y, b_x, b_y }))
            } else if arg_string.is_empty() {
                Ok((input, StyleOverride::EmptyClip))
            } else {
                let (arg_string, idx) =
                    opt(terminated(decimal_or_float, ws(char(','))))(arg_string)?;
                let (_, drawings) = drawing(arg_string)?;
                Ok((input, StyleOverride::ClipToDrawing(idx, drawings)))
            }
        }
        "t" => {
            let (arg_string, start) = opt(decimal)(arg_string)?;
            let (arg_string, end) = opt(preceded(ws(char(',')), decimal))(arg_string)?;
            let (arg_string, acceleration) =
                opt(preceded(ws(char(',')), decimal_or_float))(arg_string)?;
            let (_arg_string, styles) =
                preceded(ws(char(',')), separated_list0(char(','), style))(arg_string)?;
            Ok((
                input,
                StyleOverride::Transition {
                    start: start.map(Duration::from_millis),
                    end: end.map(Duration::from_millis),
                    acceleration,
                    styles,
                },
            ))
        }
        "move" => {
            let (arg_string, (start_x, start_y, end_x, end_y)) = tuple((
                decimal_or_float,
                preceded(ws(char(',')), decimal_or_float),
                preceded(ws(char(',')), decimal_or_float),
                preceded(ws(char(',')), decimal_or_float),
            ))(arg_string)?;
            let (arg_string, start) = opt(preceded(ws(char(',')), decimal))(arg_string)?;
            let (_arg_string, end) = opt(preceded(ws(char(',')), decimal))(arg_string)?;
            Ok((
                input,
                StyleOverride::Move {
                    start_x,
                    start_y,
                    end_x,
                    end_y,
                    start: start.map(Duration::from_millis),
                    end: end.map(Duration::from_millis),
                },
            ))
        }
        "pos" => {
            let (_arg_string, (end_x, end_y)) =
                pair(decimal_or_float, preceded(ws(char(',')), decimal_or_float))(arg_string)?;
            Ok((
                input,
                StyleOverride::Move {
                    start_x: end_x,
                    start_y: end_y,
                    end_x,
                    end_y,
                    start: Some(Duration::from_millis(0)),
                    end: Some(Duration::from_millis(0)),
                },
            ))
        }
        "org" => {
            let (_arg_string, args) = separated_list0(ws(char(',')), decimal_or_float)(arg_string)?;
            Ok((
                input,
                StyleOverride::Origin {
                    x: args[0],
                    y: args[1],
                },
            ))
        }
        "fade" => {
            let (_arg_string, args) = separated_list0(ws(char(',')), decimal)(arg_string)?;
            Ok((
                input,
                StyleOverride::Fade {
                    starting_alpha: args[0] as u8,
                    middle_alpha: args[1] as u8,
                    ending_alpha: args[2] as u8,
                    start_time: Duration::from_millis(args[3]),
                    in_between_time: Duration::from_millis(args[4]),
                    late_time: Duration::from_millis(args[5]),
                    ending_time: Duration::from_millis(args[6]),
                },
            ))
        }
        "fad" => {
            let (_arg_string, args) = separated_list0(ws(char(',')), decimal)(arg_string)?;
            Ok((
                input,
                StyleOverride::FadeInAndOut {
                    fade_in_for: Duration::from_millis(args[0]),
                    fade_out_for: Duration::from_millis(args[1]),
                },
            ))
        }
        _ => unimplemented!(), // todo: not!
    }
}

fn bool_style(input: &str) -> IResult<&str, StyleOverride> {
    use StyleOverride::*;
    let (input, (kind, n)) = alt((
        pair(tag("be"), one_of("01")),
        pair(tag("i"), one_of("01")),
        pair(tag("u"), one_of("01")),
        pair(tag("s"), one_of("01")),
    ))(input)?;

    let n = n == '1';

    Ok((
        input,
        match kind {
            "be" => BlurEdges(n),
            "i" => Italic(n),
            "u" => Underline(n),
            "s" => StrikeOut(n),
            _ => unreachable!(),
        },
    ))
}

fn number_style(input: &str) -> IResult<&str, StyleOverride> {
    use StyleOverride::*;

    let (input, (kind, n)) = alt((
        pair(tag("frx"), decimal_or_float),
        pair(tag("fry"), decimal_or_float),
        pair(tag("frz"), decimal_or_float),
        pair(tag("fr"), decimal_or_float),
        pair(tag("fs"), decimal_or_float),
        pair(tag("fscx"), decimal_or_float),
        pair(tag("fscy"), decimal_or_float),
        pair(tag("fsp"), decimal_or_float),
        pair(tag("bord"), decimal_or_float),
        pair(tag("shad"), decimal_or_float),
        pair(tag("an"), decimal_or_float),
        pair(tag("a"), decimal_or_float),
        pair(tag("k"), decimal_or_float),
        pair(tag("q"), decimal_or_float),
        pair(tag("b"), decimal_or_float),
        pair(tag("p"), decimal_or_float),
    ))(input)?;

    Ok((
        input,
        match kind {
            "frx" => RotationX(n),
            "fry" => RotationY(n),
            "frz" | "fr" => RotationZ(n),
            "fs" => FontSize(n),
            "fscx" => ScaleX(n),
            "fscy" => ScaleY(n),
            "fsp" => LetterSpacing(n),
            "a" => Alignment(n),
            "an" => NumpadLayoutAlignment(n),
            "k" => KaraokeDuration(Duration::from_millis((n as u64) * 10)),
            "q" => WrappingStyle(n),
            "b" => Bold(n),
            "shad" => Shadow(n),
            "bord" => Border(n),
            "p" => DrawingMode(n),
            _ => unreachable!(),
        },
    ))
}

fn string_style(input: &str) -> IResult<&str, StyleOverride> {
    use StyleOverride::*;
    let (input, (_, name)) = pair(tag("fn"), take_until("\\"))(input)?;
    Ok((input, FontName(name.to_owned())))
}

fn color_style(input: &str) -> IResult<&str, StyleOverride> {
    use StyleOverride::*;
    let (input, idx) = opt(decimal)(input)?;
    let (input, color) = delimited(tag("c&H"), is_not("&"), char('&'))(input)?;
    let color = color
        .chars()
        .rev()
        .chain(['0'].into_iter().cycle())
        .take(6)
        .collect::<String>();
    let (_, color) = partial_color(&color).unwrap(); // TODO: don't unwrap
    Ok((input, Color(idx.unwrap_or(1), color)))
}

fn alpha_style(input: &str) -> IResult<&str, StyleOverride> {
    use StyleOverride::*;
    let (input, idx) = opt(decimal)(input)?;
    let (input, alpha) =
        delimited(alt((tag("alpha&H"), tag("a&"))), hex_primary, char('&'))(input)?;
    Ok((input, Alpha(idx.unwrap_or(1), alpha)))
}

fn fallback_style(input: &str) -> IResult<&str, StyleOverride> {
    use StyleOverride::*;
    let (input, what) = take_until("\\")(input)?;
    Ok((input, Other(what.to_owned())))
}

fn style(input: &str) -> IResult<&str, StyleOverride> {
    let (input, _) = char('\\')(input)?;
    alt((
        bool_style,
        string_style,
        color_style,
        alpha_style,
        number_style,
        function,
        fallback_style,
    ))(input)
}

fn style_override(input: &str) -> IResult<&str, TextSection> {
    map(
        delimited(
            char('{'),
            many0(alt((
                style,
                map(is_not("}"), |v: &str| StyleOverride::Other(v.to_owned())),
            ))),
            char('}'),
        ),
        TextSection::StyleOverride,
    )(input)
}

fn text(input: &str) -> IResult<&str, TextSection> {
    map(is_not("{"), |v: &str| TextSection::Text(v.to_owned()))(input)
}

fn text_line(input: &str) -> IResult<String, Vec<TextSection>> {
    let mut sections: Vec<TextSection> = Vec::new();
    let (input, sect) = alt((text, style_override))(input).map_err(|e| e.to_owned())?;
    let mut input = input.to_owned();
    sections.push(sect);

    while !input.is_empty() {
        if let TextSection::StyleOverride(styles) = sections.last().unwrap() {
            if styles.iter().any(|v| {
                if let StyleOverride::DrawingMode(x) = v {
                    x > &0.0
                } else {
                    false
                }
            }) {
                let (remaining, new_sect) = drawing(&input).map_err(|e| e.to_owned())?;
                input = remaining.to_owned();
                sections.push(TextSection::Drawing(new_sect));
                continue;
            }
        }

        let (ninput, nsect) = alt((text, style_override))(&input).map_err(|e| e.to_owned())?;
        input = ninput.to_owned();
        sections.push(nsect);
    }

    Ok((input, sections))
}

fn line_list(input: &str) -> IResult<&str, Vec<&str>> {
    map(
        separated_list0(
            char(','),
            take_while(|c| c != '\n' && c != '\r' && c != ','),
        ),
        |l: Vec<&str>| l.into_iter().map(|v| v.trim_start()).collect(),
    )(input)
}

fn setting(input: &str) -> IResult<&str, Option<(&str, &str)>> {
    peek(not(line_ending))(input)?;
    peek(not(char('[')))(input)?;
    let (input, comment) = opt(delimited(ws(char(';')), not_line_ending, line_ending))(input)?;
    if comment.is_some() {
        Ok((input, None))
    } else {
        let (input, (k, v)) = terminated(
            separated_pair(is_not(":"), char(':'), not_line_ending),
            line_ending,
        )(input.trim_start())?;
        Ok((input, Some((k.trim_start(), v.trim_start()))))
    }
}

pub fn subtitle<'a>(input: &'a str, definition: &'a Vec<String>) -> IResult<&'a str, Entry> {
    let mut entry = Entry::default();
    let (input, kind) = terminated(is_not(":"), char(':'))(input)?;
    entry.kind = kind.to_owned();

    let (input, settings) =
        many_m_n(1, 9, terminated(opt(is_not(",")), char(',')))(input.trim_start())?;
    for (n, possible_val) in settings.into_iter().enumerate() {
        if let Some(val) = possible_val {
            match definition[n].as_str() {
                "Layer" => entry.layer = val.parse::<isize>().ok(),
                "Start" => entry.start = duration(val).ok().map(|v| v.1),
                "End" => entry.end = duration(val).ok().map(|v| v.1),
                "Style" => entry.style = Some(val.to_owned()),
                "Name" => entry.name = Some(val.to_owned()),
                "MarginL" => entry.margin_l = val.parse::<usize>().ok(),
                "MarginR" => entry.margin_r = val.parse::<usize>().ok(),
                "MarginV" => entry.margin_v = val.parse::<usize>().ok(),
                "Effect" => entry.effect = Some(val.to_owned()),
                _ => (),
            }
        }
    }

    if !input.is_empty() {
        entry.text = text_line(input).unwrap().1; // todo: don't
    }

    Ok((input, entry))
}

pub fn section(input: &str) -> IResult<&str, Section> {
    let (input, header) = delimited(char('['), is_not("]"), char(']'))(input)?;
    match header {
        "V4+ Styles" => {
            let (input, definition) = preceded(tag("Format:"), line_list)(input.trim_start())?;
            let (input, lines) = separated_list0(line_ending, preceded(tag("Style:"), line_list))(
                input.trim_start(),
            )?;
            let mut h = HashMap::new();

            for vals in lines {
                let mut style = Style::default();

                for (n, val) in vals.into_iter().enumerate() {
                    match definition[n] {
                        "Name" => style.name = val.to_owned(),
                        "Fontname" => style.fontname = val.to_owned(),
                        "Fontsize" => style.fontsize = val.parse::<usize>().unwrap(),
                        "PrimaryColour" => style.primary_color = full_color(val).ok().map(|v| v.1),
                        "SecondaryColour" => {
                            style.secondary_color = full_color(val).ok().map(|v| v.1)
                        }
                        "OutlineColour" => style.outline_color = full_color(val).ok().map(|v| v.1),
                        "BackColour" => style.back_color = full_color(val).ok().map(|v| v.1),
                        "Bold" => style.bold = Some(val == "-1"),
                        "Italic" => style.italic = Some(val == "-1"),
                        "Underline" => style.underline = Some(val == "-1"),
                        "ScaleX" => style.scale_x = val.parse::<usize>().ok(),
                        "ScaleY" => style.scale_y = val.parse::<usize>().ok(),
                        "Spacing" => style.spacing = val.parse::<usize>().ok(),
                        "Angle" => style.angle = val.parse::<f64>().ok(),
                        "BorderStyle" => style.border_style = val.parse::<usize>().ok(),
                        "Outline" => style.outline_size = val.parse::<usize>().ok(),
                        "Shadow" => style.shadow = val.parse::<usize>().ok(),
                        "Alignment" => style.alignment = val.parse::<usize>().ok(),
                        "MarginL" => style.margin_l = val.parse::<usize>().ok(),
                        "MarginR" => style.margin_r = val.parse::<usize>().ok(),
                        "MarginV" => style.margin_v = val.parse::<usize>().ok(),
                        "Encoding" => style.encoding = val.parse::<usize>().ok(),
                        _ => (),
                    }
                }
                h.insert(style.name.clone(), style);
            }

            Ok((input, Section::Styles(h)))
        }
        "Events" => {
            let (input, definition) = preceded(tag("Format:"), line_list)(input.trim_start())?;
            Ok((
                input,
                Section::EventsHeader(definition.into_iter().map(|v| v.to_owned()).collect()),
            ))
        }
        _ => {
            let (input, _) = line_ending(input)?;
            let (input, options) = many0(setting)(input)?;
            Ok((
                input,
                Section::Other {
                    name: header.to_string(),
                    settings: options
                        .into_iter()
                        .filter_map(|o| {
                            if let Some((k, v)) = o {
                                Some((k.to_owned(), v.to_owned()))
                            } else {
                                None
                            }
                        })
                        .collect(),
                },
            ))
        }
    }
}
