/// Force-terminate all processes matching an exe name (e.g. "chrome.exe").
/// Uses taskkill with no visible console window. Returns Ok even if nothing matched.
pub fn force_close(exe: &str) -> std::io::Result<()> {
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
