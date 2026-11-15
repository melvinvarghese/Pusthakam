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

export interface ClassificationScheme { id: number; code: string; name: string }

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

export interface RecordDetail extends Omit<RecordInput, "id" | "isbn" | "contributors" | "subjects"> {
  id: number;
  isbn10: string | null;
  isbn13: string | null;
  archived: boolean;
  contributors: Contributor[];
  subjects: string[];
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

export type ImportFormat = "marc" | "csv";
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
