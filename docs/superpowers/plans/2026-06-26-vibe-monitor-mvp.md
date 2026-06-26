# vibe-monitor MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first usable vibe-monitor MVP: Workspace management, Codex control, terminal, Git status/diff, and the Attention Queue in a Windows-first Tauri desktop app.

**Architecture:** Use Tauri 2 as the desktop shell, React/TypeScript for the UI, Rust commands for local process/filesystem/Git/Codex/PTY work, and SQLite for local state. Keep Codex, Workspace, Terminal, Git, and Attention as separate bounded modules with typed IPC contracts between frontend and backend.

**Tech Stack:** Tauri 2, Rust, React, TypeScript, Vite, Tailwind CSS, shadcn/ui, lucide-react, Zustand, TanStack Query, SQLite, xterm.js, Windows ConPTY-compatible PTY, Git CLI, Codex app-server JSON-RPC.

## Global Constraints

- Windows is the first supported platform; do not block future macOS/Linux support with Windows-only frontend assumptions.
- MVP scope is Workspace management, Codex control, terminal, Git status/diff, and Attention Queue only.
- Codex integration must use `codex app-server`, not CLI output scraping and not embedding the Codex UI.
- External tools are out of MVP except as future Launcher concepts; do not read WeChat, ChatGPT, browser, or music app private content.
- All persistent product data is local-first and stored under the app data directory.
- Use the project glossary in `CONTEXT.md`: Workspace, Session, Codex Thread, Attention Item, Attention Queue, External Tool, Launcher, Plugin, MVP, Cockpit.
- Prefer PowerShell commands on this Windows host and UTF-8 reads for Chinese content.
- Keep changes committed after each task.

---

## Source Notes

- Tauri recommends `create-tauri-app` and supports React templates through the official scaffold flow: https://v2.tauri.app/start/create-project/
- Tauri recommends Vite for SPA frameworks such as React: https://v2.tauri.app/start/frontend/
- Tauri project structure uses `src-tauri/`, `tauri.conf.json`, `capabilities/`, icons, and Rust build files: https://v2.tauri.app/start/project-structure/
- Tauri commands are the frontend-to-Rust bridge for typed local operations: https://v2.tauri.app/develop/calling-rust/
- Tauri SQL plugin supports SQLite through sqlx and app-relative `sqlite:` database paths: https://v2.tauri.app/plugin/sql/
- shadcn/ui has a Vite installation flow: https://ui.shadcn.com/docs/installation/vite
- xterm.js installs from npm as `@xterm/xterm`; addons such as fit are loaded with `term.loadAddon(...)`: https://xtermjs.org/ and https://xtermjs.org/docs/guides/using-addons/

## File Structure

Create or preserve this structure during MVP implementation:

```text
/
  CONTEXT.md
  docs/
    adr/
    superpowers/plans/2026-06-26-vibe-monitor-mvp.md
    vibe-monitor-long-task-design.md
  src/
    app/
      App.tsx
      layout/AppShell.tsx
      providers/AppProviders.tsx
    features/
      attention/
      codex/
      git/
      terminal/
      workspace/
    shared/
      api/
      components/
      lib/
      types/
    styles/
      globals.css
  src-tauri/
    capabilities/default.json
    src/
      attention/
      codex/
      db/
      git/
      terminal/
      workspace/
      lib.rs
      main.rs
      state.rs
      error.rs
```

Frontend feature folders own UI, hooks, and TypeScript API wrappers. Rust feature folders own command handlers and local adapters. Shared data shapes must be mirrored explicitly in `src/shared/types/*.ts` and Rust structs with `serde`.

---

### Task 1: Scaffold Tauri React App and Tooling

**Files:**
- Create: `package.json`, `index.html`, `vite.config.ts`, `tsconfig.json`, `src/`, `src-tauri/`
- Modify: `.gitignore`
- Verify: `npm run dev`, `npm run build`, `cargo check`

**Interfaces:**
- Produces: a runnable Tauri 2 + React + TypeScript desktop shell.
- Consumes: no application code from later tasks.

- [ ] **Step 1: Verify prerequisites**

Run:

```powershell
node --version
npm --version
rustc --version
cargo --version
```

Expected: Node, npm, Rust, and Cargo are installed. If Rust is missing, install Rust before continuing. If WebView2 is missing on Windows, install the Microsoft Edge WebView2 Runtime.

- [ ] **Step 2: Scaffold in a temporary directory**

Run from `D:\AIdeas\CodingMonitor`:

```powershell
npm create tauri-app@latest app-scaffold
```

