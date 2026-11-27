/** DTOs mirrored from Rust (`src-tauri/src/services` and `commands`). Keep in sync; Rust is the source of truth. */

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

export interface DashboardOverview {
  catalogue: { titles: number; copies: number; available: number } | null;
  activeMembers: number | null;
  circulation: {
    onLoan: number;
    dueToday: number;
    overdue: number;
    readyHolds: number;
    outstandingFines: number;
  } | null;
}

export interface CirculationDay {
  date: string;
  issued: number;
  returned: number;
  lost: number;
  renewed: number;
  reservations: number;
}

export interface PopularTitle {
  recordId: number;
  title: string;
  archived: boolean;
  issues: number;
}

export interface StockRow {
  locationId: number | null;
  location: string | null;
  archivedLocation: boolean;
  available: number;
  onLoan: number;
  held: number;
  lost: number;
  damaged: number;
  other: number;
  total: number;
}

export interface OverdueRow {
  loanId: number;
  memberId: number;
  memberName: string;
  memberBarcode: string;
  itemId: number;
  accessionNo: string;
  barcode: string;
  title: string;
  dueDate: string;
  daysOverdue: number;
}

export interface AccessionRow {
  itemId: number;
  accessionNo: string;
  barcode: string;
  title: string;
  acquisitionDate: string | null;
  acquisitionSource: string;
  priceMinor: number | null;
  location: string | null;
  status: string;
}

export interface ExpenditureRow {
  acquisitionSource: string;
  copies: number;
  totalMinor: number;
}

export interface LossDamageRow {
  loanId: number;
  returnCondition: string;
  returnedDate: string;
  memberId: number;
  memberName: string;
  itemId: number;
  accessionNo: string;
  title: string;
}

export interface ReportExportFile {
  fileName: string;
  mimeType: string;
  dataBase64: string;
}

// ----- Acquisitions -----

export interface Vendor {
  id: number;
  name: string;
  code: string | null;
  contactName: string | null;
  email: string | null;
  phone: string | null;
  address: string | null;
  notes: string | null;
  archived: boolean;
  createdAt: string;
}

export interface VendorInput {
  id?: number | null;
  name: string;
  code?: string | null;
  contactName?: string | null;
  email?: string | null;
  phone?: string | null;
  address?: string | null;
  notes?: string | null;
}

export interface AcquisitionBudget {
  id: number;
  name: string;
  code: string | null;
  amountMinor: number;
  currency: string;
  fiscalYear: string | null;
  allocatedMinor: number;
  archived: boolean;
  createdAt: string;
}

export interface BudgetInput {
  id?: number | null;
  name: string;
  code?: string | null;
  amountMinor: number;
  currency: string;
  fiscalYear?: string | null;
}

export interface AcquisitionRequest {
  id: number;
  title: string;
  author: string | null;
  isbn: string | null;
  publisher: string | null;
  publicationYear: number | null;
  estimatedPriceMinor: number | null;
  currency: string;
  copies: number;
  requestedBy: string;
  requestReason: string | null;
  status: "pending" | "approved" | "rejected" | "ordered" | "received";
  rejectionReason: string | null;
  budgetId: number | null;
  budgetName: string | null;
  createdAt: string;
}

export interface CreateRequestInput {
  title: string;
  author?: string | null;
  isbn?: string | null;
  publisher?: string | null;
  publicationYear?: number | null;
  estimatedPriceMinor?: number | null;
  currency?: string | null;
  copies?: number | null;
  requestedBy: string;
  requestReason?: string | null;
}

export interface PurchaseOrderItem {
  id: number;
  orderId: number;
  requestId: number | null;
  title: string;
  author: string | null;
  isbn: string | null;
  quantity: number;
  unitPriceMinor: number;
  receivedQuantity: number;
  notes: string | null;
}

export interface PurchaseOrderItemInput {
  requestId?: number | null;
  title: string;
  author?: string | null;
  isbn?: string | null;
  quantity: number;
  unitPriceMinor: number;
  notes?: string | null;
}

