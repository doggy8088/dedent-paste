# All input is passed through environment variables, never interpolated as code.
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false)

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
            $shell = New-Object -ComObject WScript.Shell
            $link = $shell.CreateShortcut($env:DEDENT_PASTE_SETUP_SHORTCUT)
            $arguments = '"' + $env:DEDENT_PASTE_SETUP_SCRIPT + '"'
            if ($link.Description -cne $env:DEDENT_PASTE_SETUP_DESCRIPTION -or $link.Arguments -ine $arguments) {
                throw "Unrelated startup shortcut at $env:DEDENT_PASTE_SETUP_SHORTCUT. Move it aside before running setup."
            }
        }
    }
    'create-shortcut' {
        $shell = New-Object -ComObject WScript.Shell
        $link = $shell.CreateShortcut($env:DEDENT_PASTE_SETUP_SHORTCUT)
        $link.TargetPath = $env:DEDENT_PASTE_SETUP_INTERPRETER
        $link.Arguments = '"' + $env:DEDENT_PASTE_SETUP_SCRIPT + '"'
        $link.WorkingDirectory = [IO.Path]::GetDirectoryName($env:DEDENT_PASTE_SETUP_SCRIPT)
        $link.Description = $env:DEDENT_PASTE_SETUP_DESCRIPTION
        $link.Save()
    }
    default { throw 'Unknown setup operation.' }
}
