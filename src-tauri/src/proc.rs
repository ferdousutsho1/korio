/// Force-terminate all processes matching an exe name (e.g. "chrome.exe").
/// Uses taskkill with no visible console window. Returns Ok even if nothing matched.
/// Refuses to terminate Korio's own process as a safety guard.
pub fn force_close(exe: &str) -> std::io::Result<()> {
    if is_own_exe(exe) {
        return Ok(()); // never close ourselves
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use std::process::Command;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        Command::new("taskkill")
            .args(["/IM", exe, "/F", "/T"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()?;
    }
    #[cfg(not(windows))]
    {
        let _ = exe;
    }
    Ok(())
}

/// Launch a program by its full path (detached). No-op off Windows.
pub fn launch_path(path: &str) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new(path).spawn()?;
    }
    #[cfg(not(windows))]
    {
        let _ = path;
    }
    Ok(())
}

/// Launch a program by exe name via the shell (resolves App Paths / PATH), no console window.
pub fn launch_by_name(name: &str) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use std::process::Command;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        // `start "" <name>` — the empty "" is the (ignored) window title arg `start` expects.
        Command::new("cmd")
            .args(["/C", "start", "", name])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()?;
    }
    #[cfg(not(windows))]
    {
        let _ = name;
    }
    Ok(())
}

/// True if `exe` matches Korio's own executable file name (case-insensitive).
fn is_own_exe(exe: &str) -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()))
        .map(|own| own == exe.to_lowercase())
        .unwrap_or(false)
}