export interface PurchaseOrderSummary {
  id: number;
  orderNumber: string;
  vendorId: number;
  vendorName: string;
  budgetId: number | null;
  budgetName: string | null;
  status: "draft" | "ordered" | "partially_received" | "received" | "cancelled";
  invoiceRef: string | null;
  orderedAt: string | null;
  itemsCount: number;
  totalMinor: number;
  createdAt: string;
}

export interface PurchaseOrderDetail {
  id: number;
  orderNumber: string;
  vendorId: number;
  vendorName: string;
  budgetId: number | null;
  budgetName: string | null;
  status: "draft" | "ordered" | "partially_received" | "received" | "cancelled";
  invoiceRef: string | null;
  orderedAt: string | null;
  expectedAt: string | null;
  notes: string | null;
  items: PurchaseOrderItem[];
  totalMinor: number;
  createdAt: string;
}

export interface CreateOrderInput {
  orderNumber: string;
  vendorId: number;
  budgetId?: number | null;
  invoiceRef?: string | null;
  expectedAt?: string | null;
  notes?: string | null;
  items: PurchaseOrderItemInput[];
}

export interface ReceiveItemInput {
  orderId: number;
  itemId: number;
  quantity: number;
  recordId?: number | null;
  itemCategoryId?: number | null;
  locationId?: number | null;
  collectionId?: number | null;
  shelfId?: number | null;
  callNumber?: string | null;
}

export interface CreatedCopyInfo {
  copyId: number;
  accessionNo: string;
  barcode: string;
}

export interface ReceiveResult {
  orderId: number;
  itemId: number;
  recordId: number;
  receivedQuantity: number;
  orderStatus: string;
  createdCopies: CreatedCopyInfo[];
}

// ----- serials / periodicals -----

export interface SerialSubscription {
  id: number;
  title: string;
  issn: string | null;
  publisher: string | null;
  frequency: string;
  vendorId: number | null;
  vendorName: string | null;
  budgetId: number | null;
  budgetName: string | null;
  costMinor: number;
  currency: string;
  startDate: string;
  endDate: string;
  status: "active" | "expired" | "cancelled";
  notes: string | null;
  archived: boolean;
  totalIssuesCount: number;
  receivedIssuesCount: number;
  missingIssuesCount: number;
  expectedIssuesCount: number;
  createdAt: string;
}

export interface SaveSubscriptionInput {
  id?: number | null;
  title: string;
  issn?: string | null;
  publisher?: string | null;
  frequency: string;
  vendorId?: number | null;
  budgetId?: number | null;
  costMinor?: number | null;
  currency?: string | null;
  startDate: string;
  endDate: string;
  notes?: string | null;
  autoGenerateIssues?: boolean | null;
  issuePatternPrefix?: string | null;
}

export interface SerialIssue {
  id: number;
  subscriptionId: number;
  subscriptionTitle: string;
  issueNumber: string;
  expectedDate: string;
  receivedDate: string | null;
  status: "expected" | "received" | "missing" | "claimed" | "cancelled";
  barcode: string | null;
  accessionNo: string | null;
  physicalItemId: number | null;
  claimCount: number;
  claimDate: string | null;
  notes: string | null;
  receivedBy: number | null;
  createdAt: string;
}

export interface AddIssueInput {
  subscriptionId: number;
  issueNumber: string;
  expectedDate: string;
  notes?: string | null;
}

export interface CheckInIssueInput {
  issueId: number;
  receivedDate?: string | null;
  accessionCopy?: boolean | null;
  itemCategoryId?: number | null;
  locationId?: number | null;
  notes?: string | null;
}

export interface CheckInResult {
  issueId: number;
  status: string;
  receivedDate: string;
  accessionNo: string | null;
  barcode: string | null;
  copyId: number | null;
}

export interface SubscriptionFilter {
  status?: string | null;
  search?: string | null;
}

