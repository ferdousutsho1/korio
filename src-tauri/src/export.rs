use crate::db::queries::ExportRow;

fn csv_field(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

pub fn to_csv(rows: &[ExportRow]) -> String {
    let mut out = String::from("app,exe,kind,started_at,ended_at,active_seconds\n");
    for r in rows {
        out.push_str(&format!(
            "{},{},{},{},{},{}\n",
            csv_field(&r.app), csv_field(&r.exe), csv_field(&r.kind),
            r.started_at, r.ended_at, r.active_seconds
        ));
    }
    out
}

pub fn to_json(rows: &[ExportRow]) -> String {
    serde_json::to_string_pretty(rows).unwrap_or_else(|_| "[]".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn row(app: &str) -> ExportRow {
        ExportRow { app: app.into(), exe: "code.exe".into(), kind: "productive".into(),
            started_at: 1000, ended_at: 1100, active_seconds: 90 }
    }
    #[test]
    fn csv_has_header_and_escapes_commas_quotes() {
        let csv = to_csv(&[row("Acme, \"Inc\"")]);
        assert!(csv.starts_with("app,exe,kind,started_at,ended_at,active_seconds\n"));
        assert!(csv.contains("\"Acme, \"\"Inc\"\"\",code.exe,productive,1000,1100,90"));
    }
    #[test]
    fn csv_plain_field_unquoted() {
        let csv = to_csv(&[row("VS Code")]);
        assert!(csv.contains("VS Code,code.exe,productive,1000,1100,90"));
    }
    #[test]
    fn json_is_array() {
        let j = to_json(&[row("VS Code")]);
        assert!(j.trim_start().starts_with('['));
        assert!(j.contains("\"app\": \"VS Code\""));
    }
    #[test]
    fn empty_csv_is_header_only() {
        assert_eq!(to_csv(&[]), "app,exe,kind,started_at,ended_at,active_seconds\n");
    }
}
