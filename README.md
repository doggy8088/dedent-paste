# dedent-paste

貼上縮排文字時，自動移除多餘的共同縮排。

`dedent-paste` 會：

1. 以純文字讀取目前剪貼簿內容。
2. 移除非空白行前方共同的空白或 Tab 縮排。
3. 移除每一行結尾多餘的空白或 Tab。
4. 如果內容符合 Codex CLI 的 `›` 或 `❯` 提示格式，移除提示符與續行前綴，並合併同一段落中的視覺換行。若為 `> ` 前綴（例如貼上時保留的引文前綴），只移除前綴與縮排空白，不會改變原有換行。
5. 將整理後的文字寫回剪貼簿並立即貼上。

Codex CLI 的單一換行通常是終端機寬度造成的視覺折行。`dedent-paste` 會依文字邊界自動接合：CJK 文字直接相接，拉丁文字或中拉丁文字交界補上一個空白。兩個以上連續的換行則視為真正的段落分隔並予以保留。未符合 Codex CLI 提示格式的文字不會套用這項段落展開處理。

另外，如果剪貼簿**沒有文字但有圖片**（例如螢幕截圖），且已設定 Gemini API 金鑰，`dedent-paste` 會自動把圖片交給 Gemini 辨識成格式化文字後貼上。詳見〈[圖片轉文字（Gemini）](#圖片轉文字gemini)〉。

建議搭配方式：

- macOS：搭配 Karabiner-Elements，使用 `Option+V`
- Windows：搭配 AutoHotkey，使用 `Win+V`

也可以透過 npm 安裝 CLI：

```sh
npm install -g dedent-paste
```

npm 套件會在安裝時從 GitHub Releases 下載符合目前平台的原生執行檔並驗證 SHA-256 checksum。

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

    ; 執行失敗時 dedent-paste 會自行顯示錯誤對話方塊，這裡不再重複提示。
    try RunWait Format("""{1}""", dedentPaste),, "Hide"
    catch Error as err {
        MsgBox "啟動 dedent-paste 失敗。`n`n" err.Message, "dedent-paste", "Iconx"
        return
    }
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

    ; 執行失敗時 dedent-paste 會自行顯示錯誤對話方塊，這裡不再重複提示。
    quotedPath := Chr(34) . dedentPaste . Chr(34)
    RunWait, %quotedPath%,, Hide UseErrorLevel
    if (ErrorLevel = "ERROR")
        MsgBox, 16, dedent-paste, 啟動 dedent-paste 失敗。
return
```

如果你想讓它每次登入 Windows 都自動生效，可以把 `.ahk` 腳本或其捷徑放到「啟動」資料夾。

### 使用

1. 先確認 AutoHotkey 腳本正在執行。
2. 複製一段帶有共同縮排的文字。
3. 按下 `Win+V`。
4. `dedent-paste` 會把剪貼簿內容轉成純文字、移除共同縮排後立即貼上。

如果你仍想保留 Windows 內建的 `Win+V` 剪貼簿歷程記錄，可以把 AutoHotkey 腳本裡的 `#v` 改成其他快捷鍵，例如 `!v`（`Alt+V`）。

## 圖片轉文字（Gemini）

當剪貼簿**沒有文字但有圖片**時，`dedent-paste` 會把圖片送到 [Gemini API](https://ai.google.dev/) 辨識：

- 圖片中有文字：忠實轉錄成格式化文字（以 Markdown 保留標題、清單、表格與程式碼區塊結構）。
- 圖片中沒有文字：改為產生圖片描述（預設使用繁體中文 `zh-TW`）。

辨識結果會直接貼上。剪貼簿有文字時行為完全不變，仍走原本的縮排整理流程。

> 隱私提醒：啟用此功能後，剪貼簿中的圖片會上傳至 Google Gemini API。

### 啟用方式

只需要設定 API 金鑰（可在 [Google AI Studio](https://aistudio.google.com/apikey) 取得）：

```sh
export GEMINI_API_KEY="你的金鑰"
```

**沒有設定金鑰時，此功能完全不會啟動**：按下快捷鍵不會有任何視窗或通知，只會在記錄檔寫入一行訊息。

> 注意：透過 Karabiner-Elements 或 AutoHotkey 觸發時，環境變數必須讓該程式看得到。在終端機 `export` 或寫進 `.zshrc` 是不夠的，請參考下一節。

### 在 Karabiner-Elements / AutoHotkey 中設定環境變數

快捷鍵工具不會載入你的 shell 設定檔（`.zshrc`、`.bash_profile` 等），所以在終端機 `export` 的環境變數對它們是看不見的。

#### macOS（Karabiner-Elements）

建議建立專用的環境變數檔 `~/.config/dedent-paste/env`：

```sh
mkdir -p ~/.config/dedent-paste
cat > ~/.config/dedent-paste/env <<'EOF'
export GEMINI_API_KEY="你的金鑰"
export DEDENT_PASTE_LANG="zh-TW"
EOF
chmod 600 ~/.config/dedent-paste/env
```

再把 Karabiner 規則（`~/.config/karabiner/karabiner.json`，修改前建議先備份）中的 `shell_command` 改成先載入這個檔案：

```json
{
  "shell_command": ". \"$HOME/.config/dedent-paste/env\" 2>/dev/null; exec $HOME/.local/bin/dedent-paste"
}
```

Karabiner 會自動套用設定變更，不需要重新啟動。`2>/dev/null` 讓檔案不存在時快捷鍵仍可正常運作（只是圖片轉文字功能不會啟用）。

另一個做法是 `launchctl setenv GEMINI_API_KEY "你的金鑰"`，但重開機後會失效（需搭配 LaunchAgent），而且金鑰會暴露給整個 GUI session 的所有程式，因此建議使用上面的環境變數檔。

#### Windows（AutoHotkey）

AutoHotkey 會繼承「使用者環境變數」。用 `setx` 或「系統內容 > 環境變數」設定後，**重新啟動 AutoHotkey 腳本**即可生效：

```powershell
setx GEMINI_API_KEY "你的金鑰"
setx DEDENT_PASTE_LANG "zh-TW"
```

`setx` 只影響之後啟動的程式，已經在執行的 AutoHotkey 腳本要重啟才會看到新值。

如果不想設定全域環境變數，也可以在 AutoHotkey 腳本裡、執行 `RunWait` 之前設定，讓變數只作用於 dedent-paste（v2 語法）：

```ahk
EnvSet "GEMINI_API_KEY", "你的金鑰"
EnvSet "DEDENT_PASTE_LANG", "zh-TW"
```

> 提醒：金鑰寫進腳本或設定檔後，請將檔案權限設為僅自己可讀（macOS 上 `chmod 600`），並避免分享或提交進版本控制。

### 環境變數

| 環境變數 | 用途 | 預設值 / 備援 |
|---|---|---|
| `DEDENT_PASTE_GEMINI_API_KEY` | 本工具專用金鑰（優先） | 未設定時改用 `GEMINI_API_KEY` |
| `GEMINI_API_KEY` | 共用金鑰 | 兩者皆未設定時功能停用 |
| `DEDENT_PASTE_GEMINI_MODEL` | 模型 ID | `gemini-3.7-flash` |
| `DEDENT_PASTE_LANG` | 輸出語言（直接使用，如 `en-US`、`ja`） | 未設定時由 `LANG` 推導（如 `zh_TW.UTF-8` → `zh-TW`）；再無則 `zh-TW` |
| `DEDENT_PASTE_GEMINI_SYSTEM_PROMPT` | 自訂 system prompt（直接內容，優先於檔案） | 內建 prompt |
| `DEDENT_PASTE_GEMINI_SYSTEM_PROMPT_FILE` | 自訂 system prompt 檔案路徑 | — |
| `DEDENT_PASTE_GEMINI_TIMEOUT_SECS` | API 逾時秒數 | `60` |
| `DEDENT_PASTE_LOG_FILE` | 記錄檔路徑 | macOS：`~/Library/Logs/dedent-paste.log`；Windows：`%LOCALAPPDATA%\dedent-paste\dedent-paste.log` |

自訂 system prompt 中可以使用 `{language}` 佔位符，執行時會代換成輸出語言。

### 錯誤處理

- 未設定金鑰、或剪貼簿既無文字也無圖片：**靜默結束**，只寫入記錄檔。
- 已設定金鑰但呼叫失敗（網路錯誤、API 錯誤、prompt 檔案無法讀取）：顯示系統通知（macOS 通知中心；Windows 錯誤對話方塊），剪貼簿內容不會被更動，並寫入記錄檔。
- 圖片過大（要求超過 20 MB 上限）會直接回報錯誤，不會送出。

> Windows 使用者請更新 AutoHotkey 腳本至最新範例：舊版腳本會在非零 exit code 時再跳出一個泛用錯誤視窗，新版已移除以避免重複提示。

### 已知限制

- 在 Finder 複製 HEIC「檔案」時，剪貼簿放的是檔案路徑而非圖片內容，會視為沒有圖片。請改用預覽程式開啟後複製，或直接使用螢幕截圖。
- Gemini 呼叫期間（數秒至數十秒）沒有進度提示，完成後才會貼上。

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
