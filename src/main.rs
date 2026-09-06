use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write as _;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(target_os = "macos")]
use dedent_paste::text_from_bytes;
use dedent_paste::update::{self, UpdateOptions};
use dedent_paste::{
    GeminiError, PasteCliOverrides, PasteSettings, dedent_text, format_log_line, format_timestamp,
    is_silent_error, resolve_api_key, resolve_gemini_settings, resolve_log_path,
    resolve_paste_settings,
};

mod gemini;
mod setup;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const HELP: &str = "\
dedent-paste {version}
Paste clipboard text with common indentation removed.

Usage: dedent-paste [PASTE OPTIONS]
       dedent-paste (-i | -u | -v | -h)
       dedent-paste update [UPDATE OPTIONS]

With no options, dedent-paste reads the clipboard as plain text, removes the
common indentation, writes the result back, and pastes it. This is what the
Left Option+V (macOS, Karabiner-Elements) or Win+V (Windows, AutoHotkey) hotkey
runs. On macOS the paste keystroke is sent only after all modifier keys have
been released (waits up to 1 s), and a second instance started while one is
still running exits immediately so key-repeat cannot paste twice.

Subcommands:
  update                     Update dedent-paste to the latest release. Run
                             'dedent-paste update --help' for options.

Paste options (may be combined):
  -n, --no-paste             Rewrite the clipboard only; do not send Cmd+V /
                             Ctrl+V. Use this when your hotkey manager (skhd,
                             Hammerspoon, ...) sends the paste keystroke itself.
      --paste-delay-ms <MS>  Extra delay before the paste keystroke, after the
                             modifier keys are released. Default 0.
  Environment equivalents: DEDENT_PASTE_NO_PASTE=1, DEDENT_PASTE_PASTE_DELAY_MS=<MS>.
  Command-line flags take precedence.

Other options (exclusive):
  -i, --install      Register the Left Option+V rule in Karabiner-Elements (macOS).
                     The rule points at this executable. Existing dedent-paste
                     rules are replaced and karabiner.json is backed up first.
  -u, --uninstall    Remove every dedent-paste rule from Karabiner-Elements (macOS)
                     and delete the imported complex-modification asset.
  -v, --version      Print the version and exit.
  -h, --help         Print this help and exit.

All environment variables (paste behavior, Gemini image-to-text, logging) are
documented at: https://github.com/doggy8088/dedent-paste#readme
";

const UPDATE_HELP: &str = "\
dedent-paste update {version}
Update dedent-paste to the latest release.

Usage: dedent-paste update [OPTIONS]

Options:
  -c, --check    Check for updates without installing
  -f, --force    Force reinstall even if already up to date
  -h, --help     Print this help and exit
";

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Paste(PasteCliOverrides),
    Help,
    Version,
    Install,
    Uninstall,
    Update(UpdateOptions),
    UpdateHelp,
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Command, String> {
    let args: Vec<String> = args.collect();

    if let Some(first) = args.first() {
        if first == "update" {
            return parse_update_args(&args[1..]);
        }
    }

    // Exclusive options must appear alone.
    if let [flag] = args.as_slice() {
        match flag.as_str() {
            "-h" | "--help" => return Ok(Command::Help),
            "-v" | "--version" => return Ok(Command::Version),
            "-i" | "--install" => return Ok(Command::Install),
            "-u" | "--uninstall" => return Ok(Command::Uninstall),
            "--update" => return Ok(Command::Update(UpdateOptions::default())),
            _ => {}
        }
    }

    let mut overrides = PasteCliOverrides::default();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-n" | "--no-paste" => overrides.no_paste = true,
            "--paste-delay-ms" => {
                let value = iter
                    .next()
                    .ok_or("--paste-delay-ms requires a value in milliseconds")?;
                overrides.delay_ms = Some(parse_delay_ms(value)?);
            }
            other if other.starts_with("--paste-delay-ms=") => {
                overrides.delay_ms = Some(parse_delay_ms(&other["--paste-delay-ms=".len()..])?);
            }
            "-h" | "--help" | "-v" | "--version" | "-i" | "--install" | "-u" | "--uninstall"
            | "--update" => {
                return Err(format!("'{arg}' cannot be combined with other options"));
            }
            "update" => {
                return Err(
                    "'update' is a subcommand and cannot be combined with other options"
                        .to_string(),
                );
            }
            other => return Err(format!("unknown option '{other}'")),
        }
    }

    Ok(Command::Paste(overrides))
}

