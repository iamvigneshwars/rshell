use std::{
    collections::HashMap,
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
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

        for command in commands {
            if self.builtins.contains_key(&command) {
                println!("{} is a shell builtin", command);
                continue;
            }
            match self.find_executable(command) {
                Some(path) => println!("{} is {}", command, path.to_string_lossy()),
                None => println!("{}: not found", command),
            }
        }
    }

    fn find_executable(&mut self, command: &str) -> Option<PathBuf> {
        let path_var = match env::var("PATH") {
            Ok(path) => path,
            Err(_) => return None,
        };
        let paths: Vec<&str> = path_var.split(":").collect();
        for path in paths {
            let full_path = Path::new(path).join(command);
            if full_path.exists() && self.is_executable(&full_path) {
                return Some(full_path);
            }
        }
        None
    }

    fn cmd_external(&mut self, command: &str, args: &str) {
        let executable_path = match self.find_executable(command) {
            Some(path) => path,
            None => {
                println!("{}: command not found", command);
                return;
            }
        };
        if !self.is_executable(&executable_path) {
            eprintln!("{} is not a executable", command);
            return;
        }
        let arg_vec: Vec<&str> = if args.is_empty() {
            Vec::new()
        } else {
            args.split_whitespace().collect()
        };
        match process::Command::new(command)
            .args(arg_vec)
            .stdout(process::Stdio::inherit())
            .stderr(process::Stdio::inherit())
            .stdin(process::Stdio::inherit())
            .status()
        {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Failed to execute {}: {}", command, e);
            }
        }
    }
    fn is_executable(&self, path: &Path) -> bool {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::metadata(path)
                .map(|metadata| {
                    let permissions = metadata.permissions();
                    metadata.is_file() && (permissions.mode() & 0o111 != 0)
                })
                .unwrap_or(false)
        }
    }
}

fn main() {
    let mut shell = Shell::new();
    shell.run();
}
