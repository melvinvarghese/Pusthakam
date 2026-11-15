/**
 * Optional web tools listed in the "Online Tools" menu. They open in the system browser and need
 * internet; Pusthakam itself never depends on them. To add a tool: add an entry here AND allow its
 * address in src-tauri/capabilities/default.json (the app can only open links that capability permits).
 */
export interface OnlineTool {
  id: string;
  name: string;
  description: string;
  /** Plain address of the tool. Use `toolUrl()` to get the address that is actually opened. */
  url: string;
}

/** Marks visits as coming from Pusthakam. The only thing added to a link; no library data is ever sent. */
const UTM_SOURCE = "pusthakam";

export const ONLINE_TOOLS: OnlineTool[] = [
  {
    id: "ddc-list",
    name: "DDC Number List",
    description: "Look up Dewey Decimal Classification numbers.",
    url: "https://melvinvarghese.in/dewey-decimal-classification-list-ddc-codes/",
  },
  {
    id: "library-toolkit",
    name: "Library Toolkit",
    description: "Free tools for librarians, such as barcode labels and CSV clean-up.",
    url: "https://melvinvarghese.github.io/Library-Toolkit/",
  },
];

/** The address to open: the tool's link plus `utm_source=pusthakam`. */
export function toolUrl(tool: OnlineTool): string {
  const u = new URL(tool.url);
  u.searchParams.set("utm_source", UTM_SOURCE);
  return u.toString();
}
