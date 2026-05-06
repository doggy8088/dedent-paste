use std::error::Error;
use std::io::Write;
use std::process::{Command, Stdio};

use dedent_paste::{dedent_text, text_from_bytes};

const UTF8_LOCALE: &str = "en_US.UTF-8";

fn main() {
    if let Err(error) = run() {
        eprintln!("dedent-paste: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    if !cfg!(target_os = "macos") {
        return Err(
            "dedent-paste requires macOS because it uses pbpaste, pbcopy, and osascript".into(),
        );
    }

    let clipboard = read_clipboard()?;
    let text = text_from_bytes(clipboard)?;
    let dedented = dedent_text(&text);

    write_clipboard(&dedented)?;
    paste_from_clipboard()?;

    Ok(())
}

fn read_clipboard() -> Result<Vec<u8>, Box<dyn Error>> {
    let output = Command::new("pbpaste")
        .args(["-Prefer", "txt"])
        .env("LANG", UTF8_LOCALE)
        .env("LC_CTYPE", UTF8_LOCALE)
        .output()?;

    if !output.status.success() {
        return Err(format!("pbpaste failed with status {}", output.status).into());
    }

    Ok(output.stdout)
}

fn write_clipboard(text: &str) -> Result<(), Box<dyn Error>> {
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

fn paste_from_clipboard() -> Result<(), Box<dyn Error>> {
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
