use std::collections::HashMap;
use std::time::Duration;

pub mod parser;
pub mod utils;

#[derive(Clone, Debug)]
pub struct Color {
    pub alpha: Option<u8>,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Default, Clone, Debug)]
pub struct Style {
    pub name: String,
    pub fontname: String,
    pub fontsize: usize,
    pub primary_color: Option<Color>,
    pub secondary_color: Option<Color>,
    pub outline_color: Option<Color>,
    pub back_color: Option<Color>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub strikeout: Option<bool>,
    pub scale_x: Option<usize>,
    pub scale_y: Option<usize>,
    pub spacing: Option<usize>,
    pub angle: Option<f64>,
    pub border_style: Option<usize>,
    pub outline_size: Option<usize>,
    pub shadow: Option<usize>,
    pub alignment: Option<usize>,
    pub margin_l: Option<usize>,
    pub margin_r: Option<usize>,
    pub margin_v: Option<usize>,
    pub encoding: Option<usize>,
}

#[derive(Default, Clone, Debug)]
pub struct Entry {
    pub kind: Option<String>,
    pub layer: Option<isize>,
    pub start: Option<Duration>,
    pub end: Option<Duration>,
    pub style: Option<String>,
    pub name: Option<String>,
    pub margin_l: Option<usize>,
    pub margin_r: Option<usize>,
    pub margin_v: Option<usize>,
    pub effect: Option<String>,
    pub read_order: Option<isize>,
    pub text: Vec<TextSection>,
}

impl Entry {
    pub fn plain_text(&self) -> String {
        let s = self
            .text
            .iter()
            .filter_map(|v| {
                if let TextSection::Text(s) = v {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();
        s.join("")
    }
}

#[derive(Clone, Debug)]
pub enum Section {
    EventsHeader(Vec<String>),
    Other {
        name: String,
        settings: HashMap<String, String>,
    },
    Styles(HashMap<String, Style>),
}

impl Section {
    pub fn as_event_header(&self) -> Option<&Vec<String>> {
        match self {
            Section::EventsHeader(v) => Some(v),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum TextSection {
    Text(String),
    StyleOverride(Vec<StyleOverride>),
    Drawing(Vec<DrawingCommand>),
}

#[derive(Clone, Debug)]
pub enum StyleOverride {
    Bold(f64),
    Italic(bool),
    Underline(bool),
    StrikeOut(bool),
    Border(f64),
    Shadow(f64),
    BlurEdges(bool),
    FontName(String),
    FontSize(f64),
    ScaleX(f64),
    ScaleY(f64),
    LetterSpacing(f64),
    RotationX(f64),
    RotationY(f64),
    RotationZ(f64),
    Charset(u64),
    Color(u64, Color), // u64: color index (Primary, Secondary, Outline, Background)
    Alpha(u64, u8),    // same color index
    Alignment(f64),
    NumpadLayoutAlignment(f64),
    KaraokeDuration(Duration),
    WrappingStyle(f64),
    Reset(Option<String>),
    DrawingMode(f64),
    BaselineOffset(f64),
    Transition {
        start: Option<Duration>,
        end: Option<Duration>,
        acceleration: Option<f64>,
        styles: Vec<StyleOverride>,
    },
    Move {
        start_x: f64,
        start_y: f64,
        end_x: f64,
        end_y: f64,
        start: Option<Duration>,
        end: Option<Duration>,
    },
    Origin {
        x: f64,
        y: f64,
    },
    Fade {
        starting_alpha: u8,
        middle_alpha: u8,
        ending_alpha: u8,
        start_time: Duration,
        in_between_time: Duration,
        late_time: Duration,
        ending_time: Duration,
    },
    FadeInAndOut {
        fade_in_for: Duration,
        fade_out_for: Duration,
    },
    Clip {
        a_x: f64,
        a_y: f64,
        b_x: f64,
        b_y: f64,
    },
    ClipToDrawing(Option<f64>, Vec<DrawingCommand>),
    EmptyClip,
    Other(String),
}

#[derive(Clone, Debug)]
pub enum DrawingCommand {
    Move {
        x: f64,
        y: f64,
    },
    MoveWithoutClosing {
        x: f64,
        y: f64,
    },
    Line {
        x: f64,
        y: f64,
    },
    Bezier {
        a_x: f64,
        a_y: f64,
        b_x: f64,
        b_y: f64,
        c_x: f64,
        c_y: f64,
    },
    UniformSpline(Vec<(f64, f64)>),
    ExtendBspline {
        x: f64,
        y: f64,
    },
    CloseBspline,
}
