use crate::hotkey::{self, HotkeyRegistration};
use crate::taskbar;
use eyre::{Context, ContextCompat, Result, eyre};
use std::ffi::c_void;
use std::sync::OnceLock;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::Console::{
    AllocConsole, CTRL_BREAK_EVENT, CTRL_C_EVENT, CTRL_CLOSE_EVENT, FreeConsole,
    GetConsoleProcessList, SetConsoleCtrlHandler,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey};
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow,
    DispatchMessageW, EnableMenuItem, GWLP_USERDATA, GetCursorPos, GetMessageW, GetWindowLongPtrW,
    HICON, IDI_APPLICATION, IDNO, IDYES, LoadIconW, MB_ICONINFORMATION, MB_ICONQUESTION, MB_OK,
    MB_YESNO, MF_BYCOMMAND, MF_GRAYED, MF_SEPARATOR, MF_STRING, MSG, MessageBoxW, PostMessageW,
    PostQuitMessage, RegisterClassW, RegisterWindowMessageW, SW_SHOW,
    SetForegroundWindow, SetWindowLongPtrW, ShowWindow, TPM_LEFTALIGN, TPM_RETURNCMD,
    TPM_RIGHTBUTTON, TPM_TOPALIGN, TrackPopupMenu, TranslateMessage, WM_CLOSE,
    WM_CONTEXTMENU, WM_CREATE, WM_DESTROY, WM_HOTKEY, WM_LBUTTONDBLCLK, WM_RBUTTONUP, WM_USER,
    WNDCLASSW, WS_OVERLAPPEDWINDOW,
};
use windows::core::{BOOL, HSTRING, PCWSTR, w};

const HOTKEY_ID: i32 = 1;
const TRAY_ICON_ID: u32 = 1;
const WM_TRAY_CALLBACK: u32 = WM_USER + 1;

const CMD_TOGGLE: usize = 0x3000;
const CMD_SHOW_LOGS: usize = 0x3001;
const CMD_HIDE_LOGS: usize = 0x3002;
const CMD_ABOUT: usize = 0x3003;
const CMD_EXIT: usize = 0x3004;

static TRAY_VERSION: OnceLock<&'static str> = OnceLock::new();
static TRAY_HOTKEY: OnceLock<HotkeyRegistration> = OnceLock::new();
static TRAY_HOTKEY_EXPRESSION: OnceLock<String> = OnceLock::new();
static WM_TASKBAR_CREATED: OnceLock<u32> = OnceLock::new();
static TRAY_HWND: OnceLock<isize> = OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConsoleMode {
    Detached,
    Owned,
}

#[derive(Debug)]
struct TrayState {
    version: &'static str,
    hotkey_expression: String,
    console_mode: ConsoleMode,
}

impl TrayState {
    fn new(version: &'static str, hotkey_expression: String) -> Self {
        Self {
            version,
            hotkey_expression,
            console_mode: ConsoleMode::Detached,
        }
    }

    fn can_show_logs(&self) -> bool {
        self.console_mode != ConsoleMode::Owned
    }

    fn can_hide_logs(&self) -> bool {
        self.console_mode == ConsoleMode::Owned
    }

    fn show_logs(&mut self) {
        if !self.can_show_logs() {
            return;
        }

        let _ = unsafe { AllocConsole() };
        let console = unsafe { windows::Win32::System::Console::GetConsoleWindow() };
        if !console.0.is_null() {
            let _ = unsafe { ShowWindow(console, SW_SHOW) };
        }
        self.console_mode = ConsoleMode::Owned;
    }

    fn hide_logs(&mut self) {
        if !self.can_hide_logs() {
            return;
        }
        let _ = unsafe { FreeConsole() };
        self.console_mode = ConsoleMode::Detached;
    }

    fn about_text(&self) -> String {
        format!(
            "tb\nVersion: {}\nHotkey: {}\n\nChoose Yes to copy this text to clipboard.",
            self.version, self.hotkey_expression
        )
    }
}

