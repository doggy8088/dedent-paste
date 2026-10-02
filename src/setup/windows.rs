//! Windows discovery, login shortcuts, and the managed AutoHotkey process.
use super::autohotkey::{self, Desktop, Interpreter, Paths, Result, SHORTCUT_DESCRIPTION, Version};
use base64::Engine as _;
use std::ffi::{OsStr, OsString};
use std::fs::{File, OpenOptions};
use std::os::windows::ffi::OsStringExt;
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::{CloseHandle, HWND, LPARAM};
use windows_sys::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetWindowTextW, GetWindowThreadProcessId, SMTO_ABORTIFHUNG,
    SendMessageTimeoutW, WM_CLOSE,
};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const WAIT: Duration = Duration::from_secs(5);

fn powershell(action: &str, values: &[(&str, &OsStr)]) -> Result<Vec<u8>> {
    let system_root = std::env::var_os("SystemRoot").ok_or("SystemRoot is not set")?;
    let executable =
        PathBuf::from(system_root).join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let mut command = Command::new(executable);
    let script: Vec<u8> = include_str!("windows.ps1")
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect();
    let encoded = base64::engine::general_purpose::STANDARD.encode(script);
    command
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-EncodedCommand",
            &encoded,
        ])
        .env("DEDENT_PASTE_SETUP_ACTION", action)
        .env("DEDENT_PASTE_SETUP_DESCRIPTION", SHORTCUT_DESCRIPTION)
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null());
    for (name, value) in values {
        command.env(format!("DEDENT_PASTE_SETUP_{name}"), value);
    }
    let output = command
        .output()
        .map_err(|e| format!("could not run Windows PowerShell for {action}: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "Windows setup {action} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    Ok(output.stdout)
}

fn paths() -> Result<Paths> {
    let value: serde_json::Value = serde_json::from_slice(&powershell("paths", &[])?)?;
    let local = value["local"]
        .as_str()
        .ok_or("Windows did not return LocalAppData")?;
    let startup = value["startup"]
        .as_str()
        .ok_or("Windows did not return Startup")?;
    Ok(Paths {
        script: PathBuf::from(local)
            .join("dedent-paste")
            .join("dedent-paste-win-v.ahk"),
        shortcut: PathBuf::from(startup).join("dedent-paste.lnk"),
    })
}

fn discover() -> Result<Interpreter> {
    let value: serde_json::Value = serde_json::from_slice(&powershell("discover", &[])?)?;
    let candidates = value
        .as_array()
        .ok_or("invalid AutoHotkey discovery result")?;
    let mut interpreters = Vec::new();
    for value in candidates {
        let major = value["major"]
            .as_u64()
            .ok_or("missing AutoHotkey major version")?;
        let minor = value["minor"]
            .as_u64()
            .ok_or("missing AutoHotkey minor version")?;
        if let Some(version) = Version::from_parts(major, minor) {
            interpreters.push(Interpreter {
                path: PathBuf::from(
                    value["path"]
                        .as_str()
                        .ok_or("missing AutoHotkey executable path")?,
                ),
                version,
            });
        }
    }
    autohotkey::select_interpreter(interpreters)
}

/// Share mode zero serializes install/uninstall, including across processes.
/// The file can remain after a crash: the OS releases the lock on handle close.
fn setup_lock(paths: &Paths) -> Result<File> {
    let directory = paths.script.parent().ok_or("missing script directory")?;
    std::fs::create_dir_all(directory)?;
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .share_mode(0)
        .open(directory.join("setup.lock"))
        .map_err(|e| {
            format!("cannot lock Windows setup (another install/uninstall may be running): {e}")
                .into()
        })
}

pub fn install() -> Result<()> {
    let interpreter = discover()?;
    let paths = paths()?;
    let _lock = setup_lock(&paths)?;
    let binary = std::env::current_exe()?;
    autohotkey::install(&paths, &binary, &interpreter, &mut WindowsDesktop)?;
    println!(
        "Using AutoHotkey {}: {}",
        match interpreter.version {
            Version::V1 => "v1.1",
            Version::V2 => "v2",
        },
        interpreter.path.display()
    );
    println!(
        "Managed script: {}\nLogin shortcut: {}\nRuns: {}",
        paths.script.display(),
        paths.shortcut.display(),
        binary.display()
    );
    println!(
        "Done. Win+V is enabled now and at login, replacing Windows clipboard history.\nDisable any independently configured Win+V scripts manually to avoid conflicts."
    );
    Ok(())
}

pub fn uninstall() -> Result<()> {
    let paths = paths()?;
    let _lock = setup_lock(&paths)?;
    autohotkey::uninstall(&paths, &mut WindowsDesktop)?;
    println!(
        "Done. The managed Win+V script and login shortcut have been removed.\nBackups and independently configured AutoHotkey scripts were kept."
    );
    Ok(())
}

struct WindowsDesktop;

struct ScriptWindow {
    handle: HWND,
    pid: u32,
}

struct WindowSearch {
    script: String,
    windows: Vec<ScriptWindow>,
}

unsafe extern "system" fn visit_window(window: HWND, context: LPARAM) -> i32 {
    // SAFETY: EnumWindows is synchronous and context points to its live WindowSearch.
    let search = unsafe { &mut *(context as *mut WindowSearch) };
    let mut class = [0u16; 64];
    // SAFETY: buffers are valid for the supplied lengths; window comes from EnumWindows.
    let len = unsafe { GetClassNameW(window, class.as_mut_ptr(), class.len() as i32) };
    if String::from_utf16_lossy(&class[..len.max(0) as usize]) != "AutoHotkey" {
        return 1;
    }
    let mut title = [0u16; 32768];
    // SAFETY: title is a writable buffer, and the length is its capacity.
    let len = unsafe { GetWindowTextW(window, title.as_mut_ptr(), title.len() as i32) };
    let title = String::from_utf16_lossy(&title[..len.max(0) as usize]);
    if autohotkey::is_script_window("AutoHotkey", &title, &search.script) {
        let mut pid = 0;
        // SAFETY: pid is a valid output pointer.
        unsafe {
            GetWindowThreadProcessId(window, &mut pid);
        }
        search.windows.push(ScriptWindow {
            handle: window,
            pid,
        });
    }
    1
}

fn windows(script: &Path) -> Result<Vec<ScriptWindow>> {
    let path = script
        .to_str()
        .ok_or("managed script path is not valid Unicode")?;
    let mut search = WindowSearch {
        script: path.to_owned(),
        windows: Vec::new(),
    };
    // SAFETY: search outlives every callback; EnumWindows does not retain context.
    if unsafe {
        EnumWindows(
            Some(visit_window),
            &mut search as *mut WindowSearch as LPARAM,
        )
    } == 0
    {
        return Err(format!(
            "could not enumerate AutoHotkey windows: {}",
            std::io::Error::last_os_error()
        )
        .into());
    }
    Ok(search.windows)
}

fn process_path(pid: u32) -> Result<PathBuf> {
    // SAFETY: requesting a query-only handle for the identified script process.
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if handle.is_null() {
        return Err(std::io::Error::last_os_error().into());
    }
    let mut buffer = [0u16; 32768];
    let mut len = buffer.len() as u32;
    // SAFETY: handle is valid, and buffer/len point to writable storage.
    let success = unsafe { QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut len) };
    let error = std::io::Error::last_os_error();
    // SAFETY: this function owns the process handle and closes it exactly once.
    unsafe {
        CloseHandle(handle);
    }
    if success == 0 {
        return Err(error.into());
    }
    Ok(PathBuf::from(OsString::from_wide(&buffer[..len as usize])))
}

