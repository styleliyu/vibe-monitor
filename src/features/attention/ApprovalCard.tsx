import { Check, X } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { ApprovalContext } from "@/features/codex/types";

type ApprovalCardProps = {
  approval: ApprovalContext;
  onApprove?: () => void;
  onReject?: () => void;
};

export function ApprovalCard({ approval, onApprove, onReject }: ApprovalCardProps) {
  return (
    <div className="rounded-md border border-amber-500/30 bg-amber-500/10 p-3">
      <div className="flex items-center gap-2">
        <h4 className="min-w-0 truncate text-sm font-medium">{approval.command}</h4>
        <Badge className="ml-auto" variant="outline">
          {approval.requestedScope}
        </Badge>
      </div>
      <p className="mt-1 truncate text-xs text-muted-foreground">{approval.workspacePath}</p>
      <div className="mt-3 flex justify-end gap-2">
        <Button
          aria-label="Reject Codex request"
          onClick={onReject}
          size="sm"
          type="button"
          variant="outline"
        >
          <X className="size-4" />
          Reject
        </Button>
        <Button aria-label="Approve Codex request" onClick={onApprove} size="sm" type="button">
          <Check className="size-4" />
          Approve
        </Button>
      </div>
    </div>
  );
}
