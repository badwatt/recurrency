import { apiClient } from "./client";
import { getAuthParams } from "./auth-helpers";

export interface MonthlySummary {
  year: number;
  month: number;
  subscriptions_total: string;
  transactions_total: string;
  overall_total: string;
  subscriptions_count: number;
  transactions_count: number;
}

export interface CategoryBreakdown {
  category_id: string;
  category_name: string;
  total: string;
  count: number;
  percentage: number;
}

export interface SpendingTrend {
  period: string;
  total: string;
}

export interface ReportResponse {
  monthly_summary: MonthlySummary;
  category_breakdown: CategoryBreakdown[];
  recent_trends: SpendingTrend[];
}

export const reportsApi = {
  async getSummary(year?: number, month?: number): Promise<ReportResponse> {
    const { token } = await getAuthParams();
    let url = `/api/reports/summary?token=${encodeURIComponent(token)}`;
    if (year) url += `&year=${year}`;
    if (month) url += `&month=${month}`;
    return apiClient.get<ReportResponse>(url);
  },
};
