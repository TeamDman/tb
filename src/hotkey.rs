use crate::paths;
use eyre::{Context, Result, bail};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN, VK_BACK, VK_DELETE, VK_DOWN,
    VK_END, VK_ESCAPE, VK_F1, VK_F2, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_F10,
    VK_F11, VK_F12, VK_F13, VK_F14, VK_F15, VK_F16, VK_F17, VK_F18, VK_F19, VK_F20, VK_F21,
    VK_F22, VK_F23, VK_F24, VK_HOME, VK_INSERT, VK_LEFT, VK_NEXT, VK_PRIOR, VK_RETURN,
    VK_RIGHT, VK_SPACE, VK_TAB, VK_UP,
};

const HOTKEY_CONFIG_FILE: &str = "hotkey.txt";
const DEFAULT_HOTKEY_EXPRESSION: &str = "Ctrl+Shift+B";

#[derive(Debug, Clone, Copy)]
pub struct HotkeyRegistration {
    pub modifiers: HOT_KEY_MODIFIERS,
    pub vk: u32,
}

#[derive(Debug, Clone)]
pub struct Hotkey {
    pub expression: String,
    pub registration: HotkeyRegistration,
}

pub fn parse_hotkey_expression(expression: &str) -> Result<Hotkey> {
    let mut has_ctrl = false;
    let mut has_shift = false;
    let mut has_alt = false;
    let mut has_win = false;
    let mut key: Option<(u32, String)> = None;

    let tokens = expression
        .split(['+', ' ', '\t'])
        .filter(|token| !token.trim().is_empty())
        .collect::<Vec<_>>();

    if tokens.is_empty() {
        bail!("Hotkey expression cannot be empty")
    }

    for token in tokens {
        let normalized = token.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "ctrl" | "control" => has_ctrl = true,
            "shift" => has_shift = true,
            "alt" => has_alt = true,
            "win" | "windows" | "meta" => has_win = true,
            _ => {
                if key.is_some() {
                    bail!("Hotkey expression must contain exactly one non-modifier key")
                }
                key = Some(parse_key_token(&normalized)?);
            }
        }
    }

    let (vk, key_label) = key.ok_or_else(|| eyre::eyre!("Hotkey key is missing"))?;
    let modifiers = build_modifiers(has_ctrl, has_shift, has_alt, has_win);

    let mut parts = Vec::new();
    if has_ctrl {
        parts.push("Ctrl".to_string());
    }
    if has_shift {
        parts.push("Shift".to_string());
    }
    if has_alt {
        parts.push("Alt".to_string());
    }
    if has_win {
        parts.push("Win".to_string());
    }
    parts.push(key_label);

    Ok(Hotkey {
        expression: parts.join("+"),
        registration: HotkeyRegistration { modifiers, vk },
    })
}

pub fn load_hotkey() -> Result<Hotkey> {
    let path = hotkey_path()?;
    if !path.exists() {
        return parse_hotkey_expression(DEFAULT_HOTKEY_EXPRESSION);
    }

    let raw = std::fs::read_to_string(&path).wrap_err_with(|| {
        format!(
            "Failed to read hotkey configuration at {}",
            path.display()
        )
    })?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return parse_hotkey_expression(DEFAULT_HOTKEY_EXPRESSION);
    }

    parse_hotkey_expression(trimmed).wrap_err_with(|| {
        format!(
            "Invalid hotkey expression in {}: {trimmed}",
            path.display()
        )
    })
}

pub fn save_hotkey_expression(expression: &str) -> Result<Hotkey> {
    let hotkey = parse_hotkey_expression(expression)?;
    let path = hotkey_path()?;
    std::fs::write(&path, format!("{}\n", hotkey.expression)).wrap_err_with(|| {
        format!(
            "Failed to write hotkey configuration at {}",
            path.display()
        )
    })?;
    Ok(hotkey)
}

fn hotkey_path() -> Result<std::path::PathBuf> {
    let home = paths::app_home()?;
    home.ensure_dir()?;
    Ok(home.path().join(HOTKEY_CONFIG_FILE))
}

fn build_modifiers(ctrl: bool, shift: bool, alt: bool, win: bool) -> HOT_KEY_MODIFIERS {
    let mut modifiers = HOT_KEY_MODIFIERS(0);
    if ctrl {
        modifiers |= MOD_CONTROL;
    }
    if shift {
        modifiers |= MOD_SHIFT;
    }
    if alt {
        modifiers |= MOD_ALT;
    }
    if win {
        modifiers |= MOD_WIN;
    }
    modifiers
}

fn parse_key_token(token: &str) -> Result<(u32, String)> {
    if token.len() == 1 {
        let character = token.chars().next().expect("single-char token has one char");
        if character.is_ascii_alphanumeric() {
            let upper = character.to_ascii_uppercase();
            return Ok((u32::from(upper), upper.to_string()));
        }
    }

    if let Some(index) = token.strip_prefix('f').and_then(|value| value.parse::<u8>().ok())
        && (1..=24).contains(&index)
    {
        let virtual_key = match index {
            1 => u32::from(VK_F1.0),
            2 => u32::from(VK_F2.0),
            3 => u32::from(VK_F3.0),
            4 => u32::from(VK_F4.0),
            5 => u32::from(VK_F5.0),
            6 => u32::from(VK_F6.0),
            7 => u32::from(VK_F7.0),
            8 => u32::from(VK_F8.0),
            9 => u32::from(VK_F9.0),
            10 => u32::from(VK_F10.0),
            11 => u32::from(VK_F11.0),
            12 => u32::from(VK_F12.0),
            13 => u32::from(VK_F13.0),
            14 => u32::from(VK_F14.0),
            15 => u32::from(VK_F15.0),
            16 => u32::from(VK_F16.0),
            17 => u32::from(VK_F17.0),
            18 => u32::from(VK_F18.0),
            19 => u32::from(VK_F19.0),
            20 => u32::from(VK_F20.0),
            21 => u32::from(VK_F21.0),
            22 => u32::from(VK_F22.0),
            23 => u32::from(VK_F23.0),
            24 => u32::from(VK_F24.0),
            _ => unreachable!(),
        };
        return Ok((virtual_key, format!("F{index}")));
    }

    let (key, label) = match token {
        "space" => (u32::from(VK_SPACE.0), "Space"),
        "tab" => (u32::from(VK_TAB.0), "Tab"),
        "enter" | "return" => (u32::from(VK_RETURN.0), "Enter"),
        "esc" | "escape" => (u32::from(VK_ESCAPE.0), "Escape"),
        "backspace" | "bksp" => (u32::from(VK_BACK.0), "Backspace"),
        "insert" | "ins" => (u32::from(VK_INSERT.0), "Insert"),
        "delete" | "del" => (u32::from(VK_DELETE.0), "Delete"),
        "home" => (u32::from(VK_HOME.0), "Home"),
        "end" => (u32::from(VK_END.0), "End"),
        "pageup" | "pgup" => (u32::from(VK_PRIOR.0), "PageUp"),
        "pagedown" | "pgdn" => (u32::from(VK_NEXT.0), "PageDown"),
        "up" | "arrowup" => (u32::from(VK_UP.0), "Up"),
        "down" | "arrowdown" => (u32::from(VK_DOWN.0), "Down"),
        "left" | "arrowleft" => (u32::from(VK_LEFT.0), "Left"),
        "right" | "arrowright" => (u32::from(VK_RIGHT.0), "Right"),
        _ => bail!("Unsupported hotkey key token: {token}"),
    };

    Ok((key, label.to_string()))
}