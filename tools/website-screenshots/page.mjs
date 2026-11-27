// Opens the app's dev preview with the synthetic data in mock.js appended to src/features/dev/preview.ts.
import { readFileSync } from "node:fs";
const mock = readFileSync(new URL("./mock.js", import.meta.url), "utf8");

export async function page(browser, options = {}) {
  const p = await browser.newPage({ viewport: { width: 1440, height: 900 }, ...options });
  await p.route("**/src/features/dev/preview.ts*", async (route) => {
    const response = await route.fetch();
    const body = (await response.text()).replace("export async function previewCall(", "async function basePreviewCall(") + "\n" + mock;
    await route.fulfill({ response, body });
  });
  p.on("pageerror", (error) => console.log("Page error:", error.message));
  return p;
}
