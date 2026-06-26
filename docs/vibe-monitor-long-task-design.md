# vibe-monitor 长任务设计文档

> 日期：2026-06-26  
> 项目名称：vibe-monitor  
> 当前阶段：产品定义与长期开发计划  
> 产品定位：AI 编程驾驶舱 + 本地开发工作台 + 注意力管理层

## 1. 背景与目标

vibe-monitor 的核心问题不是“再做一个 AI 聊天窗口”，而是解决 vibe-coding 时多个关键窗口不断切换导致的注意力损耗。

典型工作流中会同时出现：

- Codex / Claude Code / 其他 AI coding agent 的长任务执行窗口
- VS Code、Zed 等编辑器
- Git/GitHub issue、PR、diff、CI 状态
- 终端、dev server、测试输出
- ChatGPT、浏览器资料、微信、音乐等辅助工具

vibe-monitor 的目标是把这些工具中“当前需要判断、批准、处理、恢复的信息”集中到一个主控面板中，让用户在一个工作台内完成大部分观察、审批、跳转和恢复上下文动作。

### 1.1 产品一句话

vibe-monitor 是面向 AI 编程工作流的本地优先全能开发工作台，用一个主控面板管理 Codex、终端、Git/GitHub、浏览器预览、编辑器跳转和外部工具状态。

### 1.2 成功标准

MVP 成功不以“集成工具数量”衡量，而以是否减少窗口切换和恢复成本衡量：

- 用户能在一个主界面看到当前项目、Codex 线程、终端、Git 状态和阻塞项。
- 用户能直接处理 Codex 审批、终端命令、Git diff、测试失败等高频动作。
- 用户能一键跳转到 VS Code/Zed、GitHub、浏览器预览、ChatGPT、微信、音乐等外部工具。
- 系统默认只突出需要用户介入的事项，避免把所有日志平铺成噪音。

## 2. 初步市场调研

### 2.1 竞品与开源项目

#### CodexMonitor

来源：https://github.com/Dimillian/CodexMonitor

CodexMonitor 是 Tauri 应用，围绕 Codex 多 workspace/thread 管理、Codex app-server 协议、worktree、Git/GitHub、终端、prompt library、远程 daemon 等能力构建。它证明了基于 Codex app-server 做深度客户端集成是可行路线。

可借鉴点：

- 使用 Codex app-server 获取线程、消息、审批、工具调用和流式事件。
- 对 workspace、thread、worktree 做结构化管理。
- 把 Git/GitHub 与 Codex 工作流放在同一界面。

vibe-monitor 需要避开的点：

- 不只做 Codex 专属客户端，否则容易和 CodexMonitor 正面重叠。
- 不应把所有功能堆成复杂面板，需要更强调“注意力队列”和跨工具状态聚合。

#### Vibe Dash

来源：https://github.com/sgentzen/vibe-dash

Vibe Dash 是 local-first 的 AI-driven development dashboard，通过 MCP 连接 Claude Code、Cursor、Codex、Copilot、Aider 等 agent，提供任务板、活动流、成本统计、SQLite、本地优先架构。

可借鉴点：

- MCP 作为 agent 汇报任务、状态、阻塞、成本的标准入口。
- 看板、活动流、cost tracker 适合多 agent 工作流。
- SQLite local-first 降低部署和隐私风险。

vibe-monitor 需要避开的点：

- 不把产品第一核心做成 Kanban。你的真实痛点是窗口切换和主控面板，而不是项目管理软件。

#### Vibe Space

来源：https://vibe-space.ai/

Vibe Space 定位为 AI engineers 的 desktop cockpit，支持多个 AI CLI 并排、terminal split、角色化 agent、token/cost、remote access、BYOK、本地执行。

可借鉴点：

- “cockpit”表达准确，适合 vibe-monitor 的产品心智。
- 多 AI CLI 并行、广播 prompt、会话状态和成本跟踪有长期价值。
- 远程查看和移动端监控可作为后期方向。

vibe-monitor 需要避开的点：

- 不要第一版就追求 11 个终端并排或远程访问，容易拖慢核心验证。

#### AgentsRoom

来源：https://agentsroom.dev/multi-agent-dashboard

AgentsRoom 强调多 agent 网格、实时终端、角色标签、状态颜色、Git worktree、移动监控。

