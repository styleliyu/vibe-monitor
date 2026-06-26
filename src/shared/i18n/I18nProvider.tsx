import {
  createContext,
  type ReactNode,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import { dictionaries, type Language, type TranslationKey } from "./translations";

const LANGUAGE_STORAGE_KEY = "vibe-monitor.language";
const DEFAULT_LANGUAGE: Language = "zh-CN";

type I18nContextValue = {
  language: Language;
  setLanguage: (language: Language) => void;
  t: (key: TranslationKey, values?: Record<string, string | number>) => string;
};

const fallbackContext: I18nContextValue = {
  language: "en",
  setLanguage: () => {},
  t: (key, values) => translate("en", key, values),
};

const I18nContext = createContext<I18nContextValue>(fallbackContext);

export function I18nProvider({ children }: { children: ReactNode }) {
  const [language, setLanguageState] = useState<Language>(() => readStoredLanguage());

  useEffect(() => {
    localStorage.setItem(LANGUAGE_STORAGE_KEY, language);
    document.documentElement.lang = language;
    document.documentElement.classList.add("dark");
  }, [language]);

  const value = useMemo<I18nContextValue>(
    () => ({
      language,
      setLanguage: setLanguageState,
      t: (key, values) => translate(language, key, values),
    }),
    [language],
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n() {
  return useContext(I18nContext);
}

function readStoredLanguage(): Language {
  const stored = localStorage.getItem(LANGUAGE_STORAGE_KEY);
  return stored === "en" || stored === "zh-CN" ? stored : DEFAULT_LANGUAGE;
}

function translate(
  language: Language,
  key: TranslationKey,
  values?: Record<string, string | number>,
) {
  let text = dictionaries[language][key];
  if (!values) {
    return text;
  }
  for (const [name, value] of Object.entries(values)) {
    text = text.split(`{${name}}`).join(String(value));
  }
  return text;
}
