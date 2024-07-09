use regex::Regex;
use std::collections::HashMap;

pub fn parse_tool_versions_content(file_content: &str) -> HashMap<String, String> {
    let re = Regex::new(r"\s+").unwrap();
    file_content
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| {
            let parts: Vec<&str> = re.splitn(line, 2).collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tool_versions_content() {
        let file_content = "\
# This is a comment
python 3.10.4
 poetry  1.7.1
# exact ruby version required for homebrew
ruby\t3.1.4
direnv  2.32.3 ";
        let versions = parse_tool_versions_content(file_content);

        let expected = HashMap::from([
            ("direnv".to_string(), "2.32.3".to_string()),
            ("python".to_string(), "3.10.4".to_string()),
            ("ruby".to_string(), "3.1.4".to_string()),
            ("poetry".to_string(), "1.7.1".to_string()),
        ]);

        assert_eq!(versions, expected);
    }
}
