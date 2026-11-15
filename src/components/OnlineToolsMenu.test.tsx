import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { OnlineToolsMenu } from "./OnlineToolsMenu";
import { ONLINE_TOOLS, toolUrl } from "@/features/online-tools/tools";

describe("OnlineToolsMenu", () => {
  it("is closed until asked, then lists every tool with its address", async () => {
    render(<OnlineToolsMenu />);
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Online Tools" }));
    const dialog = screen.getByRole("dialog", { name: "Online Tools" });
    expect(dialog).toBeInTheDocument();
    for (const tool of ONLINE_TOOLS) {
      expect(screen.getByRole("link", { name: new RegExp(tool.name) })).toHaveAttribute("href", toolUrl(tool));
    }
    expect(screen.getByText(/need an internet connection/i)).toBeInTheDocument();
  });

  it("lists the DDC list and the Library Toolkit, each tagged utm_source=pusthakam", () => {
    expect(ONLINE_TOOLS.map(toolUrl)).toEqual([
      "https://melvinvarghese.in/dewey-decimal-classification-list-ddc-codes/?utm_source=pusthakam",
      "https://melvinvarghese.github.io/Library-Toolkit/?utm_source=pusthakam",
    ]);
  });

  it("adds the tag without disturbing an existing query string", () => {
    expect(toolUrl({ id: "x", name: "X", description: "", url: "https://example.org/a?q=1" })).toBe(
      "https://example.org/a?q=1&utm_source=pusthakam",
    );
  });

  it("opens the tool in a new tab outside the desktop app and closes the menu", async () => {
    const open = vi.spyOn(window, "open").mockImplementation(() => null);
    render(<OnlineToolsMenu />);
    await userEvent.click(screen.getByRole("button", { name: "Online Tools" }));
    await userEvent.click(screen.getByRole("link", { name: /Library Toolkit/ }));
    expect(open).toHaveBeenCalledWith(
      "https://melvinvarghese.github.io/Library-Toolkit/?utm_source=pusthakam",
      "_blank",
      "noopener,noreferrer",
    );
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    open.mockRestore();
  });

  it("closes with Escape and returns focus to the button", async () => {
    render(<OnlineToolsMenu />);
    const button = screen.getByRole("button", { name: "Online Tools" });
    await userEvent.click(button);
    expect(screen.getByRole("link", { name: /DDC Number List/ })).toHaveFocus();
    await userEvent.keyboard("{Escape}");
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(button).toHaveFocus();
  });

  it("closes when clicking elsewhere", async () => {
    render(<><OnlineToolsMenu /><p>elsewhere</p></>);
    await userEvent.click(screen.getByRole("button", { name: "Online Tools" }));
    await userEvent.click(screen.getByText("elsewhere"));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });
});
