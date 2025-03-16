use std::{
    env, fs,
    io::{self, Write},
    path::Path,
    process,
};

struct Shell {
    builtins: Vec<&'static str>,
}

impl Shell {
    fn new() -> Self {
        Shell {
            builtins: vec!["exit", "echo", "type"],
        }
    }

    fn run(&self) {
        loop {
            if let Err(e) = self.prompt_and_execute() {
                eprint!("Error: {}", e);
            }
        }
    }

    fn prompt_and_execute(&self) -> io::Result<()> {
        print!("$ ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim();
        if input.is_empty() {
            return Ok(());
        }

        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let command = parts[0];
        let args = parts.get(1).unwrap_or(&"").trim();

        match command {
            "exit" => process::exit(0),
            "echo" => self.cmd_echo(args),
            "type" => self.cmd_type(args),
            _ => self.cmd_external(command, args),
        }
        Ok(())
    }

    fn cmd_echo(&self, args: &str) {
        println!("{}", args)
    }

    fn cmd_type(&self, args: &str) {
        if args.is_empty() {
            return;
        }
        let commands: Vec<&str> = args.split_whitespace().collect();

        'command_loop: for command in commands {
            if self.builtins.contains(&command) {
                println!("{} is a shell builtin", command);
                continue;
            }
            if let Ok(path_var) = env::var("PATH") {
                let paths: Vec<&str> = path_var.split(':').collect();
                for path in paths {
                    let full_path = Path::new(path).join(command);
                    if full_path.exists()
                        && fs::metadata(&full_path)
                            .map(|x| x.is_file())
                            .unwrap_or(false)
                    {
                        println!("{} is {}", command, full_path.to_string_lossy());
                        continue 'command_loop;
                    }
                }
            }
            println!("{}: not found", command);
        }
    }

    fn cmd_external(&self, command: &str, args: &str) {
        if let Ok(path_var) = env::var("PATH") {
            let paths: Vec<&str> = path_var.split(":").collect();
            for path in paths {
                let full_path = Path::new(path).join(command);
                if full_path.exists()
                    && fs::metadata(&full_path)
                        .map(|x| x.is_file())
                        .unwrap_or(false)
                {
                    let arg_vec: Vec<&str> = if args.is_empty() {
                        Vec::new()
                    } else {
                        args.split_whitespace().collect()
                    };

                    match std::process::Command::new(command)
                        .args(arg_vec)
                        .stdout(std::process::Stdio::inherit())
                        .stderr(std::process::Stdio::inherit())
                        .stdin(std::process::Stdio::inherit())
                        .status()
                    {
                        Ok(_) => return,
                        Err(e) => {
                            eprintln!("Failed to execute {}: {}", command, e);
                            return;
                        }
                    }
                }
            }
            println!("{}: command not found", command);
        }
    }
}

fn main() {
    let shell = Shell::new();
    shell.run();
}
