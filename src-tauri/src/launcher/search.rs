use nucleo::pattern::{CaseMatching, MultiPattern, Normalization};
use nucleo::{Config, Matcher, Utf32String};

use crate::launcher::types::{AppEntry, SearchResult};

/// Weight given to launch-frequency relative to the fuzzy match score.
const FREQUENCY_WEIGHT: f32 = 0.3;

/// Fuzzy-rank `entries` against `query`, blending match score with launch frequency.
/// Returns results sorted descending by blended score, capped at `limit`.
pub fn rank(
    query: &str,
    entries: &[AppEntry],
    usage_counts: &[(String, u32)],
    limit: usize,
) -> Vec<SearchResult> {
    if entries.is_empty() {
        return Vec::new();
    }

    let max_count = usage_counts.iter().map(|(_, c)| *c).max().unwrap_or(1).max(1) as f32;

    if query.is_empty() {
        // No query — sort by usage frequency, return top-`limit`.
        let mut results: Vec<SearchResult> = entries
            .iter()
            .map(|entry| {
                let count = usage_count_for(&entry.id, usage_counts);
                SearchResult {
                    entry: entry.clone(),
                    score: count as f32 / max_count,
                    launch_count: count,
                }
            })
            .collect();
        results.sort_by(|a, b| {
            b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);
        return results;
    }

    // Build a fuzzy pattern with nucleo_matcher (synchronous, single-threaded).
    let mut matcher = Matcher::new(Config::DEFAULT);
    let mut pattern = MultiPattern::new(1);
    pattern.reparse(0, query, CaseMatching::Smart, Normalization::Smart, false);

    // Score every entry.
    let scored: Vec<(AppEntry, u32, u32)> = entries
        .iter()
        .filter_map(|entry| {
            let haystack = [Utf32String::from(build_haystack(entry).as_str())];
            let raw = pattern.score(&haystack, &mut matcher)?;
            let count = usage_count_for(&entry.id, usage_counts);
            Some((entry.clone(), raw, count))
        })
        .collect();

    if scored.is_empty() {
        return Vec::new();
    }

    let max_raw = scored.iter().map(|(_, s, _)| *s).max().unwrap_or(1).max(1) as f32;

    let mut results: Vec<SearchResult> = scored
        .into_iter()
        .map(|(entry, raw, count)| {
            let fuzzy = raw as f32 / max_raw;
            let freq = count as f32 / max_count;
            let blended = fuzzy * (1.0 - FREQUENCY_WEIGHT) + freq * FREQUENCY_WEIGHT;
            SearchResult { entry, score: blended, launch_count: count }
        })
        .collect();

    results.sort_by(|a, b| {
        b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(limit);
    results
}

fn build_haystack(entry: &AppEntry) -> String {
    let mut parts = vec![entry.name.clone()];
    if let Some(desc) = &entry.description {
        parts.push(desc.clone());
    }
    parts.extend_from_slice(&entry.keywords);
    parts.join(" ")
}

fn usage_count_for(id: &str, counts: &[(String, u32)]) -> u32 {
    counts.iter().find(|(k, _)| k == id).map(|(_, v)| *v).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(name: &str, description: &str) -> AppEntry {
        AppEntry {
            id: name.to_lowercase().replace(' ', "_"),
            name: name.to_string(),
            description: Some(description.to_string()),
            icon: None,
            exec: name.to_lowercase(),
            terminal: false,
            categories: vec![],
            keywords: vec![],
            desktop_file: format!("/usr/share/applications/{}.desktop", name.to_lowercase()),
        }
    }

    #[test]
    fn empty_query_sorts_by_frequency() {
        let entries = vec![
            make_entry("Firefox", "Web browser"),
            make_entry("Terminal", "Terminal emulator"),
            make_entry("Code", "Editor"),
        ];
        let usage = vec![
            ("firefox".to_string(), 10),
            ("code".to_string(), 5),
        ];
        let results = rank("", &entries, &usage, 10);
        assert_eq!(results[0].entry.name, "Firefox");
        assert_eq!(results[1].entry.name, "Code");
    }

    #[test]
    fn fuzzy_match_returns_relevant() {
        let entries = vec![
            make_entry("Firefox", "Web browser"),
            make_entry("Files", "File manager"),
            make_entry("Terminal", "Terminal emulator"),
        ];
        let results = rank("fire", &entries, &[], 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].entry.name, "Firefox");
    }

    #[test]
    fn no_match_returns_empty() {
        let entries = vec![make_entry("Firefox", "Web browser")];
        let results = rank("zzzzzzzzz", &entries, &[], 10);
        assert!(results.is_empty());
    }

    #[test]
    fn limit_is_respected() {
        let entries: Vec<AppEntry> =
            (0..20).map(|i| make_entry(&format!("App{i}"), "")).collect();
        let results = rank("App", &entries, &[], 5);
        assert!(results.len() <= 5);
    }
}
