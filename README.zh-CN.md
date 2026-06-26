# vibe-monitor

[English README](README.md)

vibe-monitor 是一个本地优先的 AI 开发 Cockpit，用来把 Workspace、Codex Thread、终端、Git 状态和 Attention Queue 放到同一个桌面主控面板里。

## 当前状态

本项目正在开发中。这是一个公开的早期 MVP 检查点，当前范围刻意保持收敛：

- Workspace 管理和本地持久化
- Codex app-server 检测和 Codex Thread 控制面板
- 面向当前 Workspace 的集成终端
- Git 状态、diff、stage、unstage
- 用于审批、阻塞、失败和手动测试项的 Attention Queue

GitHub、VS Code、Zed、ChatGPT、微信、浏览器窗口、音乐软件等外部工具属于后续 Launcher 或 Plugin 方向。MVP 不读取这些应用里的私人内容。

## 环境要求

- Windows 和 Microsoft Edge WebView2 Runtime
- Node.js 与 npm
- 通过 rustup 安装的 Rust 与 Cargo
- Git CLI
- 可选：`PATH` 中存在支持 `codex app-server` 的 Codex CLI

在当前开发主机上，npm 可能需要指定本地缓存：

```powershell
$env:npm_config_cache = Join-Path $env:LOCALAPPDATA 'npm-cache-codex'
```

Rust 命令可能需要加载 cargo 路径和 Visual Studio Build Tools 环境：

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
$vs = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools'
& "$vs\Common7\Tools\Launch-VsDevShell.ps1" -Arch amd64 -HostArch amd64 -SkipAutomaticLocation | Out-Null
```

## 开发命令

安装依赖：

```powershell
npm install
```

启动桌面应用：

```powershell
npm run tauri dev
```

生成可直接双击运行的 release exe：

```powershell
npm run tauri:build:exe
```

生成后打开：

```text
src-tauri\target\release\vibe-monitor.exe
```

注意：不要用 `cargo build --release` 生成可分发 exe。它不会走 Tauri 前端资源嵌入流程，直接双击可能会访问 `localhost:1420` 并显示连接失败。

运行前端测试：

```powershell
npm test -- --run
```

构建前端：

```powershell
npm run build
```

运行 Rust 测试和检查：

```powershell
cd src-tauri
cargo test
cargo check
cd ..
```

## 隐私边界

vibe-monitor 是本地优先应用。产品持久化数据保存在应用数据目录下的 SQLite 数据库中，核心应用只跟踪本地 Workspace、Codex、Terminal、Git 和 Attention Queue 状态。

MVP 可能会启动 `git`、`powershell.exe`、`codex app-server` 等本地进程，但不会抓取任意桌面窗口、浏览器页面、聊天应用、微信或音乐软件内容。后续 External Tool 支持应通过明确的 Launcher 或 Plugin 实现，并保持私人内容读取为显式 opt-in。

## 项目文档

- [产品设计文档](docs/vibe-monitor-long-task-design.md)
- [MVP 实施计划](docs/superpowers/plans/2026-06-26-vibe-monitor-mvp.md)
- [术语表](CONTEXT.md)
- [ADR 0001: 桌面技术栈](docs/adr/0001-desktop-stack.md)
- [ADR 0002: Codex app-server](docs/adr/0002-codex-app-server.md)
- [ADR 0003: 本地优先隐私边界](docs/adr/0003-local-first-privacy.md)
