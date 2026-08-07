# Bump and Release 工作流程參考

以下為 `scripts/bump-and-release.js` 的行為規格，便於在實際專案中調整。

## 流程總覽

1. 解析參數並檢查工具：
   - 預設 `patch`，可用 `minor` 取代
   - 檢查 `git`、`gh`，若有 `Cargo.toml` 會檢查 `cargo`，若有 `package.json` 會檢查 `npm`
2. 讀取版本來源：
   - 優先從 `package.json` 取版本
   - 若同時存在 `Cargo.toml`，版本必須一致
3. 寫入新版本：
   - 更新 `package.json`（存在時）
   - 更新 `Cargo.toml`（存在且可找到 `[package] version =`）
   - 將 `CHANGELOG.md` 中 `## Unreleased` 的段落移到 `## <version>`，並保留 `## Unreleased` 區段
4. 建立 commit 與 tag，將 tag 命名為 `v<version>`
5. 推播分支與 tag
6. 觸發 CI 與 release workflow
7. 等候對應 release workflow 完成後，修正 GitHub Release notes（使用 `CHANGELOG.md` 的新段落）
8. 觸發 npm 發佈 workflow（預設 `npm-publish.yml`）

## 依賴流程假設

- release workflow 為 `workflow_dispatch` 可手動觸發，並接受 `tag`（或你在參數指定的欄位）作為 release tag。
- npm workflow 可透過 `workflow_dispatch` 觸發。
- npm 發佈流程採用 trusted publishing 時，workflow 內至少需包含：
  - `permissions: id-token: write`
  - `npm publish --provenance` 指令（或同義的可信發佈參數）
- `release-notes` 修正使用 `gh release edit <tag> --notes-file` 完成。

## 常用參數

- `--ci-workflow`：CI workflow 名稱，例如 `ci.yml`
- `--release-workflow`：釋出版 workflow 名稱，例如 `release.yml`
- `--release-input`：release workflow 的輸入欄位，例如 `tag`
- `--npm-workflow`：npm publish workflow 名稱，例如 `npm-publish.yml`

## 重要限制

- 腳本預設會在 push 之前再次做一次乾淨度檢查（`--allow-dirty` 可跳過）
- `--dry-run` 僅模擬檔案變更，不會 push 與觸發任何 workflow
- 若找不到 `CHANGELOG.md`，仍可完成版本 bump，但 release notes 修正會跳過

## 針對 `minor version` 的片語

- 你可用 `minor version` 當作兩個位置參數，腳本會判斷出 `minor` 進行 bump。

