import { ApiError } from "@/lib/api";

export async function fetchIsbnCover(isbn: string): Promise<string> {
  const code = isbn.replace(/[-\s]/g, "");
  if (!/^(?:\d{13}|\d{9}[\dXx])$/.test(code)) {
    throw new ApiError({ code: "VALIDATION", message: "Enter a valid ISBN in the book record first." });
  }
  const response = await fetch(`https://covers.openlibrary.org/b/isbn/${code}-M.jpg?default=false`, {
    signal: AbortSignal.timeout(15000), credentials: "omit", referrerPolicy: "no-referrer",
  });
  if (response.status === 404) {
    throw new ApiError({ code: "NOT_FOUND", message: "No cover was found for this ISBN. You can choose a local image." });
  }
  if (!response.ok || !response.body) throw new Error("Could not load the cover. Check your internet connection and retry.");
  const reader = response.body.getReader();
  const chunks: Uint8Array<ArrayBuffer>[] = [];
  let size = 0;
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      size += value.byteLength;
      if (size > 10 * 1024 * 1024) throw new Error("The cover image is too large.");
      chunks.push(new Uint8Array(value));
    }
  } finally {
    await reader.cancel();
    reader.releaseLock();
  }
  return prepareImage(new File(chunks, "isbn-cover.jpg", { type: "image/jpeg" }));
}

export async function prepareImage(file: File): Promise<string> {
  const invalid = new ApiError({ code: "VALIDATION", message: "Choose a JPG or PNG image up to 10 MB." });
  if (file.size > 10 * 1024 * 1024 || !/\.(jpe?g|png)$/i.test(file.name)) throw invalid;
  let bitmap: ImageBitmap;
  try { bitmap = await createImageBitmap(file); }
  catch { throw invalid; }
  try {
    const scale = Math.min(1, 512 / Math.max(bitmap.width, bitmap.height));
    const canvas = document.createElement("canvas");
    canvas.width = Math.max(1, Math.round(bitmap.width * scale));
    canvas.height = Math.max(1, Math.round(bitmap.height * scale));
    const context = canvas.getContext("2d");
    if (!context) throw invalid;
    context.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
    return canvas.toDataURL("image/png").slice("data:image/png;base64,".length);
  } finally {
    bitmap.close();
  }
}
