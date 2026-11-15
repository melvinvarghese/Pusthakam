/**
 * Development-only preview switch. Opening the dev server with `?preview` (or `?preview=setup` /
 * `?preview=login`) in a plain browser lets the UI be reviewed with synthetic data. `import.meta.env.DEV`
 * is false in production builds, so none of this can run in a shipped app.
 */
export function previewMode(): string | null {
  if (!import.meta.env.DEV || typeof window === "undefined" || "__TAURI_INTERNALS__" in window) return null;
  const params = new URLSearchParams(window.location.search);
  return params.has("preview") ? params.get("preview") || "app" : null;
}
