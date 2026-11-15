import { useState, type FormEvent } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { AuthLayout } from "./AuthLayout";
import { Button } from "@/components/ui/Button";
import { TextField } from "@/components/ui/Field";
import { ApiError, authApi } from "@/lib/api";

export function LoginScreen() {
  const qc = useQueryClient();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");

  const login = useMutation({
    mutationFn: () => authApi.login(username, password),
    onSuccess: (session) => qc.setQueryData(["session"], session),
  });

  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (username.trim() && password) login.mutate();
  };

  return (
    <AuthLayout>
      <form onSubmit={submit} className="w-[min(24rem,100%)] space-y-4 rounded-lg border border-line bg-surface p-6 text-fg">
        <h1 className="text-xl font-bold leading-7 tracking-tight">Sign in</h1>
        <TextField label="Username" value={username} onChange={(e) => setUsername(e.target.value)} autoComplete="username" autoFocus />
        <TextField
          label="Password"
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          autoComplete="current-password"
        />
        {login.isError && (
          <p role="alert" className="text-xs text-danger">
            {login.error instanceof ApiError ? login.error.message : "Could not sign in."}
          </p>
        )}
        <Button type="submit" variant="primary" size="lg" className="w-full" disabled={login.isPending}>
          {login.isPending ? "Signing in…" : "Sign in"}
        </Button>
      </form>
    </AuthLayout>
  );
}
