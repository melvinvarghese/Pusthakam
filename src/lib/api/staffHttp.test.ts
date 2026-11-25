import { afterEach, describe, expect, it, vi } from "vitest";
import { appApi, ApiError } from ".";
import { staffHttpCommand, staffHttpConfig } from "./staffHttp";

const config = { endpoint: "/api/staff/commands" as const, csrfToken: "test-csrf" };
afterEach(() => {
  vi.unstubAllGlobals();
  document.head.querySelectorAll('meta[name^="pusthakam-staff-"]').forEach(meta => meta.remove());
  window.history.replaceState(null, "", "/");
});
function installWebMarkers(endpoint = config.endpoint) {
  window.history.replaceState(null, "", "/staff");
  for (const [name, content] of [["api", endpoint], ["csrf", config.csrfToken]]) {
    const meta = document.createElement("meta"); meta.name = `pusthakam-staff-${name}`; meta.content = content!; document.head.append(meta);
  }
}
describe("staff HTTP transport foundation", () => {
  it("requires the staff route and explicit server configuration", () => {
    expect(staffHttpConfig()).toBeNull();
    installWebMarkers();
    expect(staffHttpConfig()).toEqual(config);
    window.history.replaceState(null, "", "/opac");
    expect(staffHttpConfig()).toBeNull();
  });
  it("preserves typed API results and sends cookies/CSRF to the same origin", async () => {
    installWebMarkers();
    const info = { version: "test", schemaVersion: 24, dataDir: "", databasePath: "" };
    const fetch = vi.fn().mockResolvedValue({ ok: true, json: async () => ({ result: info }) });
    vi.stubGlobal("fetch", fetch);
    expect(await appApi.info()).toEqual(info);
    expect(fetch).toHaveBeenCalledWith(config.endpoint, expect.objectContaining({
      method: "POST", credentials: "same-origin", redirect: "error", cache: "no-store",
      headers: { "Content-Type": "application/json", "X-CSRF-Token": config.csrfToken },
      body: JSON.stringify({ command: "app_info", args: {} }),
    }));
  });
  it("propagates permission/conflict errors through the desktop error contract", async () => {
    installWebMarkers();
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue({ ok: false, status: 409, json: async () => ({ error: { code: "CONFLICT", message: "Record changed. Reload it." } }) }));
    await expect(appApi.info()).rejects.toBeInstanceOf(ApiError);
    await expect(appApi.info()).rejects.toMatchObject({ code: "CONFLICT", message: "Record changed. Reload it." });
  });
  it("never exposes raw internal errors or login HTML", async () => {
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue({ ok: false, status: 500, json: async () => ({ error: { code: "DATABASE", message: "secret SQL/path" } }) }));
    await expect(staffHttpCommand(config, "app_info")).rejects.toEqual({ code: "SERVER", message: "The library server could not complete this request." });
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue({ ok: true, json: async () => { throw Error("<html>login</html>"); } }));
    await expect(staffHttpCommand(config, "app_info")).rejects.toMatchObject({ code: "INVALID_RESPONSE" });
  });
  it("reports expired authentication and lost connections without exposing internals", async () => {
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue({ ok: false, status: 401, json: async () => ({}) }));
    await expect(staffHttpCommand(config, "app_info")).rejects.toMatchObject({ code: "AUTH_FAILED" });
    vi.stubGlobal("fetch", vi.fn().mockRejectedValue(Error("socket details")));
    await expect(staffHttpCommand(config, "app_info")).rejects.toMatchObject({ code: "CONNECTION" });
  });
  it("does not dispatch to arbitrary URLs or without CSRF configuration", async () => {
    const fetch = vi.fn(); vi.stubGlobal("fetch", fetch);
    await expect(staffHttpCommand({ ...config, csrfToken: "" }, "app_info")).rejects.toMatchObject({ code: "WEB_NOT_CONFIGURED" });
    await expect(staffHttpCommand(config, "../private")).rejects.toMatchObject({ code: "WEB_NOT_CONFIGURED" });
    expect(fetch).not.toHaveBeenCalled();
  });
});
