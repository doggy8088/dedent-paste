use std::error::Error;

use dedent_paste::dedent_text;
#[cfg(target_os = "macos")]
use dedent_paste::text_from_bytes;

fn main() {
    if let Err(error) = run() {
        eprintln!("dedent-paste: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let clipboard = platform::read_clipboard()?;
    let dedented = dedent_text(&clipboard);

    platform::write_clipboard(&dedented)?;
    platform::paste_from_clipboard()?;

    Ok(())
}

#[cfg(target_os = "macos")]
mod platform {
    use std::error::Error;
    use std::io::Write;
    use std::process::{Command, Stdio};

    use super::text_from_bytes;

    const UTF8_LOCALE: &str = "en_US.UTF-8";

    pub fn read_clipboard() -> Result<String, Box<dyn Error>> {
        let output = Command::new("pbpaste")
            .args(["-Prefer", "txt"])
            .env("LANG", UTF8_LOCALE)
            .env("LC_CTYPE", UTF8_LOCALE)
            .output()?;

        if !output.status.success() {
            return Err(format!("pbpaste failed with status {}", output.status).into());
        }

        Ok(text_from_bytes(output.stdout)?)
    }

    pub fn write_clipboard(text: &str) -> Result<(), Box<dyn Error>> {
        let mut child = Command::new("pbcopy")
            .env("LANG", UTF8_LOCALE)
            .env("LC_CTYPE", UTF8_LOCALE)
            .stdin(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().ok_or("failed to open pbcopy stdin")?;
        stdin.write_all(text.as_bytes())?;
        drop(stdin);

        let status = child.wait()?;
        if !status.success() {
            return Err(format!("pbcopy failed with status {status}").into());
        }

        Ok(())
    }

    pub fn paste_from_clipboard() -> Result<(), Box<dyn Error>> {
        let status = Command::new("osascript")
            .args([
                "-e",
                r#"tell application "System Events" to keystroke "v" using command down"#,
            ])
            .status()?;

        if !status.success() {
            return Err(format!("osascript paste failed with status {status}").into());
        }

        Ok(())
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use std::error::Error;
    use std::io;
    use std::mem::size_of;
    use std::thread;
    use std::time::Duration;

    use clipboard_win::{get_clipboard_string, set_clipboard_string};
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VK_CONTROL,
    };

    const VK_V: u16 = b'V' as u16;
    const CLIPBOARD_SETTLE_DELAY: Duration = Duration::from_millis(30);

    pub fn read_clipboard() -> Result<String, Box<dyn Error>> {
        Ok(get_clipboard_string()?)
    }

    pub fn write_clipboard(text: &str) -> Result<(), Box<dyn Error>> {
        set_clipboard_string(text)?;
        thread::sleep(CLIPBOARD_SETTLE_DELAY);
        Ok(())
    }

    pub fn paste_from_clipboard() -> Result<(), Box<dyn Error>> {
        let inputs = [
            keyboard_input(VK_CONTROL, 0),
            keyboard_input(VK_V, 0),
            keyboard_input(VK_V, KEYEVENTF_KEYUP),
            keyboard_input(VK_CONTROL, KEYEVENTF_KEYUP),
        ];

        let sent = unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_ptr(),
                size_of::<INPUT>() as i32,
            )
        };

        if sent != inputs.len() as u32 {
            return Err(io::Error::last_os_error().into());
        }

        Ok(())
    }

    fn keyboard_input(vk: u16, flags: u32) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform {
    use std::error::Error;

    pub fn read_clipboard() -> Result<String, Box<dyn Error>> {
        Err("dedent-paste requires macOS or Windows".into())
    }

    pub fn write_clipboard(_text: &str) -> Result<(), Box<dyn Error>> {
        Err("dedent-paste requires macOS or Windows".into())
    }

    pub fn paste_from_clipboard() -> Result<(), Box<dyn Error>> {
        Err("dedent-paste requires macOS or Windows".into())
    }
}
