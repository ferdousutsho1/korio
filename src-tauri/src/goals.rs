use serde::Serialize;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct Streaks {
    pub current: i64,
    pub best: i64,
}

/// Is a single day's `seconds` total meeting the goal?
pub fn met(seconds: i64, target: i64, gte: bool) -> bool {
    if gte { seconds >= target } else { seconds <= target }
}

fn trailing_true(m: &[bool]) -> i64 {
    m.iter().rev().take_while(|&&b| b).count() as i64
}

/// `daily` = per-local-day seconds, oldest→newest, last element = today.
/// `gte` selects direction and the today-rule:
///  - best  = longest run of met days anywhere in the window.
///  - current = trailing run of met days ending today; if today is NOT met:
///      * gte goal → today is "pending" (ignore it, keep the run ending yesterday)
///      * lte goal → today is exceeded/failed → current resets to 0.
pub fn streaks(daily: &[i64], target: i64, gte: bool) -> Streaks {
    let m: Vec<bool> = daily.iter().map(|&s| met(s, target, gte)).collect();
    let mut best = 0i64;
    let mut run = 0i64;
    for &b in &m {
        if b { run += 1; best = best.max(run); } else { run = 0; }
    }
    let current = if m.is_empty() {
        0
    } else if *m.last().unwrap() {
        trailing_true(&m)
    } else if gte {
        trailing_true(&m[..m.len() - 1]) // today pending → run ending yesterday
    } else {
        0 // lte exceeded today → broken
    };
    Streaks { current, best }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn met_both_directions() {
        assert!(met(120, 100, true));   // >= hit
        assert!(!met(80, 100, true));
        assert!(met(80, 100, false));   // <= under
        assert!(!met(120, 100, false));
    }

    #[test]
    fn empty_history_is_zero() {
        assert_eq!(streaks(&[], 100, true), Streaks { current: 0, best: 0 });
    }

    #[test]
    fn all_met_gte() {
        // five days all >= target
        assert_eq!(streaks(&[100, 100, 100, 100, 100], 100, true), Streaks { current: 5, best: 5 });
    }

    #[test]
    fn gte_today_pending_keeps_prior_streak() {
        // three met days then an unmet (in-progress) today → current = 3, best = 3
        assert_eq!(streaks(&[100, 100, 100, 0], 100, true), Streaks { current: 3, best: 3 });
    }

    #[test]
    fn gte_gap_then_recovery() {
        // met,met, miss, met, met(today) → current 2, best 2
        assert_eq!(streaks(&[100, 100, 0, 100, 100], 100, true), Streaks { current: 2, best: 2 });
    }

    #[test]
    fn lte_exceeded_today_breaks() {
        // under,under,under, then over today → current 0, best 3
        assert_eq!(streaks(&[10, 10, 10, 999], 100, false), Streaks { current: 0, best: 3 });
    }

    #[test]
    fn best_can_exceed_current() {
        // long early run, recent short run ending today
        assert_eq!(streaks(&[100, 100, 100, 100, 0, 100, 100], 100, true), Streaks { current: 2, best: 4 });
    }
}