可借鉴点：

- Agent 网格适合展示多个后台任务。
- 状态颜色和角色标签能降低扫描成本。
- “只在需要时介入”的工作流符合 vibe-monitor 注意力管理目标。

vibe-monitor 需要避开的点：

- 不把所有 agent 输出都作为主视图。vibe-monitor 的主视图应该围绕当前主任务和需要处理的事项。

#### Clopen

来源：https://github.com/myrialabs/clopen

Clopen 是 all-in-one workspace for AI coding agents，包含 chat、terminal、git、browser preview、database client、Monaco editor、Cloudflare Tunnel、MCP、checkpoint、collaboration 等。

可借鉴点：

- 证明全能开发工作台方向存在真实需求。
- 浏览器预览、数据库、编辑器、checkpoint 是长期可扩展模块。
- 各 AI engine 状态隔离和 tool installer 值得参考。

vibe-monitor 需要避开的点：

- 不在 MVP 复刻完整 IDE、数据库客户端和协作系统。
- 全能方向必须阶段化，否则范围会失控。

#### ai-agent-board

来源：https://github.com/ezeoli88/ai-agent-board

ai-agent-board 以 native binary + web dashboard 方式提供任务、agent、chat、diff、PR、MCP 集成，后端使用 Rust/Axum/SQLite/SSE。

可借鉴点：

- Rust 后端 + SQLite + SSE 的轻量本地架构适合桌面工具。
- Git worktree 隔离 agent 修改是可靠默认策略。
- Diff review 和 PR 创建是 AI coding 工具的关键闭环。

vibe-monitor 需要避开的点：

- 不把任务生命周期完全绑定到 PR 工作流。个人 vibe-coding 中也有大量探索、调试、资料整理和非 PR 长任务。

#### Pieces

来源：https://pieces.app/

Pieces 强调 OS 级记忆、跨工具捕获、自然语言检索、MCP 接入、本地优先。

可借鉴点：

- “记住我刚才在哪、看过什么、聊过什么”是减少上下文恢复成本的关键。
- 后续可以加入轻量 activity memory，而不是只做实时面板。

vibe-monitor 需要避开的点：

- 不第一版做 OS 级全量捕获，权限、隐私、复杂度都很高。

### 2.2 论文与研究启发

#### AI Chains: Transparent and Controllable Human-AI Interaction by Chaining Large Language Model Prompts

来源：https://dl.acm.org/doi/pdf/10.1145/3491102.3517582

启发：

- 长任务应该拆成透明、可控、可检查的步骤。
- vibe-monitor 应把 AI 工作流呈现为 plan、running、blocked、review、done，而不是一整段不可控聊天。

#### Generative AI and Developer Workflows

来源：https://scholarspace.manoa.hawaii.edu/bitstreams/4f29b0c8-06ac-4d5d-9fa0-52eb93321eb2/download

启发：

- 生成式 AI 已经改变 solo 和 pair programming 的工作流。
- 产品应该围绕“人类判断点 + AI 执行过程 + 代码状态”设计，而不是把 AI 聊天和开发环境割裂。

#### The Programmer's Assistant

来源：https://dl.acm.org/doi/pdf/10.1145/3581641.3584037

启发：

- 编程助手的价值来自与真实开发上下文结合。
- vibe-monitor 需要把文件、终端、Git、错误、任务目标和聊天聚合成同一个工作上下文。

#### Detecting Developers' Task Switches and Types

来源：https://ieeexplore.ieee.org/ielx7/32/9675297/09069309.pdf

启发：

- 开发者任务切换可以被检测和分类。
- 后续可以做“当前频繁切换提醒”“任务恢复卡片”“刚刚中断前状态”。

#### Sensing Interruptibility in the Office

来源：http://dl.acm.org/ft_gateway.cfm?id=3174165&type=pdf

启发：

- 通知不应该等价对待。
- vibe-monitor 的注意力队列要区分阻塞、完成、失败、普通日志、低优先提醒。

### 2.3 市场机会判断

现有产品大多在三件事上竞争：

- 多 agent 并行
- AI CLI 终端编排
- 任务看板和成本追踪

vibe-monitor 的机会是补上“开发者注意力主控层”：

