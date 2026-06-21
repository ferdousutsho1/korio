use serde::Serialize;
use std::collections::BTreeMap;
use windows::Win32::Foundation::{BOOL, CloseHandle, HWND, LPARAM, MAX_PATH, TRUE};
use windows::Win32::System::Threading::{
    GetCurrentProcessId, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
};

#[derive(Debug, Serialize, Clone)]
pub struct RunningApp {
    pub exe_name: String, // lower-cased, e.g. "code.exe"
    pub title: String,    // a representative window title
    pub path: String,     // full image path, e.g. "C:\\…\\Code.exe"
}

unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    if !IsWindowVisible(hwnd).as_bool() {
        return TRUE;
    }
    if GetWindowTextLengthW(hwnd) == 0 {
        return TRUE; // skip windows with no title (helper/background windows)
    }
    let map = &mut *(lparam.0 as *mut BTreeMap<String, (String, String)>);

    let mut title_buf = [0u16; 512];
    let len = GetWindowTextW(hwnd, &mut title_buf);
    let title = String::from_utf16_lossy(&title_buf[..len as usize]);

    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid == 0 || pid == GetCurrentProcessId() {
        return TRUE; // skip our own windows
    }

    if let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
        let mut buf = [0u16; MAX_PATH as usize];
        let mut size = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_FORMAT(0),
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut size,
        );
        let _ = CloseHandle(handle);
        if ok.is_ok() && size > 0 {
            let full = String::from_utf16_lossy(&buf[..size as usize]);
            if let Some(exe) = full.rsplit(['\\', '/']).next() {
                map.entry(exe.to_lowercase()).or_insert((title, full.clone()));
            }
        }
    }
    TRUE
}

/// List visible, titled top-level apps currently running (deduped by exe).
pub fn running_apps() -> Vec<RunningApp> {
    let mut map: BTreeMap<String, (String, String)> = BTreeMap::new();
    unsafe {
        let _ = EnumWindows(Some(enum_proc), LPARAM(&mut map as *mut _ as isize));
    }
    map.into_iter()
        .map(|(exe_name, (title, path))| RunningApp { exe_name, title, path })
        .collect()
}

/// Full image path of a currently-running process matching `exe_name` (basename), if any.
pub fn path_for_exe(exe_name: &str) -> Option<String> {
    let want = exe_name.to_lowercase();
    running_apps().into_iter().find(|r| r.exe_name == want).map(|r| r.path)
}
