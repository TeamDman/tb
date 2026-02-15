Taskbar toggle utility for programmatic control of "Automatically hide the taskbar in desktop mode".

===

## Current status

`tb` is implemented as a standalone Rust executable for Windows with:

- CLI parsing via `facet` + `figue`
- `--help` and `--version` (version includes git revision from `build.rs`)
- taskbar commands: `toggle`, `status`
- path commands: `home`, `cache`
- configurable hotkey commands: `hotkey show`, `hotkey set <EXPRESSION>`
- default no-args behavior launches tray mode (`run`)

## Tray behavior

When running in tray mode:

- starts without showing the default console
- creates a tray icon
- registers the configured global hotkey (default: `Ctrl+Shift+B`)
- toggles taskbar auto-hide when hotkey is pressed
- shows tray menu with:
  - Toggle taskbar auto-hide
  - Show logs
  - Hide logs
  - About
  - Exit
- About dialog shows version + git revision + active hotkey, with copy-to-clipboard

## Hotkey CLI

- `tb hotkey show` prints the current configured hotkey expression
- `tb hotkey set <EXPRESSION>` parses, normalizes, validates, and saves the hotkey

`tb hotkey` defaults to `show`.

Examples:

```powershell
tb hotkey
tb hotkey show
tb hotkey set ctrl+shift+b
tb hotkey set win+alt+f12
tb hotkey set f9
```

The expression is persisted under the app home directory in `hotkey.txt`.

## Usage

```powershell
tb --help
tb --version
tb status
tb toggle
tb home
tb cache
tb hotkey
tb hotkey show
tb hotkey set ctrl+shift+b
tb run
```

No arguments defaults to tray mode:

```powershell
tb
```

## Notes

- This follows an AutoHotkey/Powertoys-style workflow, implemented in Rust.
- Powertoys may still be preferable for some users depending on desired keyboard hook behavior.
