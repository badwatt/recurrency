import { apiClient } from "./client";

const API_BASE = import.meta.env.VITE_API_URL || "";

export interface User {
  id: string;
  email: string;
  name?: string;
  currency: string;
  verified: boolean;
}

export interface AuthResponse {
  token: string;
  user: User;
  requires_verification: boolean;
}

export interface RegisterRequest {
  email: string;
  password: string;
  name: string;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export const authApi = {
  async register(data: RegisterRequest): Promise<AuthResponse> {
    return apiClient.post<AuthResponse>("/api/auth/register", data);
  },

  async login(data: LoginRequest): Promise<AuthResponse> {
    return apiClient.post<AuthResponse>("/api/auth/login", data);
  },

  async verifyToken(token: string): Promise<{ valid: boolean; user_id: string; email: string }> {
    return apiClient.post("/api/auth/verify", token);
  },

  async verifyEmail(token: string, code: string): Promise<User> {
    const res = await fetch(`${API_BASE}/api/auth/verify-email?token=${encodeURIComponent(token)}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ code }),
    });
    if (!res.ok) {
      const err = await res.text().catch(() => "Verification failed");
      throw new Error(err);
    }
    return res.json();
  },

  async resendVerification(token: string): Promise<void> {
    const res = await fetch(`${API_BASE}/api/auth/resend-verification?token=${encodeURIComponent(token)}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
    });
    if (!res.ok) {
      const err = await res.text().catch(() => "Failed to resend");
      throw new Error(err);
    }
  },
};