pub fn run_tray(version: &'static str) -> Result<()> {
    let inherited_console = is_inheriting_console();
    if inherited_console {
        attach_ctrl_c_handler()?;
    } else {
        detach_default_console_if_not_inherited();
    }

    let hotkey = hotkey::load_hotkey()?;
    let _ = TRAY_VERSION.set(version);
    let _ = TRAY_HOTKEY.set(hotkey.registration);
    let _ = TRAY_HOTKEY_EXPRESSION.set(hotkey.expression);
    let taskbar_created = unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) };
    let _ = WM_TASKBAR_CREATED.set(taskbar_created);

    let hwnd = create_window()?;
    let _ = TRAY_HWND.set(hwnd.0 as isize);
    unsafe { register_hotkey(hwnd)? };
    add_tray_icon(hwnd)?;

    run_message_loop()?;
    Ok(())
}

fn detach_default_console_if_not_inherited() {
    let console = unsafe { windows::Win32::System::Console::GetConsoleWindow() };
    if console.0.is_null() {
        return;
    }

    let mut process_ids = [0u32; 8];
    let count = unsafe { GetConsoleProcessList(&mut process_ids) };

    if count == 1 {
        let _ = unsafe { FreeConsole() };
    }
}

fn is_inheriting_console() -> bool {
    let console = unsafe { windows::Win32::System::Console::GetConsoleWindow() };
    if console.0.is_null() {
        return false;
    }

    let mut process_ids = [0u32; 8];
    let count = unsafe { GetConsoleProcessList(&mut process_ids) };
    count > 1
}

fn attach_ctrl_c_handler() -> Result<()> {
    unsafe { SetConsoleCtrlHandler(Some(ctrl_c_handler), true) }
        .map_err(|error| eyre!("Failed to install Ctrl+C console handler: {error}"))
}

unsafe extern "system" fn ctrl_c_handler(ctrl_type: u32) -> BOOL {
    match ctrl_type {
        CTRL_C_EVENT | CTRL_BREAK_EVENT | CTRL_CLOSE_EVENT => {
            if let Some(hwnd_bits) = TRAY_HWND.get().copied() {
                let hwnd = HWND(hwnd_bits as *mut c_void);
                let _ = unsafe { PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0)) };
            }
            BOOL(1)
        }
        _ => BOOL(0),
    }
}

fn create_window() -> Result<HWND> {
    let hinstance = unsafe { GetModuleHandleW(None) }.wrap_err("GetModuleHandleW failed")?;
    let class_name = w!("tb_tray_window");

    let wnd_class = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        hInstance: hinstance.into(),
        lpszClassName: class_name,
        ..Default::default()
    };

    let atom = unsafe { RegisterClassW(&wnd_class) };
    if atom == 0 {
        eyre::bail!("RegisterClassW failed")
    }

    let hwnd = unsafe {
        CreateWindowExW(
            Default::default(),
            class_name,
            w!("tb"),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(hinstance.into()),
            None,
        )
    }
    .wrap_err("CreateWindowExW failed")?;

    Ok(hwnd)
}

fn run_message_loop() -> Result<()> {
    let mut msg = MSG::default();
    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.into() {
        let _ = unsafe { TranslateMessage(&msg) };
        unsafe { DispatchMessageW(&msg) };
    }
    Ok(())
}

unsafe fn register_hotkey(hwnd: HWND) -> Result<()> {
    let registration = TRAY_HOTKEY
        .get()
        .copied()
        .ok_or_else(|| eyre!("Tray hotkey not configured"))?;

    unsafe {
        RegisterHotKey(
            Some(hwnd),
            HOTKEY_ID,
            registration.modifiers,
            registration.vk,
        )
    }
        .ok()
        .wrap_err("Failed to register global hotkey")?;
    Ok(())
}

unsafe fn unregister_hotkey(hwnd: HWND) {
    let _ = unsafe { UnregisterHotKey(Some(hwnd), HOTKEY_ID) };
}

