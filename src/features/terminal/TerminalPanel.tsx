import { FitAddon } from "@xterm/addon-fit";
import { Terminal as XtermTerminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";
import { listen } from "@tauri-apps/api/event";
import { Loader2, Power, SquareTerminal, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useI18n } from "@/shared/i18n";
import {
  closeTerminal,
  openTerminal,
  resizeTerminal,
  writeTerminal,
} from "./api";
import type { TerminalOutputEvent, TerminalSession } from "./types";

type TerminalPanelProps = {
  workspaceId?: string | null;
};

export function TerminalPanel({ workspaceId }: TerminalPanelProps) {
  const { t } = useI18n();
  const containerRef = useRef<HTMLDivElement | null>(null);
  const xtermRef = useRef<XtermTerminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const sessionIdRef = useRef<string | null>(null);
  const [session, setSession] = useState<TerminalSession | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isOpening, setIsOpening] = useState(false);

  useEffect(() => {
    if (typeof window !== "undefined" && !("__TAURI_INTERNALS__" in window)) {
      return;
    }

    let unlisten: (() => void) | null = null;
    let cancelled = false;

    listen<TerminalOutputEvent>("terminal://output", (event) => {
      const payload = event.payload;
      if (payload.sessionId !== sessionIdRef.current) {
        return;
      }
      xtermRef.current?.write(payload.data);
    }).then((nextUnlisten) => {
      if (cancelled) {
        nextUnlisten();
      } else {
        unlisten = nextUnlisten;
      }
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    function fit() {
      fitAddonRef.current?.fit();
      const terminal = xtermRef.current;
      const sessionId = sessionIdRef.current;
      if (!terminal || !sessionId) {
        return;
      }
      const dimensions = terminal as XtermTerminal & { cols?: number; rows?: number };
      if (dimensions.cols && dimensions.rows) {
        void resizeTerminal(sessionId, dimensions.cols, dimensions.rows);
      }
    }

    window.addEventListener("resize", fit);
    return () => window.removeEventListener("resize", fit);
  }, []);

  useEffect(() => {
    if (!workspaceId && sessionIdRef.current) {
      void handleClose();
    }
  }, [workspaceId]);

  async function handleOpen() {
    if (!workspaceId || !containerRef.current || isOpening) {
      return;
    }

    setIsOpening(true);
    setError(null);
    try {
      const nextSession = await openTerminal(workspaceId);
      teardownXterm();

      const terminal = new XtermTerminal({
        convertEol: true,
        cursorBlink: true,
        fontFamily:
          'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace',
        fontSize: 13,
        scrollback: 4000,
        theme: {
          background: "#0a0a0a",
          foreground: "#f4f4f5",
        },
      });
      const fitAddon = new FitAddon();
      terminal.loadAddon(fitAddon);
      terminal.open(containerRef.current);
      terminal.onData((data) => {
        void writeTerminal(nextSession.id, data).catch((writeError) => {
          setError(String(writeError));
        });
      });
      fitAddon.fit();

      xtermRef.current = terminal;
      fitAddonRef.current = fitAddon;
      sessionIdRef.current = nextSession.id;
      setSession(nextSession);

      const dimensions = terminal as XtermTerminal & { cols?: number; rows?: number };
      if (dimensions.cols && dimensions.rows) {
        void resizeTerminal(nextSession.id, dimensions.cols, dimensions.rows);
      }
    } catch (openError) {
      setError(String(openError));
    } finally {
      setIsOpening(false);
    }
  }

  async function handleClose() {
    const sessionId = sessionIdRef.current;
    sessionIdRef.current = null;
    setSession(null);
    teardownXterm();

    if (sessionId) {
      try {
        await closeTerminal(sessionId);
      } catch (closeError) {
        setError(String(closeError));
      }
    }
  }

  function teardownXterm() {
    xtermRef.current?.dispose();
    xtermRef.current = null;
    fitAddonRef.current = null;
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex h-10 shrink-0 items-center gap-2 border-b px-4 text-sm font-medium">
        <SquareTerminal className="size-4" />
        <span>{t("terminal.title")}</span>
        {session ? (
          <Badge className="max-w-[220px] truncate" variant="secondary">
            {session.shell}
          </Badge>
        ) : null}
        <div className="ml-auto flex items-center gap-2">
          {session ? (
            <Button aria-label={t("terminal.close")} size="icon" variant="ghost" onClick={handleClose}>
              <X className="size-4" />
            </Button>
          ) : (
            <Button
              disabled={!workspaceId || isOpening}
              size="sm"
              variant="secondary"
              onClick={handleOpen}
            >
              {isOpening ? (
                <Loader2 className="size-4 animate-spin" />
              ) : (
                <Power className="size-4" />
              )}
              {t("terminal.open")}
            </Button>
          )}
        </div>
      </div>

      {!workspaceId ? (
        <div className="flex flex-1 items-center justify-center px-4 text-sm text-muted-foreground">
          {t("terminal.selectWorkspace")}
        </div>
      ) : null}

      {error ? <p className="border-b px-4 py-2 text-sm text-destructive">{error}</p> : null}

      <div
        ref={containerRef}
        className="min-h-0 flex-1 bg-[#0a0a0a] px-2 py-2"
        aria-label="Terminal viewport"
      />
    </div>
  );
}
