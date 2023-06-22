use std::cell::OnceCell;
use std::collections::HashMap;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_until};
use nom::character::complete::{char, multispace0, one_of};
use nom::combinator::{cut, value};
use nom::error::{ErrorKind, make_error};
use nom::{AsChar, InputTake, IResult};
use nom::Err::Error;
use nom::multi::{fold_many0, many0};
use nom::sequence::{delimited, preceded, separated_pair, terminated};
use nom_locate::LocatedSpan;

type Span<'a> = LocatedSpan<&'a str>;
const PAD: OnceCell<Span> = OnceCell::new();

fn line_comment(input: Span) -> IResult<Span, Span> {
    preceded(tag("//"), is_not("\r\n"))(input)
}
fn block_comment(input: Span) -> IResult<Span, Span> {
    delimited(tag("/*"), take_until("*/"), tag("*/"))(input)
}
fn ignore(input: Span) -> IResult<Span, Span> {
    alt((
        line_comment,
        block_comment,
        value(*PAD.get_or_init(|| {Span::new("")}), one_of(" \t\r\n"))
    ))(input)
}

fn string(input: Span) -> IResult<Span, Span> {
    let mut escaped: usize = 0xefffffff;
    let mut max: usize = 0;
    // let escaped_str = "\\\"abftnrU";
    let non_unicode = |c: char| {!c.is_hex_digit()};
    for (idx, c) in input.char_indices() {
        if idx == 0 && c != '"' {
            return Err(nom::Err::Error(make_error(input, ErrorKind::Char)));
        }
        if c == '\\' {
            escaped = idx;
            continue;
        }
        if idx == escaped + 1 {
            escaped = 0xefffffff;
            /* 
                Only unicode (\Uxxxx) should be checked, others with escape charactor is legal in macOS, for exampe: "\h" is same as "h".
            */
            // if !escaped_str.contains(c) {
            //     let (end, _) = input.take_split(idx - 1);
            //     return Err(nom::Err::Failure(make_error(end, ErrorKind::Char)));
            // }
            // detect unicode
            if c == 'U' {
                let unicode = input.get(idx + 1 .. idx + 5).ok_or(nom::Err::Failure(make_error(input, ErrorKind::Eof)))?;
                let ret = unicode.find(non_unicode);
                if let Some(_x) = ret {
                    let (end, _) = input.take_split(idx - 1);
                    return Err(nom::Err::Failure(make_error(end, ErrorKind::Char)));
                }
            }
            continue;
        }
        if idx > 0 && c == '"' {
            max = idx;
            break;
        }
    }
    if max == 0 {
        return Err(nom::Err::Failure(make_error(input, ErrorKind::Eof)));
    }
    Ok(input.take_split(max + 1))
}

fn k(input: Span) -> IResult<Span, Span> {
    preceded(multispace0, string)(input)
}
fn v(input: Span) -> IResult<Span, Span> {
    cut(terminated(k, delimited(multispace0, char(';'), many0(ignore))))(input)
}
fn key_value(input: Span) -> IResult<Span, (Span, Span)> {
    if input.is_empty() {
        return Err(Error(make_error(Span::new(""), ErrorKind::Eof)));
    }
    let sp = preceded(multispace0, char('='));
    cut(separated_pair(string, sp, v))(input)
}

fn key_value_loop(input: Span) -> IResult<Span, HashMap<Span, Span>> {
    fold_many0(
        key_value,
        HashMap::new,
        |mut acc: HashMap<Span, Span>, item| {
            acc.insert(item.0, item.1);
            acc
        }
    )(input)
}

fn parse_strings_slice(input: Span) -> IResult<Span, HashMap<Span, Span>> {
    let (input, _) = many0(ignore)(input)?;
    key_value_loop(input)
}


const DATA: &str = r#"/* Menu item to make the current document plain text */
"Make Plain Text" = "In reinen Text umwandeln";
/* Menu item to make the current document rich text */
"Make Rich Text" = "In formatierten Text umwandeln";
"#;

const EMBED_DOUBLE_QUOTE_DATA: &str = r#"/* Menu item to make the current document plain text */
"Make Plain Text" = "In reinen" Text umwandeln";
/* Menu item to make the current document rich text */
"Make Rich Text" = "In formatierten Text umwandeln";
"#;

fn main() {
    let span1 = Span::new(DATA);
    let ret1 = parse_strings_slice(span1);
    println!("{:#?}", ret1.ok().unwrap().1);
    let span2 = Span::new(EMBED_DOUBLE_QUOTE_DATA);
    let ret2 = parse_strings_slice(span2);
    println!("{:#?}", ret2.err().unwrap());
}
