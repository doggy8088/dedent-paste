# dedent-paste

貼上縮排文字時，自動移除多餘的共同縮排。

`dedent-paste` 會：

1. 以純文字讀取目前剪貼簿內容。
2. 移除非空白行前方共同的空白或 Tab 縮排。
3. 移除每一行結尾多餘的空白或 Tab。
4. 將整理後的文字寫回剪貼簿並立即貼上。

建議搭配方式：

- macOS：搭配 Karabiner-Elements，使用 `Option+V`
- Windows：搭配 AutoHotkey，使用 `Win+V`

> 注意：Windows 預設的 `Win+V` 是「剪貼簿歷程記錄」。如果你使用下面的 AutoHotkey 腳本，這個預設快捷鍵會被覆蓋。

## macOS

### 安裝

需求：macOS、Karabiner-Elements、`curl` 與 Python 3。

```sh
curl -fsSL https://raw.githubusercontent.com/doggy8088/dedent-paste/main/install.sh | bash
```

這個安裝程式會下載最新 Release 的 shell installer，並把 `dedent-paste` 安裝到：

```text
$HOME/.local/bin/dedent-paste
```

### 設定

- 安裝程式會自動把 `Option+V` 規則加入目前啟用中的 Karabiner-Elements profile。
- 修改前會先備份 Karabiner 設定。
- Karabiner 規則會直接指向 `$HOME/.local/bin/dedent-paste`，所以即使你的 shell `PATH` 尚未包含 `$HOME/.local/bin`，快捷鍵仍可正常使用。
- 如果你想在 Terminal 直接輸入 `dedent-paste`，再自行把 `$HOME/.local/bin` 加入 `PATH`。
- 如果你想手動查看或匯入規則，可以參考 [`examples/macos/paste-dedent-plain-text.json`](examples/macos/paste-dedent-plain-text.json)。

Karabiner-Elements 可能需要 macOS「輔助使用」權限，才能透過 System Events 觸發貼上動作。

請開啟：

```text
系統設定 > 隱私權與安全性 > 輔助使用
```

確認 Karabiner-Elements 已被允許。

### 使用

安裝與設定完成後，直接按：

```text
Option+V
```

## Windows

### 安裝

需求：Windows 10/11、PowerShell 5.1 或更新版本；若要綁定快捷鍵，請另外安裝 AutoHotkey。

