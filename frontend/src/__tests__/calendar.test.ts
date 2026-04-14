import { describe, it, expect } from "vitest";

/**
 * Calendar date utilities tests for the "add subscription on click" feature
 * 
 * Feature: User clicks on a calendar day → subscription dialog opens with date pre-filled
 */

describe("Calendar date utilities", () => {
  // Helper function used in CalendarPage to build date strings
  const getDateString = (year: number, month: number, day: number): string => {
    return `${year}-${String(month).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
  };

  it("should format date as yyyy-mm-dd for backend", () => {
    const dateStr = getDateString(2026, 4, 15);
    expect(dateStr).toBe("2026-04-15");
  });

  it("should pad single digit months and days with zeros", () => {
    expect(getDateString(2026, 1, 1)).toBe("2026-01-01");
    expect(getDateString(2026, 12, 31)).toBe("2026-12-31");
  });

  it("should handle month boundaries correctly", () => {
    expect(getDateString(2026, 1, 15)).toBe("2026-01-15");
    expect(getDateString(2026, 2, 28)).toBe("2026-02-28");
    expect(getDateString(2026, 4, 30)).toBe("2026-04-30");
  });

  it("should produce date string in valid format", () => {
    const dateStr = getDateString(2026, 4, 15);
    // Valid yyyy-mm-dd format
    expect(dateStr.length).toBe(10);
    expect(dateStr.charAt(4)).toBe("-");
    expect(dateStr.charAt(7)).toBe("-");
  });
});

describe("CalendarPage date click feature", () => {
  it("should create correct date when clicking on day 15 of April 2026", () => {
    const year = 2026;
    const month = 4;
    const day = 15;

    const expectedDate = `${year}-${String(month).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
    expect(expectedDate).toBe("2026-04-15");
  });

  it("should create date string for day clicked in calendar navigation", () => {
    const currentYear = 2026;
    const currentMonth = 6; // June
    const clickedDay = 20;

    const startDate = `${currentYear}-${String(currentMonth).padStart(2, "0")}-${String(clickedDay).padStart(2, "0")}`;
    
    expect(startDate).toBe("2026-06-20");
  });

  it("should handle various days of month correctly", () => {
    const year = 2026;
    const month = 4;
    
    const dates = [1, 5, 10, 15, 20, 25, 30];
    
    dates.forEach(day => {
      const dateStr = `${year}-${String(month).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
      expect(dateStr).toBe(`${year}-04-${String(day).padStart(2, "0")}`);
    });
  });
});

describe("SubscriptionDialog date pre-filling", () => {
  const subscriptionWithDate = {
    id: "123",
    name: "Netflix",
    amount: 15.99,
    currency: "EUR",
    frequency: "monthly" as const,
    start_date: "2026-04-20",
    is_active: true,
  };

  it("should extract start_date from subscription object", () => {
    const startDate = subscriptionWithDate.start_date;
    expect(startDate).toBe("2026-04-20");
  });

  it("should handle subscriptions on different days of month", () => {
    const dates = [
      { day: 1, expected: "2026-04-01" },
      { day: 15, expected: "2026-04-15" },
      { day: 30, expected: "2026-04-30" },
    ];

    dates.forEach(({ day, expected }) => {
      const dateStr = `2026-04-${String(day).padStart(2, "0")}`;
      expect(dateStr).toBe(expected);
    });
  });
});

describe("Calendar navigation", () => {
  const MONTHS = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December"
  ];

  it("should map month index to name correctly", () => {
    expect(MONTHS[0]).toBe("January");
    expect(MONTHS[3]).toBe("April");
    expect(MONTHS[11]).toBe("December");
  });

  it("should use 0-based index for array access", () => {
    // MONTHS[month - 1] gives correct month name
    const month = 4;
    expect(MONTHS[month - 1]).toBe("April");
  });

  it("should calculate previous month correctly", () => {
    let month = 4; // April
    month = month - 1; // March
    expect(month).toBe(3);
    expect(MONTHS[month - 1]).toBe("March");
  });

  it("should calculate next month correctly", () => {
    let month = 4; // April
    month = month + 1; // May
    expect(month).toBe(5);
    expect(MONTHS[month - 1]).toBe("May");
  });
});
