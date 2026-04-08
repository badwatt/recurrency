import { describe, it, expect } from "vitest";
import { cn } from "@/lib/utils";

describe("utils", () => {
  describe("cn (classnames utility)", () => {
    it("merges class names", () => {
      const result = cn("foo", "bar");
      expect(result).toBe("foo bar");
    });

    it("handles conditional classes", () => {
      const isActive = true;
      const result = cn("base", isActive && "active");
      expect(result).toBe("base active");
    });

    it("skips falsy values", () => {
      const result = cn("base", false && "hidden", undefined && "nope", "visible");
      expect(result).toBe("base visible");
    });
  });
});

describe("date formatting", () => {
  it("formats currency correctly", () => {
    const formatCurrency = (amount: number, currency = "USD") => {
      return new Intl.NumberFormat("en-US", {
        style: "currency",
        currency,
      }).format(amount);
    };

    expect(formatCurrency(9.99)).toBe("$9.99");
    expect(formatCurrency(100)).toBe("$100.00");
    expect(formatCurrency(1234.56, "EUR")).toContain("1,234.56");
  });
});

describe("frequency calculations", () => {
  const normalizeToMonthly = (amount: number, frequency: string): number => {
    switch (frequency) {
      case "weekly":
        return amount * 4.33;
      case "monthly":
        return amount;
      case "quarterly":
        return amount / 3;
      case "yearly":
        return amount / 12;
      default:
        return amount;
    }
  };

  it("calculates weekly as monthly", () => {
    expect(normalizeToMonthly(10, "weekly")).toBeCloseTo(43.3, 1);
  });

  it("keeps monthly the same", () => {
    expect(normalizeToMonthly(100, "monthly")).toBe(100);
  });

  it("divides quarterly by 3", () => {
    expect(normalizeToMonthly(30, "quarterly")).toBe(10);
  });

  it("divides yearly by 12", () => {
    expect(normalizeToMonthly(120, "yearly")).toBe(10);
  });
});

describe("validation", () => {
  const isValidEmail = (email: string): boolean => {
    return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
  };

  const isValidAmount = (amount: string): boolean => {
    const num = parseFloat(amount);
    return !isNaN(num) && num > 0;
  };

  it("validates email format", () => {
    expect(isValidEmail("test@example.com")).toBe(true);
    expect(isValidEmail("invalid")).toBe(false);
    expect(isValidEmail("test@")).toBe(false);
  });

  it("validates positive amounts", () => {
    expect(isValidAmount("10.00")).toBe(true);
    expect(isValidAmount("0")).toBe(false);
    expect(isValidAmount("-5")).toBe(false);
    expect(isValidAmount("invalid")).toBe(false);
  });
});
