import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { Session } from "@/lib/api/types";

const api = vi.hoisted(() => ({
  setupStatus: vi.fn(),
  completeSetup: vi.fn(),
  login: vi.fn(),
  logout: vi.fn(),
  currentSession: vi.fn(),
}));

vi.mock("@/lib/api", async (orig) => ({ ...(await orig<typeof import("@/lib/api")>()), authApi: api }));

import { ApiError } from "@/lib/api";
import { Gate } from "./Gate";
import { useSession } from "@/lib/session";

const SESSION: Session = { userId: 1, username: "admin", displayName: "Head Librarian", role: "administrator", permissions: ["admin.settings"] };

function Inside() {
  const { session } = useSession();
  return <p>Inside the app as {session.displayName}</p>;
}

function renderGate() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={qc}>
      <Gate><Inside /></Gate>
    </QueryClientProvider>,
  );
}

beforeEach(() => Object.values(api).forEach((f) => f.mockReset()));

describe("Gate", () => {
  it("shows a loading status while the library configuration is pending", () => {
    api.setupStatus.mockReturnValue(new Promise(() => {}));
    renderGate();
    expect(screen.getByRole("status")).toHaveTextContent("Opening your library");
    expect(screen.queryByRole("heading", { name: "Sign in" })).not.toBeInTheDocument();
  });

  it("shows the setup wizard on a fresh install", async () => {
    api.setupStatus.mockResolvedValue({ configured: false, profiles: [] });
    renderGate();
    expect(await screen.findByRole("heading", { name: "Set up your library" })).toBeInTheDocument();
    expect(screen.queryByText(/Inside the app/)).not.toBeInTheDocument();
  });

  it("shows sign-in when the library exists but nobody is signed in", async () => {
    api.setupStatus.mockResolvedValue({ configured: true, profiles: [] });
    api.currentSession.mockResolvedValue(null);
    renderGate();
    expect(await screen.findByRole("heading", { name: "Sign in" })).toBeInTheDocument();
    expect(screen.queryByText(/Inside the app/)).not.toBeInTheDocument();
  });

  it("lets the app render once signed in", async () => {
    api.setupStatus.mockResolvedValue({ configured: true, profiles: [] });
    api.currentSession.mockResolvedValue(SESSION);
    renderGate();
    expect(await screen.findByText("Inside the app as Head Librarian")).toBeInTheDocument();
  });

  it("signs in with the form and shows the app", async () => {
    api.setupStatus.mockResolvedValue({ configured: true, profiles: [] });
    api.currentSession.mockResolvedValue(null);
    api.login.mockResolvedValue(SESSION);
    renderGate();
    await userEvent.type(await screen.findByLabelText("Username"), "admin");
    await userEvent.type(screen.getByLabelText("Password"), "a long passphrase");
    await userEvent.click(screen.getByRole("button", { name: "Sign in" }));
    expect(api.login).toHaveBeenCalledWith("admin", "a long passphrase");
    expect(await screen.findByText("Inside the app as Head Librarian")).toBeInTheDocument();
  });

  it("shows the backend's message when sign-in fails and stays on the form", async () => {
    api.setupStatus.mockResolvedValue({ configured: true, profiles: [] });
    api.currentSession.mockResolvedValue(null);
    api.login.mockRejectedValue(new ApiError({ code: "AUTH_FAILED", message: "Incorrect username or password." }));
    renderGate();
    await userEvent.type(await screen.findByLabelText("Username"), "admin");
    await userEvent.type(screen.getByLabelText("Password"), "wrong");
    await userEvent.click(screen.getByRole("button", { name: "Sign in" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Incorrect username or password.");
    expect(screen.queryByText(/Inside the app/)).not.toBeInTheDocument();
  });

  it("explains when the app is opened outside the desktop shell", async () => {
    api.setupStatus.mockRejectedValue(new ApiError({ code: "NOT_DESKTOP", message: "Needs the desktop app." }));
    renderGate();
    expect(await screen.findByText("Open Pusthakam as an app")).toBeInTheDocument();
  });
});
