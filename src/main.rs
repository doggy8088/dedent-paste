use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write as _;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "macos")]
use dedent_paste::text_from_bytes;
use dedent_paste::{
    GeminiError, dedent_text, format_log_line, format_timestamp, is_silent_error, resolve_api_key,
    resolve_gemini_settings, resolve_log_path,
};

mod gemini;

fn main() {
    if let Err(error) = run() {
        eprintln!("dedent-paste: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    match platform::read_clipboard()? {
        Some(text) if !text.is_empty() => {
            let dedented = dedent_text(&text);

            platform::write_clipboard(&dedented)?;
            platform::paste_from_clipboard()?;

            Ok(())
        }
        _ => run_image_to_text(),
    }
}

fn run_image_to_text() -> Result<(), Box<dyn Error>> {
    match image_to_text() {
        Ok(()) => Ok(()),
        Err(error) => {
            let silent = error
                .downcast_ref::<GeminiError>()
                .is_some_and(is_silent_error);

            if silent {
                log_line("info", &error.to_string());
                Ok(())
            } else {
                log_line("error", &error.to_string());
                platform::notify_error(&error.to_string());
                Err(error)
            }
        }
    }
}

fn image_to_text() -> Result<(), Box<dyn Error>> {
    let get_env = |name: &str| std::env::var(name).ok();

    resolve_api_key(get_env)?;

    let image = platform::read_clipboard_image()?.ok_or(GeminiError::NoImage)?;
    let settings = resolve_gemini_settings(get_env, |path| {
        std::fs::read_to_string(path).map_err(|error| error.to_string())
    })?;

    let text = gemini::generate_text_from_image(&settings, &image)?;

    platform::write_clipboard(&text)?;
    platform::paste_from_clipboard()?;

    Ok(())
}

fn log_line(level: &str, message: &str) {
    let Some(path) = resolve_log_path(
        |name| std::env::var(name).ok(),
        platform::default_log_path(),
    ) else {
        return;
    };

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| format_timestamp(elapsed.as_secs()))
        .unwrap_or_default();
    let line = format_log_line(&timestamp, level, message);

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = file.write_all(line.as_bytes());
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use std::error::Error;
    use std::io::Write;
    use std::path::PathBuf;
    use std::process::{Command, Stdio};

    use dedent_paste::{ClipboardImage, applescript_string_literal, parse_osascript_image_data};

    use super::text_from_bytes;

    const UTF8_LOCALE: &str = "en_US.UTF-8";

    pub fn read_clipboard() -> Result<Option<String>, Box<dyn Error>> {
        let output = Command::new("pbpaste")
            .args(["-Prefer", "txt"])
            .env("LANG", UTF8_LOCALE)
            .env("LC_CTYPE", UTF8_LOCALE)
            .output()?;

        if !output.status.success() {
            return Err(format!("pbpaste failed with status {}", output.status).into());
        }

        if output.stdout.is_empty() {
            return Ok(None);
        }

        Ok(Some(text_from_bytes(output.stdout)?))
    }

    pub fn read_clipboard_image() -> Result<Option<ClipboardImage>, Box<dyn Error>> {
        let output = Command::new("osascript")
            .args(["-e", "the clipboard as «class PNGf»"])
            .output()?;

        // osascript fails with a coercion error when the clipboard holds no image.
        if !output.status.success() {
            return Ok(None);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        match parse_osascript_image_data(&stdout) {
            Some(data) => Ok(Some(ClipboardImage {
                mime_type: "image/png",
                data,
            })),
            None => Err("osascript returned unexpected clipboard image data".into()),
        }
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

    pub fn notify_error(message: &str) {
        let script = format!(
            "display notification {} with title \"dedent-paste\"",
            applescript_string_literal(message)
        );

        // Best effort: the error already reaches stderr via main.
        let _ = Command::new("osascript").args(["-e", &script]).output();
    }

    pub fn default_log_path() -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        Some(PathBuf::from(home).join("Library/Logs/dedent-paste.log"))
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use std::error::Error;
    use std::io;
    use std::mem::size_of;
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;

    use clipboard_win::{
        ErrorCode, formats, get_clipboard, get_clipboard_string, set_clipboard_string,
    };
    use dedent_paste::{ClipboardImage, bmp_to_png};
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VK_CONTROL,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        MB_ICONERROR, MB_OK, MB_SETFOREGROUND, MessageBoxW,
    };

    const VK_V: u16 = b'V' as u16;
    const CLIPBOARD_SETTLE_DELAY: Duration = Duration::from_millis(30);

    pub fn read_clipboard() -> Result<Option<String>, Box<dyn Error>> {
        if !clipboard_win::is_format_avail(formats::CF_UNICODETEXT) {
            return Ok(None);
        }

        Ok(Some(get_clipboard_string().map_err(clipboard_error)?))
    }

    pub fn read_clipboard_image() -> Result<Option<ClipboardImage>, Box<dyn Error>> {
        // Browsers and the Snipping Tool publish a lossless registered "PNG" format.
        if let Some(png_format) = clipboard_win::register_format("PNG") {
            if clipboard_win::is_format_avail(png_format.get()) {
                let data: Vec<u8> =
                    get_clipboard(formats::RawData(png_format.get())).map_err(clipboard_error)?;
                return Ok(Some(ClipboardImage {
                    mime_type: "image/png",
                    data,
                }));
            }
        }

        if clipboard_win::is_format_avail(formats::CF_BITMAP) {
            let bmp: Vec<u8> = get_clipboard(formats::Bitmap).map_err(clipboard_error)?;
            return Ok(Some(ClipboardImage {
                mime_type: "image/png",
                data: bmp_to_png(&bmp)?,
            }));
        }

        Ok(None)
    }

    pub fn write_clipboard(text: &str) -> Result<(), Box<dyn Error>> {
        set_clipboard_string(text).map_err(clipboard_error)?;
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

    pub fn notify_error(message: &str) {
        let text: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
        let title: Vec<u16> = "dedent-paste"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            MessageBoxW(
                std::ptr::null_mut(),
                text.as_ptr(),
                title.as_ptr(),
                MB_OK | MB_ICONERROR | MB_SETFOREGROUND,
            );
        }
    }

    pub fn default_log_path() -> Option<PathBuf> {
        let local_app_data = std::env::var_os("LOCALAPPDATA")?;
        Some(
            PathBuf::from(local_app_data)
                .join("dedent-paste")
                .join("dedent-paste.log"),
        )
    }

    fn clipboard_error(error: ErrorCode) -> io::Error {
        io::Error::other(error.to_string())
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
    use std::path::PathBuf;

    use dedent_paste::ClipboardImage;

    pub fn read_clipboard() -> Result<Option<String>, Box<dyn Error>> {
        Err("dedent-paste requires macOS or Windows".into())
    }

    pub fn read_clipboard_image() -> Result<Option<ClipboardImage>, Box<dyn Error>> {
        Err("dedent-paste requires macOS or Windows".into())
    }

    pub fn write_clipboard(_text: &str) -> Result<(), Box<dyn Error>> {
        Err("dedent-paste requires macOS or Windows".into())
    }

    pub fn paste_from_clipboard() -> Result<(), Box<dyn Error>> {
        Err("dedent-paste requires macOS or Windows".into())
    }

    pub fn notify_error(_message: &str) {}

    pub fn default_log_path() -> Option<PathBuf> {
        None
    }
}