export interface IssueFilter {
  subscriptionId?: number | null;
  status?: string | null;
  startDate?: string | null;
  endDate?: string | null;
}

export interface SerialRenewal {
  id: number;
  subscriptionId: number;
  startDate: string;
  endDate: string;
  costMinor: number;
  currency: string;
  budgetId: number | null;
  budgetName: string | null;
  notes: string | null;
  createdAt: string;
}

export interface RenewSubscriptionInput {
  subscriptionId: number;
  startDate: string;
  endDate: string;
  costMinor: number;
  currency?: string | null;
  budgetId?: number | null;
  notes?: string | null;
  autoGenerateIssues?: boolean | null;
  issuePatternPrefix?: string | null;
}

export interface SerialsExpenditureItem {
  id: number;
  subscriptionId: number;
  title: string;
  issn: string | null;
  itemType: "initial" | "renewal";
  date: string;
  startDate: string;
  endDate: string;
  costMinor: number;
  currency: string;
  budgetId: number | null;
  budgetName: string | null;
  notes: string | null;
}

export interface SerialsExpenditureSummary {
  fromDate: string | null;
  toDate: string | null;
  totalSpendMinor: number;
  currency: string;
  itemCount: number;
  items: SerialsExpenditureItem[];
}

export interface ExpiringSubscription {
  subscription: SerialSubscription;
  daysRemaining: number;
  isOverdue: boolean;
}

// ----- classification & authorities -----

export interface ClassificationScheme {
  id: number;
  code: string;
  name: string;
  description?: string | null;
  isCustom?: boolean;
  entryCount?: number;
  recordCount?: number;
}

export interface ClassificationSchemeInput {
  id?: number | null;
  code: string;
  name: string;
  description?: string | null;
}

export interface ClassificationEntry {
  id: number;
  schemeId: number;
  schemeCode: string;
  code: string;
  parentCode: string | null;
  title: string;
  description: string | null;
  isCustom: boolean;
  recordCount: number;
}

export interface ClassificationEntryInput {
  id?: number | null;
  schemeId: number;
  code: string;
  parentCode?: string | null;
  title: string;
  description?: string | null;
}

export interface AuthorAuthority {
  id: number;
  name: string;
  sortName: string | null;
  kind: "personal" | "corporate" | "meeting" | "uniform_title" | string;
  birthYear: number | null;
  deathYear: number | null;
  authorityNote: string | null;
  variantNames: string | null;
  marcTag: string;
  recordCount: number;
  createdAt: string;
  updatedAt: string | null;
}

export interface AuthorAuthorityInput {
  id?: number | null;
  name: string;
  sortName?: string | null;
  kind?: string | null;
  birthYear?: number | null;
  deathYear?: number | null;
  authorityNote?: string | null;
  variantNames?: string | null;
  marcTag?: string | null;
}

export interface PublisherAuthority {
  id: number;
  name: string;
  place: string | null;
  notes: string | null;
  marcTag: string;
  recordCount: number;
  createdAt: string;
  updatedAt: string | null;
}

export interface PublisherAuthorityInput {
  id?: number | null;
  name: string;
  place?: string | null;
  notes?: string | null;
  marcTag?: string | null;
}

export interface SubjectAuthority {
  id: number;
  heading: string;
  scheme: string;
  scopeNote: string | null;
  broaderTerm: string | null;
  narrowerTerms: string | null;
  seeAlso: string | null;
  marcTag: string;
  recordCount: number;
  updatedAt: string | null;
}

export interface SubjectAuthorityInput {
  id?: number | null;
  heading: string;
  scheme?: string | null;
  scopeNote?: string | null;
  broaderTerm?: string | null;
  narrowerTerms?: string | null;
  seeAlso?: string | null;
  marcTag?: string | null;
}

export interface MarcMapping {
  id: number;
  authorityType: string;
  marcTag: string;
  subfieldCode: string;
  description: string;
  targetField: string;
  isDefault: boolean;
}

