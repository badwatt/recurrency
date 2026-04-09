import { getApiBaseUrl } from "@/lib/config";

interface ApiError {
  error: string;
  details?: string;
}

class ApiClient {
  constructor() {}

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const baseUrl = getApiBaseUrl();
    const url = `${baseUrl}${endpoint}`;
    const token = localStorage.getItem("recur-token");

    const headers: HeadersInit = {
      "Content-Type": "application/json",
      ...(options.headers || {}),
    };

    if (token) {
      (headers as Record<string, string>)["Authorization"] = `Bearer ${token}`;
    }

    const response = await fetch(url, {
      ...options,
      headers,
    });

    if (!response.ok) {
      const text = await response.text().catch(() => "An error occurred");
      try {
        const error: ApiError = JSON.parse(text);
        throw new Error(error.error || "An error occurred");
      } catch {
        throw new Error(text || "An error occurred");
      }
    }

    const text = await response.text();
    if (!text) return {} as T;
    return JSON.parse(text);
  }

  async get<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: "GET" });
  }

  async post<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: "POST",
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async put<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: "PUT",
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async delete<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: "DELETE" });
  }
}

export const apiClient = new ApiClient();