- 不追求显示所有东西，而是过滤出下一步该处理的事。
- 不只服务 agent，而是服务人的开发节奏。
- 不只管理 Codex，而是把 Codex 作为核心执行引擎，再向 IDE、GitHub、浏览器、聊天、音乐扩展。

## 3. 产品定义

### 3.1 产品定位

vibe-monitor 是全能开发工作台，但必须按阶段实现：

- 长期目标：全能开发工作台。
- 第一阶段目标：Codex 主控台 + 终端 + Git + 注意力队列。
- 设计原则：主界面永远服务“我现在该看哪里、批哪里、跑哪里、回哪里”。

### 3.2 核心用户

第一用户是重度 vibe-coding 的个人开发者，尤其是：

- 同时运行多个 AI coding 任务。
- 经常在 Codex、IDE、GitHub、浏览器、ChatGPT、微信之间切换。
- 需要长任务持续执行和中途恢复。
- 希望本地优先，不愿把代码和工作流状态上传到第三方平台。

### 3.3 非目标

MVP 不做以下内容：

- 不复刻完整 VS Code/Zed 编辑器。
- 不做完整团队项目管理 SaaS。
- 不做 OS 级全量窗口录制。
- 不做微信、音乐、ChatGPT 的深度自动化控制。
- 不做远程移动端访问。
- 不做数据库客户端、云部署、团队协作。

## 4. 产品体验设计

### 4.1 主界面布局

第一屏就是工作台，不做营销页或空首页。

建议布局：

- 左侧导航栏：项目、workspace、session、常用工具。
- 中央主工作区：Codex 会话、终端、浏览器预览、diff review，可分屏。
- 右侧注意力队列：审批、失败、阻塞、完成、未读、待 review。
- 底部抽屉：terminal、logs、git diff、dev server 输出。
- 顶部命令栏：全局搜索、打开项目、启动 session、运行命令、跳转工具。

### 4.2 信息优先级

默认只高亮需要用户处理的信息：

1. 安全/权限审批。
2. Codex 等待用户输入。
3. 测试失败、构建失败、命令失败。
4. Git 冲突、未提交重要变更、PR review 请求。
5. 长任务完成。
6. 微信/ChatGPT 等外部工具未读或待处理状态。
7. 普通日志和 token/cost 信息。

普通流式日志默认折叠，只在用户展开 session 时完整显示。

### 4.3 核心工作流

#### 工作流 A：启动 AI 长任务

1. 用户选择项目。
2. 用户在命令栏或 Codex 面板输入任务。
3. 系统启动 Codex thread。
4. 任务进入 Running。
5. 右侧只显示审批、阻塞、失败、完成。
6. 用户无需盯日志，等注意力队列提示介入。

#### 工作流 B：处理 Codex 审批

1. Codex 请求 shell、网络、文件等审批。
2. 注意力队列出现审批卡。
3. 用户查看命令、影响范围、工作目录。
4. 用户批准、拒绝、编辑后批准。
5. 任务继续执行。

#### 工作流 C：测试失败恢复

1. 终端或 Codex 工具调用产生失败。
2. 系统归类为 Failure。
3. 用户可查看失败摘要、完整日志、相关文件、重跑按钮。
4. 用户可把失败上下文发送回当前 Codex thread。

#### 工作流 D：Git review

1. 系统读取 git status 和 diff。
2. 用户在工作台查看 diff。
3. 用户可 stage、unstage、discard、commit。
4. 后续接入 GitHub PR 创建与 review。

#### 工作流 E：外部工具跳转

1. 用户看到微信未读、ChatGPT 入口、音乐播放状态或 GitHub PR 状态。
2. MVP 阶段提供打开/聚焦/深链跳转。
3. 深度读取和自动化控制留到插件阶段。

## 5. 技术架构

### 5.1 推荐技术栈

- 桌面壳：Tauri 2
- 前端：React + TypeScript + Vite
- UI：Tailwind CSS + shadcn/ui + lucide-react
- 状态管理：Zustand
- 异步数据：TanStack Query
- 后端：Rust
- 本地数据库：SQLite
- 终端：xterm.js + portable PTY / Windows ConPTY
- 实时通信：Tauri event + 内部 SSE/WebSocket 抽象
- Codex 集成：codex app-server JSON-RPC
- Git 集成：本地 git CLI
- GitHub 集成：优先 gh CLI，后续 GitHub OAuth/API
- 插件接口：MCP + 本地插件 manifest

