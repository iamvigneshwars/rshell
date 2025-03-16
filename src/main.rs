use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process,
};

enum BuiltinCommand {
    Echo,
    Type,
    Exit,
    Pwd,
    Cd,
}

impl BuiltinCommand {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "echo" => Some(BuiltinCommand::Echo),
            "type" => Some(BuiltinCommand::Type),
            "exit" => Some(BuiltinCommand::Exit),
            "pwd" => Some(BuiltinCommand::Pwd),
            "cd" => Some(BuiltinCommand::Cd),
            _ => None,
        }
    }

    fn execute(&self, shell: &Shell, args: &str) {
        match self {
            BuiltinCommand::Echo => shell.cmd_echo(args),
            BuiltinCommand::Type => shell.cmd_type(args),
            BuiltinCommand::Exit => shell.cmd_exit(args),
            BuiltinCommand::Pwd => shell.cmd_pwd(args),
            BuiltinCommand::Cd => shell.cmd_cd(args),
        }
    }
}

#[derive(Default)]
struct Shell;

impl Shell {
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

        if let Some(builtin) = BuiltinCommand::from_str(command) {
            builtin.execute(self, args);
        } else {
            self.cmd_external(command, args);
        }

        Ok(())
    }

    fn cmd_echo(&self, args: &str) {
        println!("{}", args);
    }

    fn cmd_pwd(&self, args: &str) {
        if !args.is_empty() {
            eprintln!("pwd: too many arguments");
            return;
        }

        match env::current_dir() {
            Ok(path) => println!("{}", path.display()),
            Err(e) => eprintln!("pwd: error getting current directory: {}", e),
        }
    }

    fn cmd_cd(&self, args: &str) {
        let new_dir = if args.is_empty() || args == "~" {
            env::var("HOME").unwrap_or_else(|_| "/".to_string())
        } else {
            args.to_string()
        };

        if env::set_current_dir(&new_dir).is_err() {
            eprintln!("cd: {}: No such file or directory", new_dir);
        }
    }

    fn cmd_exit(&self, args: &str) {
        if args.is_empty() {
            process::exit(0);
        } else if let Ok(status) = args.parse::<i32>() {
            process::exit(status);
        } else {
            eprintln!("exit: too many arguments");
        }
    }

    fn cmd_type(&self, args: &str) {
        if args.is_empty() {
            return;
        }
        let commands: Vec<&str> = args.split_whitespace().collect();
        for command in commands {
            if BuiltinCommand::from_str(command).is_some() {
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
    let mut shell = Shell;
    shell.run();
}
