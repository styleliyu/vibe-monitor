import { Check, Languages, Moon, Settings } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { cn } from "@/shared/lib/cn";
import { useI18n, type Language } from "@/shared/i18n";

const languages: Array<{ value: Language; labelKey: "settings.zhCN" | "settings.english" }> = [
  { value: "zh-CN", labelKey: "settings.zhCN" },
  { value: "en", labelKey: "settings.english" },
];

export function SettingsPanel() {
  const { language, setLanguage, t } = useI18n();

  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button aria-label={t("app.settings")} size="sm" type="button" variant="ghost">
          <Settings className="size-4" />
          {t("app.settings")}
        </Button>
      </DialogTrigger>
      <DialogContent className="border-zinc-800 bg-zinc-950 text-zinc-50 ring-zinc-700 sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-zinc-50">
            <Settings className="size-4" />
            {t("settings.title")}
          </DialogTitle>
          <DialogDescription className="text-zinc-400">
            {t("settings.description")}
          </DialogDescription>
        </DialogHeader>

        <section className="space-y-3 rounded-md border border-zinc-800 bg-zinc-900/70 p-3">
          <div className="flex items-start gap-2">
            <Languages className="mt-0.5 size-4 text-zinc-400" />
            <div>
              <h3 className="text-sm font-medium text-zinc-50">{t("settings.language")}</h3>
              <p className="mt-1 text-xs leading-5 text-zinc-400">
                {t("settings.languageDescription")}
              </p>
            </div>
          </div>
          <div className="grid grid-cols-2 gap-2">
            {languages.map((item) => (
              <Button
                className={cn(
                  "justify-between border-zinc-700 bg-zinc-950 text-zinc-100 hover:bg-zinc-800",
                  language === item.value && "border-zinc-100 bg-zinc-800",
                )}
                key={item.value}
                onClick={() => setLanguage(item.value)}
                type="button"
                variant="outline"
              >
                {t(item.labelKey)}
                {language === item.value ? <Check className="size-4" /> : null}
              </Button>
            ))}
          </div>
        </section>

        <section className="space-y-2 rounded-md border border-zinc-800 bg-zinc-900/70 p-3">
          <div className="flex items-start gap-2">
            <Moon className="mt-0.5 size-4 text-zinc-400" />
            <div>
              <h3 className="text-sm font-medium text-zinc-50">{t("settings.appearance")}</h3>
              <p className="mt-1 text-xs leading-5 text-zinc-400">
                <span className="font-medium text-zinc-200">{t("settings.darkMode")}</span>
                {" - "}
                {t("settings.darkModeDescription")}
              </p>
            </div>
          </div>
        </section>
      </DialogContent>
    </Dialog>
  );
}
