/**
 * The ONLY module that talks to the Rust backend. One typed function per Tauri command.
 * The frontend never accesses SQLite directly.
 */
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { previewMode } from "@/lib/preview";
import type {
  AccessionFormat, AppErrorPayload, AppInfo, ArchiveKind, CirculationDay, ConfigOverview, CopyBatchInput, CopyPage, CopyQuery,
  CopyRecord, CopyStatus, CopyUpdateInput, DashboardOverview, DuplicateHit, DuplicateProbe, FineAmount, FineHistoryPage, FinePage, FineSettlement, ImportOptions, ImportPreview,
  ImportPreviewRequest, ImportReport, IssuePreview, KohaPatronPreview, LibraryInput, MemberCategoryInput, MemberInput, MemberLoanPage, MemberPage, OverduePage, RenewalPreview, ReservationPage, ReturnCondition, ReturnPreview,
  NamedInput, NamedKind, PolicyInput, PopularTitle, RecordDetail, RecordInput, SearchQuery, SearchResult, Session, SetupInput,
  SetupStatus, ShelfInput, StockRow, FineSummary, CatalogueGrowthDay, MemberSummary, BackupFile, BackupList, AuditPage, AuditQuery,
  NewStaff, StaffChanges, StaffDirectory,
  ScanOutcome, StockCheck, StockCheckPage, StockCheckReview, StockCheckView, StockTransfer,
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
      return (await previewCall(command, preview, args)) as T;
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
  setting: (key: string) => call<string | null>("get_setting", { key }),
  setSetting: (key: string, value: string) => call<void>("set_setting", { key, value }),
  info: () => call<AppInfo>("app_info"),
  overview: () => call<DashboardOverview>("dashboard_overview"),
};

export const reportsApi = {
  circulationActivity: (from: string, to: string) => call<CirculationDay[]>("reports_circulation_activity", { from, to }),
  popularTitles: (from: string, to: string) => call<PopularTitle[]>("reports_popular_titles", { from, to }),
  stockByLocation: () => call<StockRow[]>("reports_stock_by_location"),
  fineSummary: () => call<FineSummary[]>("reports_fine_summary"),
  memberSummary: () => call<MemberSummary[]>("reports_member_summary"),
  catalogueGrowth: (from: string, to: string) => call<CatalogueGrowthDay[]>("reports_catalogue_growth", { from, to }),
};

export const backupApi = {
  list: () => call<BackupList>("backup_list"),
  create: () => call<BackupFile>("backup_create"),
  copyToFolder: (name: string, folder: string) => call<BackupFile>("backup_copy_to_folder", { name, folder }),
  setDaily: (enabled: boolean) => call<void>("backup_set_daily", { enabled }),
  verify: (name: string) => call<number>("backup_verify", { name }),
  requestRestore: (name: string) => call<void>("backup_request_restore", { name }),
  cancelRestore: () => call<void>("backup_cancel_restore"),
};

export const auditApi = {
  search: (query: AuditQuery) => call<AuditPage>("audit_search", { query }),
};

export const staffApi = {
  list: () => call<StaffDirectory>("staff_list"),
  create: (input: NewStaff) => call<number>("staff_create", { input }),
  update: (id: number, input: StaffChanges) => call<void>("staff_update", { id, input }),
  resetPassword: (id: number, password: string) => call<void>("staff_reset_password", { id, password }),
  unlock: (id: number) => call<void>("staff_unlock", { id }),
  changePassword: (currentPassword: string, newPassword: string) => call<void>("staff_change_password", { currentPassword, newPassword }),
  generateRecoveryCode: (currentPassword: string) => call<string>("staff_generate_recovery_code", { currentPassword }),
  recover: (username: string, code: string, newPassword: string) => call<void>("staff_recover", { username, code, newPassword }),
};

export const inventoryApi = {
  list: (page: number) => call<StockCheckPage<StockCheck>>("stock_check_list", { page }),
  review: (id: number, view: StockCheckView, page: number) => call<StockCheckReview>("stock_check_review", { id, view, page }),
  start: (locationId: number, shelfId: number | null) => call<number>("stock_check_start", { locationId, shelfId }),
  scan: (id: number, barcode: string) => call<ScanOutcome>("stock_check_scan", { id, barcode }),
  close: (id: number) => call<void>("stock_check_close", { id }),
  transfers: (page: number) => call<StockCheckPage<StockTransfer>>("stock_transfer_list", { page }),
  sendTransfer: (barcode: string, locationId: number, shelfId: number | null) => call<number>("stock_transfer_send", { barcode, locationId, shelfId }),
  finishTransfer: (id: number, receive: boolean) => call<void>("stock_transfer_finish", { id, receive }),
};

