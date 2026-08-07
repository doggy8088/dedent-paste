#!/usr/bin/env node

"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

const DEFAULT_CI_WORKFLOW = "ci.yml";
const DEFAULT_RELEASE_WORKFLOW = "release.yml";
const DEFAULT_RELEASE_INPUT = "tag";
const DEFAULT_NPM_WORKFLOW = "npm-publish.yml";
const WAIT_MAX_ROUNDS = 40;
const WAIT_MS = 8000;

const args = parseArgs(process.argv.slice(2));

main().catch((error) => {
  console.error(`\n[error] ${error.message}`);
  process.exit(1);
});

async function main() {
  const repoRoot = path.resolve(args.repo);
  validateRepo(repoRoot);
  info(`專案路徑：${repoRoot}`);
  validateTools(repoRoot);

  const currentVersion = getCurrentVersion(repoRoot);
  const nextVersion = bumpVersion(currentVersion.version, args.bump);
  info(`版本推進：${currentVersion.version} -> ${nextVersion.version}`);
  info(`來源：${currentVersion.source}`);
  if (currentVersion.packageName) {
    info(`npm 套件名稱：${currentVersion.packageName}`);
  }

  requireCleanTree(repoRoot, args.allowDirty);

  const changedPaths = [];
  if (currentVersion.packageJsonPath) {
    updatePackageVersion(currentVersion.packageJsonPath, nextVersion.version);
    changedPaths.push(relative(repoRoot, currentVersion.packageJsonPath));
  }
  if (currentVersion.cargoTomlPath) {
    updateCargoVersion(currentVersion.cargoTomlPath, nextVersion.version);
    changedPaths.push(relative(repoRoot, currentVersion.cargoTomlPath));
  }
  if (currentVersion.cargoTomlPath) {
    if (syncCargoLock(repoRoot)) {
      changedPaths.push("Cargo.lock");
    }
  }

  const releaseNotesFromChangelog = updateChangelog(repoRoot, nextVersion.version);
  if (releaseNotesFromChangelog) {
    changedPaths.push("CHANGELOG.md");
  }

  if (args.dryRun) {
    info("dry-run 啟用：不會建立 commit/tag 或觸發 workflow");
    info(`預計變更檔案：${changedPaths.join(", ") || "（無變更）"}`);
    return;
  }

  const branch = getBranch(repoRoot);
  info(`目前分支：${branch}`);
  const headSha = getHeadSha(repoRoot);

  if (!args.skipChecks) {
    runChecks(repoRoot);
  }

  gitCommit(repoRoot, nextVersion.tag, changedPaths);

  pushBranchAndTag(repoRoot, branch, nextVersion.tag);

  const startTs = Date.now();

  if (!args.skipRelease) {
    if (!args.skipCi) {
      info(`等待 CI 流程結果（head ${headSha}）`);
      try {
        triggerWorkflow(args.ciWorkflow, branch);
      } catch (error) {
        if (isWorkflowDispatchError(error.message)) {
          warn(`CI workflow (${args.ciWorkflow}) 無 workflow_dispatch，將改以 Push 觸發驗證。`);
        } else {
          throw error;
        }
      }
      const ciRun = await waitForWorkflowRun(
        args.ciWorkflow,
        branch,
        headSha,
        startTs,
        repoRoot,
      );
      if (ciRun) {
        if (ciRun.status !== "completed") {
          throw new Error(`CI workflow (${args.ciWorkflow}) 未完成（目前 ${ciRun.status}）。`);
        }
        if (ciRun.conclusion !== "success") {
          throw new Error(`CI workflow (${args.ciWorkflow}) 未通過：${ciRun.conclusion}`);
        }
        info(`CI workflow 已通過：${ciRun.url || "（無 URL）"}`);
      } else {
        warn("未即時抓到 CI workflow 結果，將不保證通過狀態。");
      }
    }
    triggerWorkflow(
      args.releaseWorkflow,
      branch,
      {
        [args.releaseInput]: nextVersion.tag,
      },
    );
    const releaseRun = await waitForWorkflowRun(
      args.releaseWorkflow,
      branch,
      null,
      startTs,
      repoRoot,
    );
    if (releaseRun) {
      if (releaseRun.status !== "completed") {
        throw new Error(`Release workflow (${args.releaseWorkflow}) 未完成（目前 ${releaseRun.status}）。`);
      }
      if (releaseRun.conclusion !== "success") {
        throw new Error(`Release workflow (${args.releaseWorkflow}) 未通過：${releaseRun.conclusion}`);
      }
      info(`Release workflow 已完成：${releaseRun.url || "（無 URL）"}`);
    } else {
      warn("未即時抓到 Release workflow 結果，將直接嘗試修正 release notes。");
    }

    if (!args.skipReleaseNotes) {
        if (!releaseNotesFromChangelog) {
          warn("CHANGELOG.md 無法取得 release notes，將略過 release notes 修正。");
        } else {
          const publishedTag = await waitForRelease(nextVersion.tag);
          const packageName = currentVersion.packageName || "dedent-paste";
          if (publishedTag) {
            const finalNotes = buildReleaseNotes(
              nextVersion.version,
              packageName,
              releaseNotesFromChangelog,
            );
            updateReleaseNotes(nextVersion.tag, finalNotes);
            const npmRun = await waitForReleaseTriggeredWorkflow(
              args.npmWorkflow,
              startTs,
              repoRoot,
            );
            if (npmRun) {
              if (npmRun.status === "completed" && npmRun.conclusion === "success") {
                info(`npm workflow 已完成：${npmRun.url || "（無 URL）"}`);
              } else if (npmRun.status === "completed") {
                warn(`npm workflow 未通過：${npmRun.conclusion}。`);
              } else {
                warn(`npm workflow 尚未完成（目前 ${npmRun.status}）。`);
              }
            } else {
              warn("未及時抓到 npm workflow 結果，請從 GitHub Actions 確認發佈狀態。");
            }
            info("Release notes 已更新。");
          } else {
            warn(`release ${nextVersion.tag} 尚未建立，已略過 release notes 修正。`);
          }
        }
    }
  } else {
    info("skip-release 已啟用，跳過 workflow 觸發。");
  }

  if (!args.skipRelease && args.publishNpm && !args.skipNpm) {
    validateTrustedNpmWorkflow(repoRoot, args.npmWorkflow);
    triggerWorkflow(args.npmWorkflow, branch);
    const npmRun = await waitForLatestRun(args.npmWorkflow, branch, startTs, repoRoot);
    if (npmRun) {
      info(`npm workflow 已完成：${npmRun.url || "（無 URL）"}`);
    } else {
      warn("未即時抓到 npm workflow 結果，請從 GitHub Actions 確認發佈狀態。");
    }
  } else if (args.skipNpm) {
    info("skip-npm 已啟用，跳過手動 npm 發佈流程。");
  } else if (!args.publishNpm) {
    info("已關閉手動發佈（預設），將依靠 release 事件自動觸發 npm 發佈。");
  }
}

