use windows::Win32::Foundation::{CloseHandle, HWND, MAX_PATH};
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

/// Lower-cased exe name of the current foreground window, e.g. "code.exe".
pub fn foreground_exe() -> Option<String> {
    foreground_app().map(|(exe, _)| exe)
}

/// (lower-cased exe name, full image path) of the current foreground window.
pub fn foreground_app() -> Option<(String, String)> {
    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; MAX_PATH as usize];
        let mut size = buf.len() as u32;
        let result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_FORMAT(0),
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut size,
        );
        let _ = CloseHandle(handle);
        if result.is_err() || size == 0 {
            return None;
        }
        let full = String::from_utf16_lossy(&buf[..size as usize]);
        let exe = full.rsplit(['\\', '/']).next()?.to_lowercase();
        Some((exe, full))
    }
}

/// Seconds since the last keyboard/mouse input.
pub fn idle_seconds() -> u64 {
    unsafe {
        let mut info = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };
        if GetLastInputInfo(&mut info).as_bool() {
            let now = GetTickCount();
            // GetTickCount wraps ~49.7 days; wrapping_sub handles rollover.
            ((now.wrapping_sub(info.dwTime)) / 1000) as u64
        } else {
            0
        }
    }
}
