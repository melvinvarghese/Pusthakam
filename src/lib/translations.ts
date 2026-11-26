import glossary from "../../docs/translations/approved-glossary.json";
import { setupMalayalam } from "./setup-translations";
import { useLanguage } from "./language";
import { coreMalayalam } from "./core-translations";
import { useCallback } from "react";

// User records, command names and role/permission values never pass through here.
export const malayalam: Readonly<Record<string, string>> = {
  ...setupMalayalam,
  ...coreMalayalam,
  "Main": "പ്രധാനം",
  "Collections": "ശേഖരങ്ങൾ",
  "Operations": "പ്രവർത്തനങ്ങൾ",
  "Administration": "അഡ്മിൻ",
  "Circulation": "പുസ്തക വിതരണം",
  "Accession Register": "അക്സഷൻ രജിസ്റ്റർ",
  "Inventory": "സ്റ്റോക്ക് പരിശോധന",
  "Classification": "വർഗീകരണം",
  "Periodicals": "ആനുകാലികങ്ങൾ",
  "Acquisitions": "പുസ്തകം വാങ്ങൽ",
  "Reports": "റിപ്പോർട്ടുകൾ",
  "Barcode & Printing": "ബാർകോഡും പ്രിന്റിംഗും",
  "Staff & Permissions": "ജീവനക്കാരും അനുമതികളും",
  "Library Settings": "ലൈബ്രറി ക്രമീകരണങ്ങൾ",
  "Database Backups": "ഡാറ്റാബേസ് ബാക്കപ്പുകൾ",
  "Audit Log": "പ്രവർത്തന രേഖ",
  "Primary": "പ്രധാന മെനു",
  "Navigation": "മെനു",
  "Open navigation": "മെനു തുറക്കുക",
  "Close navigation": "മെനു അടയ്ക്കുക",
  "Quick navigation": "വിഭാഗത്തിലേക്ക് പോകുക",
  "Go to a section": "ഒരു വിഭാഗത്തിലേക്ക് പോകുക",
  "Go to section": "വിഭാഗം തിരയുക",
  "No matching sections": "ചേരുന്ന വിഭാഗങ്ങളില്ല",
  "Search or go to a section": "തിരയുക അല്ലെങ്കിൽ ഒരു വിഭാഗത്തിലേക്ക് പോകുക",
  "Skip to content": "ഉള്ളടക്കത്തിലേക്ക് പോകുക",
  "Not available in this version yet": "ഈ പതിപ്പിൽ ഇതുവരെ ലഭ്യമല്ല",
  "Language": "ഭാഷ",
  "Choose the language for this device.": "ഈ ഉപകരണത്തിൽ ഉപയോഗിക്കേണ്ട ഭാഷ തിരഞ്ഞെടുക്കുക.",
  "A Better Way to Manage Libraries.": "ലൈബ്രറി നടത്തിപ്പ് കൂടുതൽ എളുപ്പമാക്കാം.",
  "Signing in.": "ലോഗിൻ ചെയ്യുന്നു…",
  "Could not sign in.": "ലോഗിൻ ചെയ്യാനായില്ല.",
  "Password reset. Sign in with your new password.": "പാസ്‌വേഡ് മാറ്റി. പുതിയ പാസ്‌വേഡ് ഉപയോഗിച്ച് ലോഗിൻ ചെയ്യുക.",
  "Use a recovery code": "റിക്കവറി കോഡ് ഉപയോഗിക്കുക",
  "Recover administrator account": "അഡ്മിൻ അക്കൗണ്ട് വീണ്ടെടുക്കുക",
  "Administrator username": "അഡ്മിന്റെ ഉപയോക്തൃനാമം",
  "Recovery code": "റിക്കവറി കോഡ്",
  "New password": "പുതിയ പാസ്‌വേഡ്",
  "Confirm new password": "പുതിയ പാസ്‌വേഡ് വീണ്ടും നൽകുക",
  "The new passwords do not match.": "പുതിയ പാസ്‌വേഡുകൾ തമ്മിൽ പൊരുത്തപ്പെടുന്നില്ല.",
  "Back to sign in": "ലോഗിൻ പേജിലേക്ക് മടങ്ങുക",
  "Resetting.": "പാസ്‌വേഡ് മാറ്റുന്നു…",
  "Reset password": "പാസ്‌വേഡ് മാറ്റുക",
  "Theme": "തീം",
  "Light theme": "ലൈറ്റ് തീം",
  "Dark theme": "ഡാർക്ക് തീം",
  "Match system theme": "ഉപകരണത്തിലെ തീം ഉപയോഗിക്കുക",
  "Online Tools": "ഓൺലൈൻ ടൂളുകൾ",
  "Open in your web browser. These need an internet connection.": "വെബ് ബ്രൗസറിൽ തുറക്കും. ഇന്റർനെറ്റ് കണക്ഷൻ വേണം.",
  "Could not open the link.": "ലിങ്ക് തുറക്കാനായില്ല.",
  "Opening your library.": "ലൈബ്രറി തുറക്കുന്നു…",
  "Just a moment.": "ഒരു നിമിഷം…",
  "Application startup progress": "ആപ്പ് തുറക്കുന്നതിന്റെ പുരോഗതി",
  "administrator": "അഡ്മിൻ",
  "librarian": "ലൈബ്രേറിയൻ",
  "assistant": "അസിസ്റ്റന്റ്",
  "viewer": "വിവരങ്ങൾ കാണുന്നയാൾ",
  // Approved glossary wins over draft wording.
  ...glossary.terms,
};

export type TranslationValues = Readonly<Record<string, string | number>>;

export function translate(language: "en" | "ml", text: string, values: TranslationValues = {}) {
  const translated = language === "ml" ? malayalam[text] ?? text : text;
  return translated.replace(/\{(\w+)\}/g, (placeholder, key: string) =>
    Object.prototype.hasOwnProperty.call(values, key) ? String(values[key]) : placeholder);
}

export function useTranslation() {
  const language = useLanguage();
  const t = useCallback((text: string, values?: TranslationValues) => translate(language, text, values), [language]);
  return { language, t };
}