export interface MarcMappingInput {
  id?: number | null;
  authorityType: string;
  marcTag: string;
  subfieldCode: string;
  description: string;
  targetField: string;
}

export interface MarcTestInput {
  rawMarcLine: string;
}

export interface MarcTestResult {
  tag: string;
  indicators: string;
  subfields: Record<string, string>;
  matchedMappings: MarcMapping[];
  mappedAuthorityType: string | null;
  targetFields: Record<string, string>;
}

// ----- sign-in and setup -----

export interface Session {
  userId: number;
  username: string;
  displayName: string;
  role: string;
  permissions: string[];
}

export type ProfileCode =
  | "school" | "college" | "university" | "public" | "special" | "research" | "institutional" | "private";

export interface SetupStatus {
  configured: boolean;
  profiles: ProfileCode[];
}

export interface LibraryInput {
  name: string;
  profile: ProfileCode;
  address: string | null;
  contact: string | null;
  currency: string;
  locale: string;
}

export interface SetupInput {
  library: LibraryInput;
  admin: { username: string; displayName: string; password: string };
  applyProfileDefaults: boolean;
  /** Minor units (e.g. paise or cents). 0 = no fines until configured. */
  finePerDayMinor: number;
}

// ----- configuration -----

export type Library = LibraryInput;

export interface Named {
  id: number;
  name: string;
  description: string | null;
  archived: boolean;
}

export interface MemberCategory extends Named {
  membershipDays: number;
}

export interface Shelf {
  id: number;
  locationId: number;
  code: string;
  description: string | null;
  archived: boolean;
}

export interface Policy {
  id: number;
  memberCategoryId: number;
  itemCategoryId: number;
  loanDays: number;
  maxLoans: number;
  maxRenewals: number;
  renewalDays: number;
  finePerDayMinor: number;
  fineGraceDays: number;
  fineCapMinor: number | null;
  reservationHoldDays: number;
}

export interface ConfigOverview {
  library: Library | null;
  classificationSchemes: ClassificationScheme[];
  memberCategories: MemberCategory[];
  itemCategories: Named[];
  collections: Named[];
  locations: Named[];
  shelves: Shelf[];
  policies: Policy[];
}

// ----- catalogue -----

export type ContributorRole = "author" | "additional_author" | "editor" | "translator" | "illustrator" | "other";
export interface Contributor { name: string; role: ContributorRole }

export type CopyStatus = "available" | "on_loan" | "reserved_hold" | "in_repair" | "lost" | "damaged" | "withdrawn" | "in_transit";
export type AcquisitionSource = "purchase" | "donation" | "transfer" | "exchange" | "other";

export interface RecordInput {
  id: number | null;
  title: string;
  subtitle: string | null;
  edition: string | null;
  publisher: string | null;
  publicationPlace: string | null;
  publicationYear: number | null;
  language: string | null;
  physicalDescription: string | null;
  pagination: string | null;
  series: string | null;
  abstract: string | null;
  notes: string | null;
  isbn: string | null;
  classificationSchemeId: number | null;
  classificationNumber: string | null;
  keywords: string | null;
  contributors: Contributor[];
  subjects: string[];
  /** Browse category ids chosen by staff. Omit to keep existing choices and only re-apply DDC rules. */
  categories?: number[];
}

export interface CategoryRef {
  id: number;
  name: string;
}

/** Member-facing OPAC grouping such as "Malayalam Fiction". Separate from subject headings. */
export interface BrowseCategory {
  id: number;
  name: string;
  /** Comma-separated DDC prefixes that place books automatically, e.g. "530" or "894.8123". */
  ddcRules: string | null;
  sortOrder: number;
  isDefault: boolean;
  archived: boolean;
  recordCount: number;
}

export interface BrowseCategoryInput {
  id?: number | null;
  name: string;
  ddcRules: string | null;
  sortOrder?: number | null;
}

