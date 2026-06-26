# MVP Verification

Use this checklist before treating the public MVP as ready for the next implementation phase. Automated checks prove the code compiles and tests pass; manual scenarios prove the desktop workflow is usable.

## Automated Checks

Run from the repository root:

```powershell
npm test -- --run
npm run build
cd src-tauri
cargo test
cargo check
cd ..
```

Expected result:

- Frontend tests pass.
- Vite and TypeScript build pass.
- Rust unit and integration tests pass.
- Rust `cargo check` passes.

## Desktop Smoke

Start the app:

```powershell
npm run tauri dev
```

Expected result:

- The app opens as `vibe-monitor`.
- The Cockpit layout is stable.
- Workspace list loads.
- Attention Queue loads.
- Terminal panel can open for a selected Workspace.
- Git panel loads for Git Workspaces.
- Codex shows either a clear unavailable state or a connected state.

## Scenario 1: Workspace

1. Add a local Git Workspace, such as `D:\AIdeas\CodingMonitor`.
2. Confirm it appears in the Workspace sidebar.
3. Restart the app.
4. Confirm the Workspace still appears.
5. Remove the Workspace.
6. Confirm it no longer appears after another restart.

## Scenario 2: Attention Queue

1. Select a Workspace.
2. Create the MVP sample failed Attention Item.
3. Confirm it appears in the Attention Queue.
4. Resolve it.
5. Restart the app.
6. Confirm the resolved item stays hidden.

## Scenario 3: Terminal

1. Select a Workspace.
2. Open the terminal panel.
3. Run `git status`.
4. Run `node --version`.
5. Resize the app window.
6. Confirm terminal output remains readable and input still works.
7. Close the app.

## Scenario 4: Git

1. Select a Git Workspace.
2. Modify a tracked file.
3. Confirm the Git panel shows the changed file.
4. Select the file and confirm the diff renders.
5. Stage the file.
6. Confirm the status updates.
7. Unstage the file.
8. Confirm the status updates again.

## Scenario 5: Codex

1. Select a Workspace.
2. Confirm Codex CLI detection runs.
3. If Codex is unavailable, confirm the UI explains the unavailable state without freezing.
4. If Codex is available, list or start a Codex Thread.
5. Send a prompt.
6. Interrupt a running turn.
7. If Codex requests approval, confirm an approval Attention Item appears.

## Process Cleanup

After closing the app, run:

```powershell
Get-Process | Where-Object { $_.ProcessName -match 'codex|powershell|pwsh' } | Select-Object ProcessName,Id,StartTime
```

Expected result:

- No vibe-monitor-owned `codex app-server` process remains.
- No vibe-monitor-owned terminal child process remains.
- Existing unrelated user shells are acceptable.

## Known MVP Limits

- Codex app-server behavior depends on a runnable local Codex CLI. On hosts where the Codex binary exits with access errors, the UI should remain usable and show Codex as unavailable.
- Git diff currently focuses on unstaged file diffs. Staged-only diffs may show as empty in the MVP UI.
- External Tool Launchers and Plugins are outside this MVP checkpoint.

## Verification Record

2026-06-26 Task 10 checkpoint:

- `npm test -- --run`: passed, 7 test files and 18 tests.
- `npm run build`: passed; Vite reported the existing large chunk warning.
- `cargo test`: passed, including backend integration tests for Workspace, Attention, Codex, Terminal, and Git.
- `cargo check`: passed.
- `npm run tauri dev`: launched Vite and started `target\debug\vibe-monitor.exe`; the smoke process was stopped afterward.
- Process cleanup check: no `vibe-monitor.exe`, `tauri dev`, or vibe-monitor-owned `codex app-server` process remained. Existing Codex host processes and unrelated user shells are acceptable.
