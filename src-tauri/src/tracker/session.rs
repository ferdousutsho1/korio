use std::collections::HashSet;

/// One observation of the system at a point in time.
#[derive(Debug, Clone)]
pub struct Sample {
    pub now: i64,                    // unix seconds
    pub focused_exe: Option<String>, // lower-cased exe of the foreground window
    pub idle: bool,                  // true if user is past the idle threshold
}

/// A completed session, ready to persist.
#[derive(Debug, Clone, PartialEq)]
pub struct FinishedSession {
    pub exe: String,
    pub started_at: i64,
    pub ended_at: i64,
    pub active_seconds: i64,
}

struct Active {
    exe: String,
    started_at: i64,
    active_seconds: i64,
    last_tick: i64,
}

pub struct SessionTracker {
    watched: HashSet<String>, // lower-cased exe names
    current: Option<Active>,
}

impl SessionTracker {
    pub fn new(watched: HashSet<String>) -> Self {
        Self { watched, current: None }
    }

    pub fn set_watched(&mut self, watched: HashSet<String>) {
        self.watched = watched;
    }

    /// Info about the in-progress session for the live UI indicator.
    pub fn current_exe(&self) -> Option<(&str, i64)> {
        self.current.as_ref().map(|a| (a.exe.as_str(), a.active_seconds))
    }

    /// Feed one sample. Returns a FinishedSession when one ends this tick.
    pub fn observe(&mut self, s: Sample) -> Option<FinishedSession> {
        let counting = match &s.focused_exe {
            Some(exe) => !s.idle && self.watched.contains(exe),
            None => false,
        };

        if counting {
            let exe = s.focused_exe.clone().unwrap();
            match &mut self.current {
                Some(a) if a.exe == exe => {
                    a.active_seconds += s.now - a.last_tick;
                    a.last_tick = s.now;
                    None
                }
                _ => {
                    // different app (or none) was active: finish previous, start new
                    let finished = self.finish(s.now);
                    self.current = Some(Active {
                        exe,
                        started_at: s.now,
                        active_seconds: 0,
                        last_tick: s.now,
                    });
                    finished
                }
            }
        } else {
            self.finish(s.now)
        }
    }

    /// End the current session (on blur/idle/close/app-exit). Sessions with
    /// zero active time are discarded.
    pub fn finish(&mut self, now: i64) -> Option<FinishedSession> {
        let a = self.current.take()?;
        if a.active_seconds <= 0 {
            return None;
        }
        Some(FinishedSession {
            exe: a.exe,
            started_at: a.started_at,
            ended_at: now,
            active_seconds: a.active_seconds,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn watched(list: &[&str]) -> HashSet<String> {
        list.iter().map(|s| s.to_string()).collect()
    }
    fn sample(now: i64, exe: Option<&str>, idle: bool) -> Sample {
        Sample { now, focused_exe: exe.map(|e| e.to_string()), idle }
    }

    #[test]
    fn accumulates_active_time_for_watched_app() {
        let mut t = SessionTracker::new(watched(&["code.exe"]));
        assert_eq!(t.observe(sample(100, Some("code.exe"), false)), None); // start, 0s
        assert_eq!(t.observe(sample(101, Some("code.exe"), false)), None); // +1
        assert_eq!(t.observe(sample(103, Some("code.exe"), false)), None); // +2
        let f = t.finish(103).unwrap();
        assert_eq!(f.active_seconds, 3);
        assert_eq!(f.started_at, 100);
        assert_eq!(f.ended_at, 103);
    }

    #[test]
    fn idle_pauses_then_resumes_without_double_counting() {
        let mut t = SessionTracker::new(watched(&["code.exe"]));
        t.observe(sample(100, Some("code.exe"), false));     // start
        t.observe(sample(101, Some("code.exe"), false));     // +1  => 1s
        let f = t.observe(sample(160, Some("code.exe"), true)); // idle: finish 1s session
        assert_eq!(f.unwrap().active_seconds, 1);
        t.observe(sample(161, Some("code.exe"), false));     // resume: new session start
        t.observe(sample(163, Some("code.exe"), false));     // +2
        assert_eq!(t.finish(163).unwrap().active_seconds, 2);
    }

    #[test]
    fn switching_apps_finishes_previous_and_starts_new() {
        let mut t = SessionTracker::new(watched(&["code.exe", "figma.exe"]));
        t.observe(sample(100, Some("code.exe"), false));
        t.observe(sample(105, Some("code.exe"), false));     // 5s on code
        let f = t.observe(sample(106, Some("figma.exe"), false)); // switch
        let f = f.unwrap();
        assert_eq!(f.exe, "code.exe");
        assert_eq!(f.active_seconds, 5);
        t.observe(sample(110, Some("figma.exe"), false));
        assert_eq!(t.finish(110).unwrap().exe, "figma.exe");
    }

    #[test]
    fn unwatched_app_is_ignored() {
        let mut t = SessionTracker::new(watched(&["code.exe"]));
        assert_eq!(t.observe(sample(100, Some("notepad.exe"), false)), None);
        assert_eq!(t.observe(sample(105, None, false)), None);
        assert!(t.current_exe().is_none());
    }
}
