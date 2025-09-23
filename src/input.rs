use std::path::PathBuf;

pub enum Input {
    Command(Vec<String>),
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
        Input::Command(cmd) => {
            get_command_output(&cmd.iter().map(|s| s.as_str()).collect(), path, debug)
        }
    }
}

pub fn from_config(input: &crate::config::Input) -> Input {
    Input::Command(input.command.split_whitespace().map(|s| s.to_string()).collect())
}