# Bump and Release 工作流程參考

以下為 `scripts/bump-and-release.js` 的行為規格，便於在實際專案中調整。

## 流程總覽

1. 解析參數並檢查工具：
   - 預設 `patch`，可用 `minor` 取代
   - 檢查 `git`、`gh`，若有 `Cargo.toml` 會檢查 `cargo`，若有 `package.json` 會檢查 `npm`
2. 讀取版本來源：
   - 優先從 `package.json` 取版本
   - 若同時存在 `Cargo.toml`，版本必須一致
3. 稽核並更新 `CHANGELOG.md`：
   - 比對可達的語意化版本 Git 標籤與版本段落
   - 找出缺少版本段落、空白段落及 `尚未填寫`、`TBD`、`TODO` 等占位內容
   - 以各版本的前一標籤至目前標籤區間執行 `git log`，再用 `git show` 驗證實際差異並回補摘要
   - 以最新版本標籤至 `HEAD` 的差異更新 `## Unreleased`
   - 若版本號曾略過且沒有標籤，不得虛構變更；應明確標示未發佈或移除錯誤版本段落
4. 寫入新版本：
   - 更新 `package.json`（存在時）
   - 更新 `Cargo.toml`（存在且可找到 `[package] version =`）
   - 將 `CHANGELOG.md` 中 `## Unreleased` 的段落移到 `## <version>`，並保留 `## Unreleased` 區段
5. 建立 commit 與 tag，將 tag 命名為 `v<version>`
6. 推播分支與 tag
7. 觸發 CI 與 release workflow
8. 等候對應 release workflow 完成後，修正 GitHub Release notes（使用 `CHANGELOG.md` 的新段落）
9. 預設不手動觸發 npm 發佈 workflow；由 `npm-publish.yml` 的 `on: release`（published）負責自動發佈
   - 如需手動觸發，啟用 `--publish-npm`

## CHANGELOG 回補規則

- 先以 `git tag --sort=version:refname` 取得實際版本，不以 `CHANGELOG.md` 既有標題推定版本真的發佈過。
- 使用 `git log --no-merges --format='%H%x09%s' <前一標籤>..<目前標籤>` 列出候選提交。
- 使用 `git show --stat --summary <commit>` 與必要的檔案差異確認實際行為，再整理成使用者可理解的 Release notes。
- 排除只負責版本號、標籤或 Release 建立的純發佈提交，除非該提交同時包含其他實際變更。
- 若 Git 標籤存在但 `CHANGELOG.md` 沒有對應段落，依標籤順序插入版本段落。
- 若段落只有占位文字或完全空白，以該版本區間的真實變更取代。
- 若沒有足夠 Git 證據，停止發佈並明確指出缺漏，不得編造 Release notes。

## 依賴流程假設

- release workflow 為 `workflow_dispatch` 可手動觸發，並接受 `tag`（或你在參數指定的欄位）作為 release tag。
- npm workflow 預設由 `release` 事件自動觸發；在極少數手動補發情境下，可選擇透過參數手動觸發。
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

- 腳本要求 `CHANGELOG.md` 存在，且 `Unreleased` 與所有既有版本段落均包含非占位內容
- 腳本允許只有 `CHANGELOG.md` 預先修改，並會將該變更納入版本提交
- 其他未提交變更仍會阻止發佈；`--allow-dirty` 可明確略過此限制
- `--dry-run` 僅模擬檔案變更，不會 push 與觸發任何 workflow
- `--dry-run` 仍會實際修改版本檔案與 `CHANGELOG.md`，但不會建立提交；驗證後需自行還原測試變更

## 針對 `minor version` 的片語

- 你可用 `minor version` 當作兩個位置參數，腳本會判斷出 `minor` 進行 bump。