impl Desktop for WindowsDesktop {
    fn validate_shortcut(&mut self, paths: &Paths) -> Result<()> {
        powershell(
            "validate-shortcut",
            &[
                ("SCRIPT", paths.script.as_os_str()),
                ("SHORTCUT", paths.shortcut.as_os_str()),
            ],
        )?;
        Ok(())
    }

    fn create_shortcut(&mut self, paths: &Paths, interpreter: &Path) -> Result<()> {
        powershell(
            "create-shortcut",
            &[
                ("SCRIPT", paths.script.as_os_str()),
                ("SHORTCUT", paths.shortcut.as_os_str()),
                ("INTERPRETER", interpreter.as_os_str()),
            ],
        )?;
        Ok(())
    }

    fn running_interpreter(&mut self, script: &Path) -> Result<Option<PathBuf>> {
        windows(script)?
            .first()
            .map(|window| process_path(window.pid))
            .transpose()
    }

    fn stop(&mut self, script: &Path) -> Result<()> {
        for window in windows(script)? {
            // SAFETY: the handle matched both AutoHotkey class and the full managed
            // script path. WM_CLOSE exits AHK (unlike SC_CLOSE, which hides it).
            if unsafe {
                SendMessageTimeoutW(
                    window.handle,
                    WM_CLOSE,
                    0,
                    0,
                    SMTO_ABORTIFHUNG,
                    2000,
                    std::ptr::null_mut(),
                )
            } == 0
            {
                return Err(format!("could not stop the managed AutoHotkey script: {}. Exit it from its tray icon and retry.", std::io::Error::last_os_error()).into());
            }
        }
        let deadline = Instant::now() + WAIT;
        while !windows(script)?.is_empty() {
            if Instant::now() >= deadline {
                return Err("timed out stopping the managed AutoHotkey script; exit it from its tray icon and retry".into());
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        Ok(())
    }

    fn start(&mut self, script: &Path, interpreter: &Path) -> Result<()> {
        // /ErrorStdOut prevents startup syntax errors from leaving modal dialogs.
        // Persistent child output is not piped: waiting for EOF would never finish.
        let mut child = Command::new(interpreter)
            .arg("/ErrorStdOut")
            .arg(script)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("could not launch {}: {e}", interpreter.display()))?;
        let deadline = Instant::now() + WAIT;
        let mut appeared = None;
        let result = (|| {
            loop {
                if let Some(status) = child.try_wait()? {
                    return Err(format!(
                        "AutoHotkey exited before activating Win+V ({status}); check {}",
                        script.display()
                    )
                    .into());
                }
                if windows(script)?
                    .iter()
                    .any(|window| window.pid == child.id())
                {
                    // The main window exists during initialization too. Require
                    // it to survive startup, checking for an early exit each pass.
                    if appeared.get_or_insert_with(Instant::now).elapsed()
                        >= Duration::from_millis(250)
                    {
                        return Ok(());
                    }
                } else {
                    appeared = None;
                }
                if Instant::now() >= deadline {
                    return Err("timed out waiting for AutoHotkey to activate Win+V".into());
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        })();
        if result.is_err() && child.try_wait()?.is_none() {
            // This is only the process just launched by us, never other scripts.
            child.kill().map_err(|e| {
                format!("startup failed and could not terminate the new AutoHotkey process: {e}")
            })?;
            child.wait()?;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempDirectory(PathBuf);

    impl Drop for TempDirectory {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn startup_shortcut_round_trip_preserves_unicode_paths_and_ownership() {
        // Exercise Windows' actual .lnk implementation without launching an
        // interpreter, touching the user's Startup folder, or needing a GUI.
        let directory = TempDirectory(std::env::temp_dir().join(format!(
            "dedent-paste-保哥's setup-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        )));
        std::fs::create_dir(&directory.0).unwrap();
        let mut paths = Paths {
            script: directory.0.join("dedent-paste.ahk"),
            shortcut: directory.0.join("dedent-paste.lnk"),
        };
        let mut desktop = WindowsDesktop;
        desktop.validate_shortcut(&paths).unwrap();
        desktop
            .create_shortcut(&paths, &std::env::current_exe().unwrap())
            .unwrap();
        desktop.validate_shortcut(&paths).unwrap();
        paths.script = directory.0.join("unrelated.ahk");
        assert!(desktop.validate_shortcut(&paths).is_err());
    }
}