function parseArgs(argv) {
  const parsed = {
    bump: "patch",
    repo: process.cwd(),
    ciWorkflow: DEFAULT_CI_WORKFLOW,
    releaseWorkflow: DEFAULT_RELEASE_WORKFLOW,
    releaseInput: DEFAULT_RELEASE_INPUT,
    npmWorkflow: DEFAULT_NPM_WORKFLOW,
    dryRun: false,
    skipRelease: false,
    skipReleaseNotes: false,
    publishNpm: false,
    skipNpm: true,
    skipChecks: false,
    skipCi: false,
    allowDirty: false,
  };

  const positions = [];

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (!arg.startsWith("-")) {
      positions.push(arg);
      continue;
    }
    if (arg === "--repo") {
      if (!argv[i + 1] || String(argv[i + 1]).startsWith("-")) {
        throw new Error(`--repo 需要指定路徑`);
      }
      parsed.repo = argv[i + 1];
      i += 1;
      continue;
    }
    if (arg === "--ci-workflow") {
      if (!argv[i + 1] || String(argv[i + 1]).startsWith("-")) {
        throw new Error(`--ci-workflow 需要指定 workflow 名稱`);
      }
      parsed.ciWorkflow = argv[i + 1];
      i += 1;
      continue;
    }
    if (arg === "--release-workflow") {
      if (!argv[i + 1] || String(argv[i + 1]).startsWith("-")) {
        throw new Error(`--release-workflow 需要指定 workflow 名稱`);
      }
      parsed.releaseWorkflow = argv[i + 1];
      i += 1;
      continue;
    }
    if (arg === "--release-input") {
      if (!argv[i + 1] || String(argv[i + 1]).startsWith("-")) {
        throw new Error(`--release-input 需要指定 input 名稱`);
      }
      parsed.releaseInput = argv[i + 1];
      i += 1;
      continue;
    }
    if (arg === "--npm-workflow") {
      if (!argv[i + 1] || String(argv[i + 1]).startsWith("-")) {
        throw new Error(`--npm-workflow 需要指定 workflow 名稱`);
      }
      parsed.npmWorkflow = argv[i + 1];
      i += 1;
      continue;
    }
    if (arg === "--dry-run") parsed.dryRun = true;
    else if (arg === "--skip-release") parsed.skipRelease = true;
    else if (arg === "--skip-release-notes") parsed.skipReleaseNotes = true;
    else if (arg === "--publish-npm") parsed.publishNpm = true;
    else if (arg === "--skip-npm") parsed.skipNpm = true;
    else if (arg === "--skip-checks") parsed.skipChecks = true;
    else if (arg === "--skip-ci") parsed.skipCi = true;
    else if (arg === "--allow-dirty") parsed.allowDirty = true;
    else if (arg === "--help") usage();
    else {
      throw new Error(`未識別參數：${arg}`);
    }
  }

  if (positions.includes("major")) parsed.bump = "major";
  else if (positions.includes("minor")) parsed.bump = "minor";
  else if (positions.includes("patch")) parsed.bump = "patch";

  return parsed;
}

