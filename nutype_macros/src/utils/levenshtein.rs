/// Compute the Levenshtein edit distance between two strings.
///
/// Used to suggest a likely-intended attribute name when the user mistypes one,
/// e.g. `validte` -> `validate`.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let m = a.len();
    let n = b.len();
    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr: Vec<usize> = vec![0; n + 1];
    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (curr[j - 1] + 1)
                .min(prev[j] + 1)
                .min(prev[j - 1] + cost);
        }
        core::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

/// Return the candidate closest to `query` if its edit distance is `<= max_distance`.
/// Ties are broken by the order in `candidates`.
pub fn closest_match<'a>(
    query: &str,
    candidates: &[&'a str],
    max_distance: usize,
) -> Option<&'a str> {
    let mut best: Option<(usize, &'a str)> = None;
    for cand in candidates {
        let d = levenshtein(query, cand);
        if d <= max_distance && best.map(|(bd, _)| d < bd).unwrap_or(true) {
            best = Some((d, cand));
        }
    }
    best.map(|(_, s)| s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_basics() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", "abc"), 0);
        assert_eq!(levenshtein("validte", "validate"), 1);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }

    #[test]
    fn picks_closest() {
        let candidates = ["sanitize", "validate", "derive", "default"];
        assert_eq!(closest_match("validte", &candidates, 2), Some("validate"));
        assert_eq!(closest_match("derve", &candidates, 2), Some("derive"));
        assert_eq!(closest_match("xyz", &candidates, 2), None);
    }
}
