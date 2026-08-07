---
name: bump-and-release
description: 幫專案自動化「提升版本、更新變更紀錄、觸發 CI、建立 Release、修正 Release notes、並用 trusted publishing 發佈 npm 套件」的工作流程，適用於輸入「$bump-and-release」或「$bump-and-release minor version」等呼叫時。
---

# Bump And Release

## 適用情境

在使用 `package.json` 或 `Cargo.toml` 且以 GitHub Actions 發佈的專案，用一句命令完成版本發佈前置流程。

## 目標

一鍵完成：

- 依預設 `patch` 或指定 `minor` 提升版本
- 更新 `Cargo.toml`、`package.json` 的版本（若檔案存在）
- 將 `CHANGELOG.md` 的 `Unreleased` 內容移到新版本段落
- 觸發 CI workflow
- 透過 `workflow_dispatch` 觸發 `release` workflow 建立 Release
- 依 `CHANGELOG.md` 修正 Release notes
- 透過 trusted publishing 方式發佈 npm（依賴 Release 的 `published` 事件）
- 避免重複發佈：預設不再手動 `workflow_dispatch` 觸發 `npm-publish.yml`

## 指令

- `$bump-and-release`：預設為 `patch` bump
- `$bump-and-release minor version`：`minor` bump

## 腳本入口

### `scripts/bump-and-release.js`

#### 參數

- `--repo <path>`：指定專案根目錄（預設為目前目錄）
- `--ci-workflow <name>`：CI workflow 名稱或檔名（預設 `ci.yml`）
- `--release-workflow <name>`：Release workflow 名稱或檔名（預設 `release.yml`）
- `--release-input <name>`：`release.yml` `workflow_dispatch` 參數名稱（預設 `tag`）
- `--npm-workflow <name>`：npm workflow 名稱或檔名（預設 `npm-publish.yml`）
- `--dry-run`：僅模擬執行，不推播、不觸發 workflow
- `--skip-release`：僅做版本更新與 release notes 搬移
- `--skip-release-notes`：不更新 Release notes
- `--publish-npm`：明確啟用手動觸發 npm 發佈 workflow（預設關閉）
- `--skip-npm`：不觸發手動 npm 發佈 workflow
- `--skip-ci`：不觸發 CI workflow
- `--skip-checks`：略過本機驗證（`cargo fmt/test/build`、`npm test`）
- `--allow-dirty`：允許有未提交變更的工作目錄（不建議）
- `--help`：顯示完整說明

#### 範例

- `node scripts/bump-and-release.js`（預設 `patch`）
- `node scripts/bump-and-release.js minor version`
- `node scripts/bump-and-release.js --publish-npm --skip-release-notes`
- `node scripts/bump-and-release.js --repo /path/to/repo --skip-checks`

腳本會在每個步驟輸出結果，遇到錯誤即以 `exit 1` 結束。

## 參考

請先閱讀：[release-workflow.md](references/release-workflow.md)

* * *

本技能假設目標專案已有可用的 `release.yml` 與 `npm-publish.yml`；若 workflow 名稱或參數名稱不同，請先在腳本參數上調整對應值。
