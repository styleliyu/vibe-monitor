import { Bot, Code2, TerminalSquare, UserRound } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useI18n } from "@/shared/i18n";
import type { CodexEvent } from "./types";

type CodexEventListProps = {
  events: CodexEvent[];
};

const iconByKind = {
  user: UserRound,
  assistant: Bot,
  tool: Code2,
  command: TerminalSquare,
  approval: Code2,
  unknown: Code2,
} satisfies Record<CodexEvent["kind"], typeof Bot>;

export function CodexEventList({ events }: CodexEventListProps) {
  const { t } = useI18n();

  if (events.length === 0) {
    return (
      <div className="flex min-h-0 flex-1 items-center justify-center px-4 text-sm text-muted-foreground">
        {t("codex.noEvents")}
      </div>
    );
  }

  return (
    <ScrollArea className="min-h-0 flex-1">
      <div className="space-y-3 p-4">
        {events.map((event) => {
          const Icon = iconByKind[event.kind];
          return (
            <article className="rounded-md border bg-card p-3 text-card-foreground" key={event.id}>
              <div className="flex items-center gap-2">
                <Icon className="size-4 text-muted-foreground" />
                <h3 className="min-w-0 truncate text-sm font-medium">{event.title}</h3>
                <Badge className="ml-auto" variant="secondary">
                  {event.kind}
                </Badge>
              </div>
              {event.body ? (
                <p className="mt-2 whitespace-pre-wrap text-sm leading-6 text-muted-foreground">
                  {event.body}
                </p>
              ) : null}
              {import.meta.env.DEV && event.kind === "unknown" && event.raw ? (
                <details className="mt-2 text-xs text-muted-foreground">
                  <summary>{t("codex.rawJson")}</summary>
                  <pre className="mt-2 overflow-auto rounded bg-muted p-2">
                    {JSON.stringify(event.raw, null, 2)}
                  </pre>
                </details>
              ) : null}
            </article>
          );
        })}
      </div>
    </ScrollArea>
  );
}
