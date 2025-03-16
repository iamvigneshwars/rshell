
use std::env;
use std::fs;
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
                    let mut found = false;
                    match command {
                        "exit" => println!("exit is a shell builtin"),
                        "echo" => println!("echo is a shell builtin"),
                        "type" => println!("type is a shell builtin"),
                        _ => {
                            if let Ok(path_var) = env::var("PATH") {
                                let paths: Vec<&str> = path_var.split(":").collect();

                                'path_loop: for path in paths {
                                    if let Ok(entries) = fs::read_dir(path) {
                                        for entry in entries.flatten() {
                                            if command == entry.file_name() {
                                                println!(
                                                    "{} is {}",
                                                    command,
                                                    entry.path().to_string_lossy()
                                                );
                                                found = true;
                                                break 'path_loop;
                                            }
                                        }
                                    }
                                }
                                if !found {
                                    println!("{}: not found", command);
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                println!("{}: command not found", command);
            }
        }
    }
}