export interface CopyRecord {
  id: number;
  recordId: number;
  accessionNo: string;
  barcode: string;
  itemCategoryId: number;
  collectionId: number | null;
  locationId: number | null;
  shelfId: number | null;
  callNumber: string | null;
  acquisitionDate: string | null;
  acquisitionSource: AcquisitionSource;
  priceMinor: number | null;
  vendor: string | null;
  invoiceRef: string | null;
  notes: string | null;
  status: CopyStatus;
}

export interface RecordDetail extends Omit<RecordInput, "id" | "isbn" | "contributors" | "subjects" | "categories"> {
  id: number;
  isbn10: string | null;
  isbn13: string | null;
  archived: boolean;
  contributors: Contributor[];
  subjects: string[];
  categories: CategoryRef[];
  copies: CopyRecord[];
}

export interface RecordSummary {
  id: number;
  title: string;
  subtitle: string | null;
  authors: string;
  publisher: string | null;
  publicationYear: number | null;
  isbn13: string | null;
  classificationNumber: string | null;
  classificationSchemeId: number | null;
  totalCopies: number;
  availableCopies: number;
  archived: boolean;
}

export interface SearchQuery {
  text?: string | null;
  collectionId?: number | null;
  onlyAvailable?: boolean;
  includeArchived?: boolean;
  sort?: "relevance" | "title" | "year" | "recent";
  page: number;
  pageSize: number;
}

export interface SearchResult { total: number; page: number; pageSize: number; rows: RecordSummary[] }

export interface CopyBatchInput {
  count: number;
  accessionNumbers: string[] | null;
  barcodes: string[] | null;
  itemCategoryId: number;
  collectionId: number | null;
  locationId: number | null;
  shelfId: number | null;
  callNumber: string | null;
  acquisitionDate: string | null;
  acquisitionSource: AcquisitionSource;
  priceMinor: number | null;
  vendor: string | null;
  invoiceRef: string | null;
  notes: string | null;
}

export type CopyUpdateInput = Omit<CopyBatchInput, "count" | "accessionNumbers" | "barcodes"> & {
  accessionNo: string;
  barcode: string;
};

export interface CopyQuery { text?: string | null; status?: CopyStatus | null; collectionId?: number | null; page: number; pageSize: number }

export interface CopyRow {
  id: number;
  recordId: number;
  accessionNo: string;
  barcode: string;
  title: string;
  callNumber: string | null;
  status: CopyStatus;
  collection: string | null;
  location: string | null;
  shelf: string | null;
  acquisitionDate: string | null;
  priceMinor: number | null;
}

export interface CopyPage { total: number; page: number; pageSize: number; rows: CopyRow[] }

export interface DuplicateProbe { id: number | null; title: string; isbn: string | null; firstAuthor: string | null; publicationYear: number | null }
export interface DuplicateHit { id: number; title: string; authors: string; publicationYear: number | null; reason: string }
export interface AccessionFormat { prefix: string; padding: number; next: number }

// ----- import -----

export type ImportFormat = "marc" | "marcxml" | "csv" | "xlsx" | "json";
export type ImportProfile = "koha" | "generic";

export interface ImportCopy {
  barcode: string | null;
  accession: string | null;
  callNumber: string | null;
  location: string | null;
  shelf: string | null;
  collection: string | null;
  itemCategory: string | null;
  acquisitionDate: string | null;
  priceMinor: number | null;
  status: string | null;
  skipReason: string | null;
}

export interface ImportRecord {
  row: number;
  title: string;
  contributors: Contributor[];
  isbn: string | null;
  copies: ImportCopy[];
  warnings: string[];
  errors: string[];
  duplicateOf: number | null;
}

export interface ImportPreview {
  token: string;
  fileName: string;
  format: ImportFormat;
  profile: ImportProfile;
  totals: { records: number; copies: number; withWarnings: number; withErrors: number; duplicates: number; copiesSkipped: number };
  newLookups: { locations: string[]; collections: string[]; itemCategories: string[] };
  sample: ImportRecord[];
  parseErrors: string[];
  headers: string[] | null;
  mapping: Record<string, string> | null;
  mappingFields: string[] | null;
}