### 5.2 为什么选 Tauri

Tauri 适合本项目：

- 桌面原生能力强，能启动进程、管理窗口、读本地文件。
- Rust 后端适合 PTY、文件系统、SQLite、Git、Codex 子进程管理。
- 比 Electron 更适合长期本地工具的资源占用目标。

### 5.3 模块边界

#### Shell 层

职责：

- Tauri 窗口、菜单、系统托盘、全局快捷键。
- 启动和管理后端服务。
- 跨平台窗口聚焦和外部应用打开。

#### Workspace 层

职责：

- 管理项目列表。
- 保存 workspace 配置。
- 识别 Git root、package manager、常用命令。
- 保存最近打开文件、session 和工具布局。

#### Codex 层

职责：

- 启动 `codex app-server`。
- 管理 JSON-RPC 连接。
- 列出、启动、恢复、归档 thread。
- 接收 turn、item、approval、tool、diff 等事件。
- 将 Codex 状态映射为 UI session 状态。

#### Terminal 层

职责：

- 创建 PTY。
- 管理命令、输出、resize、kill。
- 识别 dev server、test、build 常见命令。
- 把失败摘要送入注意力队列。

#### Git 层

职责：

- 读取 status、branch、log、diff。
- stage/unstage/commit。
- 后续支持 worktree 和 PR。

#### Attention 层

职责：

- 聚合 Codex、Terminal、Git、GitHub、External Apps 事件。
- 统一分类为 approval、blocked、failed、done、unread、info。
- 控制通知优先级和面板排序。

#### External App 层

职责：

- MVP：launcher、open URL、open app、focus window。
- 后续：读取微信/ChatGPT/音乐/浏览器状态。
- 外部工具必须插件化，避免主程序耦合不可控平台。

### 5.4 数据模型草案

```ts
type Workspace = {
  id: string;
  name: string;
  path: string;
  gitRoot?: string;
  defaultAiEngine: "codex" | "claude" | "custom";
  createdAt: string;
  updatedAt: string;
};

type Session = {
  id: string;
  workspaceId: string;
  engine: "codex" | "terminal" | "browser" | "external";
  title: string;
  status: "idle" | "running" | "blocked" | "failed" | "review" | "done";
  sourceId?: string;
  createdAt: string;
  updatedAt: string;
};

type AttentionItem = {
  id: string;
  workspaceId: string;
  sessionId?: string;
  kind: "approval" | "blocked" | "failed" | "done" | "unread" | "info";
  priority: 0 | 1 | 2 | 3;
  title: string;
  summary: string;
  actionLabel?: string;
  actionRef?: string;
  createdAt: string;
  resolvedAt?: string;
};
```

## 6. 长期路线图

### Phase 0：项目基础与设计验证

目标：建立文档、原型、架构边界。

交付：

- 产品设计文档。
- 技术设计文档。
- UI wireframe。
- 数据模型草案。
- MVP acceptance criteria。

### Phase 1：MVP 本地驾驶舱

目标：做出能真实减少窗口切换的第一版。

范围：

- Tauri 桌面应用。
- Workspace 管理。
- Codex app-server 接入。
- 线程列表、主会话视图、消息发送、审批处理。
- 内置终端。
- Git status/diff。
- 注意力队列。

不做：

- 浏览器预览。
- GitHub PR。
- 微信/音乐深度集成。
- 多 AI engine。
- 团队协作。

### Phase 2：开发工作台闭环

目标：覆盖 web app 开发的日常闭环。

范围：

- 浏览器预览。
- dev server 检测与启动。
- 测试失败解析。
- Git commit。
- GitHub issue/PR 查看。
- PR 创建草案。
- diff review 与发送反馈给 Codex。

### Phase 3：外部工具聚合

目标：把常见窗口切换变成状态卡和快速跳转。

范围：

- ChatGPT、浏览器资料、微信、音乐、编辑器入口。
- 外部 app launcher。
- 窗口聚焦。
- URL deep link。
- 常用工具状态卡。

注意：

- 微信等工具第一版只做 launcher 和提醒占位，不做消息抓取。
- 深度自动化需要单独评估隐私、权限和平台限制。

### Phase 4：注意力系统

