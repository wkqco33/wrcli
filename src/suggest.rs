//! "Did you mean" candidate search for typo input.

/// Minimum edit distance threshold, same as Cobra.
const MIN_DISTANCE: usize = 2;

/// Case-insensitive Levenshtein edit distance.
fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.to_lowercase().chars().collect();
    let b: Vec<char> = b.to_lowercase().chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }

    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0usize; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

/// Returns candidates sufficiently similar to `input`, in candidate order (deduplicated).
///
/// Suggests candidates whose edit distance is 2 or less or that start with `input`, as Cobra does.
pub(crate) fn closest<'a>(
    input: &str,
    candidates: impl IntoIterator<Item = &'a str>,
) -> Vec<String> {
    let lower = input.to_lowercase();
    let mut out: Vec<String> = Vec::new();
    for candidate in candidates {
        let suggest = levenshtein(input, candidate) <= MIN_DISTANCE
            || candidate.to_lowercase().starts_with(&lower);
        if suggest && !out.iter().any(|c| c == candidate) {
            out.push(candidate.to_owned());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levenshtein_basic() {
        assert_eq!(levenshtein("greet", "greet"), 0);
        assert_eq!(levenshtein("gret", "greet"), 1);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("GREET", "greet"), 0);
    }

    #[test]
    fn closest_matches_typo_and_prefix() {
        let names = ["greet", "echo", "fail"];
        assert_eq!(closest("gret", names), vec!["greet"]);
        assert_eq!(closest("ec", names), vec!["echo"]);
        assert!(closest("zzzzzz", names).is_empty());
    }

    #[test]
    fn closest_dedupes() {
        assert_eq!(closest("greet", ["greet", "greet"]), vec!["greet"]);
    }
}