export interface ImportPreviewRequest {
  fileName: string;
  dataBase64: string;
  format: ImportFormat;
  profile: ImportProfile;
  mapping: Record<string, string> | null;
}

export interface ImportOptions {
  duplicatePolicy: "skip" | "merge" | "new";
  createMissing: boolean;
  defaultItemCategoryId: number | null;
  barcodeAsAccession: boolean;
}

export interface ImportReport {
  createdRecords: number;
  mergedRecords: number;
  skippedRecords: number;
  failedRecords: number;
  copiesAdded: number;
  copiesSkipped: number;
  createdLookups: number;
  problems: { row: number; message: string }[];
}

export type NamedKind = "collection" | "location" | "itemCategory";
export type ArchiveKind = "collection" | "location" | "itemCategory" | "memberCategory" | "shelf";

export interface NamedInput { id: number | null; name: string; description: string | null }
export interface MemberCategoryInput extends NamedInput { membershipDays: number }
export interface ShelfInput { id: number | null; locationId: number; code: string; description: string | null }
export type PolicyInput = Omit<Policy, "id"> & { id: number | null };
export interface MemberInput {
  id: number | null;
  memberNumber: string;
  name: string;
  categoryId: number;
  email: string | null;
  phone: string | null;
  expiresOn: string | null;
  address?: string | null;
  department?: string | null;
  joinedOn?: string | null;
  notes?: string | null;
}

export interface Member extends MemberInput {
  id: number;
  categoryName: string;
  expiresOn: string;
  address?: string | null;
  department?: string | null;
  joinedOn?: string | null;
  notes?: string | null;
  opacUsername?: string | null;
  hasOpacPassword?: boolean;
}

export interface MemberFineBalance {
  currency: string;
  outstandingMinor: number;
}

export interface MemberProfileSummary {
  activeLoans: number;
  overdueLoans: number;
  totalLoans: number;
  activeHolds: number;
  maxLoansLimit: number | null;
  membershipDays: number;
  eligibleToBorrow: boolean;
  unpaidFines: MemberFineBalance[];
}

export interface MemberHold {
  id: number;
  title: string;
  barcode: string;
  status: string;
  placedAt: string;
  expiresAt: string | null;
  closedAt: string | null;
  queuePosition: number | null;
}

export interface MemberActivityItem {
  id: string;
  kind: "loan" | "return" | "renewal" | "fine" | "hold" | "admin";
  occurredAt: string;
  title: string;
  detail: string;
  staffName: string | null;
}

export interface MemberPage {
  rows: Member[];
  total: number;
  page: number;
  pageSize: number;
}

export interface MemberLoan {
  renewalCount?: number;
  importSource?: string | null;
  sourceStaff?: string | null;
  id: number;
  title: string;
  barcode: string;
  issuedOn: string;
  dueOn: string;
  returnedOn: string | null;
  overdue: boolean;
  returnCondition: ReturnCondition | null;
}

export interface MemberLoanPage {
  rows: MemberLoan[];
  total: number;
  page: number;
  pageSize: number;
}

export interface KohaPatronPreview {
  ready: number;
  skipped: number;
  categories: Record<string, number>;
  sample: {
    memberNumber: string;
    name: string;
    categoryCode: string;
    expiresOn: string | null;
  }[];
  problems: string[];
}

export interface LoanDetails {
  memberName: string;
  title: string;
  dueOn: string;
}

export interface IssuePreview extends LoanDetails {
  memberId: number;
  itemId: number;
}

export interface ReturnPreview extends LoanDetails {
  loanId: number;
  estimatedFine?: FineAmount;
}

export type ReturnCondition = "available" | "damaged" | "lost";

export interface RenewalPreview extends ReturnPreview {
  previousDueOn: string;
}

