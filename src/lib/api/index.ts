/**
 * The ONLY module that talks to the Rust backend. One typed function per Tauri command.
 * The frontend never accesses SQLite directly.
 */
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { previewMode } from "@/lib/preview";
import type {
  AccessionFormat, AppErrorPayload, AppInfo, ArchiveKind, ConfigOverview, CopyBatchInput, CopyPage, CopyQuery,
  CopyRecord, CopyStatus, CopyUpdateInput, DuplicateHit, DuplicateProbe, ImportOptions, ImportPreview,
  ImportPreviewRequest, ImportReport, LibraryInput, MemberCategoryInput, MemberInput, MemberPage,
  NamedInput, NamedKind, PolicyInput, RecordDetail, RecordInput, SearchQuery, SearchResult, Session, SetupInput,
  SetupStatus, ShelfInput,
} from "./types";

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
  // Development only: synthetic data for reviewing screens in a plain browser. The `import.meta.env.DEV`
  // test is replaced by `false` in production builds, so this whole block (and the sample data) is removed.
  if (import.meta.env.DEV) {
    const preview = previewMode();
    if (preview) {
      const { previewCall } = await import("@/features/dev/preview");
      return (await previewCall(command, preview)) as T;
    }
  }
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

export const authApi = {
  setupStatus: () => call<SetupStatus>("setup_status"),
  completeSetup: (input: SetupInput) => call<Session>("complete_setup", { input }),
  login: (username: string, password: string) => call<Session>("login", { username, password }),
  logout: () => call<void>("logout"),
  currentSession: () => call<Session | null>("current_session"),
};

export const catalogueApi = {
  search: (query: SearchQuery) => call<SearchResult>("catalogue_search", { query }),
  getRecord: (id: number) => call<RecordDetail>("catalogue_get_record", { id }),
  saveRecord: (input: RecordInput) => call<number>("catalogue_save_record", { input }),
  findDuplicates: (probe: DuplicateProbe) => call<DuplicateHit[]>("catalogue_find_duplicates", { probe }),
  archiveRecord: (id: number, archived: boolean) => call<void>("catalogue_archive_record", { id, archived }),
  addCopies: (recordId: number, input: CopyBatchInput) => call<CopyRecord[]>("catalogue_add_copies", { recordId, input }),
  updateCopy: (id: number, input: CopyUpdateInput) => call<void>("catalogue_update_copy", { id, input }),
  setCopyStatus: (id: number, status: CopyStatus, note: string | null) =>
    call<void>("catalogue_set_copy_status", { id, status, note }),
  listCopies: (query: CopyQuery) => call<CopyPage>("catalogue_list_copies", { query }),
  accessionFormat: () => call<AccessionFormat>("accession_format_get"),
  setAccessionFormat: (format: AccessionFormat) => call<void>("accession_format_set", { format }),
};

export const importApi = {
  preview: (request: ImportPreviewRequest) => call<ImportPreview>("import_preview", { request }),
  commit: (token: string, options: ImportOptions) => call<ImportReport>("import_commit", { token, options }),
  cancel: () => call<void>("import_cancel"),
};

export const membersApi = {
  list: (search: string, page: number, archived: boolean) => call<MemberPage>("members_list", { search, page, archived }),
  save: (input: MemberInput) => call<number>("members_save", { input }),
  archive: (id: number, archived: boolean) => call<void>("members_archive", { id, archived }),
};

export const configApi = {
  overview: () => call<ConfigOverview>("config_overview"),
  updateLibrary: (input: LibraryInput) => call<void>("config_update_library", { input }),
  saveNamed: (kind: NamedKind, input: NamedInput) => call<number>("config_save_named", { kind, input }),
  saveMemberCategory: (input: MemberCategoryInput) => call<number>("config_save_member_category", { input }),
  saveShelf: (input: ShelfInput) => call<number>("config_save_shelf", { input }),
  savePolicy: (input: PolicyInput) => call<number>("config_save_policy", { input }),
  deletePolicy: (id: number) => call<void>("config_delete_policy", { id }),
  setArchived: (kind: ArchiveKind, id: number, archived: boolean) =>
    call<void>("config_set_archived", { kind, id, archived }),
};

/**
 * Opens an approved web link in the system browser. In the desktop app the permitted addresses are
 * restricted by src-tauri/capabilities/default.json; outside it (plain browser, tests) a new tab is used.
 * No library data is ever sent: only the address itself is opened.
 */
export async function openExternal(url: string): Promise<void> {
  if (!/^https:\/\//.test(url)) {
    throw new ApiError({ code: "VALIDATION", message: "Only secure web addresses can be opened." });
  }
  if (!isDesktop()) {
    window.open(url, "_blank", "noopener,noreferrer");
    return;
  }
  try {
    await openUrl(url);
  } catch (e) {
    throw toApiError(e);
  }
}
