import { apiClient } from "./client";
import { getAuthParams } from "./auth-helpers";

export interface CalendarItem {
  date: string;
  subscription_id: string;
  name: string;
  amount: string;
  currency: string;
  frequency: string;
  logo_url: string | null;
}

export const calendarApi = {
  async getMonth(year: number, month: number): Promise<CalendarItem[]> {
    const { token } = await getAuthParams();
    return apiClient.get<CalendarItem[]>(
      `/api/subscriptions/calendar?token=${encodeURIComponent(token)}&year=${year}&month=${month}`
    );
  },
};
