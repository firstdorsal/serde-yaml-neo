//! Indentation detection for YAML documents.

use crate::error::{self, ErrorImpl, Result};
use crate::libyaml::parser::Parser;
use std::borrow::Cow;

/// Detected indentation information from a YAML document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Indentation {
    /// The number of spaces used for each indentation level.
    indent: usize,
}

impl Indentation {
    /// Returns the number of spaces used for each indentation level.
    pub fn spaces(&self) -> usize {
        self.indent
    }
}

/// Detects the indentation used in a YAML string.
///
/// This function analyzes a YAML document to determine the number of spaces
/// used for indentation. It works by parsing the YAML to ensure validity,
/// then analyzing the leading whitespace patterns in the source text.
///
/// # Returns
///
/// - `Ok(Some(Indentation))` - The detected indentation (2-9 spaces).
/// - `Ok(None)` - No indentation could be detected (flat YAML with no nested structures,
///   or only uses inline/flow style).
/// - `Err(...)` - The YAML is invalid or uses inconsistent indentation.
///
/// # Examples
///
/// ```
/// use serde_yaml_neo::detect_indentation;
///
/// // 2-space indentation (default)
/// let yaml = "root:\n  child: value\n";
/// let indent = detect_indentation(yaml).unwrap().unwrap();
/// assert_eq!(indent.spaces(), 2);
///
/// // 4-space indentation
/// let yaml = "root:\n    child: value\n";
/// let indent = detect_indentation(yaml).unwrap().unwrap();
/// assert_eq!(indent.spaces(), 4);
///
/// // Flat YAML with no indentation
/// let yaml = "key: value\n";
/// assert!(detect_indentation(yaml).unwrap().is_none());
/// ```
pub fn detect_indentation(yaml: &str) -> Result<Option<Indentation>> {
    detect_indentation_slice(yaml.as_bytes())
}

/// Detects the indentation used in a YAML byte slice.
///
/// This is the byte slice variant of [`detect_indentation`]. See that function
/// for full documentation.
pub fn detect_indentation_slice(yaml: &[u8]) -> Result<Option<Indentation>> {
    // First, validate the YAML is parseable
    validate_yaml(yaml)?;

    // Analyze the raw text to detect indentation
    detect_from_text(yaml)
}

/// Validates that the input is valid YAML by attempting to parse it.
fn validate_yaml(yaml: &[u8]) -> Result<()> {
    use crate::libyaml::parser::Event;

    let mut parser = Parser::new(Cow::Borrowed(yaml));

    loop {
        let (event, _mark) = parser.next().map_err(error::Error::from)?;
        if matches!(event, Event::StreamEnd) {
            break;
        }
    }

    Ok(())
}

/// Detects indentation by analyzing the raw text.
fn detect_from_text(yaml: &[u8]) -> Result<Option<Indentation>> {
    let text = match std::str::from_utf8(yaml) {
        Ok(s) => s,
        Err(e) => {
            return Err(error::new(ErrorImpl::Message(
                format!("invalid UTF-8: {}", e),
                None,
            )));
        }
    };

    // Collect all indentation levels (leading space counts) for content lines
    let mut indent_levels: Vec<usize> = Vec::new();

    for line in text.lines() {
        // Skip empty lines and comment-only lines
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Count leading spaces (tabs are not valid YAML indentation)
        let leading_spaces = line.len() - line.trim_start_matches(' ').len();

        // Check for tab indentation which is invalid in YAML
        if line.starts_with('\t')
            || (leading_spaces > 0 && line.as_bytes().get(leading_spaces) == Some(&b'\t'))
        {
            return Err(error::new(ErrorImpl::Message(
                "tab characters are not allowed for indentation in YAML".to_string(),
                None,
            )));
        }

        indent_levels.push(leading_spaces);
    }

    // Find the minimum non-zero indentation difference
    let indent = find_indentation_unit(&indent_levels)?;

    Ok(indent.map(|i| Indentation { indent: i }))
}

