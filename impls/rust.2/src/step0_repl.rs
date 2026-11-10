use std::io::Write;

fn read(input: String) -> String {
    input
}

fn eval(input: String) -> String {
    input
}

fn print(input: String) -> String {
    input
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
