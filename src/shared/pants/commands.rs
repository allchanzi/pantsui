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