export const authApi = {
  setupStatus: () => call<SetupStatus>("setup_status"),
  completeSetup: (input: SetupInput) => call<Session>("complete_setup", { input }),
  login: (username: string, password: string) => call<Session>("login", { username, password }),
  logout: () => call<void>("logout"),
  currentSession: () => call<Session | null>("current_session"),
};

export const catalogueApi = {
  getCover: (id: number) => call<string | null>("catalogue_get_cover", { id }),
  setCover: (id: number, pngBase64: string | null) => call<void>("catalogue_set_cover", { id, pngBase64 }),
  search: (query: SearchQuery) => call<SearchResult>("catalogue_search", { query }),
  getRecord: (id: number) => call<RecordDetail>("catalogue_get_record", { id }),
  saveRecord: (input: RecordInput) => call<number>("catalogue_save_record", { input }),
  findDuplicates: (probe: DuplicateProbe) => call<DuplicateHit[]>("catalogue_find_duplicates", { probe }),
  archiveRecord: (id: number, archived: boolean) => call<void>("catalogue_archive_record", { id, archived }),
  archiveRecords: (ids: number[]) => call<void>("catalogue_archive_records", { ids }),
  addCopies: (recordId: number, input: CopyBatchInput) => call<CopyRecord[]>("catalogue_add_copies", { recordId, input }),
  updateCopy: (id: number, input: CopyUpdateInput) => call<void>("catalogue_update_copy", { id, input }),
  setCopyStatus: (id: number, status: CopyStatus, note: string | null) =>
    call<void>("catalogue_set_copy_status", { id, status, note }),
  listCopies: (query: CopyQuery) => call<CopyPage>("catalogue_list_copies", { query }),
  exportCopies: (query: CopyQuery) => call<string>("catalogue_export_copies", { query }),
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
  listLoans: (memberId: number, page: number) => call<MemberLoanPage>("members_list_loans", { memberId, page }),
  previewKoha: (dataBase64: string) => call<KohaPatronPreview>("members_preview_koha", { dataBase64 }),
  importKoha: (dataBase64: string, mappings: Record<string, number>, expectedReady: number) =>
    call<number>("members_import_koha", { dataBase64, mappings, expectedReady }),
  getPhoto: (id: number) => call<string | null>("members_get_photo", { id }),
  setPhoto: (id: number, pngBase64: string | null) => call<void>("members_set_photo", { id, pngBase64 }),
};

export const circulationApi = {
  prepareIssue: (memberNumber: string, barcode: string) =>
    call<IssuePreview>("circulation_prepare_issue", { memberNumber, barcode }),
  prepareReturn: (barcode: string) => call<ReturnPreview>("circulation_prepare_return", { barcode }),
  issue: (memberNumber: string, barcode: string, { memberId, itemId, dueOn }: IssuePreview) =>
    call<void>("circulation_issue", { memberNumber, barcode, memberId, itemId, dueOn }),
  returnCopy: (loanId: number, condition: ReturnCondition = "available") =>
    call<FineAmount | null>("circulation_return", condition === "available" ? { loanId } : { loanId, condition }),
  prepareRenewal: (barcode: string) => call<RenewalPreview>("circulation_prepare_renewal", { barcode }),
  listOverdue: (page: number) => call<OverduePage>("circulation_list_overdue", { page }),
  listReservations: (page: number) => call<ReservationPage>("circulation_list_reservations", { page }),
  placeReservation: (memberNumber: string, barcode: string) =>
    call<void>("circulation_place_reservation", { memberNumber, barcode }),
  cancelReservation: (id: number) => call<void>("circulation_cancel_reservation", { id }),
  listFines: (page: number, includeSettled: boolean) => call<FinePage>("circulation_list_fines", { page, includeSettled }),
  settleFine: (fineId: number, amountMinor: number, expectedBalance: number, entry: FineSettlement) =>
    call<void>("circulation_settle_fine", { fineId, amountMinor, expectedBalance, entry }),
  fineHistory: (fineId: number, page: number) => call<FineHistoryPage>("circulation_fine_history", { fineId, page }),
  renew: ({ loanId, previousDueOn, dueOn }: RenewalPreview) =>
    call<void>("circulation_renew", { confirmation: { loanId, previousDueOn, dueOn } }),
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
