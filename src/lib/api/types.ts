/** DTOs mirrored from Rust (`src-tauri/src/commands`). Keep in sync; covered by contract tests. */

export interface AppErrorPayload {
  code: string;
  message: string;
}

export interface AppInfo {
  version: string;
  schemaVersion: number;
  dataDir: string;
  databasePath: string;
}