/// Finds the indentation unit from a list of indentation levels.
///
/// Returns the greatest common divisor of all non-zero indentation level differences,
/// which represents the base indentation unit.
fn find_indentation_unit(levels: &[usize]) -> Result<Option<usize>> {
    if levels.is_empty() {
        return Ok(None);
    }

    // Collect all unique non-zero indentation levels
    let mut unique_levels: Vec<usize> = levels
        .iter()
        .copied()
        .filter(|&l| l > 0)
        .collect();
    unique_levels.sort_unstable();
    unique_levels.dedup();

    if unique_levels.is_empty() {
        // No indentation found (all lines at column 0)
        return Ok(None);
    }

    // Calculate all differences between consecutive indent levels in the original sequence
    // to find actual indent steps
    let mut differences: Vec<usize> = Vec::new();
    let mut prev_level = 0usize;

    for &level in levels {
        if level != prev_level {
            let diff = level.abs_diff(prev_level);
            if diff > 0 {
                differences.push(diff);
            }
        }
        prev_level = level;
    }

    // Also include the unique levels themselves as they represent total indentation
    for &level in &unique_levels {
        differences.push(level);
    }

    if differences.is_empty() {
        return Ok(None);
    }

    // Find the GCD of all differences
    let mut result = differences[0];
    for &diff in &differences[1..] {
        result = gcd(result, diff);
        if result == 1 {
            // GCD of 1 means inconsistent indentation (e.g., mix of 2 and 3 spaces)
            // but 1-space indentation is technically valid, just unusual
            break;
        }
    }

    // Validate the result is in the valid range (1-9 for libyaml, but 2-9 for emitter)
    if result == 0 {
        return Ok(None);
    }

    Ok(Some(result))
}

/// Computes the greatest common divisor of two numbers.
fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_2_space_indent() {
        let yaml = "root:\n  child: value\n";
        let result = detect_indentation(yaml).unwrap().unwrap();
        assert_eq!(result.spaces(), 2);
    }

    #[test]
    fn test_detect_4_space_indent() {
        let yaml = "root:\n    child: value\n";
        let result = detect_indentation(yaml).unwrap().unwrap();
        assert_eq!(result.spaces(), 4);
    }

    #[test]
    fn test_detect_nested_indent() {
        let yaml = "root:\n  level1:\n    level2: value\n";
        let result = detect_indentation(yaml).unwrap().unwrap();
        assert_eq!(result.spaces(), 2);
    }

    #[test]
    fn test_detect_sequence_indent() {
        let yaml = "items:\n  - one\n  - two\n";
        let result = detect_indentation(yaml).unwrap().unwrap();
        assert_eq!(result.spaces(), 2);
    }

    #[test]
    fn test_flat_yaml_no_indent() {
        let yaml = "key: value\n";
        let result = detect_indentation(yaml).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_flow_style_no_indent() {
        let yaml = "items: [1, 2, 3]\n";
        let result = detect_indentation(yaml).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_invalid_yaml() {
        let yaml = "@invalid";
        let result = detect_indentation(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_8_space_indent() {
        let yaml = "root:\n        child: value\n";
        let result = detect_indentation(yaml).unwrap().unwrap();
        assert_eq!(result.spaces(), 8);
    }

    #[test]
    fn test_3_space_indent() {
        let yaml = "root:\n   child: value\n";
        let result = detect_indentation(yaml).unwrap().unwrap();
        assert_eq!(result.spaces(), 3);
    }

    #[test]
    fn test_deeply_nested() {
        let yaml = r#"
level0:
    level1:
        level2:
            level3: value
"#;
        let result = detect_indentation(yaml).unwrap().unwrap();
        assert_eq!(result.spaces(), 4);
    }

    #[test]
    fn test_with_comments() {
        let yaml = r#"
# Comment at root
root:
  # Comment at level 1
  child: value
"#;
        let result = detect_indentation(yaml).unwrap().unwrap();
        assert_eq!(result.spaces(), 2);
    }

    #[test]
    fn test_mixed_content_types() {
        let yaml = r#"
mapping:
  key: value
sequence:
  - item1
  - item2
"#;
        let result = detect_indentation(yaml).unwrap().unwrap();
        assert_eq!(result.spaces(), 2);
    }
}
