import type { ReactNode } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { ApiError, authApi } from "@/lib/api";
import { SessionContext } from "@/lib/session";
import { LoginScreen } from "@/features/auth/LoginScreen";
import { AuthLayout } from "@/features/auth/AuthLayout";
import { SetupWizard } from "@/features/setup/SetupWizard";

function Message({ title, text }: { title: string; text: string }) {
  return (
    <AuthLayout>
      <div role="status" className="w-[min(26rem,100%)] rounded-lg border border-line bg-surface p-6 text-fg">
        <h1 className="text-base font-semibold">{title}</h1>
        <p className="mt-1 text-muted">{text}</p>
      </div>
    </AuthLayout>
  );
}

/**
 * Decides what to show first: the setup wizard on a fresh install, the sign-in screen when nobody is
 * signed in, otherwise the app. Sign-in state lives in the Rust backend, never in the browser.
 */
export function Gate({ children }: { children: ReactNode }) {
  const qc = useQueryClient();
  // In development, `?preview` runs the same gate against synthetic data (see src/lib/preview.ts).
  const status = useQuery({ queryKey: ["setup-status"], queryFn: authApi.setupStatus, retry: false });
  const configured = status.data?.configured === true;
  const session = useQuery({ queryKey: ["session"], queryFn: authApi.currentSession, retry: false, enabled: configured });

  if (status.isError) {
    const notDesktop = status.error instanceof ApiError && status.error.code === "NOT_DESKTOP";
    return (
      <Message
        title={notDesktop ? "Open Pusthakam as an app" : "Could not open your library"}
        text={status.error instanceof ApiError ? status.error.message : "Something went wrong while starting."}
      />
    );
  }
  if (status.isPending) return <Message title="Opening your library…" text="Just a moment." />;
  if (!configured) return <SetupWizard />;
  if (session.isPending) return <Message title="Opening your library…" text="Just a moment." />;
  if (!session.data) return <LoginScreen />;

  const signOut = async () => {
    await authApi.logout();
    qc.setQueryData(["session"], null);
    qc.removeQueries({ predicate: ({ queryKey }) => queryKey[0] !== "session" && queryKey[0] !== "setup-status" });
    qc.getMutationCache().clear();
  };
  return <SessionContext.Provider value={{ session: session.data, signOut }}>{children}</SessionContext.Provider>;
}
