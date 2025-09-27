use std::path::PathBuf;

pub enum Input {
    None,
    Command(Vec<String>),
    Files(Vec<String>),
    Dir(String),
}

pub fn get_command_output(cmd: &Vec<&str>, path: &PathBuf, debug: bool) -> anyhow::Result<String> {
    if debug {
        eprintln!("Executing command: {:?}", cmd);
        eprintln!("In directory: {:?}", path);
    }
    let output = std::process::Command::new(&cmd[0])
        .args(&cmd[1..])
        .current_dir(path)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute command {:?}: {}", cmd, e))?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Git command failed with status: {}",
            output.status
        ));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| anyhow::anyhow!("Failed to parse git output: {}", e))?;

    if debug {
        eprintln!("Command output: {}", stdout);
    }

    Ok(stdout)
}

pub fn get_input(input: &Input, path: &PathBuf, debug: bool) -> anyhow::Result<String> {
    match input {
        Input::None => Ok(String::new()),
        Input::Command(cmd) => {
            get_command_output(&cmd.iter().map(|s| s.as_str()).collect(), path, debug)
        }
        Input::Files(files) => {
            let mut contents = String::new();
            for file in files {
                let file_path = path.join(file);
                if debug {
                    eprintln!("Reading file: {:?}", file_path);
                }
                let file_content = std::fs::read_to_string(&file_path)
                    .map_err(|e| anyhow::anyhow!("Failed to read file {:?}: {}", file_path, e))?;
                contents.push_str(&file_content);
                contents.push_str("\n");
            }
            Ok(contents)
        }
        Input::Dir(dir) => {
            let dir_path = path.join(dir);
            if debug {
                eprintln!("Reading directory: {:?}", dir_path);
            }
            let mut contents = String::new();
            for entry in std::fs::read_dir(&dir_path)
                .map_err(|e| anyhow::anyhow!("Failed to read directory {:?}: {}", dir_path, e))?
            {
                let entry =
                    entry.map_err(|e| anyhow::anyhow!("Failed to read directory entry: {}", e))?;
                let path = entry.path();
                if path.is_file() {
                    if debug {
                        eprintln!("Reading file: {:?}", path);
                    }
                    let file_content = std::fs::read_to_string(&path)
                        .map_err(|e| anyhow::anyhow!("Failed to read file {:?}: {}", path, e))?;
                    contents.push_str(&file_content);
                    contents.push_str("\n");
                }
            }
            Ok(contents)
        }
    }
}

pub fn from_config(input: &crate::config::Input) -> Input {
    Input::Command(
        input
            .command
            .split_whitespace()
            .map(|s| s.to_string())
            .collect(),
    )
}