fn parse_update_args(args: &[String]) -> Result<Command, String> {
    let mut options = UpdateOptions::default();
    for arg in args {
        match arg.as_str() {
            "-c" | "--check" => options.check = true,
            "-f" | "--force" => options.force = true,
            "-h" | "--help" => return Ok(Command::UpdateHelp),
            other => return Err(format!("unknown option '{other}' for 'update'")),
        }
    }
    Ok(Command::Update(options))
}

fn parse_delay_ms(value: &str) -> Result<u64, String> {
    value
        .trim()
        .parse::<u64>()
        .map_err(|_| format!("--paste-delay-ms must be a non-negative integer, got '{value}'"))
}

fn main() {
    let command = match parse_args(std::env::args().skip(1)) {
        Ok(command) => command,
        Err(message) => {
            eprintln!("dedent-paste: {message}");
            eprintln!("Run 'dedent-paste --help' for usage.");
            std::process::exit(2);
        }
    };

    let result = match command {
        Command::Paste(overrides) => run(overrides),
        Command::Help => {
            print!("{}", HELP.replace("{version}", VERSION));
            Ok(())
        }
        Command::Version => {
            println!("dedent-paste {VERSION}");
            Ok(())
        }
        Command::Install => setup::install(),
        Command::Uninstall => setup::uninstall(),
        Command::Update(options) => update::run_update(&options),
        Command::UpdateHelp => {
            print!("{}", UPDATE_HELP.replace("{version}", VERSION));
            Ok(())
        }
    };

    if let Err(error) = result {
        eprintln!("dedent-paste: {error}");
        std::process::exit(1);
    }
}

/// Longest time to wait for the user to release the hotkey's modifier keys
/// before the paste keystroke is sent.
const MODIFIER_RELEASE_TIMEOUT: Duration = Duration::from_secs(1);

fn run(overrides: PasteCliOverrides) -> Result<(), Box<dyn Error>> {
    let settings = resolve_paste_settings(|name| std::env::var(name).ok(), overrides)?;

    // Hotkey managers with key-repeat (skhd, ...) can start many copies per
    // keypress. Only the first one may touch the clipboard and paste.
    let _instance_lock = match platform::try_lock_single_instance()? {
        Some(lock) => lock,
        None => {
            log_line("info", "another dedent-paste instance is running; skipping");
            return Ok(());
        }
    };

    match platform::read_clipboard()? {
        Some(text) if !text.is_empty() => {
            let dedented = dedent_text(&text);

            platform::write_clipboard(&dedented)?;
            finish_paste(&settings)
        }
        _ => run_image_to_text(&settings),
    }
}

/// Final step shared by the text and image paths: honor `--no-paste`, give the
/// user time to release the hotkey chord, then send the paste keystroke.
fn finish_paste(settings: &PasteSettings) -> Result<(), Box<dyn Error>> {
    if settings.no_paste {
        log_line("info", "clipboard updated; paste skipped (--no-paste)");
        return Ok(());
    }

    platform::wait_for_modifiers_released(MODIFIER_RELEASE_TIMEOUT);
    if !settings.delay.is_zero() {
        std::thread::sleep(settings.delay);
    }

    platform::paste_from_clipboard()
}

