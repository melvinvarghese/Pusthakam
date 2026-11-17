import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, expect, it, vi } from "vitest";
import { LoginScreen } from "./LoginScreen";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
afterEach(() => { vi.unstubAllGlobals(); vi.mocked(invoke).mockReset(); });

it("returns to sign in after a successful administrator recovery", async () => {
  vi.stubGlobal("__TAURI_INTERNALS__", {});
  vi.mocked(invoke).mockResolvedValue(undefined);
  render(<QueryClientProvider client={new QueryClient()}><LoginScreen /></QueryClientProvider>);
  const user = userEvent.setup();
  await user.click(screen.getByRole("button", { name: "Use a recovery code" }));
  expect(screen.getByRole("heading", { name: "Recover administrator account" })).toBeInTheDocument();
  await user.type(screen.getByLabelText("Administrator username"), "admin");
  await user.type(screen.getByLabelText("Recovery code"), "1".repeat(64));
  await user.type(screen.getByLabelText("New password", { exact: true }), "a replacement passphrase");
  await user.type(screen.getByLabelText("Confirm new password"), "a replacement passphrase");
  await user.click(screen.getByRole("button", { name: "Reset password" }));
  expect(await screen.findByText("Password reset. Sign in with your new password.")).toBeInTheDocument();
  expect(invoke).toHaveBeenCalledWith("staff_recover", {
    username: "admin", code: "1".repeat(64), newPassword: "a replacement passphrase",
  });
});
