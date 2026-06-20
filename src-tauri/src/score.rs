use crate::db::queries::UsageSlice;

/// A 0–100 focus score: productive time rewarded, distracting time penalized,
/// neutral ignored. 50 = balanced / no data.
pub fn focus_score(slices: &[UsageSlice]) -> u8 {
    let mut prod = 0i64;
    let mut dist = 0i64;
    for s in slices {
        match s.kind.as_str() {
            "productive" => prod += s.seconds,
            "distracting" => dist += s.seconds,
            _ => {}
        }
    }
    let denom = prod + dist;
    if denom == 0 {
        return 50;
    }
    ((prod as f64 / denom as f64) * 100.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    fn slice(kind: &str, secs: i64) -> UsageSlice {
        UsageSlice { app_id: 1, display_name: "x".into(), color: "#000".into(),
            kind: kind.into(), seconds: secs }
    }
    #[test]
    fn all_productive_is_100() {
        assert_eq!(focus_score(&[slice("productive", 600)]), 100);
    }
    #[test]
    fn half_and_half_is_50() {
        assert_eq!(focus_score(&[slice("productive", 600), slice("distracting", 600)]), 50);
    }
    #[test]
    fn neutral_is_ignored_and_no_data_is_50() {
        assert_eq!(focus_score(&[slice("neutral", 9999)]), 50);
        assert_eq!(focus_score(&[]), 50);
    }
}
