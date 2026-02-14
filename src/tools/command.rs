use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::io::{self, Write};
use std::process::Command;
use super::Tool;

pub struct RunCommand;

#[async_trait]
impl Tool for RunCommand {
    fn name(&self) -> &str {
        "run_command"
    }

    fn description(&self) -> &str {
        "Execute a shell command on the host system. Useful for file system operations, running scripts, or gathering system information."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The command line to execute (e.g., 'ls -la', 'cat file.txt')."
                }
            },
            "required": ["command"]
        })
    }

    async fn call(&self, arguments: &str) -> Result<String> {
        let args: Value = serde_json::from_str(arguments)?;
        let command_str = args["command"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'command' argument"))?;

        println!("\n[CAUTION] The agent wants to execute: '{}'.", command_str);
        print!("Allow this command? [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "y" {
            return Ok("User denied permission to execute command.".to_string());
        }

        let output = Command::new("sh")
            .arg("-c")
            .arg(command_str)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            Ok(format!("Output:\n{}", stdout))
        } else {
            Ok(format!("Command failed with error:\n{}\nStdout:\n{}", stderr, stdout))
        }
    }
}
