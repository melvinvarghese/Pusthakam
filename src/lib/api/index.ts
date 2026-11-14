/**
 * The ONLY module that talks to the Rust backend. One typed function per Tauri command.
 * The frontend never accesses SQLite directly.
 */
import { invoke } from "@tauri-apps/api/core";
import type { AppErrorPayload, AppInfo } from "./types";

export class ApiError extends Error {
  readonly code: string;
  constructor(payload: AppErrorPayload) {
    super(payload.message);
    this.name = "ApiError";
    this.code = payload.code;
  }
}

export function isDesktop(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function isPayload(e: unknown): e is AppErrorPayload {
  return (
    typeof e === "object" && e !== null &&
    typeof (e as AppErrorPayload).code === "string" &&
    typeof (e as AppErrorPayload).message === "string"
  );
}

export function toApiError(e: unknown): ApiError {
  if (e instanceof ApiError) return e;
  if (isPayload(e)) return new ApiError(e);
  if (typeof e === "string") return new ApiError({ code: "UNKNOWN", message: e });
  return new ApiError({ code: "UNKNOWN", message: "An unexpected error occurred." });
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isDesktop()) {
    throw new ApiError({
      code: "NOT_DESKTOP",
      message: "This feature needs the Pusthakam desktop app. It is not available in a plain browser.",
    });
  }
  try {
    return await invoke<T>(command, args);
  } catch (e) {
    throw toApiError(e);
  }
}

export const appApi = {
  info: () => call<AppInfo>("app_info"),
};
