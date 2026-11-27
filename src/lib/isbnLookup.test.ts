import { afterEach, describe, expect, it, vi } from "vitest";
import { lookupIsbnMetadata, normalizeIsbn, parseGoogleBooksVolume, parseOpenLibraryBook } from "./isbnLookup";

describe("normalizeIsbn", () => {
  it("strips hyphens and spaces for ISBN-10 and ISBN-13", () => {
    expect(normalizeIsbn("978-0-14-032872-1")).toBe("9780140328721");
    expect(normalizeIsbn("0-19-852663-x")).toBe("019852663X");
    expect(normalizeIsbn("12345")).toBeNull();
  });
});

describe("parseOpenLibraryBook", () => {
  it("extracts bibliographic fields, Dewey classification, and subjects", () => {
    const parsed = parseOpenLibraryBook({
      "ISBN:9780140328721": {
        title: "Matilda",
        subtitle: "A Novel",
        authors: [{ name: "Roald Dahl" }],
        publishers: [{ name: "Puffin Books" }],
        publish_places: [{ name: "London" }],
        publish_date: "October 1, 1988",
        number_of_pages: 240,
        classifications: { dewey_decimal_class: ["823.914"] },
        subjects: [{ name: "Children's fiction" }, { name: "Schools" }],
      },
    }, "9780140328721");

    expect(parsed).toEqual({
      title: "Matilda",
      subtitle: "A Novel",
      authors: ["Roald Dahl"],
      publisher: "Puffin Books",
      publicationPlace: "London",
      publicationYear: "1988",
      pagination: "240 p.",
      classificationNumber: "823.914",
      subjects: ["Children's fiction", "Schools"],
      abstract: undefined,
      source: "OpenLibrary",
    });
  });
});

describe("parseGoogleBooksVolume", () => {
  it("extracts bibliographic fields from Google Books volumeInfo", () => {
    const parsed = parseGoogleBooksVolume({
      items: [{
        volumeInfo: {
          title: "Randamoozham",
          authors: ["M. T. Vasudevan Nair"],
          publisher: "Current Books",
          publishedDate: "1984-05-01",
          pageCount: 310,
          language: "ml",
          categories: ["Fiction"],
          description: "Retelling of the Mahabharata from Bhima's perspective.",
        },
      }],
    });

    expect(parsed).toEqual({
      title: "Randamoozham",
      subtitle: undefined,
      authors: ["M. T. Vasudevan Nair"],
      publisher: "Current Books",
      publicationYear: "1984",
      pagination: "310 p.",
      language: "ml",
      subjects: ["Fiction"],
      abstract: "Retelling of the Mahabharata from Bhima's perspective.",
      source: "Google Books",
    });
  });
});

describe("lookupIsbnMetadata", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("falls back to Google Books when OpenLibrary has no match", async () => {
    const fetchMock = vi.fn()
      .mockResolvedValueOnce(new Response(JSON.stringify({}), { status: 200 }))
      .mockResolvedValueOnce(new Response(JSON.stringify({
        items: [{ volumeInfo: { title: "Wings of Fire", authors: ["A. P. J. Abdul Kalam"], publishedDate: "1999" } }],
      }), { status: 200 }));
    vi.stubGlobal("fetch", fetchMock);

    const res = await lookupIsbnMetadata("978-81-7371-146-6");
    expect(res.source).toBe("Google Books");
    expect(res.title).toBe("Wings of Fire");
    expect(res.authors).toEqual(["A. P. J. Abdul Kalam"]);
    expect(res.publicationYear).toBe("1999");
  });
});