fn run_image_to_text(settings: &PasteSettings) -> Result<(), Box<dyn Error>> {
    match image_to_text(settings) {
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

fn image_to_text(settings: &PasteSettings) -> Result<(), Box<dyn Error>> {
    let get_env = |name: &str| std::env::var(name).ok();

    resolve_api_key(get_env)?;

    let image = platform::read_clipboard_image()?.ok_or(GeminiError::NoImage)?;
    let gemini_settings = resolve_gemini_settings(get_env, |path| {
        std::fs::read_to_string(path).map_err(|error| error.to_string())
    })?;

    let text = gemini::generate_text_from_image(&gemini_settings, &image)?;

    platform::write_clipboard(&text)?;
    finish_paste(settings)
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
    use std::fs::File;
    use std::io::Write;
    use std::os::fd::AsRawFd;
    use std::path::PathBuf;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

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

    // CoreGraphics is a system framework; declaring the two symbols we need
    // avoids pulling in a binding crate. `CGEventSourceFlagsState` only reads
    // the current modifier state and needs no Accessibility permission.
    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGEventSourceFlagsState(state_id: i32) -> u64;
    }

    unsafe extern "C" {
        fn flock(fd: i32, operation: i32) -> i32;
    }

    const CG_EVENT_SOURCE_STATE_COMBINED_SESSION: i32 = 0;
    const CG_EVENT_FLAG_MASK_SHIFT: u64 = 0x0002_0000;
    const CG_EVENT_FLAG_MASK_CONTROL: u64 = 0x0004_0000;
    const CG_EVENT_FLAG_MASK_ALTERNATE: u64 = 0x0008_0000;
    const CG_EVENT_FLAG_MASK_COMMAND: u64 = 0x0010_0000;
    const MODIFIER_MASK: u64 = CG_EVENT_FLAG_MASK_SHIFT
        | CG_EVENT_FLAG_MASK_CONTROL
        | CG_EVENT_FLAG_MASK_ALTERNATE
        | CG_EVENT_FLAG_MASK_COMMAND;
    const MODIFIER_POLL_INTERVAL: Duration = Duration::from_millis(10);

    const LOCK_EX: i32 = 2;
    const LOCK_NB: i32 = 4;
    const EWOULDBLOCK: i32 = 35;

    /// Block until Shift, Control, Option and Command are all released, or
    /// `timeout` elapses. Sending Cmd+V while the physical Option from the
    /// hotkey is still down makes many apps see Cmd+Option+V instead.
    pub fn wait_for_modifiers_released(timeout: Duration) {
        let deadline = Instant::now() + timeout;
        loop {
            let flags = unsafe { CGEventSourceFlagsState(CG_EVENT_SOURCE_STATE_COMBINED_SESSION) };
            if flags & MODIFIER_MASK == 0 || Instant::now() >= deadline {
                return;
            }
            std::thread::sleep(MODIFIER_POLL_INTERVAL);
        }
    }

    /// Holds the single-instance lock for the lifetime of the process.
    pub struct InstanceLock(#[allow(dead_code)] File);

    /// Take a non-blocking exclusive lock on a per-user lock file. Returns
    /// `Ok(None)` when another instance already holds it.
    pub fn try_lock_single_instance() -> Result<Option<InstanceLock>, Box<dyn Error>> {
        let path = std::env::temp_dir().join("dedent-paste.lock");
        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(|error| format!("failed to open lock file {}: {error}", path.display()))?;

        let result = unsafe { flock(file.as_raw_fd(), LOCK_EX | LOCK_NB) };
        if result == 0 {
            return Ok(Some(InstanceLock(file)));
        }

        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(EWOULDBLOCK) {
            return Ok(None);
        }
        Err(format!("failed to lock {}: {error}", path.display()).into())
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

    /// AutoHotkey's `KeyWait` in the documented scripts already waits for the
    /// Win keys to be released, so nothing to do here.
    pub fn wait_for_modifiers_released(_timeout: Duration) {}

    pub struct InstanceLock;

    /// AutoHotkey runs one hotkey thread at a time and the example scripts use
    /// `RunWait`, so duplicate launches are not a problem on Windows.
    pub fn try_lock_single_instance() -> Result<Option<InstanceLock>, Box<dyn Error>> {
        Ok(Some(InstanceLock))
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
    use std::time::Duration;

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

    pub fn wait_for_modifiers_released(_timeout: Duration) {}

    pub struct InstanceLock;

    pub fn try_lock_single_instance() -> Result<Option<InstanceLock>, Box<dyn Error>> {
        Ok(Some(InstanceLock))
    }

    pub fn default_log_path() -> Option<PathBuf> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Command, String> {
        parse_args(args.iter().map(|arg| arg.to_string()))
    }

    #[test]
    fn no_args_means_default_paste() {
        assert_eq!(parse(&[]), Ok(Command::Paste(PasteCliOverrides::default())));
    }

    #[test]
    fn exclusive_flags_parse_alone() {
        assert_eq!(parse(&["-h"]), Ok(Command::Help));
        assert_eq!(parse(&["--help"]), Ok(Command::Help));
        assert_eq!(parse(&["-v"]), Ok(Command::Version));
        assert_eq!(parse(&["--version"]), Ok(Command::Version));
        assert_eq!(parse(&["-i"]), Ok(Command::Install));
        assert_eq!(parse(&["--install"]), Ok(Command::Install));
        assert_eq!(parse(&["-u"]), Ok(Command::Uninstall));
        assert_eq!(parse(&["--uninstall"]), Ok(Command::Uninstall));
    }

    #[test]
    fn paste_options_combine() {
        assert_eq!(
            parse(&["--no-paste"]),
            Ok(Command::Paste(PasteCliOverrides {
                no_paste: true,
                delay_ms: None,
            }))
        );
        assert_eq!(
            parse(&["-n", "--paste-delay-ms", "250"]),
            Ok(Command::Paste(PasteCliOverrides {
                no_paste: true,
                delay_ms: Some(250),
            }))
        );
        assert_eq!(
            parse(&["--paste-delay-ms=120"]),
            Ok(Command::Paste(PasteCliOverrides {
                no_paste: false,
                delay_ms: Some(120),
            }))
        );
    }

    #[test]
    fn invalid_arguments_are_rejected() {
        assert!(parse(&["--bogus"]).unwrap_err().contains("--bogus"));
        assert!(
            parse(&["--paste-delay-ms"])
                .unwrap_err()
                .contains("requires a value")
        );
        assert!(
            parse(&["--paste-delay-ms", "abc"])
                .unwrap_err()
                .contains("abc")
        );
        assert!(parse(&["--paste-delay-ms=-1"]).is_err());
        assert!(
            parse(&["-i", "--no-paste"])
                .unwrap_err()
                .contains("cannot be combined")
        );
        assert!(
            parse(&["--no-paste", "-h"])
                .unwrap_err()
                .contains("cannot be combined")
        );
        assert!(
            parse(&["-i", "-u"])
                .unwrap_err()
                .contains("cannot be combined")
        );
        assert!(
            parse(&["--no-paste", "update"])
                .unwrap_err()
                .contains("cannot be combined")
        );
        assert!(
            parse(&["-i", "update"])
                .unwrap_err()
                .contains("cannot be combined")
        );
    }

    #[test]
    fn update_subcommand_parses() {
        assert_eq!(
            parse(&["update"]),
            Ok(Command::Update(UpdateOptions {
                check: false,
                force: false,
            }))
        );
        assert_eq!(
            parse(&["update", "--check"]),
            Ok(Command::Update(UpdateOptions {
                check: true,
                force: false,
            }))
        );
        assert_eq!(
            parse(&["update", "-c"]),
            Ok(Command::Update(UpdateOptions {
                check: true,
                force: false,
            }))
        );
        assert_eq!(
            parse(&["update", "--force"]),
            Ok(Command::Update(UpdateOptions {
                check: false,
                force: true,
            }))
        );
        assert_eq!(
            parse(&["update", "-f"]),
            Ok(Command::Update(UpdateOptions {
                check: false,
                force: true,
            }))
        );
        assert_eq!(
            parse(&["update", "-c", "-f"]),
            Ok(Command::Update(UpdateOptions {
                check: true,
                force: true,
            }))
        );
        assert_eq!(parse(&["update", "-h"]), Ok(Command::UpdateHelp));
        assert_eq!(parse(&["update", "--help"]), Ok(Command::UpdateHelp));
        assert_eq!(
            parse(&["--update"]),
            Ok(Command::Update(UpdateOptions {
                check: false,
                force: false,
            }))
        );
        assert!(parse(&["update", "--unknown"]).is_err());
        assert!(parse(&["update", "-i"]).is_err());
    }
}
