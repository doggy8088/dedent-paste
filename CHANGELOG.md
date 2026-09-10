# Changelog

All notable changes to this project are documented in this file.

## Unreleased

## 0.5.2

- 修正提示格式（`❯`、`›`、`•`）視覺換行接合邏輯，避免編號或符號清單項目（`1. ` `2) ` `- ` `* ` 等）被誤判為換行並接續成同一行；新增判斷邏輯，當下一行以清單標記開頭時保留原有換行。
- 換行接合的句尾終止符號新增全形冒號 `：` 與半形冒號 `:`，讓以冒號結尾的行（例如條列說明的標題行）維持原有換行，不再被接到下一行。
- 修正行內程式碼片段（例如 `` `KEY: value` ``）被終端機視覺換行截斷時未正確接回同一片段的問題：新增偵測反引號數量是否為奇數，未閉合時強制接續該行與下一行，不受句尾標點或清單標記影響。

## 0.5.1

- 新增 `update` 子命令與 `--update` 參數：讓獨立安裝的 `dedent-paste` 可直接檢查並原地下載最新發佈版本進行自我更新。
- 支援 `--check`（`-c`）僅檢查更新而不進行安裝，以及 `--force`（`-f`）強制重新安裝最新版本。
- 自動識別安裝途徑（Homebrew、npm、Cargo、本機建置或獨立安裝），對套件管理器安裝的使用者主動提供對應的升級指令（例如 `brew upgrade dedent-paste`、`npm install -g dedent-paste`），避免破壞套件管理器狀態。
- macOS 與 Linux 透過官方 shell 安裝腳本原地升級二進位檔，macOS 更新後自動重新執行 `--install` 確保 Karabiner-Elements 規則維持最新；Windows 透過檔案重命名與 PowerShell 安裝腳本處理執行中檔案鎖定與失敗還原。

## 0.5.0

- **行為變更**：macOS Karabiner 規則（`examples/macos/paste-dedent-plain-text.json` 與 `dedent-paste --install`）改為只認**左** Option（`left_option`），右 Option 不再被攔截，避免與語音輸入等佈局在右 Option 的工具衝突。習慣按右 Option+V 的使用者請改用左 Option，或依 README 把規則改回 `option`。（#1）
- 新增 `-n`/`--no-paste` 與 `--paste-delay-ms <毫秒>` 參數，以及對應的 `DEDENT_PASTE_NO_PASTE`、`DEDENT_PASTE_PASTE_DELAY_MS` 環境變數：可只整理剪貼簿交由 skhd、Hammerspoon 等工具自行貼上，或在送出貼上前加入延遲。（#1）
- macOS 送出 `Command+V` 前會先等待 Shift/Control/Option/Command 全部放開（最多 1 秒），修正快捷鍵觸發時實體 Option 仍按住導致應用程式收到 `Command+Option+V`、`osascript` 回傳成功卻沒有貼上的問題。（#1）
- macOS 新增單一實例保護：同時只允許一個 `dedent-paste` 執行，後啟動者靜默結束，避免會 key-repeat 的快捷鍵工具一次貼上多次。（#1）
- README 補充輔助使用與自動化權限說明、`osascript` exit code 0 不保證貼上成功、第三方快捷鍵工具（skhd）的建議設定，以及如何改用右 Option 或左右皆可。

## 0.4.0

