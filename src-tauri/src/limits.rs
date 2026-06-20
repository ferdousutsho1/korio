use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Decision { None, Warn, Close }

#[derive(Debug, Clone, Default)]
pub struct LimitState {
    pub ignored: bool,
    pub snoozed_until: i64,
    pub warned: bool,
}

/// Decide what to do for ONE app this tick. Pure.
pub fn decide(today_seconds: i64, cap_seconds: i64, action: &str, st: &LimitState, now: i64) -> Decision {
    if cap_seconds <= 0 || today_seconds < cap_seconds {
        return Decision::None;
    }
    if st.ignored || now < st.snoozed_until {
        return Decision::None;
    }
    if action == "close" {
        return Decision::Close;
    }
    if st.warned { Decision::None } else { Decision::Warn }
}

/// In-memory per-app limit state, reset when the local day changes.
#[derive(Default)]
pub struct LimitRuntime {
    pub day: i64,
    pub states: HashMap<String, LimitState>,
}

impl LimitRuntime {
    pub fn new() -> Self { Self::default() }
    pub fn reset_if_new_day(&mut self, day_start: i64) {
        if day_start != self.day {
            self.day = day_start;
            self.states.clear();
        }
    }
    pub fn state_mut(&mut self, exe: &str) -> &mut LimitState {
        self.states.entry(exe.to_string()).or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn st() -> LimitState { LimitState::default() }

    #[test]
    fn no_cap_or_under_cap_is_none() {
        assert_eq!(decide(100, 0, "warn", &st(), 0), Decision::None);
        assert_eq!(decide(50, 100, "warn", &st(), 0), Decision::None);
    }
    #[test]
    fn over_cap_warns_once_then_silent() {
        assert_eq!(decide(120, 100, "warn", &st(), 0), Decision::Warn);
        let warned = LimitState { warned: true, ..Default::default() };
        assert_eq!(decide(120, 100, "warn", &warned, 0), Decision::None);
    }
    #[test]
    fn snooze_suppresses_until_expiry() {
        let s = LimitState { snoozed_until: 1000, ..Default::default() };
        assert_eq!(decide(120, 100, "warn", &s, 500), Decision::None);
        assert_eq!(decide(120, 100, "warn", &s, 1000), Decision::Warn);
    }
    #[test]
    fn ignore_suppresses_all_day() {
        let s = LimitState { ignored: true, ..Default::default() };
        assert_eq!(decide(999, 100, "warn", &s, 0), Decision::None);
        assert_eq!(decide(999, 100, "close", &s, 0), Decision::None);
    }
    #[test]
    fn close_action_closes_even_if_not_warned() {
        assert_eq!(decide(120, 100, "close", &st(), 0), Decision::Close);
    }
    #[test]
    fn reset_clears_states_on_new_day() {
        let mut rt = LimitRuntime::new();
        rt.reset_if_new_day(86_400);
        rt.state_mut("code.exe").warned = true;
        rt.reset_if_new_day(86_400);
        assert!(rt.states.contains_key("code.exe"));
        rt.reset_if_new_day(172_800);
        assert!(rt.states.is_empty());
    }
}
