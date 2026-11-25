import { useEffect, useState, type ReactNode } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { ApiError, authApi } from "@/lib/api";
import { SessionContext } from "@/lib/session";
import { LoginScreen } from "@/features/auth/LoginScreen";
import { AuthLayout } from "@/features/auth/AuthLayout";
import { SetupWizard } from "@/features/setup/SetupWizard";
import { StartupSplashScreen } from "@/features/boot/StartupSplashScreen";
import { staffHttpConfig } from "@/lib/api/staffHttp";

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

export interface GateProps {
  children: ReactNode;
  /**
   * Minimum duration in milliseconds to display the startup splash screen.
   * Defaults to 3000ms in production/development, or 0ms in automated test environments.
   */
  minSplashDurationMs?: number;
}

/**
 * Decides what to show first: the startup loading screen (held for at least 3 seconds),
 * the setup wizard on a fresh install, the sign-in screen when nobody is signed in,
 * or the main application when authenticated.
 */
export function Gate({ children, minSplashDurationMs }: GateProps) {
  const qc = useQueryClient();
  const browserStaff = staffHttpConfig() !== null;
  useEffect(() => {
    if (!browserStaff) return;
    const ended = () => {
      qc.setQueryData(["session"], null);
      qc.removeQueries({ predicate: ({ queryKey }) => queryKey[0] !== "session" && queryKey[0] !== "setup-status" });
      qc.getMutationCache().clear();
    };
    window.addEventListener("pusthakam-session-ended", ended);
    return () => window.removeEventListener("pusthakam-session-ended", ended);
  }, [browserStaff, qc]);
  const defaultDuration = import.meta.env.MODE === "test" ? 0 : 3000;
  const splashDuration = minSplashDurationMs ?? defaultDuration;
  const [splashFinished, setSplashFinished] = useState(splashDuration <= 0);

  useEffect(() => {
    if (splashDuration <= 0) {
      setSplashFinished(true);
      return;
    }
    const timer = setTimeout(() => {
      setSplashFinished(true);
    }, splashDuration);
    return () => clearTimeout(timer);
  }, [splashDuration]);

  // In development, `?preview` runs the same gate against synthetic data (see src/lib/preview.ts).
  const status = useQuery({ queryKey: ["setup-status"], queryFn: authApi.setupStatus, retry: false });
  const configured = status.data?.configured === true;
  const session = useQuery({ queryKey: ["session"], queryFn: authApi.currentSession, retry: false, enabled: configured, refetchInterval: browserStaff ? 60_000 : false });

  if (status.isError) {
    const notDesktop = status.error instanceof ApiError && status.error.code === "NOT_DESKTOP";
    return (
      <Message
        title={notDesktop ? "Open Pusthakam as an app" : "Could not open your library"}
        text={status.error instanceof ApiError ? status.error.message : "Something went wrong while starting."}
      />
    );
  }

  // Keep splash screen visible until minimum duration has elapsed AND initial status is resolved
  if (!splashFinished || status.isPending || (configured && session.isPending)) {
    return (
      <StartupSplashScreen
        durationMs={splashDuration}
        onComplete={() => setSplashFinished(true)}
      />
    );
  }

  if (!configured) return <div className="startup-reveal"><SetupWizard /></div>;
  if (!session.data) return <div className="startup-reveal"><LoginScreen /></div>;

  const signOut = async () => {
    await authApi.logout();
    qc.setQueryData(["session"], null);
    qc.removeQueries({ predicate: ({ queryKey }) => queryKey[0] !== "session" && queryKey[0] !== "setup-status" });
    qc.getMutationCache().clear();
  };
  return <SessionContext.Provider value={{ session: session.data, signOut }}>{children}</SessionContext.Provider>;
}
