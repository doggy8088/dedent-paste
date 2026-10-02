# All input is passed through environment variables, never interpolated as code.
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false)

# WScript.Shell uses ANSI paths for shortcut persistence. Use the Unicode
# ShellLink interface and IPersistFile for both loading and saving instead.
if ($env:DEDENT_PASTE_SETUP_ACTION -in @('validate-shortcut', 'create-shortcut')) {
    Add-Type -TypeDefinition @'
using System;
using System.IO;
using System.Text;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.ComTypes;

[ComImport, Guid("00021401-0000-0000-C000-000000000046")]
class ShellLink {}

// Declaration order must match the native IShellLinkW vtable.
[ComImport, Guid("000214F9-0000-0000-C000-000000000046"),
 InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IShellLinkW {
    void GetPath([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder path, int size, IntPtr data, uint flags);
    void GetIDList(out IntPtr id);
    void SetIDList(IntPtr id);
    void GetDescription([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder text, int size);
    void SetDescription([MarshalAs(UnmanagedType.LPWStr)] string text);
    void GetWorkingDirectory([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder path, int size);
    void SetWorkingDirectory([MarshalAs(UnmanagedType.LPWStr)] string path);
    void GetArguments([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder text, int size);
    void SetArguments([MarshalAs(UnmanagedType.LPWStr)] string text);
    void GetHotkey(out short key);
    void SetHotkey(short key);
    void GetShowCmd(out int command);
    void SetShowCmd(int command);
    void GetIconLocation([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder path, int size, out int index);
    void SetIconLocation([MarshalAs(UnmanagedType.LPWStr)] string path, int index);
    void SetRelativePath([MarshalAs(UnmanagedType.LPWStr)] string path, uint reserved);
    void Resolve(IntPtr window, uint flags);
    void SetPath([MarshalAs(UnmanagedType.LPWStr)] string path);
}

public static class DedentPasteShortcut {
    public static void Create(string path, string target, string script, string description) {
        var instance = new ShellLink();
        try {
            var link = (IShellLinkW)instance;
            link.SetPath(target);
            link.SetArguments("\"" + script + "\"");
            link.SetWorkingDirectory(Path.GetDirectoryName(script));
            link.SetDescription(description);
            ((IPersistFile)instance).Save(path, true);
        } finally { Marshal.FinalReleaseComObject(instance); }
    }

    public static bool IsOwned(string path, string script, string description) {
        var instance = new ShellLink();
        try {
            ((IPersistFile)instance).Load(path, 0);
            var link = (IShellLinkW)instance;
            var text = new StringBuilder(32768);
            link.GetDescription(text, text.Capacity);
            if (!String.Equals(text.ToString(), description, StringComparison.Ordinal)) return false;
            text.Clear();
            link.GetArguments(text, text.Capacity);
            return String.Equals(text.ToString(), "\"" + script + "\"", StringComparison.OrdinalIgnoreCase);
        } finally { Marshal.FinalReleaseComObject(instance); }
    }
}
'@
}

switch ($env:DEDENT_PASTE_SETUP_ACTION) {
    'paths' {
        $local = [Environment]::GetFolderPath('LocalApplicationData')
        $startup = [Environment]::GetFolderPath('Startup')
        if (!$local -or !$startup) { throw 'Windows did not return the current user setup folders.' }
        @{ local = $local; startup = $startup } | ConvertTo-Json -Compress
    }
    'discover' {
        $roots = New-Object 'System.Collections.Generic.List[string]'
        foreach ($hive in @([Microsoft.Win32.RegistryHive]::CurrentUser, [Microsoft.Win32.RegistryHive]::LocalMachine)) {
            foreach ($view in @([Microsoft.Win32.RegistryView]::Registry64, [Microsoft.Win32.RegistryView]::Registry32)) {
                $base = [Microsoft.Win32.RegistryKey]::OpenBaseKey($hive, $view)
                try {
                    $key = $base.OpenSubKey('Software\AutoHotkey')
                    if ($null -ne $key) {
                        try {
                            $directory = $key.GetValue('InstallDir')
                            if ($directory) { $roots.Add([string]$directory) }
                        } finally { $key.Dispose() }
                    }
                } finally { $base.Dispose() }
            }
        }
        foreach ($directory in @(
            "$env:LOCALAPPDATA\Programs\AutoHotkey",
            "$env:LOCALAPPDATA\AutoHotkey",
            "$env:ProgramFiles\AutoHotkey",
            "${env:ProgramFiles(x86)}\AutoHotkey"
        )) { $roots.Add($directory) }
        $directories = New-Object 'System.Collections.Generic.List[string]'
        foreach ($root in ($roots | Select-Object -Unique)) {
            if (Test-Path -LiteralPath $root -PathType Container) {
                $directories.Add($root)
                # Versioned installations coexist below the install root; UX is a launcher.
                foreach ($child in (Get-ChildItem -LiteralPath $root -Directory | Sort-Object Name -Descending)) {
                    if ($child.Name -match '^v[12](\.|$)') { $directories.Add($child.FullName) }
                }
            }
        }
        foreach ($directory in ($env:PATH -split ';')) {
            if ($directory) { $directories.Add($directory.Trim('"')) }
        }
        $candidates = @(
            foreach ($directory in ($directories | Select-Object -Unique)) {
                foreach ($name in @('AutoHotkey64.exe', 'AutoHotkey32.exe', 'AutoHotkeyU64.exe', 'AutoHotkeyU32.exe', 'AutoHotkey.exe')) {
                    $path = Join-Path $directory $name
                    if (Test-Path -LiteralPath $path -PathType Leaf) {
                        $info = [Diagnostics.FileVersionInfo]::GetVersionInfo($path)
                        if ($info.ProductName -like '*AutoHotkey*' -and $info.FileDescription -notlike '*ANSI*') {
                            @{ path = [IO.Path]::GetFullPath($path); major = $info.FileMajorPart; minor = $info.FileMinorPart }
                        }
                    }
                }
            }
        )
        ConvertTo-Json -InputObject $candidates -Compress
    }
    'validate-shortcut' {
        if (Test-Path -LiteralPath $env:DEDENT_PASTE_SETUP_SHORTCUT) {
            if (![DedentPasteShortcut]::IsOwned(
                $env:DEDENT_PASTE_SETUP_SHORTCUT,
                $env:DEDENT_PASTE_SETUP_SCRIPT,
                $env:DEDENT_PASTE_SETUP_DESCRIPTION)) {
                throw "Unrelated startup shortcut at $env:DEDENT_PASTE_SETUP_SHORTCUT. Move it aside before running setup."
            }
        }
    }
    'create-shortcut' {
        [DedentPasteShortcut]::Create(
            $env:DEDENT_PASTE_SETUP_SHORTCUT,
            $env:DEDENT_PASTE_SETUP_INTERPRETER,
            $env:DEDENT_PASTE_SETUP_SCRIPT,
            $env:DEDENT_PASTE_SETUP_DESCRIPTION)
    }
    default { throw 'Unknown setup operation.' }
}
