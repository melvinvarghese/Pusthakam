import type { AppErrorPayload } from "./types";

const ENDPOINT = "/api/staff/commands";

export interface StaffHttpConfig {
  endpoint: typeof ENDPOINT;
  csrfToken: string;
}

/** The server opts the shared app into web mode only after its session API is ready. */
export function staffHttpConfig(): StaffHttpConfig | null {
  if (typeof document === "undefined" || !/^\/staff\/?$/.test(window.location.pathname)) return null;
  const endpoint = document.querySelector<HTMLMetaElement>('meta[name="pusthakam-staff-api"]')?.content;
  const csrfToken = document.querySelector<HTMLMetaElement>('meta[name="pusthakam-staff-csrf"]')?.content;
  return endpoint === ENDPOINT && csrfToken ? { endpoint: ENDPOINT, csrfToken } : null;
}

function isPayload(value: unknown): value is AppErrorPayload {
  return typeof value === "object" && value !== null &&
    typeof (value as AppErrorPayload).code === "string" && typeof (value as AppErrorPayload).message === "string";
}

/** Same typed arguments/results as desktop IPC; authentication belongs to the server. */
export async function staffHttpCommand<T>(config: StaffHttpConfig, command: string, args?: Record<string, unknown>): Promise<T> {
  if (config.endpoint !== ENDPOINT || !config.csrfToken || !/^[a-z][a-z0-9_]*$/.test(command)) {
    throw { code: "WEB_NOT_CONFIGURED", message: "The staff web connection is not configured." } satisfies AppErrorPayload;
  }
  let response: Response;
  try {
    response = await fetch(ENDPOINT, {
      method: "POST",
      credentials: "same-origin",
      redirect: "error",
      cache: "no-store",
      headers: { "Content-Type": "application/json", "X-CSRF-Token": config.csrfToken },
      body: JSON.stringify({ command, args: args ?? {} }),
      signal: AbortSignal.timeout(/^(?:import_|circulation_(?:preview|commit)_|backup_|catalogue_export|reports_export)/.test(command) ? 15 * 60_000 : 60_000),
    });
  } catch {
    throw { code: "CONNECTION", message: "Connection interrupted. Reconnect and refresh the affected record before retrying." } satisfies AppErrorPayload;
  }
  const csrf = response.headers?.get("X-CSRF-Token");
  if (csrf && /^[a-f0-9]{64}$/.test(csrf)) {
    const meta = document.querySelector<HTMLMetaElement>('meta[name="pusthakam-staff-csrf"]');
    if (meta) meta.content = csrf;
  }
  let body: unknown;
  try { body = await response.json(); }
  catch {
    throw { code: "INVALID_RESPONSE", message: "The library server returned an unreadable response." } satisfies AppErrorPayload;
  }
  if (!response.ok) {
    const error = typeof body === "object" && body !== null && "error" in body ? body.error : body;
    // Internal backend errors must use the same sanitized contract as desktop IPC.
    if (response.status === 401 || isPayload(error) && error.code === "AUTH_FAILED") window.dispatchEvent(new Event("pusthakam-session-ended"));
    if (isPayload(error) && ["VALIDATION", "NOT_FOUND", "CONFLICT", "PERMISSION_DENIED", "AUTH_FAILED"].includes(error.code)) throw error;
    if (response.status === 401) throw { code: "AUTH_FAILED", message: "Your session ended. Sign in again." } satisfies AppErrorPayload;
    if (response.status === 403) throw { code: "PERMISSION_DENIED", message: "Your account cannot perform this action." } satisfies AppErrorPayload;
    throw { code: "SERVER", message: "The library server could not complete this request." } satisfies AppErrorPayload;
  }
  if (typeof body !== "object" || body === null || !("result" in body)) {
    throw { code: "INVALID_RESPONSE", message: "The library server returned an incomplete response." } satisfies AppErrorPayload;
  }
  if (command === "current_session" && body.result === null) window.dispatchEvent(new Event("pusthakam-session-ended"));
  return body.result as T;
}
