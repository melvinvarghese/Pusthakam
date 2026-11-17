import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, expect, it, vi } from "vitest";
import { SessionContext } from "@/lib/session";
import { AuditPage } from "./AuditPage";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
afterEach(() => { vi.unstubAllGlobals(); vi.mocked(invoke).mockReset(); });

it("filters displayed audit events by staff and action", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  const rows = [{ id: 1, actor: "Head Librarian", occurredAt: "2026-11-17T12:00:00Z",
    action: "catalogue.record_restored", entityType: "bibliographic_records", entityId: 17, summary: "Restored record" }];
  vi.mocked(invoke).mockResolvedValue({ rows, total: 1, page: 1, pageSize: 25 });
  render(
    <QueryClientProvider client={new QueryClient()}>
      <SessionContext.Provider value={{
        session: { userId: 1, username: "admin", displayName: "Head Librarian", role: "administrator", permissions: ["audit.read"] },
        signOut: async () => {},
      }}>
        <AuditPage />
      </SessionContext.Provider>
    </QueryClientProvider>,
  );
  expect(await screen.findByText("Restored record")).toBeInTheDocument();
  const user = userEvent.setup();
  await user.type(screen.getByLabelText("Staff name"), "Head");
  await user.type(screen.getByLabelText("Action code"), "restored");
  await user.click(screen.getByRole("button", { name: "Search audit log" }));
  await waitFor(() => expect(invoke).toHaveBeenLastCalledWith("audit_search", {
    query: { from: "", to: "", actor: "Head", action: "restored", page: 1 },
  }));
});
