import { ApiError } from "@/lib/api";

export interface IsbnMetadata {
  title?: string;
  subtitle?: string;
  authors: string[];
  publisher?: string;
  publicationPlace?: string;
  publicationYear?: string;
  pagination?: string;
  language?: string;
  classificationNumber?: string;
  subjects: string[];
  abstract?: string;
  source: "OpenLibrary" | "Google Books";
}

export function normalizeIsbn(raw: string): string | null {
  const code = raw.replace(/[-\s]/g, "").toUpperCase();
  return /^(?:\d{13}|\d{9}[\dX])$/.test(code) ? code : null;
}

function extractYear(raw?: string): string | undefined {
  if (!raw) return undefined;
  const match = raw.match(/\b(\d{4})\b/);
  return match?.[1];
}

function firstString(value: unknown): string | undefined {
  if (typeof value === "string" && value.trim()) return value.trim();
  if (Array.isArray(value)) {
    for (const item of value) {
      if (typeof item === "string" && item.trim()) return item.trim();
      if (item && typeof item === "object" && "name" in item && typeof (item as { name: unknown }).name === "string") {
        const name = (item as { name: string }).name.trim();
        if (name) return name;
      }
    }
  }
  return undefined;
}

export function parseOpenLibraryBook(payload: unknown, code: string): IsbnMetadata | null {
  if (!payload || typeof payload !== "object") return null;
  const record = (payload as Record<string, unknown>)[`ISBN:${code}`] as Record<string, unknown> | undefined;
  if (!record || typeof record !== "object") return null;

  const title = typeof record.title === "string" ? record.title.trim() : undefined;
  if (!title) return null;

  const authors = Array.isArray(record.authors)
    ? record.authors
        .map((a) => (a && typeof a === "object" && "name" in a && typeof a.name === "string" ? a.name.trim() : ""))
        .filter(Boolean)
    : [];

  const subjects = Array.isArray(record.subjects)
    ? record.subjects
        .map((s) => (s && typeof s === "object" && "name" in s && typeof s.name === "string" ? s.name.trim() : ""))
        .filter(Boolean)
        .slice(0, 8)
    : [];

  const classifications = record.classifications as Record<string, unknown> | undefined;
  const dewey = classifications ? firstString(classifications.dewey_decimal_class) : undefined;
  const pages = typeof record.number_of_pages === "number" && record.number_of_pages > 0
    ? `${record.number_of_pages} p.`
    : typeof record.pagination === "string" && record.pagination.trim()
      ? record.pagination.trim()
      : undefined;

  return {
    title,
    subtitle: typeof record.subtitle === "string" && record.subtitle.trim() ? record.subtitle.trim() : undefined,
    authors,
    publisher: firstString(record.publishers),
    publicationPlace: firstString(record.publish_places),
    publicationYear: extractYear(typeof record.publish_date === "string" ? record.publish_date : undefined),
    pagination: pages,
    classificationNumber: dewey,
    subjects,
    abstract: typeof record.notes === "string" && record.notes.trim() ? record.notes.trim() : undefined,
    source: "OpenLibrary",
  };
}

export function parseGoogleBooksVolume(payload: unknown): IsbnMetadata | null {
  if (!payload || typeof payload !== "object") return null;
  const items = (payload as { items?: unknown }).items;
  if (!Array.isArray(items) || items.length === 0) return null;
  const info = (items[0] as { volumeInfo?: Record<string, unknown> })?.volumeInfo;
  if (!info || typeof info !== "object") return null;

  const title = typeof info.title === "string" ? info.title.trim() : undefined;
  if (!title) return null;

  const authors = Array.isArray(info.authors)
    ? info.authors.filter((a): a is string => typeof a === "string" && a.trim() !== "").map((a) => a.trim())
    : [];
  const subjects = Array.isArray(info.categories)
    ? info.categories.filter((c): c is string => typeof c === "string" && c.trim() !== "").map((c) => c.trim())
    : [];

  return {
    title,
    subtitle: typeof info.subtitle === "string" && info.subtitle.trim() ? info.subtitle.trim() : undefined,
    authors,
    publisher: typeof info.publisher === "string" && info.publisher.trim() ? info.publisher.trim() : undefined,
    publicationYear: extractYear(typeof info.publishedDate === "string" ? info.publishedDate : undefined),
    pagination: typeof info.pageCount === "number" && info.pageCount > 0 ? `${info.pageCount} p.` : undefined,
    language: typeof info.language === "string" && info.language.trim() ? info.language.trim() : undefined,
    subjects,
    abstract: typeof info.description === "string" && info.description.trim() ? info.description.trim() : undefined,
    source: "Google Books",
  };
}

/** Explicit user-triggered ISBN lookup. Sends only the ISBN; never sends library data. */
export async function lookupIsbnMetadata(isbn: string): Promise<IsbnMetadata> {
  const code = normalizeIsbn(isbn);
  if (!code) {
    throw new ApiError({ code: "VALIDATION", message: "Enter a valid 10- or 13-digit ISBN first." });
  }
  if (typeof navigator !== "undefined" && navigator.onLine === false) {
    throw new ApiError({
      code: "NETWORK",
      message: "You appear to be offline. Connect to the internet to look up an ISBN, or enter the details manually.",
    });
  }

  const fetchOpts: RequestInit = {
    signal: AbortSignal.timeout(10000),
    credentials: "omit",
    referrerPolicy: "no-referrer",
  };

  try {
    const olRes = await fetch(
      `https://openlibrary.org/api/books?bibkeys=ISBN:${code}&format=json&jscmd=data`,
      fetchOpts,
    );
    if (olRes.ok) {
      const parsed = parseOpenLibraryBook(await olRes.json(), code);
      if (parsed) return parsed;
    }
  } catch {
    // Fall back to Google Books if OpenLibrary is unreachable or times out.
  }

  try {
    const gbRes = await fetch(
      `https://www.googleapis.com/books/v1/volumes?q=isbn:${code}`,
      fetchOpts,
    );
    if (gbRes.ok) {
      const parsed = parseGoogleBooksVolume(await gbRes.json());
      if (parsed) return parsed;
    }
  } catch {
    throw new ApiError({
      code: "NETWORK",
      message: "Could not reach book lookup services. Check your internet connection or enter details manually.",
    });
  }

  throw new ApiError({
    code: "NOT_FOUND",
    message: "No book details were found for this ISBN. You can enter them manually.",
  });
}
