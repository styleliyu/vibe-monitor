# vibe-monitor

[English README](README.md)

vibe-monitor 是一个本地优先的 AI 开发 Cockpit，用来把 Workspace、Codex Thread、终端、Git 状态和 Attention Queue 放到同一个桌面主控面板里。

## 当前状态

这是一个公开的早期 MVP 检查点。当前范围刻意保持收敛：

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

## 自测方式

推荐先跑自动化检查，再启动桌面应用做手动 smoke。

自动化检查：

```powershell
npm test -- --run
npm run build
cd src-tauri
cargo test
cargo check
cd ..
```

预期结果：

- 前端测试通过。
- TypeScript 和 Vite 构建通过。
- Rust 单元测试和集成测试通过。
- `cargo check` 通过。
- `npm run build` 当前可能出现 Vite 大 chunk warning，这是已知的 xterm.js 相关提醒，不等于构建失败。

桌面 smoke：

```powershell
npm run tauri dev
```

启动后检查：

- 应用窗口标题为 `vibe-monitor`。
- 三栏 Cockpit 布局稳定。
- Workspace 列表可以加载。
- Attention Queue 可以加载。
- 选择 Workspace 后可以打开 Terminal。
- Git Workspace 可以显示 Git 面板。
- Codex 可用时显示连接状态；不可用时显示明确的 unavailable 状态，界面不能卡死。

手动场景 1：Workspace

1. 添加一个本地 Git Workspace，例如 `D:\AIdeas\CodingMonitor`。
2. 确认它出现在左侧 Workspace 列表。
3. 重启应用。
4. 确认 Workspace 仍然存在。
5. 删除该 Workspace。
6. 再次重启后确认它不再出现。

手动场景 2：Attention Queue

1. 选择一个 Workspace。
2. 创建 MVP 示例 failed Attention Item。
3. 确认它出现在右侧 Attention Queue。
4. Resolve 该 item。
5. 重启应用。
6. 确认已 resolve 的 item 不再出现在 active queue。

手动场景 3：Terminal

1. 选择一个 Workspace。
2. 打开 Terminal 面板。
3. 执行 `git status`。
4. 执行 `node --version`。
5. 调整应用窗口大小。
6. 确认终端输出仍可读，输入仍可用。
7. 关闭应用。

手动场景 4：Git

1. 选择一个 Git Workspace。
2. 修改一个已跟踪文件。
3. 确认 Git 面板显示该文件。
4. 选择文件并确认 diff 渲染出来。
5. Stage 该文件。
6. 确认状态更新。
7. Unstage 该文件。
8. 确认状态再次更新。

手动场景 5：Codex

1. 选择一个 Workspace。
2. 确认 Codex CLI 检测会执行。
3. 如果 Codex 不可用，确认 UI 明确展示不可用状态且不卡死。
4. 如果 Codex 可用，尝试列出或启动一个 Codex Thread。
5. 发送一个 prompt。
6. 中断一个运行中的 turn。
7. 如果 Codex 请求审批，确认 Attention Queue 出现 approval item。

进程清理检查：

关闭应用后运行：

```powershell
Get-Process | Where-Object { $_.ProcessName -match 'codex|powershell|pwsh' } | Select-Object ProcessName,Id,StartTime
```

预期结果：

- 没有 vibe-monitor 拥有的 `codex app-server` 残留。
- 没有 vibe-monitor 拥有的终端子进程残留。
- 已经存在的用户 PowerShell 或当前 Codex 宿主进程可以忽略。

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
