//! Wave 3a: loopback receiver for the companion browser extension.
//! Reduces reported URLs to registrable domains; never stores full URLs.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Manager};
use crate::AppState;

/// The domain currently shown in the focused browser, with when it was last reported.
#[derive(Clone)]
pub struct ActiveSite {
    pub domain: String,
    pub updated_at: i64, // unix seconds
}

/// Runtime handles for the loopback server. `None` when the feature is off.
#[derive(Default)]
pub struct BrowserRuntime {
    pub active: Option<ActiveSite>,
    pub stop: Option<Arc<AtomicBool>>,
    pub blocked: std::collections::HashSet<String>,
}

/// Extract the host portion of a URL string (no scheme, no userinfo, no port, no path).
/// Returns None if there is no `scheme://host` shape.
fn host_of(raw: &str) -> Option<&str> {
    let after = raw.split_once("://")?.1;
    let host_port = after.split(['/', '?', '#']).next()?;
    let host = host_port.rsplit('@').next()?; // strip optional userinfo
    // IPv6 literals are bracketed; reject them here rather than trying to strip a port from them.
    if host.starts_with('[') { return None; }
    let host = host.split(':').next()?; // strip optional port
    if host.is_empty() { None } else { Some(host) }
}

/// Reduce a URL to its registrable domain (eTLD+1), lowercased.
/// Returns None for IPs, localhost, non-http(s) schemes, and unparseable input.
pub fn registrable_domain(raw: &str) -> Option<String> {
    let host = host_of(raw)?.to_ascii_lowercase();
    // Reject numeric IPv4 addresses: every label is all digits.
    if host.split('.').all(|label| label.chars().all(|c| c.is_ascii_digit())) {
        return None;
    }
    // psl returns the registrable domain only for hosts under a real public suffix.
    // IPs and bare "localhost" have no public suffix → None.
    psl::domain_str(&host).map(|d| d.to_string())
}

/// Recognized Chromium-family browser executables (lower-cased comparison).
pub fn is_browser_exe(exe: &str) -> bool {
    matches!(
        exe.to_ascii_lowercase().as_str(),
        "chrome.exe" | "msedge.exe" | "brave.exe" | "vivaldi.exe" | "opera.exe" | "chromium.exe" | "arc.exe"
    )
}

/// Return the persisted extension token, generating + storing one on first use.
pub fn ensure_token(conn: &Connection) -> rusqlite::Result<String> {
    if let Some(t) = crate::db::queries::get_setting(conn, "browser_token")? {
        if !t.is_empty() { return Ok(t); }
    }
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))))?;
    let token: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    crate::db::queries::set_setting(conn, "browser_token", &token)?;
    Ok(token)
}

#[derive(Debug, PartialEq)]
pub enum ActiveOutcome {
    Set(String),  // registrable domain to record as active
    Clear,        // authorized, but no trackable site
    Unauthorized, // token mismatch
    Bad,          // unparseable body
}

#[derive(Deserialize)]
struct ActiveReq { token: String, #[serde(default)] url: Option<String> }

/// Pure: classify an incoming /active request body against the expected token.
pub fn evaluate_active(expected_token: &str, body: &str) -> ActiveOutcome {
    let req: ActiveReq = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(_) => return ActiveOutcome::Bad,
    };
    if req.token != expected_token {
        return ActiveOutcome::Unauthorized;
    }
    match req.url.as_deref().and_then(registrable_domain) {
        Some(domain) => ActiveOutcome::Set(domain),
        None => ActiveOutcome::Clear,
    }
}

const DEFAULT_PORT: u16 = 7878;

// Wildcard CORS is intentional: the extension's origin differs per browser vendor.
// The shared token (checked in evaluate_active) is the actual access guard, not Origin.
fn cors(resp: tiny_http::Response<std::io::Empty>) -> tiny_http::Response<std::io::Empty> {
    use tiny_http::Header;
    resp.with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Headers"[..], &b"Content-Type"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Methods"[..], &b"POST, GET, OPTIONS"[..]).unwrap())
}

/// Read the configured port from settings, falling back to the default.
fn port_of(app: &AppHandle) -> u16 {
    let state = app.state::<AppState>();
    let conn = match state.db.lock() { Ok(c) => c, Err(_) => return DEFAULT_PORT };
    crate::db::queries::get_setting(&conn, "browser_port").ok().flatten()
        .and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_PORT)
}

