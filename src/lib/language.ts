import { useSyncExternalStore } from "react";

export type Language = "en" | "ml";
const key = "pusthakam.language";
const event = "pusthakam-language";

function readLanguage(): Language {
  try { return localStorage.getItem(key) === "ml" ? "ml" : "en"; }
  catch { return "en"; }
}

let language = readLanguage();

export function setLanguage(value: Language) {
  language = value;
  try { localStorage.setItem(key, value); } catch { /* Keep the choice for this session. */ }
  document.documentElement.lang = value;
  window.dispatchEvent(new Event(event));
}

function subscribe(listener: () => void) {
  const storage = (e: StorageEvent) => {
    if (e.key === key || e.key === null) {
      language = readLanguage();
      document.documentElement.lang = language;
      listener();
    }
  };
  window.addEventListener(event, listener);
  window.addEventListener("storage", storage);
  return () => {
    window.removeEventListener(event, listener);
    window.removeEventListener("storage", storage);
  };
}

export function useLanguage() {
  return useSyncExternalStore(subscribe, () => language);
}

document.documentElement.lang = language;
