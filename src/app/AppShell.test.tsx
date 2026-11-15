import { describe, expect, it } from "vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import { AppRoutes } from "@/App";
import { ThemeProvider } from "@/lib/theme";
import { SessionContext } from "@/lib/session";

const SESSION = { userId: 1, username: "rekha", displayName: "Rekha Nair", role: "librarian", permissions: [] };

function renderApp(path = "/") {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <ThemeProvider>
      <QueryClientProvider client={qc}>
        <SessionContext.Provider value={{ session: SESSION, signOut: async () => {} }}>
          <MemoryRouter initialEntries={[path]}>
            <AppRoutes />
          </MemoryRouter>
        </SessionContext.Provider>
      </QueryClientProvider>
    </ThemeProvider>,
  );
}

describe("AppShell", () => {
  it("renders brand, tagline and all navigation groups", () => {
    renderApp();
    expect(screen.getByText("Pusthakam")).toBeInTheDocument();
    expect(screen.getByText("A Better Way to Manage Libraries.")).toBeInTheDocument();
    expect(screen.queryByText(/Offline/)).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Online Tools" })).toBeInTheDocument();
    const rail = within(screen.getByRole("navigation", { name: "Primary" }));
    for (const g of ["Main", "Collections", "Operations", "Administration"]) {
      expect(rail.getByText(g)).toBeInTheDocument();
    }
    expect(within(screen.getByRole("navigation", { name: "Primary" })).getByRole("link", { name: "Circulation" })).toBeInTheDocument();
  });

  it("shows an honest not-available page for unbuilt modules", async () => {
    renderApp();
    await userEvent.click(within(screen.getByRole("navigation", { name: "Primary" })).getByRole("link", { name: "Circulation" }));
    expect(screen.getByRole("heading", { name: /Circulation is not available yet/ })).toBeInTheDocument();
  });

  it("shows who is signed in and lets them sign out", () => {
    renderApp();
    expect(screen.getByText("Rekha Nair")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Sign out" })).toBeInTheDocument();
  });

  it("explains outside the desktop app that status needs the desktop app", async () => {
    renderApp();
    expect(await screen.findByRole("alert")).toHaveTextContent(/desktop app/i);
  });

  it("opens the command palette with Ctrl+K and navigates with the keyboard", async () => {
    renderApp();
    await userEvent.keyboard("{Control>}k{/Control}");
    const input = screen.getByRole("combobox", { name: "Go to section" });
    expect(input).toHaveFocus();
    await userEvent.type(input, "memb");
    await userEvent.keyboard("{Enter}");
    expect(screen.getByRole("heading", { name: /Members is not available yet/ })).toBeInTheDocument();
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("closes the palette with Escape", async () => {
    renderApp();
    await userEvent.keyboard("{Control>}k{/Control}");
    expect(screen.getByRole("dialog")).toBeInTheDocument();
    await userEvent.keyboard("{Escape}");
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });
});

describe("theme", () => {
  it("switches theme, applies it to the document and persists the choice", async () => {
    renderApp();
    await userEvent.click(screen.getByRole("radio", { name: "Dark theme" }));
    expect(document.documentElement.dataset.theme).toBe("dark");
    expect(localStorage.getItem("pusthakam.theme")).toBe("dark");
    await userEvent.click(screen.getByRole("radio", { name: "Light theme" }));
    expect(document.documentElement.dataset.theme).toBe("light");
  });
});