/// The currently-reported domain, but only if updated within `max_age_secs`.
/// None when the feature is off, no site is active, or the report is stale.
pub fn current_fresh_domain(app: &AppHandle, max_age_secs: i64) -> Option<String> {
    let state = app.state::<AppState>();
    let rt = state.browser.lock().ok()?;
    let a = rt.active.as_ref()?;
    if chrono::Utc::now().timestamp() - a.updated_at <= max_age_secs {
        Some(a.domain.clone())
    } else {
        None
    }
}

/// Start the loopback server on a background thread if not already running.
pub fn start(app: AppHandle) {
    let stop = Arc::new(AtomicBool::new(false));
    {
        let state = app.state::<AppState>();
        let mut rt = match state.browser.lock() { Ok(g) => g, Err(_) => return };
        if rt.stop.is_some() { return; } // already running
        rt.stop = Some(stop.clone());
    }
    let port = port_of(&app);

    std::thread::spawn(move || {
        let server = match tiny_http::Server::http(("127.0.0.1", port)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("browser server bind failed: {e}");
                // Self-heal: clear the stop handle so a later start() can retry
                // instead of being permanently short-circuited by the is_some() guard.
                let state = app.state::<AppState>();
                if let Ok(mut rt) = state.browser.lock() { rt.stop = None; }
                return;
            }
        };
        let token = {
            let state = app.state::<AppState>();
            let conn = match state.db.lock() { Ok(c) => c, Err(_) => return };
            match ensure_token(&conn) {
                Ok(t) => t,
                Err(e) => { eprintln!("browser: failed to load token: {e}"); return; }
            }
        };

        while !stop.load(Ordering::Relaxed) {
            let mut req = match server.recv_timeout(Duration::from_millis(500)) {
                Ok(Some(r)) => r,
                Ok(None) => continue, // timeout → re-check stop flag
                Err(_) => break,
            };
            let method = req.method().clone();
            let url = req.url().to_string();

            if method == tiny_http::Method::Options {
                let _ = req.respond(cors(tiny_http::Response::empty(204)));
                continue;
            }
            if method == tiny_http::Method::Get && url.starts_with("/blocked") {
                let authed = url.split('?').nth(1)
                    .map(|q| q.split('&').any(|kv| kv == format!("token={token}")))
                    .unwrap_or(false);
                if !authed {
                    let _ = req.respond(cors(tiny_http::Response::empty(401)));
                    continue;
                }
                let list: Vec<String> = {
                    let st = app.state::<AppState>();
                    let rt = st.browser.lock();
                    rt.map(|r| r.blocked.iter().cloned().collect()).unwrap_or_default()
                };
                let body = serde_json::to_string(&list).unwrap_or_else(|_| "[]".into());
                let resp = tiny_http::Response::from_string(body)
                    .with_status_code(200)
                    .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
                    .with_header(tiny_http::Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
                let _ = req.respond(resp);
                continue;
            }
            if method == tiny_http::Method::Post && url.starts_with("/active") {
                let mut body = String::new();
                let _ = req.as_reader().read_to_string(&mut body);
                match evaluate_active(&token, &body) {
                    ActiveOutcome::Unauthorized => { let _ = req.respond(cors(tiny_http::Response::empty(401))); }
                    ActiveOutcome::Bad => { let _ = req.respond(cors(tiny_http::Response::empty(400))); }
                    ActiveOutcome::Clear => {
                        if let Ok(mut rt) = app.state::<AppState>().browser.lock() { rt.active = None; }
                        let _ = req.respond(cors(tiny_http::Response::empty(204)));
                    }
                    ActiveOutcome::Set(domain) => {
                        let blocked = {
                            let st = app.state::<AppState>();
                            let mut rt = match st.browser.lock() {
                                Ok(r) => r,
                                Err(_) => { let _ = req.respond(cors(tiny_http::Response::empty(204))); continue; }
                            };
                            rt.active = Some(ActiveSite { domain: domain.clone(), updated_at: chrono::Utc::now().timestamp() });
                            rt.blocked.contains(&domain)
                        };
                        let body = serde_json::to_string(&serde_json::json!({ "blocked": blocked }))
                            .unwrap_or_else(|_| "{\"blocked\":false}".into());
                        let resp = tiny_http::Response::from_string(body)
                            .with_status_code(200)
                            .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap())
                            .with_header(tiny_http::Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
                        let _ = req.respond(resp);
                    }
                }
                continue;
            }
            let _ = req.respond(cors(tiny_http::Response::empty(404)));
        }
    });
}

/// Signal the server thread to stop and clear the active site.
pub fn stop(app: &AppHandle) {
    let state = app.state::<AppState>();
    let mut rt = match state.browser.lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    if let Some(flag) = rt.stop.take() { flag.store(true, Ordering::Relaxed); }
    rt.active = None;
}

