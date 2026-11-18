/** Encode bytes without passing an entire import file as one function argument. */
export async function fileToBase64(file: File): Promise<string> {
  const bytes = new Uint8Array(await file.arrayBuffer());
  let binary = "";
  for (let i = 0; i < bytes.length; i += 0x8000) binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(binary);
}

export function downloadCsvFile(csv: string, filename: string) {
  downloadFile(csv, filename, "text/csv;charset=utf-8");
}

export function downloadTextFile(text: string, filename: string) {
  downloadFile(text, filename, "text/plain;charset=utf-8");
}

function downloadFile(content: string, filename: string, type: string) {
  const urls = URL;
  const url = urls.createObjectURL(new Blob([content], { type }));
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  document.body.append(link);
  link.click();
  link.remove();
  window.setTimeout(() => urls.revokeObjectURL(url), 1000);
}

export function downloadBase64File(data: string, filename: string, type: string) {
  const binary = atob(data);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  const url = URL.createObjectURL(new Blob([bytes], { type }));
  const link = document.createElement("a");
  link.href = url; link.download = filename; document.body.append(link); link.click(); link.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 1000);
}
