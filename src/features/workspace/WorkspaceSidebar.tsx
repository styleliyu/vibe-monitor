import { Plus, Trash2 } from "lucide-react";
import { type FormEvent, useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useI18n } from "@/shared/i18n";
import { cn } from "@/shared/lib/cn";
import { useWorkspaces } from "./useWorkspaces";

type WorkspaceSidebarProps = {
  onWorkspaceSelect?: (workspaceId: string | null) => void;
};

export function WorkspaceSidebar({ onWorkspaceSelect }: WorkspaceSidebarProps) {
  const { t } = useI18n();
  const [path, setPath] = useState("");
  const [selectedWorkspaceId, setSelectedWorkspaceId] = useState<string | null>(null);
  const {
    data: workspaces = [],
    isLoading,
    error,
    addWorkspace,
    removeWorkspace,
    isAddingWorkspace,
    addWorkspaceError,
  } = useWorkspaces();

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const trimmedPath = path.trim();
    if (!trimmedPath) {
      return;
    }
    addWorkspace(
      { path: trimmedPath, name: undefined },
      {
        onSuccess: (workspace) => {
          setPath("");
          setSelectedWorkspaceId(workspace.id);
          onWorkspaceSelect?.(workspace.id);
        },
      },
    );
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex h-12 shrink-0 items-center gap-2 border-b px-3 text-sm font-medium">
        {t("workspace.title")}
      </div>

      <form className="space-y-2 border-b p-3" onSubmit={handleSubmit}>
        <label className="text-xs font-medium text-muted-foreground" htmlFor="workspace-path">
          {t("workspace.path")}
        </label>
        <div className="flex gap-2">
          <Input
            id="workspace-path"
            value={path}
            onChange={(event) => setPath(event.currentTarget.value)}
            placeholder="D:\AIdeas\CodingMonitor"
          />
          <Button
            aria-label={t("workspace.add")}
            disabled={isAddingWorkspace || path.trim().length === 0}
            size="icon"
            type="submit"
          >
            <Plus className="size-4" />
          </Button>
        </div>
        {addWorkspaceError ? (
          <p className="text-xs text-destructive">{String(addWorkspaceError)}</p>
        ) : null}
      </form>

      <ScrollArea className="min-h-0 flex-1">
        <div className="space-y-1 p-2">
          {isLoading ? (
            <p className="px-2 py-3 text-sm text-muted-foreground">{t("workspace.loading")}</p>
          ) : null}

          {error ? (
            <p className="px-2 py-3 text-sm text-destructive">{String(error)}</p>
          ) : null}

          {!isLoading && !error && workspaces.length === 0 ? (
            <p className="px-2 py-3 text-sm text-muted-foreground">
              {t("workspace.empty")}
            </p>
          ) : null}

          {workspaces.map((workspace) => (
            <button
              className={cn(
                "group flex w-full items-start gap-2 rounded-md px-2 py-2 text-left hover:bg-accent",
                selectedWorkspaceId === workspace.id && "bg-accent",
              )}
              key={workspace.id}
              onClick={() => {
                setSelectedWorkspaceId(workspace.id);
                onWorkspaceSelect?.(workspace.id);
              }}
              type="button"
            >
              <span className="min-w-0 flex-1">
                <span className="block truncate text-sm font-medium">{workspace.name}</span>
                <span className="block truncate text-xs text-muted-foreground">
                  {workspace.path}
                </span>
              </span>
              <Button
                aria-label={t("workspace.remove", { name: workspace.name })}
                className="opacity-0 group-hover:opacity-100"
                onClick={(event) => {
                  event.stopPropagation();
                  removeWorkspace(workspace.id);
                  if (selectedWorkspaceId === workspace.id) {
                    setSelectedWorkspaceId(null);
                    onWorkspaceSelect?.(null);
                  }
                }}
                size="icon"
                type="button"
                variant="ghost"
              >
                <Trash2 className="size-4" />
              </Button>
            </button>
          ))}
        </div>
      </ScrollArea>
    </div>
  );
}
