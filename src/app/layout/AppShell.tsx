import { useState } from "react";
import { GitBranch, Monitor, Terminal } from "lucide-react";
import { AttentionQueue } from "@/features/attention/AttentionQueue";
import { CodexPanel } from "@/features/codex/CodexPanel";
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
