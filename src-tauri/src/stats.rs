use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct DayTotal {
    pub day: i64,     // unix seconds at the start of the (local) day
    pub seconds: i64, // total active seconds bucketed into that day
}

/// Bucket (started_at, active_seconds) pairs into per-day totals, sorted by day.
/// `day_start_of` maps a unix timestamp to the start-of-day it belongs to,
/// letting callers choose local vs UTC day boundaries (and keeping this testable).
pub fn daily_totals<F>(sessions: &[(i64, i64)], day_start_of: F) -> Vec<DayTotal>
where
    F: Fn(i64) -> i64,
{
    let mut map: BTreeMap<i64, i64> = BTreeMap::new();
    for (started_at, active) in sessions {
        *map.entry(day_start_of(*started_at)).or_insert(0) += *active;
    }
    map.into_iter().map(|(day, seconds)| DayTotal { day, seconds }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn floor_day(ts: i64) -> i64 { ts - ts.rem_euclid(86_400) }

    #[test]
    fn buckets_and_sums_by_day_sorted() {
        let sessions = vec![(10, 100), (200, 50), (86_400 + 5, 70)];
        let totals = daily_totals(&sessions, floor_day);
        assert_eq!(totals, vec![
            DayTotal { day: 0, seconds: 150 },
            DayTotal { day: 86_400, seconds: 70 },
        ]);
    }

    #[test]
    fn empty_input_yields_empty() {
        let totals = daily_totals(&[], floor_day);
        assert!(totals.is_empty());
    }
}
