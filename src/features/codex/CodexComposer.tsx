import { Send, Square } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";

type CodexComposerProps = {
  disabled?: boolean;
  isRunning?: boolean;
  onInterrupt?: () => void;
  onSend: (prompt: string) => void;
};

export function CodexComposer({
  disabled,
  isRunning,
  onInterrupt,
  onSend,
}: CodexComposerProps) {
  const [prompt, setPrompt] = useState("");

  function handleSubmit() {
    const trimmed = prompt.trim();
    if (!trimmed) {
      return;
    }
    onSend(trimmed);
    setPrompt("");
  }

  return (
    <div className="border-t p-3">
      <Textarea
        className="min-h-20 resize-none"
        disabled={disabled}
        onChange={(event) => setPrompt(event.target.value)}
        placeholder="Send a Codex instruction..."
        value={prompt}
      />
      <div className="mt-2 flex justify-end gap-2">
        {isRunning ? (
          <Button onClick={onInterrupt} size="sm" type="button" variant="outline">
            <Square className="size-4" />
            Interrupt
          </Button>
        ) : null}
        <Button
          aria-label="Send to Codex"
          disabled={disabled || !prompt.trim()}
          onClick={handleSubmit}
          size="sm"
          type="button"
        >
          <Send className="size-4" />
          Send
        </Button>
      </div>
    </div>
  );
}
