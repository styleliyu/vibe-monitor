import { useState } from "react";
import { Monitor } from "lucide-react";
import { AttentionQueue } from "@/features/attention/AttentionQueue";
import { CodexPanel } from "@/features/codex/CodexPanel";
import { GitPanel } from "@/features/git/GitPanel";
import { TerminalPanel } from "@/features/terminal/TerminalPanel";
import { WorkspaceSidebar } from "@/features/workspace/WorkspaceSidebar";

export function AppShell() {
  const [selectedWorkspaceId, setSelectedWorkspaceId] = useState<string | null>(null);
  const [selectedCodexThreadId, setSelectedCodexThreadId] = useState<string | null>(null);

  return (
    <div className="grid h-screen min-h-0 grid-cols-[280px_minmax(0,1fr)_360px] grid-rows-[48px_minmax(0,1fr)_240px] bg-background text-foreground">
      <header className="col-span-3 flex items-center gap-3 border-b px-4">
        <Monitor className="size-5" />
        <span className="font-semibold">vibe-monitor</span>
        <div className="ml-auto text-sm text-muted-foreground">MVP Cockpit</div>
      </header>

      <aside className="row-span-2 min-h-0 border-r">
        <WorkspaceSidebar onWorkspaceSelect={setSelectedWorkspaceId} />
      </aside>

      <main className="min-w-0">
        <CodexPanel selectedThreadId={selectedCodexThreadId} workspaceId={selectedWorkspaceId} />
      </main>

      <aside className="row-span-2 min-h-0 border-l">
        <AttentionQueue
          onAction={(item) => {
            const threadId = item.actionRef?.match(/^codex:\/\/thread\/([^/]+)/)?.[1];
            if (threadId) {
              setSelectedCodexThreadId(threadId);
            }
          }}
          workspaceId={selectedWorkspaceId}
        />
      </aside>

      <section className="col-start-2 row-start-3 grid min-h-0 grid-cols-[minmax(0,1fr)_320px] border-t">
        <TerminalPanel workspaceId={selectedWorkspaceId} />

        <div className="min-h-0 border-l">
          <GitPanel workspaceId={selectedWorkspaceId} />
        </div>
      </section>
    </div>
  );
}
