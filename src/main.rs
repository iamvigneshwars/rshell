use std::{
    collections::HashMap,
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process,
};

struct Shell {
    builtins: HashMap<&'static str, fn(&Shell, &str)>,
}

impl Shell {
    fn new() -> Self {
        let mut builtins = HashMap::new();
        builtins.insert("echo", Self::cmd_echo as fn(&Shell, &str));
        builtins.insert("type", Self::cmd_type as fn(&Shell, &str));
        builtins.insert("exit", Self::cmd_exit as fn(&Shell, &str));
        builtins.insert("pwd", Self::cmd_pwd as fn(&Shell, &str));
        Shell { builtins }
    }

    fn run(&mut self) {
        loop {
            if let Err(e) = self.prompt_and_execute() {
                eprintln!("Error: {}", e);
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

        if let Some(builtin) = self.builtins.get(command) {
            builtin(self, args);
        } else {
            self.cmd_external(command, args);
        }

        Ok(())
    }

    fn cmd_echo(&self, args: &str) {
        println!("{}", args);
    }

    fn cmd_pwd(&self, _args: &str) {
        match env::current_dir() {
            Ok(path) => println!("{}", path.display()),
            Err(e) => eprintln!("pwd: error getting current directory: {}", e),
        }
    }

    fn cmd_exit(&self, _args: &str) {
        process::exit(0);
    }

    fn cmd_type(&self, args: &str) {
        if args.is_empty() {
            return;
        }
        let commands: Vec<&str> = args.split_whitespace().collect();
        for command in commands {
            if self.builtins.contains_key(command) {
                println!("{} is a shell builtin", command);
            } else if let Some(path) = self.find_executable(command) {
                println!("{} is {}", command, path.display());
            } else {
                println!("{}: not found", command);
            }
        }
    }

    fn find_executable(&self, command: &str) -> Option<PathBuf> {
        env::var("PATH").ok().and_then(|path_var| {
            path_var.split(':').find_map(|path| {
                let full_path = Path::new(path).join(command);
                if full_path.exists() && self.is_executable(&full_path) {
                    Some(full_path)
                } else {
                    None
                }
            })
        })
    }

    fn cmd_external(&self, command: &str, args: &str) {
        match self.find_executable(command) {
            Some(path) if self.is_executable(&path) => {
                match process::Command::new(command)
                    .args(args.split_whitespace())
                    .stdout(process::Stdio::inherit())
                    .stderr(process::Stdio::inherit())
                    .stdin(process::Stdio::inherit())
                    .status()
                {
                    Ok(_) => (),
                    Err(e) => eprintln!("Failed to execute {}: {}", command, e),
                }
            }
            Some(_) => eprintln!("{} is not executable", command),
            None => println!("{}: command not found", command),
        }
    }

    #[cfg(unix)]
    fn is_executable(&self, path: &Path) -> bool {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path)
            .map(|metadata| {
                let permissions = metadata.permissions();
                metadata.is_file() && (permissions.mode() & 0o111 != 0)
            })
            .unwrap_or(false)
    }
}

fn main() {
    let mut shell = Shell::new();
    shell.run();
}
