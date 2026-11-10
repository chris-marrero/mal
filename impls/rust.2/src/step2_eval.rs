use crate::EvalError::{ExpectedAtom, InvalidArgument, OperatorNotFound};
use mal::printer::pr_str;
use mal::reader::{Atom, ReaderError, read_str};
use std::collections::HashMap;
use std::io::Write;
use thiserror::Error;

type ReaderOutput = Atom;
#[derive(Debug)]
enum EvalOutput {
    Atom(Atom),
    BinaryOperator(BinaryOperator),
}

#[derive(Error, Debug)]
enum EvalError {
    #[error("Reader error: {0}")]
    ReaderError(#[from] ReaderError),

    #[error("Symbol not found in environment")]
    SymbolNotFound(String),

    #[error("First token in list is not an operator")]
    OperatorNotFound(String),

    #[error("Empty list")]
    EmptyList,

    #[error("Invalid argument")]
    InvalidArgument(String),

    #[error("Expected atom in vector/hash-map")]
    ExpectedAtom(String),
}

fn read(input: String) -> Result<ReaderOutput, ReaderError> {
    read_str(input)
}

fn eval(
    input: Result<ReaderOutput, ReaderError>,
    repl_environment: &HashMap<String, BinaryOperator>,
) -> Result<EvalOutput, EvalError> {
    if let Ok(Atom::Symbol(symbol)) = input {
        repl_environment
            .get(&symbol)
            .map(|&f| EvalOutput::BinaryOperator(f))
            .ok_or(EvalError::SymbolNotFound(symbol.clone()))
    } else if let Ok(Atom::List(atoms)) = input {
        if atoms.len() == 0 {
            return Ok(EvalOutput::Atom(Atom::List(atoms)));
        }

        let evaluated_atoms = atoms
            .iter()
            .map(|atom| eval(Ok(atom.clone()), repl_environment))
            .collect::<Result<Vec<EvalOutput>, EvalError>>()?;

        let EvalOutput::BinaryOperator(operator) =
            evaluated_atoms.first().ok_or(EvalError::EmptyList)?
        else {
            Err(OperatorNotFound(pr_str(atoms.first().unwrap(), false)))?
        };

        let &EvalOutput::Atom(Atom::Integer(arg_one)) =
            evaluated_atoms.get(1).ok_or(EvalError::EmptyList)?
        else {
            Err(InvalidArgument(format!("{:?}", evaluated_atoms[1])))?
        };

        let &EvalOutput::Atom(Atom::Integer(arg_two)) =
            evaluated_atoms.get(2).ok_or(EvalError::EmptyList)?
        else {
            Err(InvalidArgument(format!("{:?}", evaluated_atoms[2])))?
        };

        Ok(EvalOutput::Atom(Atom::Integer(operator(arg_one, arg_two))))
    } else if let Ok(Atom::Vector(atoms)) = input {
        let evaluated_atoms = atoms
            .iter()
            .map(|atom| {
                let EvalOutput::Atom(evaluated_atom) = eval(Ok(atom.clone()), repl_environment)?
                else {
                    Err(ExpectedAtom(pr_str(atom, false)))?
                };
                Ok(evaluated_atom)
            })
            .collect::<Result<Vec<Atom>, EvalError>>()?;

        Ok(EvalOutput::Atom(Atom::Vector(evaluated_atoms)))
    } else if let Ok(Atom::HashMap(hash_map)) = input {
        let evaluated_hash_map = hash_map
            .iter()
            .map(|(key, value)| {
                let EvalOutput::Atom(evaluated_value) = eval(Ok(value.clone()), repl_environment)?
                else {
                    Err(ExpectedAtom(pr_str(value, false)))?
                };
                Ok((key.clone(), evaluated_value))
            })
            .collect::<Result<Vec<(Atom, Atom)>, EvalError>>()?;

        Ok(EvalOutput::Atom(Atom::HashMap(evaluated_hash_map)))
    } else {
        Ok(EvalOutput::Atom(input?))
    }
}

fn print(input: Result<EvalOutput, EvalError>) -> String {
    match input {
        Ok(EvalOutput::Atom(atom)) => pr_str(&atom, false),
        Err(err) => format!("{}", err),
        _ => "Eval did not produce an atom".to_string(),
    }
}

fn rep(input: String, repl_environment: &HashMap<String, BinaryOperator>) -> String {
    print(eval(read(input), repl_environment))
}

pub type BinaryOperator = fn(i64, i64) -> i64;

fn main() {
    let mut input_buffer = String::new();
    let stdin = std::io::stdin();
    let repl_environment: [(&str, BinaryOperator); 4] = [
        ("+", |a, b: i64| a + b),
        ("-", |a, b: i64| a - b),
        ("*", |a, b: i64| a * b),
        ("/", |a, b: i64| a / b),
    ];
    let repl_environment = HashMap::from(repl_environment.map(|(s, f)| (s.to_string(), f)));
    loop {
        print!("user> ");
        std::io::stdout().flush().unwrap();
        input_buffer.clear();
        if let Ok(0) = stdin.read_line(&mut input_buffer) {
            break;
        }
        println!("{}", rep(input_buffer.clone(), &repl_environment));
    }
}
