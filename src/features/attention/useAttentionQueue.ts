import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createAttentionItem, listAttentionItems, resolveAttentionItem } from "./api";
import type { CreateAttentionItem } from "./types";

export function attentionQueryKey(workspaceId?: string | null) {
  return ["attention", workspaceId ?? "all"] as const;
}

export function useAttentionQueue(workspaceId?: string | null) {
  const queryClient = useQueryClient();
  const queryKey = attentionQueryKey(workspaceId);
  const query = useQuery({
    queryKey,
    queryFn: () => listAttentionItems(workspaceId),
  });
  const create = useMutation({
    mutationFn: (input: CreateAttentionItem) => createAttentionItem(input),
    onSuccess: () => queryClient.invalidateQueries({ queryKey }),
  });
  const resolve = useMutation({
    mutationFn: resolveAttentionItem,
    onSuccess: () => queryClient.invalidateQueries({ queryKey }),
  });

  return {
    ...query,
    createAttentionItem: create.mutate,
    createAttentionItemAsync: create.mutateAsync,
    resolveAttentionItem: resolve.mutate,
    resolveAttentionItemAsync: resolve.mutateAsync,
    isCreatingAttentionItem: create.isPending,
    isResolvingAttentionItem: resolve.isPending,
    createAttentionItemError: create.error,
    resolveAttentionItemError: resolve.error,
  };
}
