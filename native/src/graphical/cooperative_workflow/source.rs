//! Linear bounded comparison of actual retained editable source states.
/// Show changed lines after common prefix/suffix removal. This does not claim
/// a minimal diff, HMC resource comparison or a historical revision number.
pub fn source_difference(previous: Option<&str>, current: &str) -> String {
    let Some(previous) = previous else {
        return "No retained undo predecessor. Inspect the full current source below.".into();
    };
    let old: Vec<_> = previous.lines().collect();
    let new: Vec<_> = current.lines().collect();
    let prefix = old.iter().zip(&new).take_while(|(a, b)| a == b).count();
    let suffix = old[prefix..]
        .iter()
        .rev()
        .zip(new[prefix..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    if prefix == old.len() && prefix == new.len() {
        return "No editable source difference from the retained undo predecessor.".into();
    }
    let mut output =
        format!("Unchanged prefix: {prefix} lines; unchanged suffix: {suffix} lines\n");
    for (marker, lines) in [
        ("-", &old[prefix..old.len() - suffix]),
        ("+", &new[prefix..new.len() - suffix]),
    ] {
        for line in lines {
            if output.len().saturating_add(line.len()).saturating_add(3) > 128 * 1024 {
                output.push_str("\nComparison display truncated at 128 KiB; inspect full source.");
                return output;
            }
            output.push_str(marker);
            output.push(' ');
            output.push_str(line);
            output.push('\n');
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn comparison_retains_replaced_lines_and_empty_edges() {
        assert!(source_difference(Some("a\nb\nz"), "a\nc\nz").contains("- b\n+ c"));
        assert!(source_difference(Some(""), "a").contains("+ a"));
        assert!(source_difference(Some("a"), "").contains("- a"));
    }
}