Prompt choices:

```text
Package manager: npm
UI template: React
Language: TypeScript
Project name: vibe-monitor
Identifier: com.styleliyu.vibemonitor
```

Expected: `D:\AIdeas\CodingMonitor\app-scaffold` contains a working Tauri project.

- [ ] **Step 3: Merge scaffold into repository root**

Move generated app files into the repository root and keep existing `docs/`, `CONTEXT.md`, `.git/`.

Run:

```powershell
Move-Item -LiteralPath .\app-scaffold\src -Destination .\src
Move-Item -LiteralPath .\app-scaffold\src-tauri -Destination .\src-tauri
Move-Item -LiteralPath .\app-scaffold\public -Destination .\public -ErrorAction SilentlyContinue
Move-Item -LiteralPath .\app-scaffold\index.html -Destination .\index.html
Move-Item -LiteralPath .\app-scaffold\package.json -Destination .\package.json
Move-Item -LiteralPath .\app-scaffold\package-lock.json -Destination .\package-lock.json -ErrorAction SilentlyContinue
Move-Item -LiteralPath .\app-scaffold\tsconfig.json -Destination .\tsconfig.json
Move-Item -LiteralPath .\app-scaffold\tsconfig.node.json -Destination .\tsconfig.node.json -ErrorAction SilentlyContinue
Move-Item -LiteralPath .\app-scaffold\vite.config.ts -Destination .\vite.config.ts
Remove-Item -LiteralPath .\app-scaffold -Recurse -Force
```

Expected: no app files remain under `app-scaffold`.

- [ ] **Step 4: Install frontend dependencies**

Run:

```powershell
npm install zustand @tanstack/react-query lucide-react clsx tailwind-merge class-variance-authority
npm install @xterm/xterm @xterm/addon-fit
npm install -D tailwindcss @tailwindcss/vite
```

Expected: dependencies are recorded in `package.json`.

- [ ] **Step 5: Initialize shadcn/ui**

Run:

```powershell
npx shadcn@latest init
```

Prompt choices:

```text
Style: New York
Base color: Neutral
CSS variables: yes
```

Then add MVP components:

```powershell
npx shadcn@latest add button card dialog dropdown-menu input scroll-area separator sheet tabs textarea tooltip badge
```

Expected: `src/components/ui/` exists, or the generated alias is adjusted to `src/shared/components/ui/` in Task 2.

- [ ] **Step 6: Install Tauri plugins needed for MVP**

Run:

```powershell
npm install @tauri-apps/plugin-sql @tauri-apps/plugin-fs @tauri-apps/plugin-dialog @tauri-apps/plugin-shell
cd src-tauri
cargo add tauri-plugin-sql --features sqlite
cargo add tauri-plugin-fs
cargo add tauri-plugin-dialog
cargo add tauri-plugin-shell
cargo add serde --features derive
cargo add serde_json
cargo add thiserror
cargo add uuid --features v4,serde
cargo add chrono --features serde
cargo add tokio --features process,io-util,sync,rt-multi-thread,macros
cd ..
```

Expected: `src-tauri/Cargo.toml` includes the plugins and supporting crates.

- [ ] **Step 7: Verify scaffold**

Run:

```powershell
npm run build
cd src-tauri
cargo check
cd ..
```

Expected: both commands pass.

- [ ] **Step 8: Commit**

Run:

```powershell
git add .
git commit -m "chore: scaffold tauri react app"
git push
```

Expected: commit is pushed to `origin/main`.

---

### Task 2: Establish App Shell and Frontend Boundaries

**Files:**
- Create: `src/app/App.tsx`, `src/app/layout/AppShell.tsx`, `src/app/providers/AppProviders.tsx`
- Create: `src/shared/lib/cn.ts`, `src/shared/types/session.ts`
- Modify: generated `src/main.tsx`, `src/styles/globals.css`
- Test: `npm run build`

**Interfaces:**
- Produces: `SessionSummary`, `AppShell`, and stable UI regions for later features.
- Consumes: scaffold from Task 1.

- [x] **Step 1: Define session types**

Create `src/shared/types/session.ts`:

```ts
export type SessionEngine = "codex" | "terminal" | "browser" | "external";

export type SessionStatus =
  | "idle"
  | "running"
  | "blocked"
  | "failed"
  | "review"
  | "done";

export type SessionSummary = {
  id: string;
  workspaceId: string;
  engine: SessionEngine;
  title: string;
  status: SessionStatus;
  sourceId?: string;
  createdAt: string;
  updatedAt: string;
};
```

