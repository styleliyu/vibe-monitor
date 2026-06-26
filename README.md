# vibe-monitor

vibe-monitor is a local-first AI development Cockpit for keeping Workspace, Codex Thread, terminal, Git state, and Attention Queue work in one desktop control surface.

## Status

This repository is a public early-stage MVP checkpoint. The current MVP scope is intentionally narrow:

- Workspace management with local persistence
- Codex app-server detection and Thread control surface
- Integrated terminal for the selected Workspace
- Git status, diff, stage, and unstage
- Attention Queue for approvals, blockers, failures, and manual test items

External tools such as GitHub, VS Code, Zed, ChatGPT, WeChat, browser windows, and music apps are future Launcher or Plugin concepts. The MVP does not read private content from those applications.

## Prerequisites

- Windows with Microsoft Edge WebView2 Runtime
- Node.js and npm
- Rust and Cargo through rustup
- Git CLI
- Optional: Codex CLI with `codex app-server` available on `PATH`

On this development host, npm may need a local cache override:

```powershell
$env:npm_config_cache = Join-Path $env:LOCALAPPDATA 'npm-cache-codex'
```

Rust commands may need the cargo path and Visual Studio Build Tools environment:

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
$vs = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools'
& "$vs\Common7\Tools\Launch-VsDevShell.ps1" -Arch amd64 -HostArch amd64 -SkipAutomaticLocation | Out-Null
```

## Development

Install dependencies:

```powershell
npm install
```

Run frontend tests:

```powershell
npm test -- --run
```

Build the frontend:

```powershell
npm run build
```

Run Rust checks:

```powershell
cd src-tauri
cargo test
cargo check
cd ..
```

Start the desktop app:

```powershell
npm run tauri dev
```

## Verification

Use [docs/mvp-verification.md](docs/mvp-verification.md) for the full MVP smoke checklist.

The minimum automated checkpoint before merging MVP work is:

```powershell
npm test -- --run
npm run build
cd src-tauri
cargo test
cargo check
cd ..
```

## Privacy Boundary

vibe-monitor is local-first. Persistent product data is stored under the app data directory in SQLite, and the core app only tracks local Workspace, Codex, terminal, Git, and Attention Queue state.

The MVP may launch local processes such as `git`, `powershell.exe`, and `codex app-server`, but it does not scrape arbitrary desktop windows, browser pages, chat applications, WeChat, or music apps. Future External Tool support should use explicit Launchers or Plugins and keep private-content access opt-in.

## Project Documents

- [Product design document](docs/vibe-monitor-long-task-design.md)
- [MVP implementation plan](docs/superpowers/plans/2026-06-26-vibe-monitor-mvp.md)
- [Context glossary](CONTEXT.md)
- [ADR 0001: Desktop stack](docs/adr/0001-desktop-stack.md)
- [ADR 0002: Codex app-server](docs/adr/0002-codex-app-server.md)
- [ADR 0003: Local-first privacy](docs/adr/0003-local-first-privacy.md)