fn add_tray_icon(hwnd: HWND) -> Result<()> {
    let icon = load_tray_icon()?;
    let data = notify_data(hwnd, icon);
    unsafe { Shell_NotifyIconW(NIM_ADD, &data).ok() }.wrap_err("Failed to add tray icon")?;
    Ok(())
}

fn re_add_tray_icon(hwnd: HWND) -> Result<()> {
    let icon = load_tray_icon()?;
    let data = notify_data(hwnd, icon);
    unsafe { Shell_NotifyIconW(NIM_ADD, &data).ok() }.wrap_err("Failed to re-add tray icon")?;
    Ok(())
}

fn load_tray_icon() -> Result<HICON> {
    let module = unsafe { GetModuleHandleW(None) }.wrap_err("GetModuleHandleW failed")?;

    match unsafe { LoadIconW(Some(module.into()), w!("main_icon")) } {
        Ok(icon) => Ok(icon),
        Err(error) => {
            tracing::warn!("Failed to load embedded tray icon 'main_icon': {error}");
            unsafe { LoadIconW(None, IDI_APPLICATION) }
                .wrap_err("Failed to load fallback tray icon")
        }
    }
}

fn delete_tray_icon(hwnd: HWND) -> Result<()> {
    let data = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ICON_ID,
        ..Default::default()
    };
    unsafe { Shell_NotifyIconW(NIM_DELETE, &data).ok() }.wrap_err("Failed to delete tray icon")?;
    Ok(())
}

fn notify_data(hwnd: HWND, icon: HICON) -> NOTIFYICONDATAW {
    let mut data = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ICON_ID,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
        uCallbackMessage: WM_TRAY_CALLBACK,
        hIcon: icon,
        ..Default::default()
    };

    let tip: Vec<u16> = "tb".encode_utf16().chain(Some(0)).collect();
    let tip_len = tip.len().min(data.szTip.len());
    data.szTip[..tip_len].copy_from_slice(&tip[..tip_len]);

    data
}

fn show_context_menu(hwnd: HWND) {
    with_state(hwnd, |state| {
        let _ = unsafe { SetForegroundWindow(hwnd) }.ok();

        let menu = match unsafe { CreatePopupMenu() } {
            Ok(menu) => menu,
            Err(error) => {
                tracing::error!("Failed to create tray menu: {error}");
                return;
            }
        };

        unsafe { AppendMenuW(menu, MF_STRING, CMD_TOGGLE, w!("Toggle taskbar auto-hide")) }.ok();
        unsafe { AppendMenuW(menu, MF_STRING, CMD_SHOW_LOGS, w!("Show logs")) }.ok();
        unsafe { AppendMenuW(menu, MF_STRING, CMD_HIDE_LOGS, w!("Hide logs")) }.ok();
        unsafe { AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null()) }.ok();
        unsafe { AppendMenuW(menu, MF_STRING, CMD_ABOUT, w!("About")) }.ok();
        unsafe { AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null()) }.ok();
        unsafe { AppendMenuW(menu, MF_STRING, CMD_EXIT, w!("Exit")) }.ok();

        if !state.can_show_logs() {
            let _ = unsafe {
                EnableMenuItem(
                    menu,
                    CMD_SHOW_LOGS.try_into().expect("menu id fits u32"),
                    MF_BYCOMMAND | MF_GRAYED,
                )
            };
        }
        if !state.can_hide_logs() {
            let _ = unsafe {
                EnableMenuItem(
                    menu,
                    CMD_HIDE_LOGS.try_into().expect("menu id fits u32"),
                    MF_BYCOMMAND | MF_GRAYED,
                )
            };
        }

        let mut cursor = POINT::default();
        unsafe { GetCursorPos(&raw mut cursor) }.ok();

        let selection = unsafe {
            TrackPopupMenu(
                menu,
                TPM_RIGHTBUTTON | TPM_TOPALIGN | TPM_LEFTALIGN | TPM_RETURNCMD,
                cursor.x,
                cursor.y,
                None,
                hwnd,
                None,
            )
        }
        .0;

        unsafe { DestroyMenu(menu) }.ok();

        match usize::try_from(selection).unwrap_or_default() {
            CMD_TOGGLE => handle_toggle(),
            CMD_SHOW_LOGS => state.show_logs(),
            CMD_HIDE_LOGS => state.hide_logs(),
            CMD_ABOUT => show_about_dialog(hwnd, state),
            CMD_EXIT => {
                unsafe { PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0)) }.ok();
            }
            _ => {}
        }
    });
}

