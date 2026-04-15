import { describe, it, expect } from "vitest";
import { CURRENCIES } from "@/lib/constants";

describe("CURRENCIES", () => {
  it("contains EUR and USD", () => {
    const codes = CURRENCIES.map(c => c.value);
    expect(codes).toContain("EUR");
    expect(codes).toContain("USD");
  });

  it("has value and label for each entry", () => {
    for (const c of CURRENCIES) {
      expect(c.value).toBeTruthy();
      expect(c.label).toBeTruthy();
    }
  });

  it("has no duplicate values", () => {
    const values = CURRENCIES.map(c => c.value);
    expect(new Set(values).size).toBe(values.length);
  });

  it("has 6 currencies", () => {
    expect(CURRENCIES.length).toBe(6);
  });
});