目标：从“面板”升级为“开发节奏管理器”。

范围：

- 任务切换记录。
- 中断恢复卡。
- 勿扰模式。
- 通知优先级策略。
- 长任务摘要。
- 每日开发回顾。

### Phase 5：插件与生态

目标：让所有工具接入都插件化。

范围：

- 本地插件 manifest。
- MCP server 管理。
- Agent/tool marketplace。
- 自定义面板。
- 自定义 attention item provider。

## 7. 任务拆分

### Task 1：创建产品与工程骨架

目标：建立可运行桌面项目和基础目录。

交付：

- `vibe-monitor` 项目目录。
- Tauri + React + TypeScript + Vite。
- Rust 后端基础命令。
- Tailwind + shadcn/ui + lucide-react。
- SQLite 初始化。
- 基础应用窗口。

验收：

- Windows 本机能启动桌面窗口。
- 前端热更新可用。
- 后端命令可从前端调用。

### Task 2：Workspace 管理

目标：能添加、保存、切换本地项目。

交付：

- Workspace 列表。
- 添加项目路径。
- 自动识别 Git root。
- 最近打开 workspace。
- SQLite 持久化。

验收：

- 重启应用后 workspace 仍存在。
- 非 Git 项目可添加，但 Git 功能显示不可用。

### Task 3：Codex app-server 连接

目标：把 Codex 接入主控台。

交付：

- 检测 Codex CLI。
- 启动 `codex app-server` 子进程。
- 建立 JSON-RPC stdio 连接。
- 实现 thread list/start/resume。
- 接收 streamed events。

验收：

- 能在 vibe-monitor 中创建 Codex thread。
- 能发送消息并看到回复。
- Codex 子进程异常退出时 UI 有明确错误。

### Task 4：Codex 主会话视图

目标：把 Codex thread 变成主工作区。

交付：

- 消息列表。
- 工具调用展示。
- reasoning/tool/diff/log 折叠。
- steer/interrupt。
- thread rename/archive。

验收：

- 长任务执行时 UI 不阻塞。
- 用户能中断正在运行的任务。
- 普通日志默认不淹没主视图。

### Task 5：审批处理

目标：把 Codex approval 变成注意力队列中的一等事件。

交付：

- Approval item 解析。
- 审批卡片。
- 批准、拒绝、编辑后批准。
- 审批历史。

验收：

- Codex 请求权限时右侧出现高优先级卡片。
- 用户处理后任务继续或停止。

### Task 6：内置终端

目标：减少切换到系统 terminal 的频率。

交付：

- xterm.js 面板。
- Rust PTY 管理。
- 多 terminal tab。
- cwd 绑定 workspace。
- command history。

验收：

- 能运行 `npm test`、`git status` 等命令。
- 输出支持 ANSI color。
- 终端可 resize 和 kill。

### Task 7：Git 状态与 Diff

目标：让用户不用离开工作台即可判断代码状态。

交付：

- 当前 branch。
- changed files。
- staged/unstaged。
- diff viewer。
- stage/unstage。

验收：

- Git 状态变化能刷新。
- diff 可读。
- stage/unstage 不破坏用户未提交文件。

### Task 8：注意力队列 MVP

目标：建立 vibe-monitor 的核心差异点。

交付：

- AttentionItem 数据表。
- 统一事件入口。
- approval、failed、blocked、done 分类。
- 右侧队列 UI。
- resolve/snooze。

验收：

- Codex 审批、终端失败、任务完成都会进入队列。
- 已处理事项不再干扰主视图。

### Task 9：Dev Server 与测试失败

目标：覆盖 web 开发常见反馈循环。

交付：

- package manager 检测。
- 常用命令快捷按钮。
- dev server 状态。
- test/build 失败摘要。
- 一键把失败发送给 Codex。

验收：

- 用户能在工作台启动测试。
- 测试失败后能生成清晰 attention item。

### Task 10：浏览器预览

目标：减少切换到浏览器验证 UI 的次数。

交付：

- 内嵌 webview/browser panel。
- local dev server URL 管理。
- refresh/open external browser。
- 截图或评论能力预留。

验收：

- 能预览本地 dev server。
- 预览面板不会影响 Codex 和终端运行。

### Task 11：GitHub 集成

目标：把 issue/PR review 拉进工作台。

交付：