function usage() {
  const text = `
用途：
  node scripts/bump-and-release.js [minor|patch]
  node scripts/bump-and-release.js minor version

預設行為：
  當未指定版本類型時預設為 patch

參數：
  --repo <path>             指定專案根目錄
  --ci-workflow <name>      CI workflow 名稱（預設 ci.yml）
  --release-workflow <name>  release workflow 名稱（預設 release.yml）
  --release-input <name>     release workflow 的 input 名稱（預設 tag）
  --npm-workflow <name>      npm workflow 名稱（預設 npm-publish.yml）
  --dry-run                 僅模擬，不推送與觸發 workflow
  --skip-release            只做版本更新與 CHANGELOG 處理
  --skip-release-notes      不修正 release notes
  --publish-npm             明確啟用手動觸發 npm 發佈 workflow（預設關閉）
  --skip-npm                跳過手動 npm 發佈 workflow
  --skip-ci                 跳過 CI workflow 觸發
  --skip-checks             跳過本機檢查
  --allow-dirty             允許未提交變更
`;
  console.log(text.trim());
  process.exit(0);
}

function validateRepo(repoRoot) {
  const gitDir = run("git", ["rev-parse", "--git-dir"], { cwd: repoRoot, silent: true });
  if (!gitDir) {
    throw new Error(`找不到 Git repo：${repoRoot}`);
  }
}

function validateTools(repoRoot) {
  requireCommand("git", repoRoot);
  requireCommand("gh", repoRoot);
  if (fs.existsSync(path.join(repoRoot, "Cargo.toml"))) {
    requireCommand("cargo", repoRoot);
  }
  if (fs.existsSync(path.join(repoRoot, "package.json"))) {
    requireCommand("npm", repoRoot);
  }
}

function runChecks(repoRoot) {
  if (fs.existsSync(path.join(repoRoot, "Cargo.toml"))) {
    run("cargo", ["fmt", "--check"], { cwd: repoRoot, stream: true, label: "cargo fmt" });
    run("cargo", ["test", "--locked"], { cwd: repoRoot, stream: true, label: "cargo test" });
    run("cargo", ["build", "--locked"], { cwd: repoRoot, stream: true, label: "cargo build" });
  }
  if (fs.existsSync(path.join(repoRoot, "package.json"))) {
    run("npm", ["test"], { cwd: repoRoot, stream: true, label: "npm test" });
  }
}

function getCurrentVersion(repoRoot) {
  const result = {
    source: "",
    version: "",
    packageName: "",
    packageJsonPath: null,
    cargoTomlPath: null,
  };

  const packagePath = path.join(repoRoot, "package.json");
  if (fs.existsSync(packagePath)) {
    const pkg = JSON.parse(fs.readFileSync(packagePath, "utf8"));
    if (pkg.version) {
      result.source = "package.json";
      result.version = pkg.version;
      if (typeof pkg.name === "string") {
        result.packageName = pkg.name;
      }
      result.packageJsonPath = packagePath;
    }
  }

  const cargoPath = path.join(repoRoot, "Cargo.toml");
  if (fs.existsSync(cargoPath)) {
    const cargoVersion = extractCargoVersion(cargoPath);
    if (cargoVersion) {
      if (!result.version) {
        result.source = "Cargo.toml";
        result.version = cargoVersion;
      } else if (result.version !== cargoVersion) {
        throw new Error(
          `package.json(${result.version}) 與 Cargo.toml(${cargoVersion}) 版本不同，請先同步版本。`,
        );
      }
      result.cargoTomlPath = cargoPath;
    }
  }

  if (!result.version) {
    throw new Error("找不到可用版本來源（需有 package.json 或 Cargo.toml）。");
  }
  return result;
}