fn show_about_dialog(hwnd: HWND, state: &TrayState) {
    let text = state.about_text();
    let response = unsafe {
        MessageBoxW(
            Some(hwnd),
            &HSTRING::from(text.clone()),
            w!("About tb"),
            MB_YESNO | MB_ICONQUESTION,
        )
    };

    if response == IDYES {
        if let Err(error) = copy_to_clipboard(text) {
            tracing::error!("Failed to copy text to clipboard: {error}");
        } else {
            unsafe {
                MessageBoxW(
                    Some(hwnd),
                    w!("Copied version info to clipboard."),
                    w!("About tb"),
                    MB_OK | MB_ICONINFORMATION,
                )
            };
        }
    } else if response != IDNO {
        tracing::debug!("Unexpected About dialog result: {}", response.0);
    }
}

fn copy_to_clipboard(text: String) -> Result<()> {
    let mut clipboard = arboard::Clipboard::new().wrap_err("Failed to open clipboard")?;
    clipboard
        .set_text(text)
        .wrap_err("Failed to write clipboard text")
}

fn store_state(hwnd: HWND, state: Box<TrayState>) {
    unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize) };
}

fn with_state(hwnd: HWND, action: impl FnOnce(&mut TrayState)) {
    let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) };
    if ptr == 0 {
        return;
    }

    let state = unsafe { &mut *(ptr as *mut TrayState) };
    action(state);
}

fn drop_state(hwnd: HWND) {
    let ptr = unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) };
    if ptr != 0 {
        unsafe { drop(Box::from_raw(ptr as *mut TrayState)) };
    }
}

fn handle_toggle() {
    match taskbar::toggle_taskbar_auto_hide() {
        Ok(enabled) => {
            tracing::info!(
                "Taskbar auto-hide {}",
                if enabled { "enabled" } else { "disabled" }
            );
        }
        Err(error) => tracing::error!("Failed to toggle taskbar: {error}"),
    }
}

pub unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_CREATE => {
            let version = TRAY_VERSION
                .get()
                .copied()
                .ok_or_else(|| eyre!("Tray version not set"));

            match version {
                Ok(version) => {
                    let hotkey_expression = TRAY_HOTKEY_EXPRESSION
                        .get()
                        .cloned()
                        .unwrap_or_else(|| "Ctrl+Shift+B".to_string());
                    store_state(hwnd, Box::new(TrayState::new(version, hotkey_expression)));
                    LRESULT(0)
                }
                Err(error) => {
                    tracing::error!("Failed to initialize tray state: {error}");
                    LRESULT(-1)
                }
            }
        }
        WM_HOTKEY => {
            if i32::try_from(wparam.0).ok() == Some(HOTKEY_ID) {
                handle_toggle();
            }
            LRESULT(0)
        }
        WM_TRAY_CALLBACK => {
            match lparam.0 as u32 {
                WM_RBUTTONUP | WM_CONTEXTMENU => show_context_menu(hwnd),
                WM_LBUTTONDBLCLK => handle_toggle(),
                _ => {}
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            unsafe { DestroyWindow(hwnd) }.ok();
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe { unregister_hotkey(hwnd) };
            if let Err(error) = delete_tray_icon(hwnd) {
                tracing::error!("Failed to delete tray icon: {error}");
            }
            drop_state(hwnd);
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ => {
            if WM_TASKBAR_CREATED.get().copied() == Some(message) {
                if let Err(error) = re_add_tray_icon(hwnd) {
                    tracing::error!("Failed to restore tray icon: {error}");
                }
                LRESULT(0)
            } else {
                unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
            }
        }
    }
}
