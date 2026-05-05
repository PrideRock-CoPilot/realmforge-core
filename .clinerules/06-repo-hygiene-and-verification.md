# Repository Hygiene and Verification

Keep the workspace clean, keep canonical docs current, and leave enough evidence for the next agent to continue without rediscovery.

## Start-of-Task Hygiene

Before changing any file:

1. Run `git status --short`.
2. Run `git ls-files --others --exclude-standard`.
3. Identify which dirty files are pre-existing and which files the current task will touch.
4. Do not revert, delete, move, or reformat pre-existing user changes unless the user explicitly asks.
5. If untracked generated/local files appear, prefer updating `.gitignore` only when the files are reproducible artifacts, credentials, logs, local caches, or temporary outputs.

If the repo is dirty from prior intended work, continue with it carefully and keep the new change scoped. Report the pre-existing dirty state in the final response.

## Artifact Placement

Use ignored locations for build and review artifacts:

- Rust quality gates: `--target-dir target-quality`
- Review-only builds or experiments: `--target-dir target-review`
- Runtime state, local snapshots, and generated project state: `.realmforge/`
- Logs: `*.log`, `*.err`
- Local credentials or login request fixtures: `login_*.json`

Do not create ad-hoc output files at the repo root. If a tool must emit a report, either use an existing ignored artifact path or add a narrowly scoped ignore rule first.

## Docs Freshness

When behavior, gates, contracts, metadata, or roadmap status changes, update the relevant docs in the same task:

- Roadmap state: `docs/spec/20_IMPLEMENTATION_ROADMAP.md`
- Acceptance tests: `docs/spec/21_ACCEPTANCE_TEST_PLAN.md`
- File registry and metadata: `docs/spec/04_METADATA_STANDARD.md`
- Open decisions and blockers: `docs/spec/22_OPEN_DECISIONS.md`
- Phase gates and current evidence: `docs/plan/Phase*.md`
- Cross-phase index: `docs/plan/README.md`

Do not mark a checkbox complete unless the evidence is reproducible and named. Use exact dates for verification notes. If a gate is only partially proven, leave it unchecked and add a current verification note that states the remaining gap.

## Required Verification

Always run lightweight hygiene checks after edits:

```powershell
git diff --check
git status --short
git ls-files --others --exclude-standard
```

For Rust implementation changes, also run:

```powershell
cargo fmt --all -- --check
cargo check --workspace --target-dir target-quality
cargo clippy --workspace --all-targets --target-dir target-quality -- -D warnings
cargo test --workspace --target-dir target-quality
```

For docs-only or instruction-only changes, `git diff --check` is the minimum required verification. Run broader gates only when the change affects code, contracts, test expectations, phase certification, or generated artifacts.

## Final Report

Every final response must include:

- What changed.
- Which verification commands ran.
- Whether the worktree has only intended changes.
- Any remaining open gate, blocker, or dirty file not caused by the current task.

Never claim the repo is clean unless both tracked and untracked status are checked.
