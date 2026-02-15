use eyre::Context;
use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::UI::Shell::{
    ABM_GETSTATE, ABM_SETSTATE, ABS_ALWAYSONTOP, ABS_AUTOHIDE, APPBARDATA, SHAppBarMessage,
};
use windows::Win32::UI::WindowsAndMessaging::FindWindowW;
use windows::core::w;

pub fn is_taskbar_auto_hide_enabled() -> eyre::Result<bool> {
    let hwnd = find_taskbar_window()?;
    let mut data = APPBARDATA {
        cbSize: std::mem::size_of::<APPBARDATA>() as u32,
        hWnd: hwnd,
        ..Default::default()
    };

    let state = unsafe { SHAppBarMessage(ABM_GETSTATE, &mut data) };
    Ok((state & ABS_AUTOHIDE as usize) != 0)
}

pub fn toggle_taskbar_auto_hide() -> eyre::Result<bool> {
    let hwnd = find_taskbar_window()?;
    let mut data = APPBARDATA {
        cbSize: std::mem::size_of::<APPBARDATA>() as u32,
        hWnd: hwnd,
        ..Default::default()
    };

    let current_state = unsafe { SHAppBarMessage(ABM_GETSTATE, &mut data) };
    let currently_enabled = (current_state & ABS_AUTOHIDE as usize) != 0;

    let mut next_state = if currently_enabled {
        current_state & !(ABS_AUTOHIDE as usize)
    } else {
        current_state | ABS_AUTOHIDE as usize
    };

    if (next_state & ABS_ALWAYSONTOP as usize) == 0 {
        next_state |= ABS_ALWAYSONTOP as usize;
    }

    data.lParam = LPARAM(next_state as isize);
    let result = unsafe { SHAppBarMessage(ABM_SETSTATE, &mut data) };
    if result == 0 {
        eyre::bail!("Failed to set taskbar state")
    }

    Ok(!currently_enabled)
}

fn find_taskbar_window() -> eyre::Result<HWND> {
    let hwnd = unsafe { FindWindowW(w!("Shell_TrayWnd"), None) }
        .wrap_err("Failed to locate Shell_TrayWnd")?;
    if hwnd.0.is_null() {
        eyre::bail!("Taskbar window handle was null")
    }
    Ok(hwnd)
}
