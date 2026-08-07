---
name: bump-and-release
description: 自動稽核並更新 CHANGELOG、依 Git 歷史回補缺漏 Release notes、提升版本、觸發 CI、建立 GitHub Release，並用 trusted publishing 發佈 npm 套件。使用於「$bump-and-release」、「$bump-and-release minor version」或要求建立新版本與發佈套件時。
---

# Bump And Release

## 目標

一鍵完成：

- 依預設 `patch` 或指定 `minor` 提升版本
- 更新 `Cargo.toml`、`package.json` 的版本（若檔案存在）
- 比對 Git 標籤與版本區間提交，回補 `CHANGELOG.md` 缺少、空白或占位的歷史 Release notes
- 將目前尚未發佈的真實變更寫入 `CHANGELOG.md` 的 `Unreleased`
- 將 `CHANGELOG.md` 的 `Unreleased` 內容移到新版本段落
- 觸發 CI workflow
- 透過 `workflow_dispatch` 觸發 `release` workflow 建立 Release
- 依 `CHANGELOG.md` 修正 Release notes
- 透過 trusted publishing 方式發佈 npm（依賴 Release 的 `published` 事件）
- 避免重複發佈：預設不再手動 `workflow_dispatch` 觸發 `npm-publish.yml`

## 必要流程

1. 先讀取目標專案的 `AGENTS.md` 與提交規範。
2. 執行 `git status`、`git tag --sort=version:refname`，並比對 `CHANGELOG.md` 的版本段落。
3. 將下列情況視為缺漏：已發佈標籤沒有版本段落、版本段落為空、內容只有 `尚未填寫`、`TBD`、`TODO` 等占位文字。
4. 針對缺漏版本，以 `git log <前一標籤>..<版本標籤>` 找出候選提交，再用 `git show` 驗證實際差異後撰寫摘要。不得只依提交標題臆測，也不得把變更歸入未建立標籤的版本。
5. 以最新版本標籤至 `HEAD` 的實際差異更新 `Unreleased`。若沒有可驗證變更，停止發佈，不得產生占位 Release notes。
6. 執行腳本。腳本允許只有 `CHANGELOG.md` 預先修改並將其納入版本提交；其他未提交檔案仍會阻止發佈，除非明確使用 `--allow-dirty`。
7. 確認新版本段落、GitHub Release notes 與 npm 版本資訊一致。

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

腳本會在改動版本檔案前稽核 `CHANGELOG.md`。若檔案不存在、`Unreleased` 無實際內容、歷史版本有空白或占位 Release notes，或 Git 版本標籤缺少對應段落，腳本會停止；先依上述必要流程回補後再執行。

## 參考

請先閱讀：[release-workflow.md](references/release-workflow.md)

* * *

本技能假設目標專案已有可用的 `release.yml` 與 `npm-publish.yml`；若 workflow 名稱或參數名稱不同，請先在腳本參數上調整對應值。
