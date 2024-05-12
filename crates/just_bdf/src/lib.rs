mod lexer;
mod parser;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct Location {
    pub offset: usize,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct Span {
    pub start: Location,
    pub end: Location,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParserError {
    DuplicateGlobalProperty(&'static str),
    InvalidGlobalProperty(String),
    MissingGlobalProperty(&'static str),

    DuplicateGlyphProperty(String, &'static str),
    InvalidGlyphProperty(String, String),
    MissingGlyphProperty(String, &'static str),

    InvalidArgument(Span),
    UnclosedString(Span),
    UnexpectedEof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Encoding {
    AdobeStandard(u32),
    NonStandard(Option<i32>),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Number {
    Float(f32),
    Integer(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Size {
    pub point_size: i32,
    pub x_res: i32,
    pub y_res: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FontBoundingBox {
    pub width: u32,
    pub height: u32,
    pub x_off: i32,
    pub y_off: i32,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Glyph {
    pub name: String,
    pub encoding: Encoding,
    pub s_width: Vector2<Number>,
    pub d_width: Vector2<i32>,
    pub s_width1: Vector2<Number>,
    pub d_width1: Vector2<i32>,
    pub v_vector: Option<Vector2<i32>>,
    pub bounding_box: FontBoundingBox,
    pub bitmap: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vector2<T> {
    pub width: T,
    pub height: T,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Property {
    pub name: String,
    pub value: PropertyValue,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum PropertyValue {
    String(String),
    Number(Number),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Font {
    pub version: Number,
    pub content_version: Option<i32>,
    pub font: String,
    pub size: Size,
    pub font_bounding_box: FontBoundingBox,
    pub properties: Vec<Property>,
    pub metric_set: i32,
    pub s_width: Option<Vector2<Number>>,
    pub d_width: Option<Vector2<i32>>,
    pub s_width1: Option<Vector2<Number>>,
    pub d_width1: Option<Vector2<i32>>,
    pub v_vector: Option<Vector2<i32>>,
    pub glyphs: Vec<Glyph>,
}

pub fn parse(input: &str) -> Result<Font, ParserError> {
    let lexer = lexer::Lexer::new(input);
    let parser = parser::Parser::new(lexer);
    parser.parse()
}