export interface OverdueLoan extends ReturnPreview {
  barcode: string;
  memberNumber: string;
  memberEmail?: string | null;
  memberPhone?: string | null;
  daysOverdue: number;
}

export interface OverduePage {
  rows: OverdueLoan[];
  page: number;
  pageSize: number;
  total: number;
}

export interface Reservation {
  id: number;
  title: string;
  barcode: string;
  memberName: string;
  memberNumber: string;
  status: "pending" | "ready";
  expiresAt: string | null;
}

export interface ReservationPage {
  rows: Reservation[];
  page: number;
  pageSize: number;
  total: number;
}

export interface FineAmount {
  amountMinor: number;
  currency: string;
}

export interface FineSummary {
  currency: string;
  assessed: string;
  paid: string;
  waived: string;
  outstanding: string;
}

export interface CatalogueGrowthDay {
  date: string;
  titlesAdded: number;
  copiesAdded: number;
}

export interface MemberSummary {
  categoryId: number;
  category: string;
  archivedCategory: boolean;
  current: number;
  expired: number;
  archived: number;
  total: number;
}

export interface BackupFile {
  name: string;
  sizeBytes: number;
  /** Folder holding the file: the Pusthakam data folder or the daily backup folder. */
  folder: string;
}

export interface BackupList {
  folder: string;
  files: BackupFile[];
  dailyEnabled: boolean;
  /** Folder chosen for daily backups; null means the Pusthakam data folder. */
  dailyFolder: string | null;
  /** False when the chosen folder cannot be reached, e.g. an unplugged USB drive. */
  dailyFolderAvailable: boolean;
  dailyKeep: number;
  lastDaily: string | null;
  pendingRestore: string | null;
  lastRestore: string | null;
}

export interface AuditQuery {
  from?: string;
  to?: string;
  actor?: string;
  action?: string;
  page: number;
}

export interface StaffRow {
  id: number;
  username: string;
  displayName: string;
  role: string;
  active: boolean;
  locked: boolean;
}

export interface StaffDirectory {
  users: StaffRow[];
  roles: { code: string; name: string }[];
}

export interface StockCheck {
  id: number;
  location: string;
  shelf: string | null;
  startedAt: string;
  closedAt: string | null;
  expected: number;
  matched: number;
  unexpected: number;
}

export interface StockCheckPage<T> {
  rows: T[];
  total: number;
  page: number;
  pageSize: number;
}

export type StockCheckView = "matched" | "missing" | "unexpected";
export type ScanOutcome = "matched" | "unexpected" | "duplicate";

export interface StockCheckReview {
  check: StockCheck;
  entries: StockCheckPage<{ barcode: string; accessionNo: string | null; title: string | null }>;
}

export interface StockTransfer {
  id: number;
  barcode: string;
  title: string;
  source: string;
  destination: string;
  status: "in_transit" | "received" | "cancelled";
  sentAt: string;
  finishedAt: string | null;
}

export type StaffChanges = Pick<StaffRow, "displayName" | "role" | "active">;
export type NewStaff = Pick<StaffRow, "username" | "displayName" | "role"> & { password: string };

export interface AuditPage {
  rows: {
    id: number;
    occurredAt: string;
    actor: string | null;
    action: string;
    entityType: string;
    entityId: number | null;
    summary: string;
  }[];
  total: number;
  page: number;
  pageSize: number;
}

export interface Fine extends FineAmount {
  id: number;
  memberName: string;
  memberNumber: string;
  title: string;
  paidMinor: number;
  waivedMinor: number;
  outstandingMinor: number;
}

export interface FinePage {
  rows: Fine[];
  page: number;
  pageSize: number;
  total: number;
}

export type FineSettlement =
  | { kind: "payment"; method: string }
  | { kind: "waiver"; reason: string };

export interface FineHistoryPage {
  rows: {
    id: number;
    kind: FineSettlement["kind"];
    amountMinor: number;
    method: string | null;
    reason: string | null;
    recordedAt: string;
    staffName: string;
  }[];
  page: number;
  pageSize: number;
  total: number;
}