- [x] **Step 2: Add utility helper**

Create `src/shared/lib/cn.ts`:

```ts
import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
```

- [x] **Step 3: Create providers**

Create `src/app/providers/AppProviders.tsx`:

```tsx
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { type ReactNode, useState } from "react";

export function AppProviders({ children }: { children: ReactNode }) {
  const [queryClient] = useState(() => new QueryClient());

  return (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
}
```

- [x] **Step 4: Build fixed MVP layout**

Create `src/app/layout/AppShell.tsx`:

```tsx
import { Bell, GitBranch, Monitor, PanelLeft, Terminal } from "lucide-react";

export function AppShell() {
  return (
    <div className="grid h-screen grid-cols-[280px_1fr_360px] grid-rows-[48px_1fr_240px] bg-background text-foreground">
      <header className="col-span-3 flex items-center gap-3 border-b px-4">
        <Monitor className="size-5" />
        <span className="font-semibold">vibe-monitor</span>
        <div className="ml-auto text-sm text-muted-foreground">
          MVP Cockpit
        </div>
      </header>
      <aside className="row-span-2 border-r">
        <div className="flex h-12 items-center gap-2 border-b px-3 text-sm font-medium">
          <PanelLeft className="size-4" />
          Workspaces
        </div>
      </aside>
      <main className="min-w-0">
        <div className="flex h-12 items-center gap-2 border-b px-4 text-sm font-medium">
          Codex Session
        </div>
      </main>
      <aside className="row-span-2 border-l">
        <div className="flex h-12 items-center gap-2 border-b px-3 text-sm font-medium">
          <Bell className="size-4" />
          Attention Queue
        </div>
      </aside>
      <section className="border-t">
        <div className="flex h-10 items-center gap-2 border-b px-4 text-sm font-medium">
          <Terminal className="size-4" />
          Terminal
        </div>
      </section>
      <section className="border-t">
        <div className="flex h-10 items-center gap-2 border-b px-4 text-sm font-medium">
          <GitBranch className="size-4" />
          Git
        </div>
      </section>
    </div>
  );
}
```

- [x] **Step 5: Wire app entry**

Create `src/app/App.tsx`:

```tsx
import { AppShell } from "./layout/AppShell";
import { AppProviders } from "./providers/AppProviders";

export function App() {
  return (
    <AppProviders>
      <AppShell />
    </AppProviders>
  );
}
```

Update `src/main.tsx` to render `App`.

- [x] **Step 6: Verify**

Run:

```powershell
npm run build
```

Expected: TypeScript and Vite build pass.

- [x] **Step 7: Commit**

Run:

```powershell
git add src package.json package-lock.json
git commit -m "feat: add cockpit app shell"
git push
```

---

### Task 3: Add SQLite State and Workspace Backend