- 新增 Homebrew tap（[doggy8088/homebrew-dedent-paste](https://github.com/doggy8088/homebrew-dedent-paste)），macOS 可透過 `brew install doggy8088/dedent-paste/dedent-paste` 安裝；cargo-dist 發佈流程會自動產生並推送 Homebrew formula，tap 儲存庫以 `homebrew-tap/` 子模組納入本專案。
- 升級 cargo-dist 至 0.32.0，並在 `Cargo.toml` 補上 `description` 與 `homepage`。
- 新增命令列參數：`-h`/`--help` 顯示說明、`-v`/`--version` 顯示版本、`-i`/`--install` 初始化 Karabiner-Elements 設定、`-u`/`--uninstall` 移除 Karabiner-Elements 規則。`--install` 以 `examples/macos/paste-dedent-plain-text.json` 為範本，規則路徑改為目前執行檔的實際安裝路徑（Homebrew `Cellar` 路徑會改寫成不含版本的 `opt` 路徑），並以 `shell_command` 是否包含 `dedent-paste` 判斷既有規則，而非規則名稱；修改前會備份 `karabiner.json`。
- `install.sh` 改為呼叫 `dedent-paste --install` 完成 Karabiner-Elements 設定，不再需要 Python 3；並拒絕在不支援參數的舊版執行檔上執行。
- Homebrew 安裝完成後顯示 caveats，提醒使用者執行 `dedent-paste --install` 設定 Karabiner-Elements（由 tap 儲存庫的 workflow 在 cargo-dist 更新 formula 後自動補上）。
- 範例 `examples/macos/paste-dedent-plain-text.json` 的執行路徑改為 `$HOME/.local/bin/dedent-paste`。
- 修正 `bump-and-release` 腳本在串流模式下的輸出處理，保留 stdout 與 stderr 以便正確辨識 workflow 錯誤訊息（例如缺少 `workflow_dispatch`）。

## 0.3.2

- 支援辨識 `•` 提示／項目符號格式，移除提示符與續行縮排並展開段落中的視覺換行。
- 支援辨識 `▸` 與 `▾` 前綴（如 Copilot CLI 思考區塊），移除前綴與續行縮排並保留原有換行。
- 提示格式換行接合支援保留逗號（`，`、`,`）結尾的換行，視為手動換行而不與下一行接合。
- 新增以 Remotion 製作的產品中文介紹影片專案 intro-video。

## 0.3.1

- 修正 Codex CLI 提示格式（`❯`、`›`）的換行接合：行尾為句末標點（`。！？.!?…`，後方可接右引號或右括號）時，視為使用者實際輸入的換行並保留，不再與下一行接合。
- 修正提示格式偵測：提示符（`❯`、`›`、`>`）後方與續行縮排的空白不再限定 ASCII 空白，也接受不換行空白（U+00A0）、全形空白（U+3000）等 Unicode 空白字元，避免終端機複製出的 NBSP 導致整段文字未被處理。
- `install.sh` 產生的 Karabiner 規則改為先載入 `~/.config/dedent-paste/env` 再執行 `dedent-paste`，讓 Karabiner 觸發時也能讀到 `GEMINI_API_KEY`，圖片轉文字功能不需再手動修改 `karabiner.json`；安裝完成時若環境變數檔不存在，會顯示建立方式的提示。
- GitHub Pages 站台目錄由 `docs/` 改為 `public/`，並限制自動部署只在 `public/**` 變更時觸發，仍保留手動重新部署功能。
- 升級 GitHub Pages workflow 使用 Node.js 24 相容的 actions，避免 runner 的 Node.js 20 淘汰警告。

## 0.3.0

- 新增 Gemini 圖片轉文字功能：剪貼簿沒有文字但有圖片時，自動以 Gemini（預設 `gemini-3.7-flash`）辨識圖片文字為格式化文字，或在圖片沒有文字時產生圖片描述（預設 `zh-TW`），並貼上結果。
- 新增 `DEDENT_PASTE_GEMINI_API_KEY`（備援 `GEMINI_API_KEY`）、`DEDENT_PASTE_GEMINI_MODEL`、`DEDENT_PASTE_LANG`（備援 `LANG`）、`DEDENT_PASTE_GEMINI_SYSTEM_PROMPT`、`DEDENT_PASTE_GEMINI_SYSTEM_PROMPT_FILE`、`DEDENT_PASTE_GEMINI_TIMEOUT_SECS` 與 `DEDENT_PASTE_LOG_FILE` 環境變數。
- 新增記錄檔功能：未設定 API 金鑰或剪貼簿沒有內容時靜默結束並只寫入記錄檔；其他圖片轉文字錯誤會顯示系統通知（macOS 通知中心、Windows 錯誤對話方塊）且不更動剪貼簿。
- 更新 AutoHotkey 範例腳本，移除非零 exit code 的泛用錯誤視窗，避免與程式自身的錯誤對話方塊重複。
- 在 README 與 GitHub Pages 首頁加入 Karabiner-Elements 與 AutoHotkey 的環境變數設定說明（env 檔、`setx`、`EnvSet`），並更新首頁對圖片轉文字功能的描述。
- 強化 `bump-and-release` 的變更紀錄流程，在發佈前依 Git 標籤與提交差異回補缺漏 Release notes，並拒絕空白或占位內容。
- 強化 `bump-and-release` 的 npm 發佈回退機制：未偵測到 Release 事件觸發的工作流程時，僅手動補觸發一次。
- 改善 `workflow_dispatch` 錯誤判斷，並避免 `--publish-npm` 在同一流程中重複觸發 npm 發佈。

## 0.2.6

- 修正 `bump-and-release`，在更新 `Cargo.toml` 後同步 `Cargo.lock`，並將鎖檔納入版本提交。
- 改善 CI、Release 與 npm 工作流程追蹤，使用提交 SHA、分支與事件辨識正確的執行項目，並在執行失敗時中止發佈。
- 以 `CHANGELOG.md` 內容產生 Release notes，並附上 npm 套件頁與指定版本頁連結。
- 偵測由 GitHub Release 事件觸發的 npm 工作流程，避免再次手動發佈相同版本。
- 將工作目錄乾淨度檢查移至版本檔案更新之前，避免正常的鎖檔同步阻斷發佈。
- 同步 Rust crate、Cargo 鎖檔與 npm 套件版本至 `0.2.6`。

## 0.2.5

- 支援移除 Codex CLI 複製內容開頭的 `> ` 前綴與續行縮排，同時保留原始換行。
- 新增專案本地的 `bump-and-release` 技能，支援預設 patch、指定 minor 版本推進、CI、GitHub Release 與 npm trusted publishing 發佈流程。
- 將 npm 發佈預設調整為由 GitHub Release 的 `published` 事件觸發，並保留明確指定時才使用的手動發佈選項，以避免重複發佈。
- 同步 Rust crate 與 npm 套件版本至 `0.2.5`。

## 0.2.4（未發佈）

- Git 紀錄中沒有此版本的獨立版本提交或標籤；版本號在建立 `v0.2.5` 時略過。

## 0.2.3（未發佈）

- Git 紀錄中沒有此版本的獨立版本提交或標籤；版本號在建立 `v0.2.5` 時略過。

## 0.2.2

- 支援移除 Codex CLI 的 `›` 提示符，並保留既有的 `❯` 提示符相容性。
- 自動合併 Codex CLI 同一段落中的單一視覺換行，依 CJK 與拉丁文字邊界決定是否插入空白。
- 保留連續換行所代表的真正段落分隔，並維持 LF 與 CRLF 行尾格式。
- 同步 Rust crate 與 npm 套件版本至 `0.2.2`。

## 0.2.1

- Added support for stripping common terminal prompt prefixes when normalizing pasted text.

## 0.2.0

- Added Windows runtime support with native clipboard handling and simulated `Ctrl+V` paste.
- Added AutoHotkey v1/v2 examples for using `dedent-paste` with `Win+V` on Windows.
- Updated `README.md` with separate macOS and Windows installation, setup, and usage guides.
- Documented the PowerShell Release installer and clarified `PATH` vs fixed-path AutoHotkey setup.
- Updated `docs/index.html` and `DEVELOPMENT.md` to reflect macOS + Windows support.

## 0.1.1

- Simplified `README.md` for end users.
- Added `DEVELOPMENT.md` for build, install, Karabiner, and CI/CD details.
- Added this changelog.
- Updated GitHub Actions to publish cargo-dist release installers and platform archives.
- Updated GitHub Actions to dispatch cargo-dist releases only when `Cargo.toml` package version changes.
- Added GitHub Pages landing page and deployment workflow.
- Optimized the GitHub Pages landing page with small WebP visual assets.

## 0.1.0

- Added `dedent-paste` command-line helper for Karabiner-Elements.
- Added `Option+V` Karabiner-Elements integration.
- Added UTF-8 clipboard handling for Karabiner `shell_command` execution.
- Added one-line installer.
- Added MIT license.