#[derive(Serialize)]
pub struct BrowserStatus {
    pub enabled: bool,
    pub port: u16,
    pub token: String,
    pub connected: bool,
    pub last_seen_secs: Option<i64>,
    pub domain: Option<String>,
}

#[tauri::command]
pub fn browser_status(app: AppHandle) -> Result<BrowserStatus, String> {
    let state = app.state::<AppState>();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let enabled = crate::db::queries::get_setting(&conn, "browser_enabled")
        .map_err(|e| e.to_string())?.as_deref() == Some("true");
    let port = crate::db::queries::get_setting(&conn, "browser_port").ok().flatten()
        .and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_PORT);
    let token = ensure_token(&conn).map_err(|e| e.to_string())?;
    drop(conn);
    let (last_seen_secs, domain) = {
        let rt = state.browser.lock().map_err(|e| e.to_string())?;
        match &rt.active {
            Some(a) => (Some(chrono::Utc::now().timestamp() - a.updated_at), Some(a.domain.clone())),
            None => (None, None),
        }
    };
    let connected = last_seen_secs.map(|s| s <= 15).unwrap_or(false);
    Ok(BrowserStatus { enabled, port, token, connected, last_seen_secs, domain })
}

#[tauri::command]
pub fn set_browser_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    {
        let state = app.state::<AppState>();
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        crate::db::queries::set_setting(&conn, "browser_enabled", if enabled { "true" } else { "false" })
            .map_err(|e| e.to_string())?;
    }
    if enabled { start(app); } else { stop(&app); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduces_common_sites() {
        assert_eq!(registrable_domain("https://www.youtube.com/watch?v=abc"), Some("youtube.com".into()));
        assert_eq!(registrable_domain("https://m.youtube.com/"), Some("youtube.com".into()));
        assert_eq!(registrable_domain("https://sub.a.example.com/x"), Some("example.com".into()));
        assert_eq!(registrable_domain("https://example.co.uk/path"), Some("example.co.uk".into()));
        assert_eq!(registrable_domain("HTTPS://WWW.GitHub.COM"), Some("github.com".into()));
    }

    #[test]
    fn rejects_non_trackable() {
        assert_eq!(registrable_domain("http://localhost:5173/"), None);
        assert_eq!(registrable_domain("http://127.0.0.1/"), None);
        assert_eq!(registrable_domain("http://192.168.1.4:8080/x"), None);
        assert_eq!(registrable_domain("chrome://settings"), None);
        assert_eq!(registrable_domain("about:blank"), None);
        assert_eq!(registrable_domain("not a url"), None);
        assert_eq!(registrable_domain(""), None);
        assert_eq!(registrable_domain("http://[::1]/"), None);
        assert_eq!(registrable_domain("http://[2001:db8::1]:8080/x"), None);
    }

    #[test]
    fn token_is_created_once_and_stable() {
        let conn = crate::db::open_in_memory().unwrap();
        let a = ensure_token(&conn).unwrap();
        assert_eq!(a.len(), 32); // 16 random bytes hex-encoded
        let b = ensure_token(&conn).unwrap();
        assert_eq!(a, b); // second call returns the same persisted token
    }

    #[test]
    fn recognizes_browser_exes() {
        assert!(is_browser_exe("chrome.exe"));
        assert!(is_browser_exe("msedge.exe"));
        assert!(is_browser_exe("brave.exe"));
        assert!(is_browser_exe("CHROME.EXE"));
        assert!(!is_browser_exe("code.exe"));
        assert!(!is_browser_exe("explorer.exe"));
    }

    #[test]
    fn evaluate_active_outcomes() {
        let tok = "abc123";
        assert_eq!(
            evaluate_active(tok, r#"{"token":"abc123","url":"https://www.reddit.com/r/x"}"#),
            ActiveOutcome::Set("reddit.com".into())
        );
        assert_eq!(evaluate_active(tok, r#"{"token":"abc123","url":null}"#), ActiveOutcome::Clear);
        assert_eq!(evaluate_active(tok, r#"{"token":"abc123"}"#), ActiveOutcome::Clear);
        assert_eq!(evaluate_active(tok, r#"{"token":"abc123","url":"chrome://newtab"}"#), ActiveOutcome::Clear);
        assert_eq!(evaluate_active(tok, r#"{"token":"nope","url":"https://x.com"}"#), ActiveOutcome::Unauthorized);
        assert_eq!(evaluate_active(tok, "not json"), ActiveOutcome::Bad);
    }
}
