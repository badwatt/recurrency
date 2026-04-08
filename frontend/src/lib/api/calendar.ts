import { apiClient } from "./client";
import { auth } from "@/lib/auth";

export interface CalendarItem {
  date: string;
  subscription_id: string;
  name: string;
  amount: string;
  currency: string;
  frequency: string;
}

async function getAuthHeaders() {
  const token = auth.getToken();
  if (!token) throw new Error("Not authenticated");
  return { token };
}

export const calendarApi = {
  async getMonth(year: number, month: number): Promise<CalendarItem[]> {
    const { token } = await getAuthHeaders();
    return apiClient.get<CalendarItem[]>(
      `/api/subscriptions/calendar?token=${encodeURIComponent(token)}&year=${year}&month=${month}`
    );
  },
};
