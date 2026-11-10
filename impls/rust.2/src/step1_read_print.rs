use std::io::Write;
use thiserror::Error;
use mal::printer::pr_str;
use mal::reader::{read_str, Atom, ReaderError};

type ReaderOutput = Result<Atom, ReaderError>;
type EvalOutput = Result<Atom, EvalError>;

#[derive(Error, Debug)]
enum EvalError {
    #[error("Reader error: {0}")]
    ReaderError(#[from] ReaderError),
}

fn read(input: String) -> ReaderOutput {
    read_str(input)
}

fn eval(input: ReaderOutput) -> EvalOutput {
    Ok(input?)
}

fn print(input: EvalOutput) -> String {
    match input {
        Ok(atom) => pr_str(&atom, false),
        Err(err) => format!("{}", err),
    }
}

fn rep(input: String) -> String {
    print(eval(read(input)))
}

fn main() {
    let mut input_buffer = String::new();
    let stdin = std::io::stdin();
    loop {
        print!("user> ");
        std::io::stdout().flush().unwrap();
        input_buffer.clear();
        if let Ok(0) = stdin.read_line(&mut input_buffer) {
            break;
        }
        println!("{}", rep(input_buffer.clone()));
    }
}
