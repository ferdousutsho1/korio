//! Wave 3b: per-domain session accumulation, parallel to the app SessionTracker.
//! The tracker loop decides when a domain "counts" (browser focused, user active,
//! ActiveSite fresh) and passes Some(domain); this type accumulates contiguous time.

use serde::Serialize;

pub struct SiteSample {
    pub now: i64,                       // unix seconds
    pub focused_domain: Option<String>, // domain to count this tick, or None
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FinishedSiteSession {
    pub domain: String,
    pub started_at: i64,
    pub ended_at: i64,
    pub seconds: i64,
}

struct Active {
    domain: String,
    started_at: i64,
    seconds: i64,
    last_tick: i64,
}

#[derive(Default)]
pub struct SiteTracker {
    current: Option<Active>,
}

impl SiteTracker {
    pub fn new() -> Self {
        Self { current: None }
    }

    pub fn current_domain(&self) -> Option<(&str, i64)> {
        self.current.as_ref().map(|a| (a.domain.as_str(), a.seconds))
    }

    pub fn observe(&mut self, s: SiteSample) -> Option<FinishedSiteSession> {
        match s.focused_domain {
            Some(domain) => match &mut self.current {
                Some(a) if a.domain == domain => {
                    a.seconds += s.now - a.last_tick;
                    a.last_tick = s.now;
                    None
                }
                _ => {
                    let finished = self.finish(s.now);
                    self.current = Some(Active { domain, started_at: s.now, seconds: 0, last_tick: s.now });
                    finished
                }
            },
            None => self.finish(s.now),
        }
    }

    pub fn finish(&mut self, now: i64) -> Option<FinishedSiteSession> {
        let a = self.current.take()?;
        if a.seconds <= 0 {
            return None;
        }
        Some(FinishedSiteSession { domain: a.domain, started_at: a.started_at, ended_at: now, seconds: a.seconds })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample(now: i64, domain: Option<&str>) -> SiteSample {
        SiteSample { now, focused_domain: domain.map(|d| d.to_string()) }
    }

    #[test]
    fn accumulates_time_for_a_domain() {
        let mut t = SiteTracker::new();
        assert_eq!(t.observe(sample(100, Some("youtube.com"))), None);
        assert_eq!(t.observe(sample(101, Some("youtube.com"))), None);
        assert_eq!(t.observe(sample(104, Some("youtube.com"))), None);
        let f = t.finish(104).unwrap();
        assert_eq!(f.domain, "youtube.com");
        assert_eq!(f.seconds, 4);
        assert_eq!(f.started_at, 100);
        assert_eq!(f.ended_at, 104);
    }

    #[test]
    fn switching_domains_finishes_previous_and_starts_new() {
        let mut t = SiteTracker::new();
        t.observe(sample(100, Some("youtube.com")));
        t.observe(sample(105, Some("youtube.com")));
        let f = t.observe(sample(106, Some("reddit.com"))).unwrap();
        assert_eq!(f.domain, "youtube.com");
        assert_eq!(f.seconds, 5);
        t.observe(sample(110, Some("reddit.com")));
        assert_eq!(t.finish(110).unwrap().domain, "reddit.com");
    }

    #[test]
    fn none_finishes_current() {
        let mut t = SiteTracker::new();
        t.observe(sample(100, Some("youtube.com")));
        t.observe(sample(103, Some("youtube.com")));
        let f = t.observe(sample(104, None)).unwrap();
        assert_eq!(f.seconds, 3);
        assert!(t.current_domain().is_none());
        assert_eq!(t.observe(sample(105, None)), None);
    }
}