- 检测 `gh` CLI。
- 读取 issue/PR 列表。
- PR diff/评论摘要。
- 打开 GitHub 链接。
- 创建 PR 草案。

验收：

- 没有 `gh` 时显示安装/登录提示。
- 已登录时能查看当前 repo PR。

### Task 12：外部工具 Launcher

目标：第一阶段解决微信、ChatGPT、音乐等工具的快速切换。

交付：

- Tool registry。
- 打开 URL。
- 打开本地应用。
- 聚焦窗口能力评估。
- 常用工具栏。

验收：

- 用户可配置 ChatGPT、微信、音乐、VS Code、Zed、浏览器等入口。
- 点击后能打开或聚焦目标工具。

### Task 13：插件接口雏形

目标：避免外部工具集成污染核心代码。

交付：

- Plugin manifest 草案。
- attention item provider 接口。
- command provider 接口。
- panel provider 接口。

验收：

- 至少一个内置工具以插件形式注册。
- 核心 UI 不依赖具体外部工具实现。

### Task 14：用户设置与隐私控制

目标：让本地优先和权限边界明确。

交付：

- 设置页。
- 数据目录显示。
- Codex path 配置。
- GitHub/gh 状态。
- 外部工具配置。
- 隐私说明。

验收：

- 用户能看到哪些数据保存在本地。
- 用户能关闭外部工具状态读取。

### Task 15：稳定性与打包

目标：形成可日常使用的本地版本。

交付：

- Windows 打包。
- 崩溃日志。
- 子进程清理。
- 数据库迁移。
- 基础 E2E 测试。

验收：

- 安装包可运行。
- 关闭应用后 Codex/terminal 子进程不会残留。

## 8. MVP 验收标准

MVP 必须完成 Task 1 到 Task 8。

MVP 可接受状态：

- 只支持 Windows 优先。
- 只支持 Codex 一个 AI engine。
- GitHub 只做后续任务。
- 外部工具只做后续任务。
- UI 不追求完整视觉系统，但必须可长时间使用。

MVP 不可接受状态：

- 只能聊天，不能处理审批。
- 只能看日志，不能把阻塞项聚合出来。
- 不能从工作台看到 Git 状态。
- 不能运行终端命令。
- 所有信息混在一个日志流里。

## 9. 风险与应对

### 9.1 范围失控

风险：全能工作台容易变成 IDE、浏览器、聊天客户端、项目管理工具的合集。

应对：

- MVP 锁定 Codex + terminal + Git + attention queue。
- 外部工具第一版只做 launcher。
- 插件化承接长期扩展。

### 9.2 Codex app-server 协议变化

风险：app-server 仍可能演进，接口兼容性需要维护。

应对：

- Codex adapter 单独隔离。
- JSON-RPC 类型集中维护。
- 每次 Codex CLI 升级跑协议兼容测试。

### 9.3 跨平台 PTY 与窗口控制复杂

风险：Windows、macOS、Linux 的 terminal 和窗口聚焦差异明显。

应对：

- Windows 优先。
- PTY 单独模块。
- 外部 app focus 作为可选能力。

### 9.4 隐私与权限

风险：如果读取微信、浏览器、窗口状态，用户会担心隐私。

应对：

- 默认本地优先。
- 默认不读取聊天内容。
- 所有外部工具状态读取都必须显式开启。

### 9.5 UI 噪音

风险：集成越多，面板越乱。

应对：

- 注意力队列只放需要处理的事项。
- 普通日志默认折叠。
- 主视图围绕当前任务，而不是所有工具平铺。

## 10. 后续立即行动

下一步建议按以下顺序推进：

1. 画出低保真 UI wireframe，确认主界面布局。
2. 创建 `vibe-monitor` 项目骨架。
3. 完成 Task 1 和 Task 2。
4. 单独 spike Codex app-server 连接，验证 thread start/resume/send/event。
5. 再进入 Task 3 到 Task 8 的 MVP 开发。

## 11. 文档审查结论

当前方案选择“全能开发工作台”作为长期方向，但明确 MVP 只做：

- Codex 主控
- 本地终端
- Git 状态与 diff
- 注意力队列
- Workspace 管理

这能保留长期野心，同时避免第一版被微信、音乐、完整编辑器、浏览器、数据库、团队协作拖死。