function extractCargoVersion(filePath) {
  const lines = fs.readFileSync(filePath, "utf8").split(/\r?\n/);
  let inPackage = false;
  for (const line of lines) {
    if (/^\s*\[package\]\s*$/.test(line)) {
      inPackage = true;
      continue;
    }
    if (inPackage && /^\s*\[/.test(line)) {
      inPackage = false;
    }
    if (!inPackage) {
      continue;
    }
    const match = line.match(/^\s*version\s*=\s*"([^"]+)"\s*$/);
    if (match) return match[1];
  }
  return "";
}

function bumpVersion(current, type) {
  const parts = current.split(".").map((part) => parseInt(part, 10));
  if (parts.length < 3 || parts.some((part) => Number.isNaN(part))) {
    throw new Error(`無法解析版本號：${current}`);
  }
  const next = [...parts];
  if (type === "major") {
    next[0] += 1;
    next[1] = 0;
    next[2] = 0;
  } else if (type === "minor") {
    next[1] += 1;
    next[2] = 0;
  } else if (type === "patch") {
    next[2] += 1;
  } else {
    throw new Error(`不支援 bump 類型：${type}`);
  }
  const version = `${next[0]}.${next[1]}.${next[2]}`;
  return {
    version,
    tag: `v${version}`,
  };
}

function updatePackageVersion(filePath, nextVersion) {
  const data = JSON.parse(fs.readFileSync(filePath, "utf8"));
  if (typeof data.version !== "string") {
    throw new Error(`package.json 缺少 version 欄位：${filePath}`);
  }
  data.version = nextVersion;
  fs.writeFileSync(
    filePath,
    `${JSON.stringify(data, null, 2)}\n`,
    "utf8",
  );
}

