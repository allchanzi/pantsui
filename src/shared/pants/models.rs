// The structs below mirror the `pants help-all` JSON schema in full, so fields are
// modelled even while nothing consumes them yet.
#![allow(dead_code)]

use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PantsFlag {
    pub config_key_label: String,
    pub cli_labels: Vec<String>,
    pub env_var_label: String,
    pub help: String,
    pub typ: String,
    pub default: Option<String>,
    pub choices: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct PantsGoal {
    pub name: String,
    pub description: String,
    pub flags: Vec<PantsFlag>,
}

/// Top-level object returned by `pants help-all`.
#[derive(Debug, Clone, Deserialize)]
pub struct AllHelpInfo {
    pub scope_to_help_info: HashMap<String, OptionScopeHelpInfo>,
    pub name_to_goal_info: HashMap<String, GoalHelpInfo>,
    pub name_to_target_type_info: HashMap<String, TargetTypeHelpInfo>,
    pub env_var_to_help_info: HashMap<String, OptionHelpInfo>,
    // Kept as raw JSON for now — model them when you actually consume them.
    pub name_to_rule_info: HashMap<String, Value>,
    pub name_to_api_type_info: HashMap<String, Value>,
    pub name_to_backend_help_info: HashMap<String, Value>,
    pub name_to_build_file_info: HashMap<String, Value>,
}

impl AllHelpInfo {
    pub fn goals(&self) -> Vec<PantsGoal> {
        let mut goals = self
            .name_to_goal_info
            .values()
            .filter(|goal| goal.is_implemented)
            .map(|goal| self.to_pants_goal(goal))
            .collect::<Vec<_>>();
        goals.sort_by(|left, right| left.name.cmp(&right.name));
        goals
    }

    fn to_pants_goal(&self, goal_help_info: &GoalHelpInfo) -> PantsGoal {
        let flags = goal_help_info
            .consumed_scopes
            .iter()
            .filter_map(|consumed_scope| self.scope_to_help_info.get(consumed_scope))
            .flat_map(|consumed_scope| consumed_scope.basic.iter())
            .map(|option_help| PantsFlag {
                config_key_label: option_help.config_key.clone(),
                cli_labels: option_help.scoped_cmd_line_args.clone(),
                env_var_label: option_help.env_var.clone(),
                help: option_help.help.clone(),
                typ: option_help.typ.clone(),
                default: option_help.default.as_str().map(String::from),
                choices: option_help.choices.clone(),
            })
            .collect();
        PantsGoal {
            name: goal_help_info.name.clone(),
            description: goal_help_info.description.clone(),
            flags,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoalHelpInfo {
    pub name: String,
    pub description: String,
    pub provider: String,
    pub is_implemented: bool,
    pub consumed_scopes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OptionScopeHelpInfo {
    pub scope: String,
    pub description: String,
    pub provider: String,
    pub is_goal: bool,
    pub deprecated_scope: Option<String>,
    pub basic: Vec<OptionHelpInfo>,
    pub advanced: Vec<OptionHelpInfo>,
    pub deprecated: Vec<OptionHelpInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OptionHelpInfo {
    pub display_args: Vec<String>,
    pub comma_separated_display_args: String,
    pub scoped_cmd_line_args: Vec<String>,
    pub unscoped_cmd_line_args: Vec<String>,
    pub env_var: String,
    pub config_key: String,
    pub target_field_name: Option<String>,
    pub typ: String,
    /// Can be null / int / string / list, so keep it as raw JSON.
    pub default: Value,
    pub help: String,
    pub deprecation_active: bool,
    pub deprecated_message: Option<String>,
    pub removal_version: Option<String>,
    pub removal_hint: Option<String>,
    pub choices: Option<Vec<String>>,
    pub comma_separated_choices: Option<String>,
    pub value_history: ValueHistory,
    pub fromfile: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ValueHistory {
    pub ranked_values: Vec<RankedValue>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RankedValue {
    pub details: Option<String>,
    pub rank: String,
    pub value: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TargetTypeHelpInfo {
    pub alias: String,
    pub provider: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub fields: Vec<TargetFieldHelpInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TargetFieldHelpInfo {
    pub alias: String,
    pub provider: Option<String>,
    pub description: Option<String>,
    pub type_hint: String,
    pub required: bool,
    pub default: Option<String>,
}
