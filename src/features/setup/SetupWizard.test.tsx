import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const api = vi.hoisted(() => ({ completeSetup: vi.fn() }));
const backup = vi.hoisted(() => ({ pickFolder: vi.fn() }));
vi.mock("@/lib/api", async (orig) => ({ ...(await orig<typeof import("@/lib/api")>()), authApi: api, backupApi: backup }));

import { ApiError } from "@/lib/api";
import { SetupWizard, validateStep } from "./SetupWizard";
import { setLanguage } from "@/lib/language";

const FORM = {
  name: "Demo Library", profile: "school" as const, address: "", currency: "INR", applyDefaults: true, fine: "0", backupFolder: "",
  displayName: "Head Librarian", username: "admin", password: "a long passphrase", confirm: "a long passphrase",
};

function renderWizard() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(<QueryClientProvider client={qc}><SetupWizard /></QueryClientProvider>);
}

// Braces matter: returning the mock from beforeEach would make Vitest call it again as "cleanup".
beforeEach(() => {
  setLanguage("en");
  api.completeSetup.mockReset();
  backup.pickFolder.mockReset();
});

describe("validateStep", () => {
  it("accepts a complete form on every step", () => {
    for (const step of [0, 1, 2, 3]) expect(validateStep(step, FORM)).toEqual({});
  });

  it("flags each problem on its own field", () => {
    expect(validateStep(0, { ...FORM, name: " ", currency: "RS" })).toEqual({
      name: expect.any(String), currency: expect.any(String),
    });
    expect(validateStep(1, { ...FORM, fine: "1,5" })).toHaveProperty("fine");
    expect(validateStep(2, { ...FORM, backupFolder: "" })).toEqual({});
    const e = validateStep(3, { ...FORM, displayName: "", username: "a b", password: "short", confirm: "other" });
    expect(Object.keys(e).sort()).toEqual(["confirm", "displayName", "password", "username"]);
  });
});

