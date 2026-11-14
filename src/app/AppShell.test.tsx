import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import { AppRoutes } from "@/App";
import { ThemeProvider } from "@/lib/theme";

function renderApp(path = "/") {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <ThemeProvider>
      <QueryClientProvider client={qc}>
        <MemoryRouter initialEntries={[path]}>
          <AppRoutes />
        </MemoryRouter>
      </QueryClientProvider>
    </ThemeProvider>,
  );
}

describe("AppShell", () => {
  it("renders brand, tagline and all navigation groups", () => {
    renderApp();
    expect(screen.getByText("PUSTHAKAM")).toBeInTheDocument();
    expect(screen.getByText("A Better Way to Manage Libraries.")).toBeInTheDocument();
    for (const g of ["Main", "Collections", "Operations", "Administration"]) {
      expect(screen.getByText(g)).toBeInTheDocument();
    }
    expect(screen.getByRole("link", { name: "Catalogue" })).toBeInTheDocument();
  });

  it("shows an honest not-available page for unbuilt modules", async () => {
    renderApp();
    await userEvent.click(screen.getByRole("link", { name: "Catalogue" }));
    expect(screen.getByRole("heading", { name: /Catalogue is not available yet/ })).toBeInTheDocument();
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
