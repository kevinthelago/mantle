/// Permitted npm packages. Anything not in this list is rejected before a
/// network request is even made.
pub static ALLOWED_PACKAGES: &[&str] = &[
    "sass",
    "sass-embedded",
    "tailwindcss",
    "postcss",
    "postcss-import",
    "autoprefixer",
    "cssnano",
    "postcss-nested",
    "postcss-custom-properties",
];

pub fn is_allowed(pkg: &str) -> bool {
    ALLOWED_PACKAGES.contains(&pkg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_packages_pass() {
        assert!(is_allowed("sass"));
        assert!(is_allowed("sass-embedded"));
        assert!(is_allowed("tailwindcss"));
        assert!(is_allowed("postcss"));
        assert!(is_allowed("autoprefixer"));
    }

    #[test]
    fn unknown_packages_are_rejected() {
        assert!(!is_allowed("lodash"));
        assert!(!is_allowed("express"));
        assert!(!is_allowed("rm -rf /"));
        assert!(!is_allowed("../../etc/passwd"));
        assert!(!is_allowed(""));
    }
}
