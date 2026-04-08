import { apiClient } from "./client";
import { auth } from "@/lib/auth";

export interface DemoStatus {
  enabled: boolean;
  email: string;
}

export interface DemoInitResponse {
  email: string;
  password: string;
}

export interface DemoResetResponse {
  success: boolean;
  message: string;
}

export const demoApi = {
  async getStatus(): Promise<DemoStatus> {
    return apiClient.get<DemoStatus>("/api/demo/status");
  },

  async init(): Promise<DemoInitResponse> {
    return apiClient.get<DemoInitResponse>("/api/demo/init");
  },

  async reset(): Promise<DemoResetResponse> {
    const token = auth.getToken();
    if (!token) throw new Error("Not authenticated");
    return apiClient.post<DemoResetResponse>(
      `/api/demo/reset?token=${encodeURIComponent(token)}`
    );
  },
};
