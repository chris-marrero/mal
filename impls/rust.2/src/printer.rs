use regex::{Captures, Regex};
use crate::reader::Atom;

fn list_items(atoms: &Vec<Atom>, print_readably: bool) -> String {
    atoms.iter().map(|atom| pr_str(atom, print_readably)).collect::<Vec<String>>().join(" ")
}

pub fn pr_str(atom: &Atom, print_readably: bool) -> String {
    match atom {
        Atom::Symbol(symbol) => symbol.to_string(),
        Atom::Keyword(keyword) => format!(":{}", keyword),
        Atom::Integer(integer) => format!("{}", integer),
        Atom::List(atoms) => {
            format!("({})", list_items(atoms, print_readably))
        },
        Atom::Vector(atoms) => {
            format!("[{}]", list_items(atoms, print_readably))
        }
        Atom::HashMap(atoms) => {
            format!("{{{}}}", atoms.iter().map(|(key, value)| {
                format!("{} {}", pr_str(key, print_readably), pr_str(value, print_readably))
            }).collect::<Vec<String>>().join(" "))
        }
        Atom::String(string) => {
            let string = if print_readably {
                string
            } else {
                let re = Regex::new("(\\\\|\\\"|\\n)").unwrap();
                &re.replace_all(string, |capture: &Captures| {
                    match &capture[0] {
                        "\\" => r"\\",
                        "\"" => r#"\""#,
                        "\n" => r"\n",
                        _ => unreachable!(),
                    }
                }).into_owned()
            };

            format!("\"{}\"", string)
        }
        Atom::Quote(atom) => {
            format!("(quote {})", pr_str(atom, print_readably))
        }
        Atom::Quasiquote(atom) => {
            format!("(quasiquote {})", pr_str(atom, print_readably))
        }
        Atom::Unquote(atom) => {
            format!("(unquote {})", pr_str(atom, print_readably))
        }
        Atom::SpliceUnquote(atom) => {
            format!("(splice-unquote {})", pr_str(atom, print_readably))
        }
        Atom::Deref(atom) => {
            format!("(deref {})", pr_str(atom, print_readably))
        }
        Atom::Metadata { data, metadata } => {
            format!("(with-meta {} {})", pr_str(data, print_readably), pr_str(metadata, print_readably))
        }
        Atom::Nil => "nil".to_string(),
        Atom::True => "true".to_string(),
        Atom::False => "false".to_string(),
        Atom::Comment => "".to_string(),
    }
}