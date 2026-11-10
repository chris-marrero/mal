use crate::reader::ReaderError::ExpectedToken;
use regex::Regex;
use std::hash::Hash;
use thiserror::Error;

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Atom {
    List(Vec<Atom>),
    Vector(Vec<Atom>),
    Integer(i64),
    Symbol(String),
    HashMap(Vec<(Atom, Atom)>),
    String(String),
    Quote(Box<Atom>),
    Quasiquote(Box<Atom>),
    Unquote(Box<Atom>),
    SpliceUnquote(Box<Atom>),
    Deref(Box<Atom>),
    Metadata {
        data: Box<Atom>,
        metadata: Box<Atom>
    },
    Comment,
}

#[derive(Copy, Clone)]
struct Reader<'t> {
    tokens: &'t [&'t str],
}

impl<'t> Reader<'t> {
    fn new(tokens: &'t [&'t str]) -> Reader<'t> {
        Self { tokens }
    }

    fn empty() -> Reader<'t> { Self { tokens: &[] } }

    fn next(self) -> Option<(&'t str, Reader<'t>)> {
        if let Some((&first_token, rest)) = self.tokens.split_first() {
            Some((first_token, Reader::new(rest)))
        } else {
            None
        }
    }

    // fn peek(&self) -> Option<&'t str> {
    //     self.tokens.first().map(|&s| s)
    // }

    // fn skip(self) -> Option<Reader<'t>> {
    //     self.next().map(|(_, reader)| reader)
    // }
}

#[derive(Error, Debug)]
pub enum ReaderError {
    #[error("Unable to parse input. Contains invalid character or character sequence")]
    Regex(#[from] regex::Error),

    #[error("Failed to parse integer. Likely regex error")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("unbalanced parenthesis in input")]
    UnbalancedParenthesis,

    #[error("EOF")]
    ExpectedToken,

    #[error("Map missing value")]
    MapMissingValue,

    #[error("String not terminated. EOF")]
    StringNotTerminated,
}

pub fn read_str(input: String) -> Result<Atom, ReaderError> {
    let tokens = tokenize(&input)?;
    let reader = Reader::new(&tokens);
    read_form(reader).map(|(atom, _)| atom)
}

fn tokenize(input: &str) -> Result<Vec<&str>, ReaderError> {
    let re =
        Regex::new(r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)"#)?;
    let tokens = re
        .captures_iter(&input)
        .map(|capture| {
            let (_, [c]) = capture.extract();
            c.into()
        })
        .collect();
    Ok(tokens)
}

fn read_form(reader: Reader) -> Result<(Atom, Reader), ReaderError> {
    let (token, next_reader) = reader.next().ok_or(ReaderError::ExpectedToken)?;
    match token {
        "(" => read_list(next_reader, ListKind::List),
        "[" => read_list(next_reader, ListKind::Vector),
        "{" => read_list(next_reader, ListKind::HashMap),
        "^" => read_metadata(next_reader),
        "'" | "`" | "~" | "~@" | "@" => read_quote(reader),
        _ => read_atom(reader),
    }
}

fn read_metadata(reader: Reader) -> Result<(Atom, Reader), ReaderError> {
    let (metadata, reader) = read_form(reader)?;
    let (data, reader) = read_form(reader)?;

    let data = Box::new(data);
    let metadata = Box::new(metadata);

    Ok((Atom::Metadata { data, metadata }, reader))
}

fn read_quote(reader: Reader) -> Result<(Atom, Reader), ReaderError> {
    let (quote, reader) = reader.next().ok_or(ReaderError::ExpectedToken)?;
    let (quoted_atom, reader) = read_form(reader)?;
    let atom = match quote {
        "'" => Atom::Quote(Box::new(quoted_atom)),
        "`" => Atom::Quasiquote(Box::new(quoted_atom)),
        "~" => Atom::Unquote(Box::new(quoted_atom)),
        "~@" => Atom::SpliceUnquote(Box::new(quoted_atom)),
        "@" => Atom::Deref(Box::new(quoted_atom)),
        _ => unreachable!()
    };

    Ok((atom, reader))
}

enum ListKind {
    List,
    Vector,
    HashMap,
}

impl ListKind {
    fn close_delimiter(&self) -> String {
        match self {
            ListKind::List => ")".to_string(),
            ListKind::Vector => "]".to_string(),
            ListKind::HashMap => "}".to_string(),
        }
    }
}

fn read_list(mut reader: Reader, kind: ListKind) -> Result<(Atom, Reader), ReaderError> {
    let mut list = Vec::new();

    let mut done = false;
    while !done {
        if let Some((first_token, new_reader)) = reader.next() {
            if first_token == kind.close_delimiter() {
                done = true;
                reader = new_reader;
            } else {
                let (atom, new_reader) = read_form(reader)?;
                list.push(atom);
                reader = new_reader;
            }
        } else {
            Err(ReaderError::UnbalancedParenthesis)?;
        }
    }

    let list_atom = match kind {
        ListKind::List => Atom::List(list),
        ListKind::Vector => Atom::Vector(list),
        ListKind::HashMap => {
            if list.len() % 2 != 0 {
                Err(ReaderError::MapMissingValue)?
            }

            Atom::HashMap(
                list.chunks(2)
                    .map(|atoms| (atoms[0].clone(), atoms[1].clone()))
                    .collect(),
            )
        }
    };
    Ok((list_atom, reader))
}


fn read_atom(reader: Reader) -> Result<(Atom, Reader), ReaderError> {
    let Some((token, reader)) = reader.next() else {
        Err(ExpectedToken)?
    };

    let starts_with_integer = Regex::new(r"^\d")?;

    let atom = match token {
        _ if token.starts_with(";") => { return Ok((Atom::Comment, Reader::empty())) }
        _ if token.starts_with("\"") => parse_string(token)?,
        _ if starts_with_integer.is_match(token) => Atom::Integer(token.parse::<i64>()?),
        _ => Atom::Symbol(token.to_string())
    };

    Ok((atom, reader))
}

fn parse_string(s: &str) -> Result<Atom, ReaderError> {
    let re = Regex::new(r#"^".*[^\\]\\(\\\\)*"$"#)?;
    if re.is_match(s) || !s.ends_with("\"") {
        Err(ReaderError::StringNotTerminated)?
    }
    let string = s[1..s.len() - 1]
        .to_string()
        .replace(r#"\""#, "\"")
        .replace(r"\n", "\n")
        .replace(r"\\", "\\");

    Ok(Atom::String(string))
}