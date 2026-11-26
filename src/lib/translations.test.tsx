import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";
import glossary from "../../docs/translations/approved-glossary.json";
import { malayalam, translate } from "./translations";
import { setLanguage } from "./language";
import { LoginScreen } from "@/features/auth/LoginScreen";
import { CommandPalette } from "@/components/CommandPalette";

const api = vi.hoisted(() => ({ login: vi.fn() }));
vi.mock("@/lib/api", async original => ({ ...await original<typeof import("@/lib/api")>(), authApi: api }));
vi.mock("@/app/nav", () => ({ useVisibleNavGroups: () => [{ label: "Main", items: [
  { path: "/catalogue", label: "Catalogue", available: true },
  { path: "/members", label: "Members", available: true },
] }] }));

beforeEach(() => { setLanguage("en"); api.login.mockReset(); });

describe("Malayalam shared screens", () => {
  it("keeps every named placeholder in the Malayalam resources", () => {
    const placeholders = (text: string) => [...text.matchAll(/\{(\w+)\}/g)].map(match => match[1]).sort();
    for (const [english, translated] of Object.entries(malayalam)) {
      expect(placeholders(translated), english).toEqual(placeholders(english));
    }
  });

  it("inserts data literally without interpreting HTML, replacement tokens or nested placeholders", () => {
    const title = '<img src=x> $& {dueOn}';
    expect(translate("ml", "Issued {title}. Due {dueOn}.", { title, dueOn: "2026-12-01" }))
      .toBe(`${title} നൽകി. തിരികെ നൽകേണ്ട തീയതി: 2026-12-01.`);
    expect(translate("en", "Returned {title}.", { title })).toBe(`Returned ${title}.`);
  });

  it("honours every approved term, preserves English and falls back for unconverted text", () => {
    for (const [english, malayalam] of Object.entries(glossary.terms)) {
      expect(translate("ml", english)).toBe(malayalam);
      expect(translate("en", english)).toBe(english);
    }
    expect(translate("ml", "An unconverted message")).toBe("An unconverted message");
  });

  it("switches sign-in language without changing entered credentials or the API contract", async () => {
    api.login.mockResolvedValue({ userId: 1 });
    render(<QueryClientProvider client={new QueryClient()}><LoginScreen /></QueryClientProvider>);
    await userEvent.type(screen.getByLabelText("Username"), "melvin");
    await userEvent.type(screen.getByLabelText("Password"), "secret-password");
    await userEvent.selectOptions(screen.getByLabelText("Language / ഭാഷ"), "ml");
    expect(screen.getByRole("heading", { name: "ലോഗിൻ ചെയ്യുക" })).toBeInTheDocument();
    expect(screen.getByLabelText("ഉപയോക്തൃനാമം")).toHaveValue("melvin");
    expect(screen.getByLabelText("പാസ്‌വേഡ്")).toHaveValue("secret-password");
    await userEvent.click(screen.getByRole("button", { name: "ലോഗിൻ ചെയ്യുക" }));
    expect(api.login).toHaveBeenCalledWith("melvin", "secret-password");
  });

  it("finds translated navigation by Malayalam and English names", async () => {
    act(() => setLanguage("ml"));
    render(<MemoryRouter><CommandPalette open onClose={() => {}} /></MemoryRouter>);
    const input = screen.getByRole("combobox", { name: "വിഭാഗം തിരയുക" });
    await userEvent.type(input, "പുസ്തക");
    expect(screen.getByRole("option", { name: /പുസ്തകപ്പട്ടിക/ })).toBeInTheDocument();
    expect(screen.queryByRole("option", { name: /അംഗങ്ങൾ/ })).not.toBeInTheDocument();
    await userEvent.clear(input);
    await userEvent.type(input, "Catalogue");
    expect(screen.getByRole("option", { name: /പുസ്തകപ്പട്ടിക/ })).toBeInTheDocument();
  });
});
