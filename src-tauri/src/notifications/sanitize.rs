/// Sanitize FDO notification body markup.
/// Spec allows only: <b>, <i>, <u>, <a href="...">, <img src="..." alt="..."/>
/// Everything else is stripped; bare text is preserved.
pub fn sanitize_body(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }
    // Use ammonia with a strict allowlist matching the FDO spec subset.
    ammonia::Builder::new()
        .tags(std::collections::HashSet::from(["b", "i", "u", "a", "img"]))
        .tag_attributes({
            let mut map = std::collections::HashMap::new();
            map.insert("a", std::collections::HashSet::from(["href"]));
            map.insert("img", std::collections::HashSet::from(["src", "alt"]));
            map
        })
        // Strip all other attributes from allowed tags
        .generic_attributes(std::collections::HashSet::new())
        .clean(input)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_fdo_subset() {
        let input = r#"<b>Bold</b> and <i>italic</i> and <a href="https://example.com">link</a>"#;
        let out = sanitize_body(input);
        assert!(out.contains("<b>Bold</b>"));
        assert!(out.contains("<i>italic</i>"));
        assert!(out.contains(r#"href="https://example.com""#));
    }

    #[test]
    fn strips_script() {
        let out = sanitize_body("<script>alert(1)</script>hello");
        assert!(!out.contains("<script>"));
        assert!(out.contains("hello"));
    }

    #[test]
    fn strips_unknown_tags() {
        let out = sanitize_body("<span class=\"x\">text</span>");
        assert!(!out.contains("<span"));
        assert!(out.contains("text"));
    }

    #[test]
    fn empty_input() {
        assert_eq!(sanitize_body(""), "");
    }
}
