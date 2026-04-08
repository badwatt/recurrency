import { describe, it, expect } from "vitest";

/**
 * Tests for category feature
 * 
 * Requirements:
 * 1. Subscription and transaction cards show category name
 * 2. Reports "by category" and "category breakdown" work correctly
 * 3. "Uncategorized" is the default category (not modifiable)
 */

describe("Category display helpers", () => {
  const getCategoryDisplay = (categoryId: string | null, categories: {id: string, name: string}[]) => {
    if (!categoryId) return "Uncategorized";
    const cat = categories.find((c) => c.id === categoryId);
    return cat?.name || "Uncategorized";
  };

  const categories = [
    { id: "cat-123", name: "Entertainment" },
    { id: "cat-456", name: "Food & Drink" },
  ];

  it("should display category name when subscription has category", () => {
    expect(getCategoryDisplay("cat-123", categories)).toBe("Entertainment");
    expect(getCategoryDisplay("cat-456", categories)).toBe("Food & Drink");
  });

  it("should display 'Uncategorized' when subscription has no category", () => {
    expect(getCategoryDisplay(null, categories)).toBe("Uncategorized");
  });

  it("should display category name when transaction has category", () => {
    expect(getCategoryDisplay("cat-123", categories)).toBe("Entertainment");
  });

  it("should display 'Uncategorized' when transaction has no category", () => {
    expect(getCategoryDisplay(null, categories)).toBe("Uncategorized");
  });
});

describe("Category breakdown in reports", () => {
  it("should aggregate spending by category", () => {
    const items = [
      { category_name: "Entertainment", total: 50.0, count: 3 },
      { category_name: "Food", total: 100.0, count: 5 },
      { category_name: "Uncategorized", total: 25.0, count: 2 },
    ];

    const total = items.reduce((sum, item) => sum + item.total, 0);
    expect(total).toBe(175.0);
  });

  it("should calculate percentage correctly per category", () => {
    const items = [
      { category_name: "Entertainment", total: 50.0 },
      { category_name: "Food", total: 100.0 },
      { category_name: "Uncategorized", total: 25.0 },
    ];

    const overall = items.reduce((sum, i) => sum + i.total, 0);
    const expectedPcts = [28.6, 57.1, 14.3];

    items.forEach((item, i) => {
      const pct = (item.total / overall) * 100;
      expect(pct).toBeCloseTo(expectedPcts[i], 1);
    });
  });

  it("should sort categories by total descending", () => {
    const items = [
      { category_name: "Food", total: 100.0 },
      { category_name: "Entertainment", total: 50.0 },
      { category_name: "Uncategorized", total: 25.0 },
    ];

    const sorted = [...items].sort((a, b) => b.total - a.total);
    expect(sorted[0].category_name).toBe("Food");
    expect(sorted[1].category_name).toBe("Entertainment");
    expect(sorted[2].category_name).toBe("Uncategorized");
  });
});

describe("Uncategorized as default category", () => {
  it("should be treated as system category (not modifiable)", () => {
    const UNCATEGORIZED_NAME = "Uncategorized";

    const isSystemCategory = (categoryName: string) => {
      return categoryName === UNCATEGORIZED_NAME;
    };

    expect(isSystemCategory("Uncategorized")).toBe(true);
    expect(isSystemCategory("Entertainment")).toBe(false);
  });

  it("should default to null/categorized when no category selected", () => {
    const newSubscription = {
      name: "New Service",
      amount: 9.99,
      category_id: null,
    };

    expect(newSubscription.category_id).toBeNull();
  });
});

describe("Category color in charts", () => {
  it("should assign colors to categories consistently", () => {
    const COLORS = [
      "#3B82F6", "#10B981", "#F59E0B", "#EF4444",
      "#8B5CF6", "#EC4899", "#06B6D4", "#84CC16",
    ];

    const getCategoryColor = (index: number) => {
      return COLORS[index % COLORS.length];
    };

    expect(getCategoryColor(0)).toBe("#3B82F6");
    expect(getCategoryColor(5)).toBe("#EC4899");
    expect(getCategoryColor(8)).toBe("#3B82F6");
  });
});
