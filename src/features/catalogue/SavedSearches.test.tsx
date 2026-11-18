import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";
import { SavedSearches } from "./SavedSearches";

it("rejects saving a name without search words or filters", async () => {
  const user = userEvent.setup();
  const filter = { text: "", collectionId: null, onlyAvailable: false, includeArchived: false, sort: "relevance" as const };
  const view = render(<SavedSearches userId={9001} filter={filter} onApply={vi.fn()} />);
  await user.type(screen.getByLabelText("Search name"), "Animal Farm");
  await user.click(screen.getByRole("button", { name: "Save search" }));
  expect(screen.getByRole("alert")).toHaveTextContent("Nothing to save yet");
  expect(localStorage.getItem("pusthakam.saved-searches.9001")).toBeNull();
  view.rerender(<SavedSearches userId={9001} filter={{ ...filter, onlyAvailable: true }} onApply={vi.fn()} />);
  await user.click(screen.getByRole("button", { name: "Save search" }));
  expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  expect(screen.getByRole("option", { name: "Animal Farm" })).toBeInTheDocument();
});

it("repairs a legacy empty search only when explicitly requested", async () => {
  const user = userEvent.setup();
  const filter = { text: "", collectionId: null, onlyAvailable: false, includeArchived: false, sort: "relevance" as const };
  localStorage.setItem("pusthakam.saved-searches.9001", JSON.stringify([{ name: "Animal Farm", filter }]));
  const apply = vi.fn();
  render(<SavedSearches userId={9001} filter={filter} onApply={apply} />);
  await user.selectOptions(screen.getByLabelText("Saved searches"), "Animal Farm");
  expect(apply).toHaveBeenLastCalledWith(filter);
  await user.click(screen.getByRole("button", { name: "Use saved name as search words" }));
  expect(apply).toHaveBeenLastCalledWith({ ...filter, text: "Animal Farm" });
  expect(JSON.parse(localStorage.getItem("pusthakam.saved-searches.9001")!)[0].filter.text).toBe("Animal Farm");
});

it("restores filters after remount and isolates staff accounts", async () => {
  const user = userEvent.setup();
  const filter = { text: "Basheer", collectionId: null, onlyAvailable: true, includeArchived: false, sort: "title" as const };
  const apply = vi.fn();
  const first = render(<SavedSearches userId={9001} filter={filter} onApply={apply} />);
  await user.type(screen.getByLabelText("Search name"), "Available fiction");
  await user.click(screen.getByRole("button", { name: "Save search" }));
  await user.click(screen.getByRole("button", { name: "Run saved search" }));
  expect(apply).toHaveBeenLastCalledWith(filter);
  await user.click(screen.getByRole("button", { name: "Save search" }));
  expect(screen.getByRole("alert")).toHaveTextContent("already exists");
  first.unmount();
  const second = render(<SavedSearches userId={9001} filter={filter} onApply={apply} />);
  await user.selectOptions(screen.getByLabelText("Saved searches"), "Available fiction");
  expect(apply).toHaveBeenCalledWith(filter);
  second.unmount();
  render(<SavedSearches userId={9002} filter={filter} onApply={apply} />);
  expect(screen.queryByRole("option", { name: "Available fiction" })).not.toBeInTheDocument();
});

it("persists updated filters and removal", async () => {
  const user = userEvent.setup();
  const filter = { text: "fiction", collectionId: null, onlyAvailable: false, includeArchived: false, sort: "title" as const };
  const apply = vi.fn();
  const first = render(<SavedSearches userId={9001} filter={filter} onApply={apply} />);
  await user.type(screen.getByLabelText("Search name"), "Fiction");
  await user.click(screen.getByRole("button", { name: "Save search" }));
  const updated = { ...filter, onlyAvailable: true, collectionId: 2, sort: "recent" as const };
  first.rerender(<SavedSearches userId={9001} filter={updated} onApply={apply} />);
  await user.click(screen.getByRole("button", { name: "Run saved search" }));
  expect(apply).toHaveBeenLastCalledWith(filter);
  await user.click(screen.getByRole("button", { name: "Update saved search" }));
  await user.click(screen.getByRole("button", { name: "Run saved search" }));
  expect(apply).toHaveBeenLastCalledWith(updated);
  first.unmount();
  const second = render(<SavedSearches userId={9001} filter={filter} onApply={apply} />);
  await user.selectOptions(screen.getByLabelText("Saved searches"), "Fiction");
  expect(apply).toHaveBeenLastCalledWith(updated);
  await user.click(screen.getByRole("button", { name: "Remove saved search" }));
  second.unmount();
  render(<SavedSearches userId={9001} filter={filter} onApply={apply} />);
  expect(screen.queryByRole("option", { name: "Fiction" })).not.toBeInTheDocument();
});
