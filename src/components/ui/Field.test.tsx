import { expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { TextField } from "./Field";

it("keeps file selection labeled, keyboard focusable and restricted to accepted files", async () => {
  const change=vi.fn();
  render(<TextField label="Import file" type="file" accept=".csv" hint="Choose a CSV." onChange={change} />);
  const user=userEvent.setup();
  const input=screen.getByLabelText("Import file") as HTMLInputElement;
  expect(input).toHaveAccessibleDescription("Choose a CSV.");
  await user.tab();
  expect(input).toHaveFocus();
  await user.upload(input,new File(["x"],"notes.txt",{type:"text/plain"}));
  expect(change).not.toHaveBeenCalled();
  const file=new File(["title"],"books.csv",{type:"text/csv"});
  await user.upload(input,file);
  expect(input.files?.[0]).toBe(file);
  expect(change).toHaveBeenCalledTimes(1);
});

it("keeps disabled file inputs inert and exposes errors", async () => {
  const change=vi.fn();
  render(<TextField label="Import file" type="file" disabled error="Import in progress." onChange={change} />);
  const input=screen.getByLabelText("Import file") as HTMLInputElement;
  expect(input).toBeDisabled();
  expect(input).toHaveAttribute("aria-invalid","true");
  expect(input).toHaveAccessibleDescription("Import in progress.");
  await userEvent.setup().upload(input,new File(["x"],"books.csv",{type:"text/csv"}));
  expect(change).not.toHaveBeenCalled();
});
