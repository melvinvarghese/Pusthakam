import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { CallNumberLabel } from "./CallNumberLabel";

describe("CallNumberLabel", () => {
  it("stacks the call number and exposes it as one accessible name", () => {
    render(<CallNumberLabel callNumber="894.8123 BAS" />);
    const label = screen.getByRole("img", { name: "Call number 894.8123 BAS" });
    expect(label.textContent).toBe("894.8123BAS");
    expect(label.children).toHaveLength(3);
  });

  it("colours the band by Dewey main class", () => {
    render(<CallNumberLabel callNumber="629.1092 KAL" />);
    expect(screen.getByRole("img").style.getPropertyValue("--cls")).toBe("var(--cls-6)");
  });

  it("leaves non-Dewey numbers on the neutral band", () => {
    render(<CallNumberLabel callNumber="PR9199.3 .R69" />);
    expect(screen.getByRole("img").style.getPropertyValue("--cls")).toBe("");
  });

  it("keeps numeric UDC labels neutral", () => {
    render(<CallNumberLabel callNumber="004.42 LOV" schemeCode="udc" />);
    const label = screen.getByRole("img", { name: "Call number 004.42 LOV" });
    expect(label.style.getPropertyValue("--cls")).toBe("");
    expect(Array.from(label.children, (line) => line.textContent)).toEqual(["004.42", "LOV"]);
  });
});
