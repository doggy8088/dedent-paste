# Changelog

All notable changes to this project are documented in this file.

## Unreleased

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
