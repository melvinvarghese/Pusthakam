import { useState, type FormEvent } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { BrandMark } from "@/components/BrandMark";
import { Button } from "@/components/ui/Button";
import { CheckField, SelectField, TextField } from "@/components/ui/Field";
import { ApiError, authApi, backupApi } from "@/lib/api";
import { staffHttpConfig } from "@/lib/api/staffHttp";
import type { ProfileCode, SetupInput } from "@/lib/api/types";
import { parseMinor } from "@/lib/money";
import { setLanguage, type Language } from "@/lib/language";
import { useTranslation } from "@/lib/translations";
import { cn } from "@/lib/utils";

const PROFILES: { code: ProfileCode; label: string; hint: string }[] = [
  { code: "school", label: "School library", hint: "Students, teachers and staff." },
  { code: "college", label: "College library", hint: "Students, faculty and staff." },
  { code: "university", label: "University library", hint: "Undergraduates, postgraduates, researchers and faculty." },
  { code: "public", label: "Public library", hint: "Adults and children." },
  { code: "special", label: "Special library", hint: "A company, hospital, museum or similar organisation." },
  { code: "research", label: "Research library", hint: "Researchers and staff." },
  { code: "institutional", label: "Institutional library", hint: "Staff and guests of an institution." },
  { code: "private", label: "Private collection", hint: "One person or family." },
];

const STEPS = ["Your library", "Loan rules", "Backups", "Administrator"] as const;
const ADMIN_STEP = STEPS.indexOf("Administrator");

interface Form {
  name: string;
  profile: ProfileCode;
  address: string;
  currency: string;
  applyDefaults: boolean;
  fine: string;
  backupFolder: string;
  displayName: string;
  username: string;
  password: string;
  confirm: string;
}

const INITIAL: Form = {
  name: "", profile: "school", address: "", currency: "INR", applyDefaults: true, fine: "0", backupFolder: "",
  displayName: "", username: "", password: "", confirm: "",
};

/** Returns an error message per field for one step, or an empty object when the step is complete. */
export function validateStep(step: number, f: Form): Partial<Record<keyof Form, string>> {
  const e: Partial<Record<keyof Form, string>> = {};
  if (step === 0) {
    if (!f.name.trim()) e.name = "Enter your library's name.";
    if (!/^[A-Za-z]{3}$/.test(f.currency.trim())) e.currency = "Use a 3-letter currency code, such as INR or USD.";
  } else if (step === 1) {
    if (parseMinor(f.fine) === null) e.fine = "Enter an amount such as 0, 1 or 2.50.";
  } else if (step === ADMIN_STEP) {
    // The backup folder step has nothing to check here: Pusthakam checks the folder itself
    // when the library is created, and an empty folder means the data folder.
    if (!f.displayName.trim()) e.displayName = "Enter your name.";
    if (!/^[A-Za-z0-9._-]{3,40}$/.test(f.username.trim())) {
      e.username = "3 to 40 characters: letters, digits, '.', '_' or '-'.";
    }
    if (f.password.length < 10) e.password = "Use at least 10 characters.";
    if (f.confirm !== f.password) e.confirm = "The passwords do not match.";
  }
  return e;
}

