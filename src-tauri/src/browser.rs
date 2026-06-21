//! Wave 3a: loopback receiver for the companion browser extension.
//! Reduces reported URLs to registrable domains; never stores full URLs.

use rusqlite::Connection;
use serde::Deserialize;

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
