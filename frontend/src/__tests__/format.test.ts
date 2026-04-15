import { describe, it, expect } from "vitest";
import { formatCurrency } from "@/lib/format";

describe("formatCurrency", () => {
  it("formats a number with EUR", () => {
    expect(formatCurrency(9.99, "EUR")).toBe("€9.99");
  });

  it("formats a number with USD", () => {
    expect(formatCurrency(100, "USD")).toBe("$100.00");
  });

  it("formats a string amount", () => {
    expect(formatCurrency("1234.56", "EUR")).toContain("1,234.56");
  });

  it("formats zero", () => {
    expect(formatCurrency(0, "EUR")).toBe("€0.00");
  });

  it("formats a large number with grouping", () => {
    const result = formatCurrency(1000000, "USD");
    expect(result).toContain("1,000,000.00");
  });
});