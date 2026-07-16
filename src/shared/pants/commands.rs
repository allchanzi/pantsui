use crate::shared::pants::{
    errors::{PantsCommandError, PantsResult},
    models::AllHelpInfo,
};

fn execute_pants_command(args: Vec<String>) -> PantsResult<std::process::Output> {
    std::process::Command::new("pants")
        .args(&args)
        .output()
        .map_err(|source| PantsCommandError::ExecutionError {
            command: format!("pants {}", args.join(" ")),
            source,
        })
}

pub fn pants_help_all() -> PantsResult<AllHelpInfo> {
    serde_json::from_slice(&execute_pants_command(vec![String::from("help-all")])?.stdout)
        .map_err(|err| PantsCommandError::from(err))
}

pub fn pants_version() -> PantsResult<String> {
    Ok(
        String::from_utf8_lossy(&execute_pants_command(vec![String::from("version")])?.stdout)
            .to_string(),
    )
}

pub fn pants_list_python_tests() -> PantsResult<String> {
    Ok(String::from_utf8_lossy(
        &execute_pants_command(vec![
            String::from("list"),
            String::from("--filter-target-type=python_test"),
            String::from("::"),
        ])?
        .stdout,
    )
    .to_string())
}

pub fn pants_test_target(spec: &str) -> PantsResult<bool> {
    let output = execute_pants_command(vec![String::from("test"), spec.to_string()])?;
    Ok(output.status.success())
}

pub fn pants_test_target_output(spec: &str) -> PantsResult<std::process::Output> {
    execute_pants_command(vec![String::from("test"), spec.to_string()])
}
