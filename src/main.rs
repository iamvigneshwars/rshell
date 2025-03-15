#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let (command, rest) = match input.trim().split_once(" ") {
            Some((command, rest)) => (command, rest),
            None => (input.trim(), ""),
        };

        match command {
            "exit" => std::process::exit(0),
            "echo" => println!("{}", rest),
            "type" => {
                let commands: Vec<&str> = rest.split_whitespace().collect();
                for command in commands {
                    match command {
                        "exit" => println!("exit is a shell builtin"),
                        "echo" => println!("echo is a shell builtin"),
                        "type" => println!("type is a shell builtin"),
                        _ => println!("{}: not found", command),
                    }
                }
            }
            _ => {
                println!("{}: command not found", command);
            }
        }
    }
}
