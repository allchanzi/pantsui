use serde_json::Value;

use super::model::TargetItem;

pub fn parse_target_specs(json: &str) -> Vec<TargetItem> {
    let value: Value = serde_json::from_str(json).unwrap_or(Value::Null);

    match value {
        Value::Array(values) => values
            .into_iter()
            .filter_map(|value| value.as_str().map(TargetItem::new))
            .collect(),
        Value::String(spec) => vec![TargetItem::new(spec)],
        _ => Vec::new(),
    }
}

pub fn parse_target_specs_text(text: &str) -> Vec<TargetItem> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(TargetItem::new)
        .collect()
}

pub fn parse_target_specs_response(text: &str) -> Vec<TargetItem> {
    let parsed = parse_target_specs(text);

    if parsed.is_empty() {
        parse_target_specs_text(text)
    } else {
        parsed
    }
}

#[cfg(test)]
use super::model::TargetStatus;

#[cfg(test)]
pub fn apply_selection(targets: &mut [TargetItem], selected_index: usize) {
    for (index, target) in targets.iter_mut().enumerate() {
        target.status = if index == selected_index {
            TargetStatus::Selected
        } else {
            TargetStatus::Loaded
        };
    }
}

pub fn sort_targets(mut targets: Vec<TargetItem>) -> Vec<TargetItem> {
    targets.sort_by(|left, right| left.spec.cmp(&right.spec));
    targets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json_array_of_specs() {
        let targets = parse_target_specs(r#"["src/app:tests", "src/lib:unit_tests"]"#);

        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0].spec, "src/app:tests");
        assert_eq!(targets[1].spec, "src/lib:unit_tests");
    }

    #[test]
    fn parses_json_string_spec() {
        let targets = parse_target_specs(r#""src/app:tests""#);

        assert_eq!(targets, vec![TargetItem::new("src/app:tests")]);
    }

    #[test]
    fn parses_newline_separated_specs() {
        let targets = parse_target_specs_text("src/a:test\nsrc/b:test\n");

        assert_eq!(
            targets,
            vec![TargetItem::new("src/a:test"), TargetItem::new("src/b:test")]
        );
    }

    #[test]
    fn parses_response_preferring_json() {
        let targets = parse_target_specs_response(r#"["src/a:test"]"#);

        assert_eq!(targets, vec![TargetItem::new("src/a:test")]);
    }

    #[test]
    fn selects_one_target_and_marks_others_loaded() {
        let mut targets = vec![TargetItem::new("b"), TargetItem::new("a"), TargetItem::new("c")];

        apply_selection(&mut targets, 1);

        assert_eq!(targets[0].status, TargetStatus::Loaded);
        assert_eq!(targets[1].status, TargetStatus::Selected);
        assert_eq!(targets[2].status, TargetStatus::Loaded);
    }

    #[test]
    fn sorts_targets_by_spec() {
        let targets = sort_targets(vec![TargetItem::new("b"), TargetItem::new("a")]);

        assert_eq!(targets[0].spec, "a");
        assert_eq!(targets[1].spec, "b");
    }
}
