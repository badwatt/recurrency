import { describe, it, expect } from "vitest";
import { getIcon } from "@/lib/icons";
import { Tag, Home, ShoppingBag, Gamepad2 } from "lucide-react";

describe("getIcon", () => {
  it("returns Tag as default for null", () => {
    expect(getIcon(null)).toBe(Tag);
  });

  it("returns Tag as default for undefined", () => {
    expect(getIcon(undefined)).toBe(Tag);
  });

  it("returns Tag as default for empty string", () => {
    expect(getIcon("")).toBe(Tag);
  });

  it("returns Tag as default for unknown icon name", () => {
    expect(getIcon("nonexistent-icon")).toBe(Tag);
  });

  it("returns correct icon for known kebab-case names", () => {
    expect(getIcon("home")).toBe(Home);
    expect(getIcon("shopping-bag")).toBe(ShoppingBag);
  });

  it("handles multi-segment kebab-case names", () => {
    expect(getIcon("gamepad-2")).toBe(Gamepad2);
  });
});