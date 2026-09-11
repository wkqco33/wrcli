//! 오타 입력에 대한 "did you mean" 후보 탐색.

/// Cobra와 동일한 최소 편집 거리 임계값.
const MIN_DISTANCE: usize = 2;

/// 대소문자를 무시하는 Levenshtein 편집 거리.
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

/// `input`과 충분히 유사한 후보를 후보 순서대로 반환 (중복 제거).
///
/// Cobra와 같이 편집 거리가 2 이하이거나 후보가 `input`으로 시작하면 제안한다.
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
