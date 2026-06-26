import { useMutation, useQuery } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { AlertTriangle, Bot, CheckCircle2, Loader2 } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { Badge } from "@/components/ui/badge";
import {
  detectCodex,
  interruptCodexTurn,
  sendCodexTurn,
  startCodexThread,
} from "./api";
import { CodexComposer } from "./CodexComposer";
import { CodexEventList } from "./CodexEventList";
import type { CodexEvent, CodexThreadSummary } from "./types";

type CodexPanelProps = {
  selectedThreadId?: string | null;
  workspaceId?: string | null;
};

export function CodexPanel({ selectedThreadId, workspaceId }: CodexPanelProps) {
  const [thread, setThread] = useState<CodexThreadSummary | null>(null);
  const [events, setEvents] = useState<CodexEvent[]>([]);
  const [panelError, setPanelError] = useState<string | null>(null);

  const detection = useQuery({
    enabled: Boolean(workspaceId),
    queryKey: ["codex", "detect"],
    queryFn: detectCodex,
  });

  const startThread = useMutation({
    mutationFn: (prompt: string) => startCodexThread(workspaceId ?? "", prompt),
    onSuccess: (nextThread) => {
      setThread(nextThread);
      setPanelError(null);
      setEvents((current) => [
        ...current,
        {
          id: `user-${Date.now()}`,
          kind: "user",
          title: nextThread.title,
          body: nextThread.preview ?? undefined,
        },
      ]);
    },
    onError: (error) => setPanelError(String(error)),
  });

  const sendTurn = useMutation({
    mutationFn: (prompt: string) => sendCodexTurn(thread?.id ?? "", prompt),
    onSuccess: (_, prompt) => {
      setPanelError(null);
      setEvents((current) => [
        ...current,
        {
          id: `user-${Date.now()}`,
          kind: "user",
          title: "User instruction",
          body: prompt,
        },
      ]);
    },
    onError: (error) => setPanelError(String(error)),
  });

  const interruptTurn = useMutation({
    mutationFn: () => interruptCodexTurn(thread?.id ?? selectedThreadId ?? ""),
    onError: (error) => setPanelError(String(error)),
  });

  useEffect(() => {
    if (typeof window !== "undefined" && !("__TAURI_INTERNALS__" in window)) {
      return;
    }

    const unlistenFns: Array<() => void> = [];
    let cancelled = false;

    async function bindEvents() {
      const pairs: Array<[string, CodexEvent["kind"], string]> = [
        ["codex://item", "assistant", "Codex item"],
        ["codex://approval-requested", "approval", "Approval requested"],
        ["codex://thread-updated", "assistant", "Thread updated"],
        ["codex://turn-finished", "assistant", "Turn finished"],
      ];

      for (const [eventName, kind, title] of pairs) {
        const unlisten = await listen(eventName, (event) => {
          setEvents((current) => [
            ...current,
            {
              id: `${eventName}-${Date.now()}-${current.length}`,
              kind,
              title,
              body: summarizePayload(event.payload),
              raw: event.payload,
            },
          ]);
          if (kind === "approval") {
            setThread((current) =>
              current ? { ...current, status: "blocked" } : current,
            );
          }
        });
        if (cancelled) {
          unlisten();
        } else {
          unlistenFns.push(unlisten);
        }
      }
    }

    bindEvents();
    return () => {
      cancelled = true;
      unlistenFns.forEach((unlisten) => unlisten());
    };
  }, []);

  const status = useMemo(() => {
    if (!workspaceId) {
      return "idle";
    }
    if (thread?.status) {
      return thread.status;
    }
    return "idle";
  }, [thread?.status, workspaceId]);

  function handleSend(prompt: string) {
    if (!thread) {
      startThread.mutate(prompt);
      return;
    }
    sendTurn.mutate(prompt);
  }

  const unavailable = workspaceId && detection.data && !detection.data.available;
  const disabled =
    !workspaceId ||
    detection.isLoading ||
    Boolean(unavailable) ||
    startThread.isPending ||
    sendTurn.isPending;

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex h-12 shrink-0 items-center gap-2 border-b px-4 text-sm font-medium">
        <Bot className="size-4" />
        <span>Codex Session</span>
        <Badge className="ml-auto" variant={status === "blocked" ? "destructive" : "secondary"}>
          {status}
        </Badge>
      </div>

      {!workspaceId ? (
        <StateMessage text="Select a workspace to start a Codex session." />
      ) : null}

      {workspaceId && detection.isLoading ? (
        <StateMessage icon="loading" text="Checking Codex CLI..." />
      ) : null}

      {unavailable ? (
        <StateMessage
          icon="warning"
          text="Codex CLI unavailable"
          detail={detection.data?.error ?? "Codex could not be detected."}
        />
      ) : null}

      {workspaceId && detection.data?.available ? (
        <>
          <div className="flex h-10 shrink-0 items-center gap-2 border-b px-4 text-xs text-muted-foreground">
            <CheckCircle2 className="size-4 text-emerald-600" />
            <span>{detection.data.version ?? "Codex available"}</span>
            {selectedThreadId ? <span className="ml-auto">Selected {selectedThreadId}</span> : null}
          </div>
          {panelError ? (
            <p className="border-b px-4 py-2 text-sm text-destructive">{panelError}</p>
          ) : null}
          <CodexEventList events={events} />
        </>
      ) : null}

      <CodexComposer
        disabled={disabled}
        isRunning={thread?.status === "running"}
        onInterrupt={() => interruptTurn.mutate()}
        onSend={handleSend}
      />
    </div>
  );
}

function StateMessage({
  detail,
  icon,
  text,
}: {
  detail?: string;
  icon?: "loading" | "warning";
  text: string;
}) {
  return (
    <div className="flex min-h-0 flex-1 flex-col items-center justify-center gap-2 px-6 text-center">
      {icon === "loading" ? <Loader2 className="size-5 animate-spin text-muted-foreground" /> : null}
      {icon === "warning" ? <AlertTriangle className="size-5 text-amber-600" /> : null}
      <p className="text-sm font-medium">{text}</p>
      {detail ? <p className="max-w-md text-sm text-muted-foreground">{detail}</p> : null}
    </div>
  );
}

function summarizePayload(payload: unknown) {
  if (typeof payload === "string") {
    return payload;
  }
  if (payload && typeof payload === "object") {
    const value = payload as Record<string, unknown>;
    return String(value.message ?? value.text ?? value.summary ?? JSON.stringify(payload));
  }
  return undefined;
}