function updateCargoVersion(filePath, nextVersion) {
  const lines = fs.readFileSync(filePath, "utf8").split(/\r?\n/);
  let inPackage = false;
  let changed = false;

  for (let i = 0; i < lines.length; i += 1) {
    const line = lines[i];
    if (/^\s*\[package\]\s*$/.test(line)) {
      inPackage = true;
      continue;
    }
    if (inPackage && /^\s*\[/.test(line)) {
      inPackage = false;
    }
    if (!inPackage) continue;
    const match = line.match(/^(\s*version\s*=\s*)".*?"(\s*)$/);
    if (match) {
      lines[i] = `${match[1]}"${nextVersion}"${match[2]}`;
      changed = true;
      break;
    }
  }
  if (!changed) {
    throw new Error(`找不到 [package] 版本設定：${filePath}`);
  }
  fs.writeFileSync(filePath, `${lines.join("\n")}\n`, "utf8");
}

function syncCargoLock(repoRoot) {
  const lockPath = path.join(repoRoot, "Cargo.lock");
  if (!fs.existsSync(lockPath)) {
    return false;
  }

  const before = fs.readFileSync(lockPath, "utf8");
  run("cargo", ["generate-lockfile", "--offline"], { cwd: repoRoot, stream: true, label: "cargo generate-lockfile" });
  const after = fs.readFileSync(lockPath, "utf8");
  return before !== after;
}

function updateChangelog(repoRoot, version) {
  const changelogPath = path.join(repoRoot, "CHANGELOG.md");
  if (!fs.existsSync(changelogPath)) {
    warn("找不到 CHANGELOG.md，跳過 notes 移轉。");
    return "";
  }

  const text = fs.readFileSync(changelogPath, "utf8");
  const lines = text.split(/\r?\n/);
  const unreleasedIndex = lines.findIndex((line) => line.trim() === "## Unreleased");
  if (unreleasedIndex === -1) {
    throw new Error("CHANGELOG.md 找不到 ## Unreleased 區段");
  }

  let nextIndex = unreleasedIndex + 1;
  while (nextIndex < lines.length && !/^##\s+/.test(lines[nextIndex])) {
    nextIndex += 1;
  }

  const unreleasedRaw = lines
    .slice(unreleasedIndex + 1, nextIndex)
    .join("\n")
    .replace(/^\s*[\r\n]+|[\r\n]+\s*$/g, "");
  const releaseNotes = unreleasedRaw || "- 尚未填寫";
  const remainder = lines.slice(nextIndex);

  const nextLines = [];
  nextLines.push(...lines.slice(0, unreleasedIndex + 1));
  nextLines.push("");
  nextLines.push(`## ${version}`);
  nextLines.push("");
  nextLines.push(...releaseNotes.split("\n"));
  nextLines.push("");
  nextLines.push(...remainder);

  let normalized = nextLines.join("\n");
  normalized = normalized.replace(/\n{3,}/g, "\n\n\n");

  fs.writeFileSync(changelogPath, `${normalized.trimEnd()}\n`, "utf8");
  return releaseNotes;
}

function getBranch(repoRoot) {
  const branch = run("git", ["rev-parse", "--abbrev-ref", "HEAD"], { cwd: repoRoot, silent: true });
  if (!branch || branch === "HEAD") {
    throw new Error("目前為 detached HEAD，請先 checkout 到分支後再執行。");
  }
  return branch;
}

function getHeadSha(repoRoot) {
  return run("git", ["rev-parse", "HEAD"], { cwd: repoRoot, silent: true });
}

function requireCleanTree(repoRoot, allowDirty) {
  const status = run("git", ["status", "--porcelain"], { cwd: repoRoot, silent: true });
  if (status) {
    if (allowDirty) {
      warn("allow-dirty 啟用，將繼續，但有未提交變更。");
      return;
    }
    throw new Error("工作目錄有未提交變更，請先提交後再執行（或加 --allow-dirty）。");
  }
}

function gitCommit(repoRoot, tag, changedPaths) {
  const files = Array.from(new Set(changedPaths.filter(Boolean)));
  if (files.length === 0) {
    throw new Error("沒有可提交的檔案變更。");
  }
  run("git", ["add", ...files], { cwd: repoRoot, label: "git add" });
  const changed = run(
    "git",
    ["diff", "--cached", "--name-only"],
    { cwd: repoRoot, silent: true },
  );
  if (!changed) {
    throw new Error("版本更新未改變內容，請確認腳本邏輯。");
  }
  run(
    "git",
    ["commit", "-m", `chore: release ${tag}`],
    { cwd: repoRoot, stream: true, label: "git commit" },
  );
  run("git", ["tag", tag], { cwd: repoRoot, label: "git tag" });
}

function pushBranchAndTag(repoRoot, branch, tag) {
  run("git", ["push", "origin", branch], { cwd: repoRoot, label: "git push branch" });
  run("git", ["push", "origin", tag], { cwd: repoRoot, label: "git push tag" });
}

function triggerWorkflow(workflow, ref, inputs = {}) {
  const args = ["workflow", "run", workflow, "--ref", ref];
  Object.keys(inputs).forEach((key) => {
    args.push("-f", `${key}=${inputs[key]}`);
  });
  run("gh", args, { stream: true, label: `gh workflow run ${workflow}` });
}

async function waitForLatestRun(workflow, branch, since, repoRoot) {
  return waitForWorkflowRun(workflow, branch, null, since, repoRoot);
}

async function waitForRelease(tag) {
  for (let i = 0; i < WAIT_MAX_ROUNDS; i += 1) {
    try {
      const result = run(
        "gh",
        ["release", "view", tag, "--json", "tagName,body,url"],
        { silent: true },
      );
      if (result) {
        const data = JSON.parse(result);
        if (data.tagName === tag) return data;
      }
    } catch {
      // no-op: wait
    }
    await delay(WAIT_MS);
  }
  return null;
}

function updateReleaseNotes(tag, releaseNotes) {
  const tmp = path.join(process.cwd(), `.tmp-release-notes-${tag}.md`);
  try {
    fs.writeFileSync(tmp, `${releaseNotes.trimEnd()}\n`, "utf8");
    run(
      "gh",
      ["release", "edit", tag, "--notes-file", tmp],
      { stream: true, label: "gh release edit" },
    );
  } finally {
    if (fs.existsSync(tmp)) {
      fs.unlinkSync(tmp);
    }
  }
}

function buildReleaseNotes(version, packageName, changelogNotes) {
  const npmPackage = packageName || "dedent-paste";
  const npmDisplay = npmPackage;
  const npmUrlName = encodeURIComponent(npmPackage);
  const npmPackageUrl = `https://www.npmjs.com/package/${npmUrlName}`;
  const npmVersionUrl = `https://www.npmjs.com/package/${npmUrlName}/v/${version}`;

  const safeNotes = (changelogNotes || "").trim() || "- 尚未填寫";
  const notes = safeNotes
    .split("\n")
    .map((line) => line.trimEnd())
    .join("\n");

  return [
    `## 變更內容`,
    "",
    notes,
    "",
    `## npm 套件發佈資訊`,
    "",
    `- 套件：${npmDisplay}`,
    `- npm 套件頁面：${npmPackageUrl}`,
    `- 本次 npm 版本：${npmVersionUrl}`,
    `- 安裝方式：\`npm install ${npmPackage}@${version}\``,
  ].join("\n");
}

function requireCommand(command) {
  const cmd = process.platform === "win32" ? "where" : "which";
  try {
    const which = run(cmd, [command], { silent: true });
    if (!which) {
      throw new Error();
    }
  } catch {
    throw new Error(`缺少指令：${command}`);
  }
}

function validateTrustedNpmWorkflow(repoRoot, workflowName) {
  if (!workflowName.endsWith(".yml") && !workflowName.endsWith(".yaml")) {
    return;
  }
  const workflowPath = path.join(repoRoot, ".github", "workflows", workflowName);
  if (!fs.existsSync(workflowPath)) {
    return;
  }
  const text = fs.readFileSync(workflowPath, "utf8");
  const hasToken = text.includes("id-token: write");
  const hasProvenance = text.includes("--provenance");
  if (!hasToken || !hasProvenance) {
    warn(`npm workflow (${workflowName}) 目前未明確檢測到 trusted publishing 條件（id-token: write 與 npm publish --provenance）。`);
  }
}

function run(cmd, args = [], options = {}) {
  const {
    cwd = process.cwd(),
    silent = false,
    stream = false,
    label = cmd,
  } = options;

  if (stream) {
    info(`執行：${label}`);
  }
  const res = spawnSync(cmd, args, {
    cwd,
    encoding: "utf8",
    env: process.env,
    stdio: stream ? "inherit" : "pipe",
  });
  if (res.error) {
    throw new Error(`${label} 執行失敗：${res.error.message}`);
  }
  if (res.status !== 0) {
    const err = (res.stderr || "").toString().trim();
    const out = (res.stdout || "").toString().trim();
    const detail = [err, out].filter(Boolean).join("\n");
    throw new Error(`${label} 回傳錯誤 (${res.status})${detail ? `: ${detail}` : ""}`);
  }
  return (res.stdout || "").toString().trim();
}

async function waitForWorkflowRun(
  workflow,
  branch,
  headSha,
  since,
  repoRoot,
  opts = {},
) {
  for (let i = 0; i < WAIT_MAX_ROUNDS; i += 1) {
    const result = run(
      "gh",
      [
        "run",
        "list",
        "--workflow",
        workflow,
        "--limit",
        "20",
        "--json",
        "databaseId,status,conclusion,url,createdAt,headBranch,headSha,event",
      ],
      { cwd: repoRoot, silent: true },
    );
    if (result) {
      const runs = JSON.parse(result);
      const picked = runs
        .filter((r) => {
          if (branch && r.headBranch && r.headBranch !== branch) return false;
          if (opts.event && r.event !== opts.event) return false;
          if (headSha && r.headSha && r.headSha !== headSha) return false;
          if (since !== undefined && since !== null) {
            return Date.parse(r.createdAt) >= since - 60_000;
          }
          return true;
        })
        .sort((a, b) => Date.parse(b.createdAt) - Date.parse(a.createdAt))[0];

      if (picked) {
        if (picked.status === "completed") {
          return picked;
        }
        info(`workflow 仍在執行：${picked.status} ${picked.url || ""}`);
      }
    }
    await delay(WAIT_MS);
  }
  return null;
}

function isWorkflowDispatchError(message) {
  return String(message).includes("does not have 'workflow_dispatch' trigger");
}

function relative(base, target) {
  return target.startsWith(base) ? target.slice(base.length + 1) : target;
}

function info(msg) {
  console.log(`[info] ${msg}`);
}

function warn(msg) {
  console.warn(`[warn] ${msg}`);
}

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function waitForReleaseTriggeredWorkflow(workflow, since, repoRoot) {
  return waitForWorkflowRun(workflow, null, null, since, repoRoot, { event: "release" });
}
