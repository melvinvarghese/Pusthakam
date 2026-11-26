import { useTranslation } from "@/lib/translations";
import { useState, type FormEvent } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { AuthLayout } from "./AuthLayout";
import { RecoverAccount } from "./RecoverAccount";
import { Button } from "@/components/ui/Button";
import { TextField } from "@/components/ui/Field";
import { ApiError, authApi } from "@/lib/api";

export function LoginScreen() {
  const { t } = useTranslation();
  const qc = useQueryClient();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [recovering, setRecovering] = useState(false);
  const [recovered, setRecovered] = useState(false);

  const login = useMutation({
    mutationFn: () => authApi.login(username, password),
    onSuccess: (session) => qc.setQueryData(["session"], session),
  });

  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (username.trim() && password) login.mutate();
  };

  if (recovering) return <RecoverAccount onCancel={() => setRecovering(false)} onRecovered={() => {
    setRecovered(true);
    setRecovering(false);
  }} />;

  return (
    <AuthLayout>
      <form onSubmit={submit} className="w-[min(24rem,100%)] space-y-4 rounded-lg border border-line bg-surface p-6 text-fg">
        <h1 className="text-xl font-bold leading-7 tracking-tight">{t("Sign in")}</h1>
        {recovered && <p role="status" className="text-sm">{t("Password reset. Sign in with your new password.")}</p>}
        <TextField label={t("Username")} value={username} onChange={(e) => setUsername(e.target.value)} autoComplete="username" autoFocus />
        <TextField
          label={t("Password")}
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          autoComplete="current-password"
        />
        {login.isError && (
          <p role="alert" className="text-xs text-danger">
            {login.error instanceof ApiError ? t(login.error.message) : t("Could not sign in.")}
          </p>
        )}
        <Button type="submit" variant="primary" size="lg" className="w-full" disabled={login.isPending}>
          {t(login.isPending ? "Signing in…" : "Sign in")}
        </Button>
        <Button variant="ghost" disabled={login.isPending} onClick={() => {
          login.reset();
          setPassword("");
          setRecovered(false);
          setRecovering(true);
        }}>{t("Use a recovery code")}</Button>
      </form>
    </AuthLayout>
  );
}
