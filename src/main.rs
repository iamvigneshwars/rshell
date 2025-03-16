use std::{
    collections::HashMap,
    env, fs,
    io::{self, Write},
    path::Path,
    process,
};

struct Shell {
    builtins: HashMap<&'static str, fn(&mut Shell, &str)>,
}

impl Shell {
    fn new() -> Self {
        let mut builtins = HashMap::new();
        builtins.insert("echo", Self::cmd_echo as fn(&mut Shell, &str));
        builtins.insert("type", Self::cmd_type as fn(&mut Shell, &str));
        builtins.insert("exit", Self::cmd_exit as fn(&mut Shell, &str));
        Shell { builtins }
    }

    fn run(&mut self) {
        loop {
            if let Err(e) = self.prompt_and_execute() {
                eprint!("Error: {}", e);
            }
        }
    }

    fn prompt_and_execute(&mut self) -> io::Result<()> {
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

        if let Some(builtin) = self.builtins.get(&command) {
            builtin(self, args);
        } else {
            self.cmd_external(command, args);
        }

        Ok(())
    }

    fn cmd_echo(&mut self, args: &str) {
        println!("{}", args)
    }

    fn cmd_exit(&mut self, _args: &str) {
        process::exit(0)
    }

    fn cmd_type(&mut self, args: &str) {
        if args.is_empty() {
            return;
        }
        let commands: Vec<&str> = args.split_whitespace().collect();

        'command_loop: for command in commands {
            if self.builtins.contains_key(&command) {
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
    let mut shell = Shell::new();
    shell.run();
}
