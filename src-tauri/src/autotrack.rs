//! Auto-discovery of watchlist apps.
//!
//! When enabled, any non-system app you keep focused for longer than a threshold
//! is added to the watchlist automatically, so you don't have to remember to add
//! things by hand. All the decision logic here is pure and unit-tested; the tracker
//! loop only supplies ticks.

use std::collections::HashMap;

/// Windows shell / background processes that own a foreground window but are not
/// "apps" a user would want timed.
pub const SYSTEM_EXES: &[&str] = &[
    "explorer.exe",
    "searchapp.exe",
    "searchhost.exe",
    "shellexperiencehost.exe",
    "startmenuexperiencehost.exe",
    "applicationframehost.exe",
    "systemsettings.exe",
    "textinputhost.exe",
    "lockapp.exe",
    "sihost.exe",
    "dwm.exe",
    "runtimebroker.exe",
    "taskmgr.exe",
    "wininit.exe",
    "winlogon.exe",
    "csrss.exe",
    "ctfmon.exe",
    "peopleapp.exe",
    "widgets.exe",
    "openwith.exe",
    "rundll32.exe",
    "korio.exe",
];

/// Should this foreground app ever be auto-added to the watchlist?
///
/// Rejects known shell processes and anything living under the Windows directory,
/// which together cover the "system app" case without a hand-maintained allowlist.
/// `windows_dir` is the `%SystemRoot%` value (e.g. `C:\Windows`); an empty string
/// disables the path check.
pub fn is_auto_trackable(exe: &str, full_path: &str, windows_dir: &str) -> bool {
    let exe = exe.to_lowercase();
    if exe.is_empty() || !exe.ends_with(".exe") {
        return false;
    }
    if SYSTEM_EXES.contains(&exe.as_str()) {
        return false;
    }
    if !windows_dir.is_empty() && !full_path.is_empty() {
        let path = full_path.to_lowercase().replace('/', "\\");
        let mut root = windows_dir.to_lowercase().replace('/', "\\");
        if !root.ends_with('\\') {
            root.push('\\');
        }
        if path.starts_with(&root) {
            return false;
        }
    }
    true
}

/// "chrome.exe" → "Chrome". Best-effort label for an auto-added app.
pub fn pretty_name(exe: &str) -> String {
    let stem = exe.rsplit(['\\', '/']).next().unwrap_or(exe);
    let stem = stem.strip_suffix(".exe").or_else(|| stem.strip_suffix(".EXE")).unwrap_or(stem);
    let mut chars = stem.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => exe.to_string(),
    }
}

/// Per-day focus accumulator for apps that are NOT on the watchlist yet.
#[derive(Default)]
pub struct AutoTracker {
    day: i64,
    secs: HashMap<String, i64>,
    /// Exes already promoted today, so a crossing fires exactly once.
    promoted: HashMap<String, ()>,
}

impl AutoTracker {
    pub fn new() -> Self {
        Self { day: 0, secs: HashMap::new(), promoted: HashMap::new() }
    }

    /// Add `delta` focused seconds for `exe`. Returns true exactly once — on the
    /// tick where the running total first reaches `threshold_secs`. Counters reset
    /// when the local day changes.
    pub fn observe(&mut self, day: i64, exe: &str, delta: i64, threshold_secs: i64) -> bool {
        if day != self.day {
            self.day = day;
            self.secs.clear();
            self.promoted.clear();
        }
        if threshold_secs <= 0 || delta <= 0 {
            return false;
        }
        let key = exe.to_lowercase();
        if self.promoted.contains_key(&key) {
            return false;
        }
        let total = self.secs.entry(key.clone()).or_insert(0);
        *total += delta;
        if *total >= threshold_secs {
            self.promoted.insert(key, ());
            return true;
        }
        false
    }

    /// Forget an exe's progress — used once it joins the watchlist by any route.
    pub fn forget(&mut self, exe: &str) {
        self.secs.remove(&exe.to_lowercase());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_shell_and_windows_dir_processes() {
        assert!(!is_auto_trackable("explorer.exe", "C:\\Windows\\explorer.exe", "C:\\Windows"));
        assert!(!is_auto_trackable("notepad.exe", "C:\\Windows\\System32\\notepad.exe", "C:\\Windows"));
        assert!(!is_auto_trackable("korio.exe", "D:\\Apps\\korio.exe", "C:\\Windows"));
        assert!(!is_auto_trackable("", "", "C:\\Windows"));
        assert!(!is_auto_trackable("something", "D:\\x\\something", "C:\\Windows"));
    }

    #[test]
    fn accepts_normal_user_apps() {
        assert!(is_auto_trackable("Code.exe", "C:\\Users\\me\\AppData\\Local\\Programs\\Code.exe", "C:\\Windows"));
        assert!(is_auto_trackable("chrome.exe", "C:\\Program Files\\Google\\chrome.exe", "C:\\Windows"));
        // forward slashes and a trailing separator in the root both normalise
        assert!(!is_auto_trackable("notepad.exe", "C:/Windows/System32/notepad.exe", "C:\\Windows\\"));
    }

    #[test]
    fn pretty_name_strips_extension_and_capitalises() {
        assert_eq!(pretty_name("chrome.exe"), "Chrome");
        assert_eq!(pretty_name("Code.exe"), "Code");
        assert_eq!(pretty_name("obs64.EXE"), "Obs64");
    }

    #[test]
    fn fires_once_when_threshold_is_crossed() {
        let mut a = AutoTracker::new();
        let day = 1_000;
        for _ in 0..9 {
            assert!(!a.observe(day, "chrome.exe", 1, 10));
        }
        assert!(a.observe(day, "chrome.exe", 1, 10), "crossing tick fires");
        for _ in 0..5 {
            assert!(!a.observe(day, "chrome.exe", 1, 10), "no repeat fires");
        }
    }

    #[test]
    fn counters_reset_on_a_new_day() {
        let mut a = AutoTracker::new();
        assert!(a.observe(1, "code.exe", 10, 10));
        // new day → progress and the promoted mark are both cleared
        assert!(!a.observe(2, "code.exe", 5, 10));
        assert!(a.observe(2, "code.exe", 5, 10));
    }

    #[test]
    fn zero_threshold_never_fires() {
        let mut a = AutoTracker::new();
        assert!(!a.observe(1, "code.exe", 100, 0));
    }
}