describe("SetupWizard", () => {
  it("defaults to English and switches setup language without losing entered data", async () => {
    const view = renderWizard();
    expect(screen.getByLabelText("Language / ഭാഷ")).toHaveValue("en");
    await userEvent.type(screen.getByLabelText("Library name"), "My Library");
    await userEvent.selectOptions(screen.getByLabelText("Language / ഭാഷ"), "ml");
    expect(screen.getByLabelText("ലൈബ്രറിയുടെ പേര്")).toHaveValue("My Library");
    expect(document.documentElement.lang).toBe("ml");
    expect(localStorage.getItem("pusthakam.language")).toBe("ml");
    await userEvent.click(screen.getByRole("button", { name: "തുടരുക" }));
    await userEvent.click(screen.getByRole("button", { name: "തുടരുക" }));
    await userEvent.click(screen.getByRole("button", { name: "തുടരുക" }));
    await userEvent.type(screen.getByLabelText("നിങ്ങളുടെ പേര്"), "Melvin");
    await userEvent.type(screen.getByLabelText("ഉപയോക്തൃനാമം"), "admin");
    await userEvent.type(screen.getByLabelText("പാസ്‌വേഡ്"), "long password");
    await userEvent.type(screen.getByLabelText("പാസ്‌വേഡ് വീണ്ടും നൽകുക"), "long password");
    api.completeSetup.mockResolvedValue({});
    await userEvent.click(screen.getByRole("button", { name: "ലൈബ്രറി സൃഷ്ടിക്കുക" }));
    expect(api.completeSetup.mock.calls[0]?.[0].library).toMatchObject({ name: "My Library", locale: "ml" });
    view.unmount();
    renderWizard();
    expect(screen.getByLabelText("Language / ഭാഷ")).toHaveValue("ml");
    await userEvent.selectOptions(screen.getByLabelText("Language / ഭാഷ"), "en");
    expect(screen.getByLabelText("Library name")).toBeInTheDocument();
    expect(document.documentElement.lang).toBe("en");
  });

  it("will not move on without a library name", async () => {
    renderWizard();
    await userEvent.click(screen.getByRole("button", { name: "Continue" }));
    expect(screen.getByText("Enter your library's name.")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Your library" })).toBeInTheDocument();
  });

  it("walks through all steps and sends the setup request with money in minor units", async () => {
    api.completeSetup.mockResolvedValue({ userId: 1, username: "admin", displayName: "Head Librarian", role: "administrator", permissions: [] });
    renderWizard();

    await userEvent.type(screen.getByLabelText("Library name"), "Demo Library");
    await userEvent.selectOptions(screen.getByLabelText("Type of library"), "college");
    await userEvent.click(screen.getByRole("button", { name: "Continue" }));

    const fine = screen.getByLabelText(/Late fine per day/);
    await userEvent.clear(fine);
    await userEvent.type(fine, "2.50");
    await userEvent.click(screen.getByRole("button", { name: "Continue" }));
    // Backups step: leaving the folder empty keeps backups in the data folder.
    await userEvent.click(screen.getByRole("button", { name: "Continue" }));

    await userEvent.type(screen.getByLabelText("Your name"), "Head Librarian");
    await userEvent.type(screen.getByLabelText("Username"), "admin");
    await userEvent.type(screen.getByLabelText("Password"), "a long passphrase");
    await userEvent.type(screen.getByLabelText("Repeat password"), "a long passphrase");
    await userEvent.click(screen.getByRole("button", { name: "Create library" }));

    expect(api.completeSetup).toHaveBeenCalledTimes(1);
    expect(api.completeSetup.mock.calls[0]?.[0]).toEqual({
      library: { name: "Demo Library", profile: "college", address: null, contact: null, currency: "INR", locale: "en" },
      admin: { username: "admin", displayName: "Head Librarian", password: "a long passphrase" },
      applyProfileDefaults: true,
      finePerDayMinor: 250,
    });
  });

  it("asks for the daily backup folder during setup and sends the chosen folder", async () => {
    api.completeSetup.mockResolvedValue({});
    backup.pickFolder.mockResolvedValueOnce(null).mockResolvedValueOnce("E:\\Library Backups");
    renderWizard();
    await userEvent.type(screen.getByLabelText("Library name"), "Demo");
    await userEvent.click(screen.getByRole("button", { name: "Continue" }));
    await userEvent.click(screen.getByRole("button", { name: "Continue" }));
    expect(screen.getByRole("heading", { name: "Backups" })).toBeInTheDocument();
    const folder = screen.getByLabelText("Backup folder (optional)");
    // Cancelling the dialog leaves the field as it was; choosing a folder fills it in.
    await userEvent.click(screen.getByRole("button", { name: "Choose folder…" }));
    expect(folder).toHaveValue("");
    await userEvent.click(screen.getByRole("button", { name: "Choose folder…" }));
    expect(folder).toHaveValue("E:\\Library Backups");
    await userEvent.click(screen.getByRole("button", { name: "Continue" }));
    await userEvent.type(screen.getByLabelText("Your name"), "A B");
    await userEvent.type(screen.getByLabelText("Username"), "admin");
    await userEvent.type(screen.getByLabelText("Password"), "a long passphrase");
    await userEvent.type(screen.getByLabelText("Repeat password"), "a long passphrase");
    await userEvent.click(screen.getByRole("button", { name: "Create library" }));
    expect(api.completeSetup.mock.calls[0]?.[0]).toMatchObject({ backupFolder: "E:\\Library Backups" });
  });

  it("does not send mismatched passwords", async () => {
    renderWizard();
    await userEvent.type(screen.getByLabelText("Library name"), "Demo");
    await userEvent.click(screen.getByRole("button", { name: "Continue" }));
    await userEvent.click(screen.getByRole("button", { name: "Continue" }));
    await userEvent.click(screen.getByRole("button", { name: "Continue" }));
    await userEvent.type(screen.getByLabelText("Your name"), "A B");
    await userEvent.type(screen.getByLabelText("Username"), "admin");
    await userEvent.type(screen.getByLabelText("Password"), "a long passphrase");
    await userEvent.type(screen.getByLabelText("Repeat password"), "different one!!");
    await userEvent.click(screen.getByRole("button", { name: "Create library" }));
    expect(screen.getByText("The passwords do not match.")).toBeInTheDocument();
    expect(api.completeSetup).not.toHaveBeenCalled();
  });

  it("shows the backend's message if setup fails", async () => {
    api.completeSetup.mockImplementation(async () => {
      throw new ApiError({ code: "CONFLICT", message: "This library has already been set up." });
    });
    renderWizard();
    await userEvent.type(screen.getByLabelText("Library name"), "Demo");
    await userEvent.click(screen.getByRole("button", { name: "Continue" }));
    await userEvent.click(screen.getByRole("button", { name: "Continue" }));
    await userEvent.click(screen.getByRole("button", { name: "Continue" }));
    await userEvent.type(screen.getByLabelText("Your name"), "A B");
    await userEvent.type(screen.getByLabelText("Username"), "admin");
    await userEvent.type(screen.getByLabelText("Password"), "a long passphrase");
    await userEvent.type(screen.getByLabelText("Repeat password"), "a long passphrase");
    await userEvent.click(screen.getByRole("button", { name: "Create library" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("already been set up");
  });

  it("can go back without losing what was typed", async () => {
    renderWizard();
    await userEvent.type(screen.getByLabelText("Library name"), "Demo");
    await userEvent.click(screen.getByRole("button", { name: "Continue" }));
    await userEvent.click(screen.getByRole("button", { name: "Back" }));
    expect(screen.getByLabelText("Library name")).toHaveValue("Demo");
  });
});
