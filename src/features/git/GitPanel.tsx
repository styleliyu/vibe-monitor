import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  AlertTriangle,
  GitBranch,
  GitCompare,
  Loader2,
  RefreshCw,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/shared/lib/cn";
import {
  getGitDiff,
  getGitStatus,
  stageGitFile,
  unstageGitFile,
} from "./api";
import type { GitFileStatus } from "./types";

type GitPanelProps = {
  workspaceId?: string | null;
};

export function GitPanel({ workspaceId }: GitPanelProps) {
  const queryClient = useQueryClient();
  const [selectedPath, setSelectedPath] = useState<string | null>(null);

  const statusQuery = useQuery({
    enabled: Boolean(workspaceId),
    queryKey: ["git", "status", workspaceId],
    queryFn: () => getGitStatus(workspaceId ?? ""),
  });

  const files = statusQuery.data?.files ?? [];

  useEffect(() => {
    if (!files.some((file) => file.path === selectedPath)) {
      setSelectedPath(files[0]?.path ?? null);
    }
  }, [files, selectedPath]);

  const diffQuery = useQuery({
    enabled: Boolean(workspaceId && selectedPath && statusQuery.data?.available),
    queryKey: ["git", "diff", workspaceId, selectedPath],
    queryFn: () => getGitDiff(workspaceId ?? "", selectedPath ?? undefined),
  });

  const stage = useMutation({
    mutationFn: (path: string) => stageGitFile(workspaceId ?? "", path),
    onSuccess: () => invalidateGitQueries(queryClient, workspaceId),
  });

  const unstage = useMutation({
    mutationFn: (path: string) => unstageGitFile(workspaceId ?? "", path),
    onSuccess: () => invalidateGitQueries(queryClient, workspaceId),
  });

  const selectedFile = useMemo(
    () => files.find((file) => file.path === selectedPath) ?? null,
    [files, selectedPath],
  );

  function handleStage() {
    if (selectedPath) {
      stage.mutate(selectedPath);
    }
  }

  function handleUnstage() {
    if (selectedPath) {
      unstage.mutate(selectedPath);
    }
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex h-10 shrink-0 items-center gap-2 border-b px-4 text-sm font-medium">
        <GitBranch className="size-4" />
        <span>Git</span>
        {statusQuery.data?.branch ? (
          <Badge className="max-w-[128px] truncate" variant="secondary">
            {statusQuery.data.branch}
          </Badge>
        ) : null}
        {statusQuery.data?.available ? (
          <span className="text-xs text-muted-foreground">
            +{statusQuery.data.ahead} / -{statusQuery.data.behind}
          </span>
        ) : null}
        <Button
          aria-label="Refresh Git status"
          className="ml-auto"
          disabled={!workspaceId || statusQuery.isFetching}
          onClick={() => statusQuery.refetch()}
          size="icon"
          type="button"
          variant="ghost"
        >
          {statusQuery.isFetching ? (
            <Loader2 className="size-4 animate-spin" />
          ) : (
            <RefreshCw className="size-4" />
          )}
        </Button>
      </div>

      {!workspaceId ? (
        <StateMessage text="Select a workspace to inspect Git changes." />
      ) : null}

      {workspaceId && statusQuery.isLoading ? (
        <StateMessage icon="loading" text="Loading Git status..." />
      ) : null}

      {workspaceId && statusQuery.error ? (
        <StateMessage
          icon="warning"
          text="Git status failed"
          detail={String(statusQuery.error)}
        />
      ) : null}

      {workspaceId && statusQuery.data && !statusQuery.data.available ? (
        <StateMessage
          icon="warning"
          text="Git unavailable"
          detail={statusQuery.data.error ?? "Git status could not be loaded."}
        />
      ) : null}

      {workspaceId && statusQuery.data?.available ? (
        <div className="grid min-h-0 flex-1 grid-rows-[112px_minmax(0,1fr)]">
          <ScrollArea className="border-b">
            <div className="space-y-1 p-2">
              {files.length === 0 ? (
                <p className="px-2 py-4 text-sm text-muted-foreground">No Git changes.</p>
              ) : null}
              {files.map((file) => (
                <GitFileButton
                  file={file}
                  isSelected={file.path === selectedPath}
                  key={`${file.indexStatus}${file.worktreeStatus}${file.path}`}
                  onClick={() => setSelectedPath(file.path)}
                />
              ))}
            </div>
          </ScrollArea>

          <div className="flex min-h-0 flex-col">
            <div className="flex h-10 shrink-0 items-center gap-2 border-b px-3">
              <GitCompare className="size-4 text-muted-foreground" />
              <span className="min-w-0 flex-1 truncate text-xs text-muted-foreground">
                {selectedPath ?? "No file selected"}
              </span>
              <Button
                disabled={!selectedPath || stage.isPending}
                onClick={handleStage}
                size="sm"
                type="button"
                variant="outline"
              >
                Stage
              </Button>
              <Button
                disabled={!selectedPath || unstage.isPending}
                onClick={handleUnstage}
                size="sm"
                type="button"
                variant="outline"
              >
                Unstage
              </Button>
            </div>

            {stage.error ? (
              <p className="border-b px-3 py-2 text-xs text-destructive">{String(stage.error)}</p>
            ) : null}
            {unstage.error ? (
              <p className="border-b px-3 py-2 text-xs text-destructive">{String(unstage.error)}</p>
            ) : null}

            <ScrollArea className="min-h-0 flex-1">
              {diffQuery.isFetching ? (
                <p className="p-3 text-sm text-muted-foreground">Loading diff...</p>
              ) : null}
              {diffQuery.error ? (
                <p className="p-3 text-sm text-destructive">{String(diffQuery.error)}</p>
              ) : null}
              {!diffQuery.isFetching && !diffQuery.error ? (
                <pre className="whitespace-pre-wrap break-words p-3 font-mono text-[11px] leading-5 text-muted-foreground">
                  {diffQuery.data?.text ||
                    (selectedFile ? "No unstaged diff for this file." : "Select a file to view diff.")}
                </pre>
              ) : null}
            </ScrollArea>
          </div>
        </div>
      ) : null}
    </div>
  );
}

function GitFileButton({
  file,
  isSelected,
  onClick,
}: {
  file: GitFileStatus;
  isSelected: boolean;
  onClick: () => void;
}) {
  return (
    <button
      aria-label={file.path}
      className={cn(
        "flex h-8 w-full items-center gap-2 rounded-md px-2 text-left text-xs hover:bg-accent",
        isSelected && "bg-accent text-accent-foreground",
      )}
      onClick={onClick}
      type="button"
    >
      <span className="w-8 shrink-0 font-mono text-muted-foreground">
        {file.indexStatus}
        {file.worktreeStatus}
      </span>
      <span className="min-w-0 flex-1 truncate">{file.path}</span>
    </button>
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
    <div className="flex min-h-0 flex-1 flex-col items-center justify-center gap-2 px-4 text-center">
      {icon === "loading" ? <Loader2 className="size-5 animate-spin text-muted-foreground" /> : null}
      {icon === "warning" ? <AlertTriangle className="size-5 text-amber-600" /> : null}
      <p className="text-sm font-medium">{text}</p>
      {detail ? <p className="max-w-[240px] text-sm text-muted-foreground">{detail}</p> : null}
    </div>
  );
}

function invalidateGitQueries(
  queryClient: ReturnType<typeof useQueryClient>,
  workspaceId?: string | null,
) {
  void queryClient.invalidateQueries({ queryKey: ["git", "status", workspaceId] });
  void queryClient.invalidateQueries({ queryKey: ["git", "diff", workspaceId] });
}
