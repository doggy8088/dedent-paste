# dedent-paste

貼上縮排文字時，自動移除多餘的共同縮排。

`dedent-paste` 會：

1. 以純文字讀取目前剪貼簿內容。
2. 移除非空白行前方共同的空白或 Tab 縮排。
3. 移除每一行結尾多餘的空白或 Tab。
4. 如果內容符合 `›`、`❯` 或 `•` 提示格式，移除提示符與續行前綴，並合併同一段落中的視覺換行。若為 `>`、`▸` 或 `▾` 前綴（例如引文或 CLI 思考區塊前綴），只移除前綴與縮排空白，不會改變原有換行。提示符後與續行縮排的空白不限 ASCII 空白，也接受不換行空白（U+00A0）、全形空白（U+3000）等 Unicode 空白字元。
5. 將整理後的文字寫回剪貼簿並立即貼上。

提示格式中的單一換行通常是終端機寬度造成的視覺折行。`dedent-paste` 會依文字邊界自動接合：CJK 文字直接相接，拉丁文字或中拉丁文字交界補上一個空白。兩個以上連續的換行則視為真正的段落分隔並予以保留。若某一行的結尾是標點符號（`。`、`！`、`？`、`：`、`.`、`!`、`?`、`:`、`…`、`，`、`,`，後方可接右引號或右括號），或下一行以清單標記開頭（`1. `、`2) `、`- `、`* `），則視為使用者實際輸入的換行並予以保留，不會與下一行接合。若某一行的反引號（`` ` ``）數量為奇數，表示折行發生在尚未閉合的程式碼片段內，此時無論行尾標點或下一行開頭為何，都會與下一行接合。未符合提示格式的文字不會套用這項段落展開處理。

另外，如果剪貼簿**沒有文字但有圖片**（例如螢幕截圖），且已設定 Gemini API 金鑰，`dedent-paste` 會自動把圖片交給 Gemini 辨識成格式化文字後貼上。詳見〈[圖片轉文字（Gemini）](#圖片轉文字gemini)〉。

建議搭配方式：

- macOS：搭配 Karabiner-Elements，使用 `左 Option+V`（右 Option 保留給其他工具）
- Windows：搭配 AutoHotkey，使用 `Win+V`

也可以透過 npm 安裝 CLI：

```sh
npm install -g dedent-paste
```

npm 套件會在安裝時從 GitHub Releases 下載符合目前平台的原生執行檔並驗證 SHA-256 checksum。

macOS 也可以透過 Homebrew tap（[doggy8088/homebrew-dedent-paste](https://github.com/doggy8088/homebrew-dedent-paste)）安裝：

```sh
brew tap doggy8088/dedent-paste
brew trust doggy8088/dedent-paste
brew install dedent-paste
```

Homebrew 6 之後對第三方 tap 需要先執行一次 `brew trust`，較舊版本可略過該行。Homebrew 只會安裝 `dedent-paste` 執行檔，安裝完成後會提示你還需要設定 Karabiner-Elements。請接著執行：

```sh
dedent-paste --install
```

這會把 `左 Option+V` 規則寫進目前啟用中的 Karabiner-Elements profile，規則會直接指向 Homebrew 安裝的執行檔路徑。詳見〈[命令列參數](#命令列參數)〉。

> 注意：Windows 預設的 `Win+V` 是「剪貼簿歷程記錄」。如果你使用下面的 AutoHotkey 腳本，這個預設快捷鍵會被覆蓋。

## macOS

### 安裝

需求：macOS、Karabiner-Elements 與 `curl`。

```sh
curl -fsSL https://raw.githubusercontent.com/doggy8088/dedent-paste/main/install.sh | bash
```

這個安裝程式會下載最新 Release 的 shell installer，並把 `dedent-paste` 安裝到：

```text
$HOME/.local/bin/dedent-paste
```

### 設定

- 安裝程式在放好執行檔後會執行 `dedent-paste --install`，自動把 `左 Option+V` 規則加入目前啟用中的 Karabiner-Elements profile。規則只認**左** Option（`left_option`），右 Option 不受影響，可留給語音輸入等需要獨佔右 Option 的工具。
- 修改前會先備份 Karabiner 設定（`~/.config/karabiner/karabiner.json.bak-<時間戳記>`）。
- Karabiner 規則會直接指向 `$HOME/.local/bin/dedent-paste`，所以即使你的 shell `PATH` 尚未包含 `$HOME/.local/bin`，快捷鍵仍可正常使用。
- 如果你想在 Terminal 直接輸入 `dedent-paste`，再自行把 `$HOME/.local/bin` 加入 `PATH`。
- 如果你想手動查看或匯入規則，可以參考 [`examples/macos/paste-dedent-plain-text.json`](examples/macos/paste-dedent-plain-text.json)。
- 想移除快捷鍵時執行 `dedent-paste --uninstall`。

#### 改用右 Option 或左右皆可

規則的修飾鍵定義在 `~/.config/karabiner/karabiner.json`（以及 `~/.config/karabiner/assets/complex_modifications/paste-dedent-plain-text.json`）中：

```json
"modifiers": { "mandatory": ["left_option"] }
```

把 `left_option` 改成 `right_option`（只認右 Option）或 `option`（左右皆可）即可，Karabiner 會自動重新載入。注意：之後再執行 `dedent-paste --install` 會把規則重設回 `left_option`。

#### 權限

貼上動作是由 Karabiner 啟動的 `dedent-paste` 透過 `osascript`（System Events）送出 `Command+V`，因此需要兩項授權：

- **輔助使用**：`系統設定 > 隱私權與安全性 > 輔助使用`，確認 Karabiner-Elements（`karabiner_console_user_server`）已被允許。若看到 `System Events 發生錯誤：不允許「osascript」傳送按鍵。 (1002)`，就是這項權限不足。
- **自動化**：第一次執行時 macOS 可能會詢問是否允許控制「System Events」，請允許。之後可在 `系統設定 > 隱私權與安全性 > 自動化` 檢視。

> 注意：`osascript` 回傳成功（exit code 0）只代表 System Events 接受了按鍵事件，不保證目標欄位真的收到貼上。若剪貼簿已整理好但畫面上沒有貼上，多半是修飾鍵時序問題，請參考下方〈[貼上時序與第三方快捷鍵工具](#貼上時序與第三方快捷鍵工具)〉。

### 使用

安裝與設定完成後，直接按：

```text
左 Option+V
```

### 命令列參數

不帶參數執行 `dedent-paste` 就是快捷鍵觸發的貼上流程。另外提供下列參數：

| 子命令 / 參數 | 說明 |
|---|---|
| `update` | 檢查並自動更新 `dedent-paste` 至最新版本（支援 `-c`/`--check` 與 `-f`/`--force`） |
| `-n`, `--no-paste` | 只整理並寫回剪貼簿，不送出 `Command+V` / `Ctrl+V`；由呼叫端自行貼上 |
| `--paste-delay-ms <毫秒>` | 修飾鍵放開後、送出貼上按鍵前額外等待的毫秒數（預設 `0`） |
| `-h`, `--help` | 顯示參數說明 |
| `-v`, `--version` | 顯示目前版本號 |
| `-i`, `--install` | 初始化 Karabiner-Elements 設定（僅 macOS） |
| `-u`, `--uninstall` | 移除 Karabiner-Elements 中的 dedent-paste 規則（僅 macOS） |

`--no-paste` 與 `--paste-delay-ms` 可以同時使用，也可以改用環境變數 `DEDENT_PASTE_NO_PASTE`、`DEDENT_PASTE_PASTE_DELAY_MS`（見〈[環境變數](#環境變數)〉）；命令列參數優先。`-h`、`-v`、`-i`、`-u` 與 `update` 必須單獨使用。

`update` 的行為：

1. 檢查目前執行的 `dedent-paste` 安裝方式（獨立二進位檔、Homebrew、npm、Cargo 或本機建置）。
2. 連線至 GitHub Releases 檢查是否有更新的版本。
3. 若指定 `--check`（或 `-c`），只顯示是否有更新與建議的更新指令，不進行安裝。
4. 若為獨立安裝的二進位檔（例如透過官方 shell / PowerShell 安裝腳本安裝到 `$HOME/.local/bin`）：自動下載對應平台的最新版本並原地升級；在 macOS 上更新後會自動重新套用 `--install` 確保 Karabiner-Elements 規則維持最新。
5. 若為 Homebrew、npm 或 Cargo 安裝，會提示使用對應套件管理器的升級指令（例如 `brew upgrade dedent-paste`、`npm install -g dedent-paste`），避免破壞套件管理器狀態。
6. 可使用 `--force`（或 `-f`）強制重新安裝最新版本。

`--install` 的行為：

1. 以 [`examples/macos/paste-dedent-plain-text.json`](examples/macos/paste-dedent-plain-text.json) 為範本，把規則中的執行路徑改成**目前執行的這個 `dedent-paste` 的實際安裝路徑**（例如 Homebrew 的 `/opt/homebrew/bin/dedent-paste`、npm 的安裝目錄或 `$HOME/.local/bin/dedent-paste`）。若路徑位於 Homebrew 的 `Cellar/<版本>/` 目錄，會改寫成不含版本號的 `opt/dedent-paste/bin/dedent-paste`，以免 `brew upgrade` 之後失效。
2. 把範本寫到 `~/.config/karabiner/assets/complex_modifications/paste-dedent-plain-text.json`，方便在 Karabiner-Elements 介面中手動匯入。
3. 備份 `~/.config/karabiner/karabiner.json`，再把規則寫進目前啟用中的 profile。判斷「是否已有 dedent-paste 規則」的依據是規則裡 `shell_command` 是否包含 `dedent-paste`，而不是規則名稱；既有規則會原地取代，多餘的重複規則會一併移除。
4. 如果找得到 `karabiner_cli`，會順便驗證產生的規則。

`--uninstall` 會從**所有** profile 移除 `shell_command` 包含 `dedent-paste` 的規則（同樣會先備份），並刪除上述的 asset 檔案。

### 貼上時序與第三方快捷鍵工具

在 macOS 上，`dedent-paste` 寫回剪貼簿後會依序：

1. **等待修飾鍵全部放開**（Shift、Control、Option、Command，最多等 1 秒）。快捷鍵觸發時實體 Option 通常還按著，若此時就送出 `Command+V`，許多應用程式會收到 `Command+Option+V` 而不是一般貼上，結果就是「exit code 0 但沒貼上」。
2. 套用 `--paste-delay-ms` / `DEDENT_PASTE_PASTE_DELAY_MS` 指定的額外延遲（預設 0）。
3. 透過 System Events 送出 `Command+V`。

另外，同一時間只允許一個 `dedent-paste` 執行：後啟動的實例會立刻靜默結束（只寫一行記錄檔）。這是為了對付會產生 key-repeat 的快捷鍵工具（例如 skhd 的 `lalt - v` 在按住時會連續觸發數十次），避免一次按鍵貼上多次。

如果你使用 skhd、Hammerspoon 等自己就能可靠送出按鍵的工具，建議改成「`dedent-paste` 只整理剪貼簿，由快捷鍵工具貼上」：

```text
# skhd 範例：左 Option+V
lalt - v : ~/.local/bin/dedent-paste --no-paste && skhd -k "cmd - v"
```

`--no-paste` 模式下不會等待修飾鍵，也不會送出任何按鍵；請讓快捷鍵工具在 Option 與 V 都放開後再送出 `Command+V`。

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
| `DEDENT_PASTE_NO_PASTE` | 設為 `1`/`true`/`yes`/`on` 時只整理剪貼簿、不送出貼上按鍵（同 `--no-paste`） | 未設定：會貼上 |
| `DEDENT_PASTE_PASTE_DELAY_MS` | 修飾鍵放開後、送出貼上按鍵前的額外延遲毫秒數（同 `--paste-delay-ms`） | `0` |

貼上相關的兩個變數在文字與圖片轉文字兩種流程都適用；`--install` 產生的 Karabiner 規則會先載入 `~/.config/dedent-paste/env`，所以直接寫在該檔案即可，不需修改 `karabiner.json`。

自訂 system prompt 中可以使用 `{language}` 佔位符，執行時會代換成輸出語言。

### 錯誤處理

- 未設定金鑰、或剪貼簿既無文字也無圖片：**靜默結束**，只寫入記錄檔。
- 已設定金鑰但呼叫失敗（網路錯誤、API 錯誤、prompt 檔案無法讀取）：顯示系統通知（macOS 通知中心；Windows 錯誤對話方塊），剪貼簿內容不會被更動，並寫入記錄檔。
- 圖片過大（要求超過 20 MB 上限）會直接回報錯誤，不會送出。

> Windows 使用者請更新 AutoHotkey 腳本至最新範例：舊版腳本會在非零 exit code 時再跳出一個泛用錯誤視窗，新版已移除以避免重複提示。

### 已知限制

- macOS 的貼上是透過 System Events 模擬 `Command+V`；`osascript` 成功結束不代表目標欄位一定收到內容。已知的原因與對策見〈[貼上時序與第三方快捷鍵工具](#貼上時序與第三方快捷鍵工具)〉。
- 單一實例保護僅在 macOS 實作（Windows 的 AutoHotkey 範例已透過 `RunWait` 與 `KeyWait` 避免重複執行）。
- 在 Finder 複製 HEIC「檔案」時，剪貼簿放的是檔案路徑而非圖片內容，會視為沒有圖片。請改用預覽程式開啟後複製，或直接使用螢幕截圖。
- Gemini 呼叫期間（數秒至數十秒）沒有進度提示，完成後才會貼上。

## 相關連結

- [Karabiner-Elements](https://karabiner-elements.pqrs.org/)
- [Karabiner-Elements 使用手冊](https://karabiner-elements.pqrs.org/docs/)
- [Karabiner complex modifications](https://karabiner-elements.pqrs.org/docs/manual/configuration/configure-complex-modifications/)
- [AutoHotkey](https://www.autohotkey.com/)
- [dedent-paste GitHub Releases](https://github.com/doggy8088/dedent-paste/releases)
- [dedent-paste Homebrew tap](https://github.com/doggy8088/homebrew-dedent-paste)
- [cargo-dist](https://opensource.axo.dev/cargo-dist/)
- [macOS 輔助使用權限說明](https://support.apple.com/guide/mac-help/allow-accessibility-apps-to-access-your-mac-mh43185/mac)

## 更多資訊

- 網站：[dedent-paste GitHub Pages](https://dedent-paste.gh.miniasp.com/)
- 開發筆記：[DEVELOPMENT.md](DEVELOPMENT.md)
- 變更紀錄：[CHANGELOG.md](CHANGELOG.md)
- 授權：[MIT](LICENSE)
