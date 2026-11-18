import { afterEach, expect, it, vi } from "vitest";
import { fetchIsbnCover } from "./images";

afterEach(() => vi.unstubAllGlobals());

it("reports a missing ISBN cover without saving a placeholder", async () => {
  const fetch = vi.fn().mockResolvedValue({ status: 404 });
  vi.stubGlobal("fetch", fetch);
  await expect(fetchIsbnCover("978-0140328721")).rejects.toThrow("No cover was found");
  expect(fetch).toHaveBeenCalledWith(
    "https://covers.openlibrary.org/b/isbn/9780140328721-M.jpg?default=false",
    expect.objectContaining({ credentials: "omit", referrerPolicy: "no-referrer" }),
  );
});

it("rejects an invalid identifier before contacting the cover source", async () => {
  const fetch = vi.fn();
  vi.stubGlobal("fetch", fetch);
  await expect(fetchIsbnCover("../private")).rejects.toThrow("Enter a valid ISBN");
  expect(fetch).not.toHaveBeenCalled();
});