Release 頁面除了壓縮檔之外，也有 cargo-dist 產生的 PowerShell 安裝器 [`dedent-paste-installer.ps1`](https://github.com/doggy8088/dedent-paste/releases/latest/download/dedent-paste-installer.ps1)。可以直接執行：

```powershell
irm https://github.com/doggy8088/dedent-paste/releases/latest/download/dedent-paste-installer.ps1 | iex
```

安裝完成後，`dedent-paste.exe` 預設會放在：

```text
$HOME/.local/bin/dedent-paste.exe
```

在 Windows 上，上面的路徑通常等同於：

```text
%USERPROFILE%\.local\bin\dedent-paste.exe
```

你也可以直接從 [GitHub Releases](https://github.com/doggy8088/dedent-paste/releases) 手動下載：

- `dedent-paste-installer.ps1`
- `dedent-paste-x86_64-pc-windows-msvc.zip`

### 設定

#### PATH 環境變數

PowerShell 安裝器通常會嘗試把 `$HOME/.local/bin` 加進 `PATH`，這樣你可以在新的 Terminal / PowerShell 視窗直接輸入 `dedent-paste.exe`。

如果安裝後仍然找不到指令，請先：

1. 關閉再重開 Terminal / PowerShell
2. 或重新登入 Windows

不過如果你是透過 AutoHotkey 來觸發 `dedent-paste`，**建議直接在腳本中寫固定路徑**，不要依賴 `PATH`。這樣最不容易因為 PATH 尚未刷新而失敗。以下範例都直接使用：

```text
A_Home "\.local\bin\dedent-paste.exe"
```

如果你已經確認 `PATH` 生效，也可以把腳本裡的完整路徑改成單純的 `dedent-paste.exe`。

#### AutoHotkey v2

先安裝 [AutoHotkey v2](https://www.autohotkey.com/)，然後建立一個 `dedent-paste-win-v.ahk` 檔案，內容如下（範例檔：[`examples/windows/dedent-paste-win-v-v2.ahk`](examples/windows/dedent-paste-win-v-v2.ahk)）：

```ahk
#Requires AutoHotkey v2.0
#SingleInstance Force

dedentPaste := A_Home "\.local\bin\dedent-paste.exe"

#v::{
    global dedentPaste

    KeyWait "LWin"
    KeyWait "RWin"

    if !FileExist(dedentPaste) {
        MsgBox "找不到：`n" dedentPaste "`n`n請先安裝 dedent-paste，或修改腳本中的路徑。", "dedent-paste", "Icon!"
        return
    }

    try exitCode := RunWait Format("""{1}""", dedentPaste),, "Hide"
    catch Error as err {
        MsgBox "啟動 dedent-paste 失敗。`n`n" err.Message, "dedent-paste", "Iconx"
        return
    }

    if (exitCode != 0)
        MsgBox "dedent-paste 執行失敗，exit code: " exitCode, "dedent-paste", "Iconx"
}
```

#### AutoHotkey v1

如果你還在使用 v1，請安裝 [AutoHotkey v1.1](https://www.autohotkey.com/download/1.1/)，然後建立腳本（範例檔：[`examples/windows/dedent-paste-win-v-v1.ahk`](examples/windows/dedent-paste-win-v-v1.ahk)）：

```ahk
#NoEnv
#SingleInstance Force
SendMode Input
SetWorkingDir %A_ScriptDir%

dedentPaste := A_Home . "\.local\bin\dedent-paste.exe"

#v::
    KeyWait, LWin
    KeyWait, RWin

    if !FileExist(dedentPaste)
    {
        MsgBox, 48, dedent-paste, 找不到：`n%dedentPaste%`n`n請先安裝 dedent-paste，或修改腳本中的路徑。
        return
    }

    quotedPath := Chr(34) . dedentPaste . Chr(34)
    RunWait, %quotedPath%,, Hide UseErrorLevel
    if (ErrorLevel = "ERROR")
    {
        MsgBox, 16, dedent-paste, 啟動 dedent-paste 失敗。
        return
    }

    if (ErrorLevel != 0)
        MsgBox, 16, dedent-paste, dedent-paste 執行失敗，exit code: %ErrorLevel%
return
```

如果你想讓它每次登入 Windows 都自動生效，可以把 `.ahk` 腳本或其捷徑放到「啟動」資料夾。

### 使用

1. 先確認 AutoHotkey 腳本正在執行。
2. 複製一段帶有共同縮排的文字。
3. 按下 `Win+V`。
4. `dedent-paste` 會把剪貼簿內容轉成純文字、移除共同縮排後立即貼上。

如果你仍想保留 Windows 內建的 `Win+V` 剪貼簿歷程記錄，可以把 AutoHotkey 腳本裡的 `#v` 改成其他快捷鍵，例如 `!v`（`Alt+V`）。

## 相關連結

- [Karabiner-Elements](https://karabiner-elements.pqrs.org/)
- [Karabiner-Elements 使用手冊](https://karabiner-elements.pqrs.org/docs/)
- [Karabiner complex modifications](https://karabiner-elements.pqrs.org/docs/manual/configuration/configure-complex-modifications/)
- [AutoHotkey](https://www.autohotkey.com/)
- [dedent-paste GitHub Releases](https://github.com/doggy8088/dedent-paste/releases)
- [cargo-dist](https://opensource.axo.dev/cargo-dist/)
- [macOS 輔助使用權限說明](https://support.apple.com/guide/mac-help/allow-accessibility-apps-to-access-your-mac-mh43185/mac)

## 更多資訊

- 網站：[dedent-paste GitHub Pages](https://dedent-paste.gh.miniasp.com/)
- 開發筆記：[DEVELOPMENT.md](DEVELOPMENT.md)
- 變更紀錄：[CHANGELOG.md](CHANGELOG.md)
- 授權：[MIT](LICENSE)
