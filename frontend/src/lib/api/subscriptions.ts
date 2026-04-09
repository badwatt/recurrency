import { apiClient } from "./client";
import { auth } from "@/lib/auth";

export interface Subscription {
  id: string;
  user_id: string;
  category_id: string | null;
  name: string;
  amount: number;
  currency: string;
  frequency: "weekly" | "monthly" | "quarterly" | "yearly";
  start_date: string;
  next_billing_date: string | null;
  is_active: boolean;
  notes: string | null;
  logo_url: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateSubscriptionRequest {
  name: string;
  amount: number;
  currency: string;
  frequency: string;
  start_date: string;
  category_id?: string;
  notes?: string;
  logo_url?: string;
}

export interface UpdateSubscriptionRequest {
  name?: string;
  amount?: number;
  currency?: string;
  frequency?: string;
  start_date?: string;
  next_billing_date?: string;
  category_id?: string;
  is_active?: boolean;
  notes?: string;
  logo_url?: string;
}

async function getAuthHeaders() {
  const token = auth.getToken();
  if (!token) throw new Error("Not authenticated");
  return { token };
}

export const subscriptionsApi = {
  async list(): Promise<Subscription[]> {
    const { token } = await getAuthHeaders();
    return apiClient.get<Subscription[]>(`/api/subscriptions?token=${encodeURIComponent(token)}`);
  },

  async get(id: string): Promise<Subscription> {
    const { token } = await getAuthHeaders();
    return apiClient.get<Subscription>(`/api/subscriptions/${id}?token=${encodeURIComponent(token)}`);
  },

  async create(data: CreateSubscriptionRequest): Promise<Subscription> {
    const { token } = await getAuthHeaders();
    return apiClient.post<Subscription>(`/api/subscriptions?token=${encodeURIComponent(token)}`, data);
  },

  async update(id: string, data: UpdateSubscriptionRequest): Promise<Subscription> {
    const { token } = await getAuthHeaders();
    return apiClient.put<Subscription>(`/api/subscriptions/${id}?token=${encodeURIComponent(token)}`, data);
  },

  async delete(id: string): Promise<void> {
    const { token } = await getAuthHeaders();
    return apiClient.delete(`/api/subscriptions/${id}?token=${encodeURIComponent(token)}`);
  },

  async upcoming(): Promise<Subscription[]> {
    const { token } = await getAuthHeaders();
    return apiClient.get<Subscription[]>(`/api/subscriptions/upcoming?token=${encodeURIComponent(token)}`);
  },
};
