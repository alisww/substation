use nom::{
    bytes::complete::{is_not, tag, take_while, take_while_m_n},
    character::complete::{char, line_ending, not_line_ending},
    combinator::{map, map_res, not, peek},
    multi::{many0, separated_list0},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    IResult,
};
use std::collections::HashMap;

#[derive(Debug)]
struct Color {
    alpha: u8,
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Default, Debug)]
struct Style {
    name: String,
    fontname: String,
    fontsize: usize,
    primary_color: Option<Color>,
    secondary_color: Option<Color>,
    outline_color: Option<Color>,
    back_color: Option<Color>,
    bold: Option<bool>,
    italic: Option<bool>,
    underline: Option<bool>,
    strikeout: Option<bool>,
    scale_x: Option<usize>,
    scale_y: Option<usize>,
    spacing: Option<usize>,
    angle: Option<f64>,
    border_style: Option<usize>,
    outline_size: Option<usize>,
    shadow: Option<usize>,
    alignment: Option<usize>,
    margin_l: Option<usize>,
    margin_r: Option<usize>,
    margin_v: Option<usize>,
    encoding: Option<usize>,
}

#[derive(Debug)]
enum Section {
    Other {
        name: String,
        settings: HashMap<String, String>,
    },
    Styles(HashMap<String, Style>),
}

fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
    c.is_digit(16)
}

fn hex_primary(input: &str) -> IResult<&str, u8> {
    map_res(take_while_m_n(2, 2, is_hex_digit), from_hex)(input)
}

// AABBGGRR, hex
fn parse_color(input: &str) -> IResult<&str, Color> {
    let (input, _) = tag("&H")(input)?;
    let (input, (alpha, blue, green, red)) =
        tuple((hex_primary, hex_primary, hex_primary, hex_primary))(input)?;
    Ok((
        input,
        Color {
            alpha,
            red,
            green,
            blue,
        },
    ))
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

fn setting(input: &str) -> IResult<&str, (&str, &str)> {
    peek(not(line_ending))(input)?;
    peek(not(char('[')))(input)?;
    terminated(
        separated_pair(is_not(":"), char(':'), not_line_ending),
        line_ending,
    )(input.trim_start())
}

fn section(input: &str) -> IResult<&str, Section> {
    let (input, header) = delimited(char('['), is_not("]"), char(']'))(input)?;
    match header {
        "V4+ Styles" => {
            let (input, definition) = preceded(tag("Format:"), line_list)(input.trim_start())?;
            let (input, lines) =
                separated_list0(line_ending, preceded(tag("Style:"), line_list))(input.trim_start())?;
            let mut h = HashMap::new();
            println!("{:?}",lines);

            for vals in lines {
                let mut style = Style::default();

                for (n, val) in vals.into_iter().enumerate() {
                    let field = definition[n];

                    match field {
                        "Name" => style.name = val.to_owned(),
                        "Fontname" => style.fontname = val.to_owned(),
                        "Fontsize" => style.fontsize = val.parse::<usize>().unwrap(),
                        "PrimaryColour" => style.primary_color = parse_color(val).ok().map(|v| v.1),
                        "SecondaryColour" => {
                            style.secondary_color = parse_color(val).ok().map(|v| v.1)
                        }
                        "OutlineColour" => style.outline_color = parse_color(val).ok().map(|v| v.1),
                        "BackColour" => style.back_color = parse_color(val).ok().map(|v| v.1),
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
                        _ => ()
                    }
                }
                h.insert(style.name.clone(), style);
            }

            Ok((input,Section::Styles(h)))
        }
        _ => {
            let (input, _) = line_ending(input)?;
            let (input, options) = many0(setting)(input)?;
            Ok((input, Section::Other {
                name: header.to_string(),
                settings: options
                    .into_iter()
                    .map(|(k, v)| (k.to_owned(), v.to_owned()))
                    .collect(),
            }))
        }
    }
}

// fn main() {
//     let header = include_str!("../header.ssa");
//     let (input, res) = section(header).unwrap();
//     // println!("{}",input);
//     let (input, second) = section(input.trim_start()).unwrap();
//     let (input, third) = section(input.trim_start()).unwrap();
//     // let (input, other) = section(input).unwrap();
//     println!("{:#?}", res);
//     println!("{:#?}", second);
//     println!("{:#?}", third);
//     // println!("{:#?}", third);
//     // println!("{:#?}", other);
//     // let format = r#"Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding"#;
//     // println!("{:?}", line_list(format).unwrap());
// }
