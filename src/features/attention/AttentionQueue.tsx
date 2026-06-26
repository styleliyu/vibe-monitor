import { AlertCircle, Bell, Check, Plus } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useI18n } from "@/shared/i18n";
import { cn } from "@/shared/lib/cn";
import type { AttentionItem, AttentionKind } from "./types";
import { useAttentionQueue } from "./useAttentionQueue";

type AttentionQueueProps = {
  onAction?: (item: AttentionItem) => void;
  workspaceId?: string | null;
};

const kindTone: Record<AttentionKind, string> = {
  approval: "border-amber-500/30 bg-amber-500/10 text-amber-700",
  blocked: "border-red-500/30 bg-red-500/10 text-red-700",
  failed: "border-red-500/30 bg-red-500/10 text-red-700",
  done: "border-emerald-500/30 bg-emerald-500/10 text-emerald-700",
  unread: "border-sky-500/30 bg-sky-500/10 text-sky-700",
  info: "border-zinc-500/30 bg-zinc-500/10 text-zinc-700",
};

export function AttentionQueue({ onAction, workspaceId }: AttentionQueueProps) {
  const { t } = useI18n();
  const {
    data: items = [],
    isLoading,
    error,
    createAttentionItem,
    isCreatingAttentionItem,
    resolveAttentionItem,
    isResolvingAttentionItem,
    createAttentionItemError,
    resolveAttentionItemError,
  } = useAttentionQueue(workspaceId);

  function handleCreateSample() {
    if (!workspaceId) {
      return;
    }

    createAttentionItem({
      workspaceId,
      kind: "failed",
      priority: 3,
      title: t("attention.sampleTitle"),
      summary: t("attention.sampleSummary"),
      actionLabel: t("attention.sampleAction"),
      actionRef: "dev-seed",
    });
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex h-12 shrink-0 items-center gap-2 border-b px-3 text-sm font-medium">
        <Bell className="size-4" />
        <span>{t("attention.title")}</span>
        <Badge className="ml-auto" variant="secondary">
          {t("attention.activeCount", { count: items.length })}
        </Badge>
      </div>

      {import.meta.env.DEV ? (
        <div className="border-b p-3">
          <Button
            className="w-full"
            disabled={!workspaceId || isCreatingAttentionItem}
            onClick={handleCreateSample}
            size="sm"
            type="button"
            variant="outline"
          >
            <Plus className="size-4" />
            {t("attention.seed")}
          </Button>
        </div>
      ) : null}

      <ScrollArea className="min-h-0 flex-1">
        <div className="space-y-2 p-3">
          {isLoading ? (
            <p className="text-sm text-muted-foreground">{t("attention.loading")}</p>
          ) : null}

          {error ? <p className="text-sm text-destructive">{String(error)}</p> : null}
          {createAttentionItemError ? (
            <p className="text-sm text-destructive">{String(createAttentionItemError)}</p>
          ) : null}
          {resolveAttentionItemError ? (
            <p className="text-sm text-destructive">{String(resolveAttentionItemError)}</p>
          ) : null}

          {!isLoading && !error && items.length === 0 ? (
            <p className="text-sm text-muted-foreground">{t("attention.empty")}</p>
          ) : null}

          {items.map((item) => (
            <AttentionCard
              isResolving={isResolvingAttentionItem}
              item={item}
              key={item.id}
              onAction={onAction ? () => onAction(item) : undefined}
              onResolve={() => resolveAttentionItem(item.id)}
              resolveAriaLabel={t("attention.resolveAria", { title: item.title })}
              resolveLabel={t("attention.resolve")}
            />
          ))}
        </div>
      </ScrollArea>
    </div>
  );
}

function AttentionCard({
  isResolving,
  item,
  onAction,
  onResolve,
  resolveAriaLabel,
  resolveLabel,
}: {
  isResolving: boolean;
  item: AttentionItem;
  onAction?: () => void;
  onResolve: () => void;
  resolveAriaLabel: string;
  resolveLabel: string;
}) {
  return (
    <article className="rounded-md border bg-card p-3 text-card-foreground shadow-xs">
      <div className="flex items-start gap-2">
        <AlertCircle className="mt-0.5 size-4 shrink-0 text-muted-foreground" />
        <div className="min-w-0 flex-1">
          <div className="flex min-w-0 items-center gap-2">
            <h3 className="truncate text-sm font-medium">{item.title}</h3>
            <Badge className="ml-auto shrink-0" variant="outline">
              P{item.priority}
            </Badge>
          </div>
          <p className="mt-1 line-clamp-3 text-xs leading-5 text-muted-foreground">
            {item.summary}
          </p>
        </div>
      </div>

      <div className="mt-3 flex items-center gap-2">
        <Badge className={cn("border", kindTone[item.kind])} variant="outline">
          {item.kind}
        </Badge>
        {item.actionLabel ? (
          <Button
            className="h-auto min-w-0 truncate px-1 text-xs text-muted-foreground"
            disabled={!onAction}
            onClick={onAction}
            size="sm"
            type="button"
            variant="link"
          >
            {item.actionLabel}
          </Button>
        ) : null}
        <Button
          aria-label={resolveAriaLabel}
          className="ml-auto"
          disabled={isResolving}
          onClick={onResolve}
          size="sm"
          type="button"
          variant="ghost"
        >
          <Check className="size-4" />
          {resolveLabel}
        </Button>
      </div>
    </article>
  );
}
