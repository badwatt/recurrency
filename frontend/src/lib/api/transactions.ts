import { apiClient } from "./client";
import { getAuthParams } from "./auth-helpers";

export interface Transaction {
  id: string;
  user_id: string;
  category_id: string;
  name: string;
  amount: number;
  currency: string;
  date: string;
  notes: string | null;
  logo_url: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateTransactionRequest {
  name: string;
  amount: number;
  currency: string;
  date: string;
  category_id?: string;
  notes?: string;
  logo_url?: string;
}

export interface UpdateTransactionRequest {
  name?: string;
  amount?: number;
  currency?: string;
  date?: string;
  category_id?: string;
  notes?: string;
  logo_url?: string;
}

export const transactionsApi = {
  async list(): Promise<Transaction[]> {
    const { token } = await getAuthParams();
    return apiClient.get<Transaction[]>(`/api/transactions?token=${encodeURIComponent(token)}`);
  },

  async get(id: string): Promise<Transaction> {
    const { token } = await getAuthParams();
    return apiClient.get<Transaction>(`/api/transactions/${id}?token=${encodeURIComponent(token)}`);
  },

  async create(data: CreateTransactionRequest): Promise<Transaction> {
    const { token } = await getAuthParams();
    return apiClient.post<Transaction>(`/api/transactions?token=${encodeURIComponent(token)}`, data);
  },

  async update(id: string, data: UpdateTransactionRequest): Promise<Transaction> {
    const { token } = await getAuthParams();
    return apiClient.put<Transaction>(`/api/transactions/${id}?token=${encodeURIComponent(token)}`, data);
  },

  async delete(id: string): Promise<void> {
    const { token } = await getAuthParams();
    return apiClient.delete(`/api/transactions/${id}?token=${encodeURIComponent(token)}`);
  },

  async getByDateRange(fromDate: string, toDate: string): Promise<Transaction[]> {
    const { token } = await getAuthParams();
    return apiClient.get<Transaction[]>(
      `/api/transactions/date-range?token=${encodeURIComponent(token)}&from_date=${fromDate}&to_date=${toDate}`
    );
  },
};