export type CatalogueExportFormat = "json" | "csv" | "xlsx" | "marc" | "marcxml";
export interface CatalogueExportFile {
  fileName: string; mimeType: string; dataBase64: string; records: number; copies: number; warnings: string[];
}

export interface Holiday { id: number; date: string; name: string }
export interface LibraryCalendar { closedWeekdays: number[]; holidays: Holiday[] }

export interface MemberFineAccount {
  rows: (Fine & { barcode: string; assessedAt: string })[];
  balances: { currency: string; assessedMinor: number; paidMinor: number; waivedMinor: number; outstandingMinor: number }[];
  page: number; pageSize: number; total: number;
}
export interface LoanImportPreview {
  fingerprint: string;
  total: number;
  open: number;
  closed: number;
  invalid: number;
  problems: string[];
}

export interface HistorySummary {
  counts: { reservations: number; fines: number; payments: number; waivers: number };
  balances: { currency: string; assessedMinor: number; paidMinor: number; waivedMinor: number; outstandingMinor: number }[];
}
export interface HistoryPreview {
  sourceSystem: string;
  summary: HistorySummary;
  expected: HistorySummary;
  problems: string[];
  canCommit: boolean;
  fingerprint: string;
}
export interface HistoryBatchSummary { id: number; sourceSystem: string; importedAt: string; summary: HistorySummary }

export interface RenewalImportPreview {
  total: number;
  loans: number;
  invalid: number;
  problems: string[];
  fingerprint: string;
}

// ============================================================================
// LAN Multi-computer Mode & OPAC Types
// ============================================================================

export interface LanConfig {
  listenerRunning?: boolean;
  listenerError?: string | null;
  enabled: boolean;
  bindAddress: string;
  port: number;
  serverName: string;
  serverFingerprint: string;
  opacEnabled: boolean;
  browserStaffEnabled: boolean;
  publicBaseUrl?: string | null;
  activeInterfaces: string[];
}

export interface UpdateLanConfigInput {
  enabled: boolean;
  bindAddress: string;
  port: number;
  serverName: string;
  opacEnabled: boolean;
  browserStaffEnabled: boolean;
  publicBaseUrl?: string | null;
}

export interface LanPairingCode {
  code: string;
  expiresAt: string;
  createdAt: string;
  isValid: boolean;
}

export interface PairedClient {
  clientId: string;
  clientName: string;
  deviceKind: string;
  ipAddress?: string;
  pairedAt: string;
  lastSeenAt: string;
  revokedAt?: string;
}

export interface PairRequest {
  pairingCode: string;
  clientName: string;
  deviceKind?: string;
  clientIp?: string;
}

export interface PairResponse {
  clientId: string;
  serverName: string;
  serverFingerprint: string;
}

export interface LanLoginRequest {
  clientId: string;
  serverFingerprint: string;
  username: string;
  password: string;
}

export interface LanLoginResponse {
  token: string;
  expiresAt: string;
  userId: number;
  username: string;
  displayName: string;
  role: string;
  permissions: string[];
}

export interface PublicOpacItem {
  id: number;
  title: string;
  subtitle?: string;
  authors: string[];
  publisher?: string;
  year?: number;
  isbn?: string;
  classification?: string;
  subjects: string[];
  availableCopies: number;
  totalCopies: number;
  locations: string[];
}

export interface PublicOpacSearchResult {
  items: PublicOpacItem[];
  total: number;
  page: number;
  pageSize: number;
}


export interface OpenLoan {
  loanId: number; title: string; barcode: string; accessionNo: string;
  memberName: string; memberNumber: string; dueOn: string;
}
export interface OpenLoanPage { rows: OpenLoan[]; page: number; pageSize: number; total: number }

export interface StartupSetting {
  /** False where Pusthakam cannot start itself when the user signs in (non-Windows). */
  supported: boolean;
  enabled: boolean;
}