**Files:**
- Create: `src-tauri/src/error.rs`, `src-tauri/src/state.rs`
- Create: `src-tauri/src/db/mod.rs`, `src-tauri/src/workspace/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `cd src-tauri && cargo test && cargo check`

**Interfaces:**
- Produces Rust commands:
  - `workspace_list() -> Result<Vec<Workspace>, AppError>`
  - `workspace_add(path: String, name: Option<String>) -> Result<Workspace, AppError>`
  - `workspace_remove(id: String) -> Result<(), AppError>`
- Produces Rust type `Workspace`.

- [x] **Step 1: Define Rust error type**

Create `src-tauri/src/error.rs`:

```rust
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("io error: {0}")]
    Io(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("command failed: {0}")]
    CommandFailed(String),
    #[error("database error: {0}")]
    Database(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
```

- [x] **Step 2: Define application state**

Create `src-tauri/src/state.rs`:

```rust
use std::path::PathBuf;

#[derive(Clone)]
pub struct AppState {
    pub app_data_dir: PathBuf,
}
```

- [x] **Step 3: Initialize database file and schema**

Create `src-tauri/src/db/mod.rs` with an `init_db(app_data_dir: &Path) -> Result<(), AppError>` function that creates `vibe-monitor.db` and a `workspaces` table:

```sql
CREATE TABLE IF NOT EXISTS workspaces (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  path TEXT NOT NULL UNIQUE,
  git_root TEXT,
  default_ai_engine TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

Use `rusqlite` if selected during implementation, or `tauri-plugin-sql` if using plugin-managed SQL. Keep the database path under Tauri app data.

- [x] **Step 4: Add `rusqlite` if using backend-owned SQLite**

If backend-owned SQLite is chosen for MVP commands, run:

```powershell
cd src-tauri
cargo add rusqlite --features bundled
cd ..
```

Expected: Rust commands can read/write SQLite without frontend SQL access.

- [x] **Step 5: Implement Workspace type and commands**

Create `src-tauri/src/workspace/mod.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub path: String,
    pub git_root: Option<String>,
    pub default_ai_engine: String,
    pub created_at: String,
    pub updated_at: String,
}
```

Implement:

```rust
#[tauri::command]
pub async fn workspace_list(state: tauri::State<'_, crate::state::AppState>) -> Result<Vec<Workspace>, crate::error::AppError>;

#[tauri::command]
pub async fn workspace_add(
    state: tauri::State<'_, crate::state::AppState>,
    path: String,
    name: Option<String>,
) -> Result<Workspace, crate::error::AppError>;

#[tauri::command]
pub async fn workspace_remove(
    state: tauri::State<'_, crate::state::AppState>,
    id: String,
) -> Result<(), crate::error::AppError>;
```

`workspace_add` must reject non-directory paths and derive `name` from the directory name when `name` is absent.

- [x] **Step 6: Detect Git root**

In `workspace_add`, run:

```powershell
git -C <path> rev-parse --show-toplevel
```

Behavior:

- Exit code 0: store stdout as `gitRoot`.
- Non-zero: store `gitRoot = null`; workspace is still valid.

- [x] **Step 7: Register commands**

Update `src-tauri/src/lib.rs` to manage `AppState`, call database initialization during setup, and register Workspace commands with `tauri::generate_handler!`.

- [x] **Step 8: Verify**

Run:

```powershell
cd src-tauri
cargo test
cargo check
cd ..
npm run build
```

Expected: all pass.

- [x] **Step 9: Commit**

Run:

```powershell
git add src-tauri package.json package-lock.json
git commit -m "feat: add workspace persistence backend"
git push
```

---

### Task 4: Build Workspace UI and Frontend API

**Files:**
- Create: `src/shared/api/tauri.ts`
- Create: `src/features/workspace/types.ts`, `src/features/workspace/api.ts`, `src/features/workspace/useWorkspaces.ts`
- Create: `src/features/workspace/WorkspaceSidebar.tsx`
- Modify: `src/app/layout/AppShell.tsx`
- Test: `npm run build`

**Interfaces:**
- Consumes Rust commands from Task 3.
- Produces `useWorkspaces()` and `WorkspaceSidebar`.

- [x] **Step 1: Add typed Tauri invoke wrapper**

Create `src/shared/api/tauri.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";

export function callTauri<T>(command: string, args?: Record<string, unknown>) {
  return invoke<T>(command, args);
}
```

- [x] **Step 2: Add Workspace frontend type**

Create `src/features/workspace/types.ts` matching Rust camelCase output:

```ts
export type Workspace = {
  id: string;
  name: string;
  path: string;
  gitRoot?: string | null;
  defaultAiEngine: "codex" | "claude" | "custom";
  createdAt: string;
  updatedAt: string;
};
```

- [x] **Step 3: Add Workspace API**

Create `src/features/workspace/api.ts`:

```ts
import { callTauri } from "@/shared/api/tauri";
import type { Workspace } from "./types";

export function listWorkspaces() {
  return callTauri<Workspace[]>("workspace_list");
}

export function addWorkspace(path: string, name?: string) {
  return callTauri<Workspace>("workspace_add", { path, name });
}

export function removeWorkspace(id: string) {
  return callTauri<void>("workspace_remove", { id });
}
```

- [x] **Step 4: Add hook**

Create `src/features/workspace/useWorkspaces.ts` with TanStack Query:

```ts
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { addWorkspace, listWorkspaces, removeWorkspace } from "./api";

export function useWorkspaces() {
  const queryClient = useQueryClient();
  const query = useQuery({ queryKey: ["workspaces"], queryFn: listWorkspaces });
  const add = useMutation({
    mutationFn: ({ path, name }: { path: string; name?: string }) =>
      addWorkspace(path, name),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["workspaces"] }),
  });
  const remove = useMutation({
    mutationFn: removeWorkspace,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["workspaces"] }),
  });

  return { ...query, addWorkspace: add.mutate, removeWorkspace: remove.mutate };
}
```

- [x] **Step 5: Build sidebar**

Create `WorkspaceSidebar` with:

- list of workspaces
- empty state text: `Add a workspace to start monitoring AI development work.`
- manual path input for MVP
- add button
- selected workspace local state

- [x] **Step 6: Wire into shell**

Update `AppShell` left sidebar to render `WorkspaceSidebar`.

- [x] **Step 7: Verify**

Run:

```powershell
npm run build
npm run tauri dev
```

Manual check:

- Add `D:\AIdeas\CodingMonitor` as a workspace.
- Restart the app.
- Workspace still appears.

- [x] **Step 8: Commit**

Run:

```powershell
git add src src-tauri
git commit -m "feat: add workspace sidebar"
git push
```

---

### Task 5: Implement Attention Domain MVP

**Files:**
- Create: `src-tauri/src/attention/mod.rs`
- Create: `src/features/attention/types.ts`, `src/features/attention/api.ts`, `src/features/attention/useAttentionQueue.ts`
- Create: `src/features/attention/AttentionQueue.tsx`
- Modify: `src-tauri/src/db/mod.rs`, `src-tauri/src/lib.rs`, `src/app/layout/AppShell.tsx`
- Test: `cargo test`, `npm run build`

**Interfaces:**
- Produces Rust commands:
  - `attention_list(workspace_id: Option<String>) -> Vec<AttentionItem>`
  - `attention_create(input: CreateAttentionItem) -> AttentionItem`
  - `attention_resolve(id: String) -> ()`
- Produces frontend `AttentionItem` and queue UI.

- [x] **Step 1: Add database table**

Add migration/schema:

```sql
CREATE TABLE IF NOT EXISTS attention_items (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  session_id TEXT,
  kind TEXT NOT NULL,
  priority INTEGER NOT NULL,
  title TEXT NOT NULL,
  summary TEXT NOT NULL,
  action_label TEXT,
  action_ref TEXT,
  created_at TEXT NOT NULL,
  resolved_at TEXT
);
```

- [x] **Step 2: Define Rust types**

Create `src-tauri/src/attention/mod.rs` with:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttentionItem {
    pub id: String,
    pub workspace_id: String,
    pub session_id: Option<String>,
    pub kind: String,
    pub priority: i64,
    pub title: String,
    pub summary: String,
    pub action_label: Option<String>,
    pub action_ref: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}
```

Accept only `kind` values `approval`, `blocked`, `failed`, `done`, `unread`, `info`; reject anything else.

- [x] **Step 3: Implement commands**

Register:

```rust
attention_list
attention_create
attention_resolve
```

Sort unresolved items by:

1. `priority DESC`
2. `created_at ASC`

Resolved items are excluded from the default queue.

- [x] **Step 4: Add frontend types**

Create `src/features/attention/types.ts`:

```ts
export type AttentionKind =
  | "approval"
  | "blocked"
  | "failed"
  | "done"
  | "unread"
  | "info";

export type AttentionItem = {
  id: string;
  workspaceId: string;
  sessionId?: string | null;
  kind: AttentionKind;
  priority: 0 | 1 | 2 | 3;
  title: string;
  summary: string;
  actionLabel?: string | null;
  actionRef?: string | null;
  createdAt: string;
  resolvedAt?: string | null;
};
```

- [x] **Step 5: Add queue UI**

Create `AttentionQueue.tsx` with:

- title
- grouped count
- card per item
- badge for `kind`
- Resolve button
- empty state: `No attention items.`

- [x] **Step 6: Add manual test seed command**

During MVP only, add a development-only button or command path to create a sample `failed` item for the selected Workspace. Remove or hide this behind a dev flag before public release.

- [x] **Step 7: Verify**

Run:

```powershell
cd src-tauri
cargo test
cargo check
cd ..
npm run build
```

Manual check:

- Create sample attention item.
- It appears in right queue.
- Resolve removes it from active queue.
- Restart app; unresolved items persist.

- [x] **Step 8: Commit**

Run:

```powershell
git add src src-tauri
git commit -m "feat: add attention queue"
git push
```

---

### Task 6: Add Codex app-server Adapter

**Files:**
- Create: `src-tauri/src/codex/mod.rs`, `src-tauri/src/codex/jsonrpc.rs`, `src-tauri/src/codex/process.rs`
- Create: `src/features/codex/types.ts`, `src/features/codex/api.ts`
- Modify: `src-tauri/src/lib.rs`
- Test: `cargo test`, manual Codex CLI detection

**Interfaces:**
- Produces Rust commands:
  - `codex_detect() -> CodexAvailability`
  - `codex_thread_list(workspace_id: String) -> Vec<CodexThreadSummary>`
  - `codex_thread_start(workspace_id: String, prompt: String) -> CodexThreadSummary`
  - `codex_turn_send(thread_id: String, prompt: String) -> ()`
  - `codex_turn_interrupt(thread_id: String) -> ()`
- Produces Tauri events:
  - `codex://thread-updated`
  - `codex://item`
  - `codex://approval-requested`
  - `codex://turn-finished`

- [x] **Step 1: Implement Codex detection**

`codex_detect` runs:

```powershell
codex --version
```

Return:

```ts
type CodexAvailability = {
  available: boolean;
  version?: string;
  error?: string;
};
```

- [x] **Step 2: Create JSON-RPC helper**

Implement `jsonrpc.rs` with:

```rust
pub struct JsonRpcRequest<T> {
    pub id: u64,
    pub method: String,
    pub params: T,
}
```

Do not include a `"jsonrpc": "2.0"` field on the wire for Codex app-server messages unless current Codex testing proves it is required.

- [x] **Step 3: Start app-server process**

`process.rs` starts:

```powershell
codex app-server
```

Use piped stdin/stdout. Keep one app-server process per application instance for MVP. Store process handle in shared Rust state behind `tokio::sync::Mutex`.

- [x] **Step 4: Implement thread list/start/send**

Map commands:

- `codex_thread_list` -> `thread/list` with `cwd` equal to Workspace path
- `codex_thread_start` -> `thread/start` with Workspace path and MVP default sandbox/approval settings
- `codex_turn_send` -> `turn/start`
- `codex_turn_interrupt` -> `turn/interrupt`

If exact Codex params differ in current CLI, update adapter tests and keep frontend API stable.

- [x] **Step 5: Emit events**

Read app-server stdout continuously and emit known notifications to frontend. Unknown notifications should be logged and ignored, not crash the adapter.

- [x] **Step 6: Create frontend API and types**

Create `src/features/codex/types.ts`:

```ts
export type CodexThreadSummary = {
  id: string;
  workspaceId: string;
  title: string;
  status: "idle" | "running" | "blocked" | "failed" | "done";
  preview?: string;
  createdAt?: string;
  updatedAt?: string;
};
```

Create API wrappers in `src/features/codex/api.ts`.

- [x] **Step 7: Connect approval events to Attention**

When Rust sees a Codex approval request, create an `approval` Attention Item with priority `3`, title `Codex approval required`, and action ref containing thread id plus approval id.

- [x] **Step 8: Verify**

Run:

```powershell
codex --version
cd src-tauri
cargo test
cargo check
cd ..
npm run build
```

Manual check:

- If Codex CLI is missing, UI shows unavailable state.
- If Codex CLI exists, starting app-server does not freeze UI.

Implementation note 2026-06-26: on this Windows host, `codex --version` resolves to the WindowsApps Codex binary but exits with `Access is denied`; the MVP adapter therefore treats Codex as unavailable and keeps the app responsive. JSON-RPC method names and frontend APIs are isolated so exact app-server params can be updated inside `src-tauri/src/codex` when a runnable Codex CLI is available.

- [x] **Step 9: Commit**

Run:

```powershell
git add src src-tauri
git commit -m "feat: add codex app-server adapter"
git push
```

---

### Task 7: Build Codex Session View and Approval UI

**Files:**
- Create: `src/features/codex/CodexPanel.tsx`, `src/features/codex/CodexComposer.tsx`, `src/features/codex/CodexEventList.tsx`
- Create: `src/features/attention/ApprovalCard.tsx`
- Modify: `src/features/attention/AttentionQueue.tsx`, `src/app/layout/AppShell.tsx`
- Test: `npm run build`

**Interfaces:**
- Consumes Codex commands/events from Task 6.
- Consumes Attention Queue from Task 5.
- Produces user-facing Codex control surface.

- [x] **Step 1: Add Codex panel states**

CodexPanel must render:

- no Workspace selected
- Codex unavailable
- no thread selected
- running thread
- blocked on approval
- error state

- [x] **Step 2: Add composer**

`CodexComposer` has textarea and send button. Send calls `codex_thread_start` if no thread exists, otherwise `codex_turn_send`.

- [x] **Step 3: Add event list**

`CodexEventList` renders:

- user messages
- assistant text
- tool call summaries
- command output summaries
- collapsible raw JSON for unknown events in development only

Default: tool/log details collapsed.

- [x] **Step 4: Add approval card**

`ApprovalCard` displays:

- command or tool name
- workspace path
- requested scope
- Approve button
- Reject button

MVP can implement Approve/Reject as adapter calls once exact approval response method is confirmed. If the current Codex app-server approval method differs, isolate that change inside `src-tauri/src/codex`.

- [x] **Step 5: Wire Attention Queue actions**

When an `approval` item has `actionRef`, clicking the card selects the related Codex thread and scrolls to the approval context.

- [x] **Step 6: Verify**

Run:

```powershell
npm run build
npm run tauri dev
```

Manual check:

- Send a simple Codex prompt in a Workspace.
- UI remains responsive.
- Approval items appear in right queue when Codex requests user action.

Implementation note 2026-06-26: live prompt/approval smoke testing is blocked by the local Codex binary access denial above; automated tests cover unavailable UI state, thread start API wiring, approval card actions, event rendering, and approval-to-Attention mapping.

- [x] **Step 7: Commit**

Run:

```powershell
git add src src-tauri
git commit -m "feat: add codex session view"
git push
```

---

### Task 8: Add Terminal MVP

**Files:**
- Create: `src-tauri/src/terminal/mod.rs`
- Create: `src/features/terminal/types.ts`, `src/features/terminal/api.ts`, `src/features/terminal/TerminalPanel.tsx`
- Modify: `src-tauri/src/lib.rs`, `src/app/layout/AppShell.tsx`
- Test: `cargo check`, `npm run build`

**Interfaces:**
- Produces Rust commands:
  - `terminal_open(workspace_id: String) -> TerminalSession`
  - `terminal_write(session_id: String, data: String) -> ()`
  - `terminal_resize(session_id: String, cols: u16, rows: u16) -> ()`
  - `terminal_close(session_id: String) -> ()`
- Produces event `terminal://output`.

- [ ] **Step 1: Choose PTY crate**

Use a Windows-compatible PTY crate. Preferred default: `portable-pty` unless implementation testing proves it incompatible with Tauri async handling.

Run:

```powershell
cd src-tauri
cargo add portable-pty
cd ..
```

- [ ] **Step 2: Implement terminal session registry**

Store terminal sessions in Rust state:

```rust
HashMap<String, TerminalHandle>
```

Each session has id, workspace id, cwd, cols, rows, and process handle.

- [ ] **Step 3: Open PowerShell by default on Windows**

For MVP, `terminal_open` starts:

```powershell
powershell.exe
```

with cwd set to Workspace path.

- [ ] **Step 4: Stream output to frontend**

Read PTY output and emit:

```ts
type TerminalOutputEvent = {
  sessionId: string;
  data: string;
};
```

on `terminal://output`.

- [ ] **Step 5: Build xterm panel**

`TerminalPanel.tsx` uses:

```ts
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
```

Load `FitAddon` with `term.loadAddon(fitAddon)` and call `fitAddon.fit()` after mount and on resize.

- [ ] **Step 6: Convert terminal failures into Attention Items**

For MVP, detect obvious command failure only when a user runs a command through future shortcuts. Raw interactive terminal output should not create Attention Items yet to avoid false positives.

- [ ] **Step 7: Verify**

Run:

```powershell
cd src-tauri
cargo check
cd ..
npm run build
npm run tauri dev
```

Manual check:

- Open terminal.
- Run `git status`.
- Run `node --version`.
- Resize app; terminal remains usable.
- Close app; terminal process exits.

- [ ] **Step 8: Commit**

Run:

```powershell
git add src src-tauri package.json package-lock.json
git commit -m "feat: add integrated terminal"
git push
```

---

### Task 9: Add Git Status and Diff MVP

**Files:**
- Create: `src-tauri/src/git/mod.rs`
- Create: `src/features/git/types.ts`, `src/features/git/api.ts`, `src/features/git/GitPanel.tsx`
- Modify: `src-tauri/src/lib.rs`, `src/app/layout/AppShell.tsx`
- Test: `cargo test`, `npm run build`

**Interfaces:**
- Produces Rust commands:
  - `git_status(workspace_id: String) -> GitStatus`
  - `git_diff(workspace_id: String, path: Option<String>) -> GitDiff`
  - `git_stage(workspace_id: String, path: String) -> ()`
  - `git_unstage(workspace_id: String, path: String) -> ()`

- [ ] **Step 1: Define types**

Create TypeScript shape:

```ts
export type GitFileStatus = {
  path: string;
  indexStatus: string;
  worktreeStatus: string;
};

export type GitStatus = {
  available: boolean;
  branch?: string;
  ahead: number;
  behind: number;
  files: GitFileStatus[];
  error?: string;
};

export type GitDiff = {
  path?: string;
  text: string;
};
```

Mirror in Rust with `serde`.

- [ ] **Step 2: Implement status command**

Run:

```powershell
git -C <gitRoot> status --porcelain=v1 --branch
```

Behavior:

- no `gitRoot`: return `available=false`
- git command failure: return `available=false` with error
- success: parse branch and file lines

- [ ] **Step 3: Implement diff command**

Run:

```powershell
git -C <gitRoot> diff -- <path>
```

If `path` is absent, run:

```powershell
git -C <gitRoot> diff
```

- [ ] **Step 4: Implement stage/unstage**

Run:

```powershell
git -C <gitRoot> add -- <path>
git -C <gitRoot> restore --staged -- <path>
```

Never run destructive commands such as reset or checkout in MVP Git UI.

- [ ] **Step 5: Build Git panel**

`GitPanel` shows:

- branch
- ahead/behind
- changed files
- selected file diff
- stage/unstage buttons
- empty state

- [ ] **Step 6: Connect Git conflicts to Attention**

If status output includes unmerged states, create a `blocked` Attention Item with priority `3` and title `Git conflict requires attention`.

- [ ] **Step 7: Verify**

Run:

```powershell
cd src-tauri
cargo test
cargo check
cd ..
npm run build
```

Manual check:

- Modify a tracked file.
- Git panel shows it.
- Diff renders.
- Stage and unstage work.

- [ ] **Step 8: Commit**

Run:

```powershell
git add src src-tauri
git commit -m "feat: add git status and diff"
git push
```

---

### Task 10: MVP Integration, Verification, and Developer Handoff

**Files:**
- Create: `docs/mvp-verification.md`
- Modify: `README.md`
- Modify: any files needed to fix integration defects
- Test: full MVP commands below

**Interfaces:**
- Consumes all Tasks 1-9.
- Produces a coherent MVP checkpoint ready for the next implementation phase.

- [ ] **Step 1: Add README**

Create `README.md` with:

- product one-liner
- private MVP status
- prerequisites
- development commands
- privacy boundary
- link to design doc and ADRs

- [ ] **Step 2: Add MVP verification doc**

Create `docs/mvp-verification.md` with manual scenarios:

```md
# MVP Verification

## Scenario 1: Workspace
- Add a Git workspace.
- Restart app.
- Workspace persists.

## Scenario 2: Attention Queue
- Create a sample failed item.
- Resolve it.
- Restart app.
- Resolved item stays hidden.

## Scenario 3: Terminal
- Open terminal.
- Run git status.
- Resize app.
- Close app.

## Scenario 4: Git
- Modify a tracked file.
- View diff.
- Stage and unstage file.

## Scenario 5: Codex
- Detect Codex CLI.
- Start or list a Codex Thread.
- Send a prompt.
- Interrupt a running turn.
```

- [ ] **Step 3: Run full automated checks**

Run:

```powershell
npm run build
cd src-tauri
cargo test
cargo check
cd ..
```

Expected: all pass.

- [ ] **Step 4: Run desktop smoke test**

Run:

```powershell
npm run tauri dev
```

Manual expected:

- app opens
- layout stable
- workspace list loads
- attention queue loads
- terminal opens
- git panel loads for Git workspaces
- Codex unavailable state or Codex connected state is clear

- [ ] **Step 5: Check process cleanup**

After closing the app, run:

```powershell
Get-Process | Where-Object { $_.ProcessName -match 'codex|powershell|pwsh' } | Select-Object ProcessName,Id,StartTime
```

Expected: no vibe-monitor-owned Codex app-server or terminal child process remains. Existing unrelated user shells are acceptable.

- [ ] **Step 6: Commit**

Run:

```powershell
git add README.md docs src src-tauri package.json package-lock.json
git commit -m "docs: add mvp verification guide"
git push
```

---

## Acceptance Criteria

The MVP is complete when:

- A Windows user can launch the Tauri app locally.
- A Workspace can be added, persisted, listed, and removed.
- The app can detect Codex CLI and either connect to Codex app-server or show a clear unavailable state.
- A Codex Thread can be started or resumed through the app when Codex is available.
- Codex approval requests become high-priority Attention Items.
- The integrated terminal can run commands in the selected Workspace.
- Git status and diff can be viewed for Git Workspaces.
- Git stage/unstage works and no destructive Git action is exposed.
- The Attention Queue persists unresolved items and hides resolved items.
- `npm run build`, `cargo test`, and `cargo check` pass.
- Closing the app cleans up owned child processes.