export function SetupWizard() {
  const { language, t } = useTranslation();
  const translatedError = (text?: string) => text ? t(text) : undefined;
  const qc = useQueryClient();
  const [step, setStep] = useState(0);
  const [form, setForm] = useState<Form>(INITIAL);
  const [errors, setErrors] = useState<Partial<Record<keyof Form, string>>>({});
  const set = <K extends keyof Form>(key: K, value: Form[K]) => setForm((f) => ({ ...f, [key]: value }));

  const create = useMutation({
    mutationFn: () => {
      const input: SetupInput = {
        library: {
          name: form.name, profile: form.profile, address: form.address || null, contact: null,
          currency: form.currency, locale: language,
        },
        admin: { username: form.username, displayName: form.displayName, password: form.password },
        applyProfileDefaults: form.applyDefaults,
        finePerDayMinor: parseMinor(form.fine) ?? 0,
        ...(form.backupFolder.trim() ? { backupFolder: form.backupFolder.trim() } : {}),
      };
      return authApi.completeSetup(input);
    },
    onSuccess: async (session) => {
      qc.setQueryData(["session"], session);
      await qc.invalidateQueries({ queryKey: ["setup-status"] });
    },
  });
  // The folder dialog opens on the computer running Pusthakam, so a browser gets no Browse button.
  const canBrowse = staffHttpConfig() === null;
  const browse = useMutation({
    mutationFn: backupApi.pickFolder,
    onSuccess: (folder) => { if (folder) set("backupFolder", folder); },
  });

  const next = (e: FormEvent) => {
    e.preventDefault();
    const found = validateStep(step, form);
    setErrors(found);
    if (Object.keys(found).length > 0) return;
    if (step < STEPS.length - 1) setStep(step + 1);
    else create.mutate();
  };

  const profile = PROFILES.find((p) => p.code === form.profile);

  return (
    <div className="grid min-h-dvh grid-cols-[280px_minmax(0,1fr)]">
      <aside className="flex flex-col gap-8 bg-ink p-6 text-rail-text">
        <div className="flex items-center gap-3 text-white">
          <BrandMark />
          <div className="text-[15px] font-bold">Pusthakam</div>
        </div>
        <div>
          <h1 className="text-xl font-bold leading-7 text-white">{t("Set up your library")}</h1>
          <p className="mt-1 text-rail-mute">{t("This takes about two minutes. You can change everything later.")}</p>
        </div>
        <ol className="space-y-1" aria-label={t("Setup steps")}>
          {STEPS.map((label, i) => (
            <li
              key={label}
              aria-current={i === step ? "step" : undefined}
              className={cn("rounded-md px-3 py-1.5", i === step ? "bg-ink-2 font-semibold text-white" : "text-rail-mute")}
            >
              {i + 1}. {t(label)}
            </li>
          ))}
        </ol>
      </aside>

      <main className="flex items-start justify-center overflow-auto bg-bg p-10">
        <form onSubmit={next} className="w-[min(32rem,100%)] space-y-5" noValidate>
          <SelectField label="Language / ഭാഷ" value={language} onChange={(e) => setLanguage(e.target.value as Language)}>
            <option value="en">English</option>
            <option value="ml">മലയാളം</option>
          </SelectField>
          <h2 className="text-lg font-bold leading-7 tracking-tight">{t(STEPS[step] ?? STEPS[0])}</h2>

          {step === 0 && (
            <>
              <TextField label={t("Library name")} value={form.name} onChange={(e) => set("name", e.target.value)} error={translatedError(errors.name)} autoFocus />
              <SelectField
                label={t("Type of library")}
                value={form.profile}
                onChange={(e) => set("profile", e.target.value as ProfileCode)}
                hint={profile ? t(profile.hint) : undefined}
              >
                {PROFILES.map((p) => <option key={p.code} value={p.code}>{t(p.label)}</option>)}
              </SelectField>
              <TextField label={t("Address (optional)")} value={form.address} onChange={(e) => set("address", e.target.value)} />
              <TextField
                label={t("Currency")}
                value={form.currency}
                onChange={(e) => set("currency", e.target.value.toUpperCase())}
                error={translatedError(errors.currency)}
                hint={t("Used for fines and purchase prices.")}
                maxLength={3}
                className="w-28"
              />
            </>
          )}

          {step === 1 && (
            <>
              <CheckField
                label={t("Start with typical member categories and loan rules")}
                hint={language === "ml" ? `${profile ? t(profile.label) : "ലൈബ്രറി"}: സാധാരണ അംഗവിഭാഗങ്ങൾ, ശേഖരങ്ങൾ, പുസ്തക വിതരണ കാലാവധികൾ എന്നിവ സൃഷ്ടിക്കും. ലൈബ്രറി ക്രമീകരണങ്ങളിൽ ഇവ മാറ്റാം.` : `Creates the usual categories, collections and loan periods for a ${profile?.label.toLowerCase()}. You can edit all of it in Library Settings.`}
                checked={form.applyDefaults}
                onChange={(e) => set("applyDefaults", e.target.checked)}
              />
              <TextField
                label={`${language === "ml" ? "വൈകിയാൽ ദിവസേനയുള്ള പിഴ" : "Late fine per day"} (${form.currency || t("Currency")})`}
                value={form.fine}
                onChange={(e) => set("fine", e.target.value)}
                error={translatedError(errors.fine)}
                hint={t("Enter 0 if you do not charge fines. This applies to every starting loan rule.")}
                inputMode="decimal"
                className="w-32"
              />
            </>
          )}

          {step === 2 && (
            <>
              <p className="text-muted">{t("Pusthakam backs up your library the first time it opens each day and keeps the last 7 days.")}</p>
              <p className="text-muted">{t("Choose a folder on a different drive or a USB drive. If this computer's disk fails, your backups are still safe there.")}</p>
              <TextField
                label={t("Backup folder (optional)")}
                value={form.backupFolder}
                onChange={(e) => set("backupFolder", e.target.value)}
                hint={t("Leave empty to keep backups in Pusthakam's own data folder. You can change this later under Database Backups.")}
                placeholder="E:\Pusthakam Backups"
                spellCheck={false}
              />
              {canBrowse && (
                <Button variant="secondary" disabled={browse.isPending} onClick={() => browse.mutate()}>
                  {t(browse.isPending ? "Choosing folder." : "Choose folder…")}
                </Button>
              )}
              {browse.isError && (
                <p role="alert" className="text-xs text-danger">
                  {browse.error instanceof ApiError ? browse.error.message : t("The folder dialog could not open. Type the folder path instead.")}
                </p>
              )}
            </>
          )}

          {step === ADMIN_STEP && (
            <>
              <p className="text-muted">{t("You will use this account to sign in. It has full access to everything.")}</p>
              <TextField label={t("Your name")} value={form.displayName} onChange={(e) => set("displayName", e.target.value)} error={translatedError(errors.displayName)} autoFocus />
              <TextField label={t("Username")} value={form.username} onChange={(e) => set("username", e.target.value)} error={translatedError(errors.username)} autoComplete="username" />
              <TextField
                label={t("Password")}
                type="password"
                value={form.password}
                onChange={(e) => set("password", e.target.value)}
                error={translatedError(errors.password)}
                hint={t("At least 10 characters. A short phrase you can remember works well.")}
                autoComplete="new-password"
              />
              <TextField label={t("Repeat password")} type="password" value={form.confirm} onChange={(e) => set("confirm", e.target.value)} error={translatedError(errors.confirm)} autoComplete="new-password" />
            </>
          )}

          {create.isError && (
            <p role="alert" className="text-xs text-danger">
              {create.error instanceof ApiError ? create.error.message : t("Setup could not be completed.")}
            </p>
          )}

          <div className="flex justify-between pt-2">
            <Button variant="secondary" size="lg" disabled={step === 0 || create.isPending} onClick={() => { setErrors({}); setStep(step - 1); }}>
              {t("Back")}
            </Button>
            <Button type="submit" variant="primary" size="lg" disabled={create.isPending}>
              {t(step < STEPS.length - 1 ? "Continue" : create.isPending ? "Creating." : "Create library")}
            </Button>
          </div>
        </form>
      </main>
    </div>
  );
}
