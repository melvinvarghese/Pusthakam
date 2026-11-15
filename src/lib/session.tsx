import { createContext, useContext } from "react";
import type { Session } from "@/lib/api/types";

interface SessionValue {
  session: Session;
  signOut: () => Promise<void>;
}

export const SessionContext = createContext<SessionValue | null>(null);

export function useSession(): SessionValue {
  const ctx = useContext(SessionContext);
  if (!ctx) throw new Error("useSession must be used inside a signed-in area");
  return ctx;
}

export function can(session: Session, permission: string): boolean {
  return session.permissions.includes(permission);
}
