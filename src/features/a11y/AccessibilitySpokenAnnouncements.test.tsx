import { describe, expect, it, vi } from "vitest";
import { render, screen, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { LiveAnnouncer } from "@/components/LiveAnnouncer";
import { announce } from "@/lib/announcer";
import { Modal } from "@/components/ui/Modal";
import { Pager } from "@/components/Pager";

describe("Accessibility and Screen Reader Spoken Announcements (CORE-A11Y)", () => {
  it("announces polite status updates to screen readers", async () => {
    render(<LiveAnnouncer />);

    const politeRegion = screen.getByTestId("live-region-polite");
    expect(politeRegion).toHaveAttribute("role", "status");
    expect(politeRegion).toHaveAttribute("aria-live", "polite");
    expect(politeRegion).toHaveAttribute("aria-atomic", "true");
    expect(politeRegion).toHaveTextContent("");

    act(() => {
      announce("Issued Dune. Due 2026-12-04.", "polite");
    });

    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 100));
    });

    expect(politeRegion).toHaveTextContent("Issued Dune. Due 2026-12-04.");
  });

  it("announces assertive alerts that interrupt speech immediately", async () => {
    render(<LiveAnnouncer />);

    act(() => {
      announce("Patron account is blocked due to unpaid fines.", "assertive");
    });

    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 100));
    });

    const assertiveRegion = screen.getByTestId("live-region-assertive");
    expect(assertiveRegion).toHaveAttribute("role", "alert");
    expect(assertiveRegion).toHaveAttribute("aria-live", "assertive");
    expect(assertiveRegion).toHaveAttribute("aria-atomic", "true");
    expect(assertiveRegion).toHaveTextContent("Patron account is blocked due to unpaid fines.");
  });

  it("handles sequential announcements without losing updates", async () => {
    render(<LiveAnnouncer />);

    const politeRegion = screen.getByTestId("live-region-polite");

    act(() => {
      announce("Searching catalogue...", "polite");
    });

    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 80));
    });
    expect(politeRegion).toHaveTextContent("Searching catalogue...");

    act(() => {
      announce("Found 42 catalogue records.", "polite");
    });

    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 80));
    });
    expect(politeRegion).toHaveTextContent("Found 42 catalogue records.");
  });

  it("pager provides polite live announcement of current range and page total", () => {
    const onPage = vi.fn();
    render(<Pager page={2} pageSize={25} total={120} onPage={onPage} />);

    const range = screen.getByText("26-50 of 120");
    expect(range).toHaveAttribute("aria-live", "polite");
  });

  it("modal traps focus and restores focus to triggering element upon dismissal", async () => {
    function TestHost() {
      const [open, setOpen] = useState(false);
      return (
        <div>
          <button type="button" onClick={() => setOpen(true)}>
            Open patron lookup
          </button>
          {open && (
            <Modal title="Find patron" onClose={() => setOpen(false)}>
              <input type="text" placeholder="Member number" aria-label="Member number" />
              <button type="button" onClick={() => setOpen(false)}>
                Dismiss
              </button>
            </Modal>
          )}
        </div>
      );
    }

    render(<TestHost />);
    const openButton = screen.getByRole("button", { name: "Open patron lookup" });

    // Focus and click open
    openButton.focus();
    expect(document.activeElement).toBe(openButton);

    await userEvent.click(openButton);

    // Modal is open, focus should move inside modal to first focusable element
    const input = screen.getByRole("textbox", { name: "Member number" });
    expect(input).toBeInTheDocument();
    expect(document.activeElement).toBe(input);

    // Press Escape to dismiss modal
    await userEvent.keyboard("{Escape}");

    // Modal is closed, focus must be restored to the open button
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(document.activeElement).toBe(openButton);
  });

  it("modal wraps Tab navigation within the dialog", async () => {
    function DialogHost() {
      return (
        <Modal title="Accessibility Dialog" onClose={() => {}}>
          <input aria-label="First input" type="text" />
          <button type="button">Secondary action</button>
          <button type="button">Close action</button>
        </Modal>
      );
    }

    render(<DialogHost />);
    const firstInput = screen.getByRole("textbox", { name: "First input" });
    const closeBtn = screen.getByRole("button", { name: "Close action" });

    // Focus starts on first element
    expect(document.activeElement).toBe(firstInput);

    // Tab to last element
    closeBtn.focus();
    expect(document.activeElement).toBe(closeBtn);

    // Tab from last element wraps to first
    await userEvent.tab();
    expect(document.activeElement).toBe(firstInput);

    // Shift+Tab from first element wraps to last
    await userEvent.tab({ shift: true });
    expect(document.activeElement).toBe(closeBtn);
  });
});
