use crate::shared::pants::{
    errors::{PantsCommandError, PantsResult},
    models::AllHelpInfo,
};

use std::path::Path;

fn execute_pants_command(cwd: &Path, args: Vec<String>) -> PantsResult<std::process::Output> {
    std::process::Command::new("pants")
        .current_dir(cwd)
        .args(&args)
        .output()
        .map_err(|source| PantsCommandError::ExecutionError {
            command: format!("pants {}", args.join(" ")),
            source,
        })
}

fn execute_checked(cwd: &Path, args: Vec<String>) -> PantsResult<std::process::Output> {
    let command = format!("pants {}", args.join(" "));
    let output = execute_pants_command(cwd, args)?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(PantsCommandError::CommandFailed {
            command,
            status: output
                .status
                .code()
                .map_or_else(|| String::from("signal"), |code| code.to_string()),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        })
    }
}

pub fn pants_help_all(cwd: &Path) -> PantsResult<AllHelpInfo> {
    serde_json::from_slice(&execute_checked(cwd, vec![String::from("help-all")])?.stdout)
        .map_err(PantsCommandError::from)
}

pub fn pants_version(cwd: &Path) -> PantsResult<String> {
    Ok(
        String::from_utf8_lossy(&execute_checked(cwd, vec![String::from("version")])?.stdout)
            .to_string(),
    )
}

pub fn pants_list_targets(cwd: &Path) -> PantsResult<String> {
    Ok(String::from_utf8_lossy(
        &execute_checked(cwd, vec![String::from("list"), String::from("::")])?.stdout,
    )
    .to_string())
}

pub fn pants_list_python_tests(cwd: &Path) -> PantsResult<String> {
    Ok(String::from_utf8_lossy(
        &execute_checked(
            cwd,
            vec![
                String::from("--filter-target-type=python_test"),
                String::from("list"),
                String::from("::"),
            ],
        )?
        .stdout,
    )
    .to_string())
}

pub fn pants_test_target_output(cwd: &Path, spec: &str) -> PantsResult<std::process::Output> {
    execute_pants_command(cwd, vec![String::from("test"), spec.to_string()])
}

pub fn pants_affected_python_tests(cwd: &Path, spec: &str) -> PantsResult<String> {
    Ok(String::from_utf8_lossy(
        &execute_checked(
            cwd,
            vec![
                String::from("dependents"),
                String::from("--transitive"),
                String::from("--closed"),
                spec.to_string(),
            ],
        )?
        .stdout,
    )
    .to_string())
}

pub fn pants_goal_output(
    cwd: &Path,
    goal: &str,
    specs: &[String],
) -> PantsResult<std::process::Output> {
    let mut args = vec![goal.to_string()];
    args.extend_from_slice(specs);
    execute_pants_command(cwd, args)
}

pub fn pants_interactive_goal(cwd: &Path, goal: &str, specs: &[String]) -> PantsResult<bool> {
    let mut args = vec![goal.to_string()];
    args.extend_from_slice(specs);
    std::process::Command::new("pants")
        .current_dir(cwd)
        .args(&args)
        .status()
        .map(|status| status.success())
        .map_err(|source| PantsCommandError::ExecutionError {
            command: format!("pants {}", args.join(" ")),
            source,
        })
}
