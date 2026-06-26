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

  return {
    ...query,
    addWorkspace: add.mutate,
    addWorkspaceAsync: add.mutateAsync,
    removeWorkspace: remove.mutate,
    removeWorkspaceAsync: remove.mutateAsync,
    isAddingWorkspace: add.isPending,
    addWorkspaceError: add.error,
  };
}
