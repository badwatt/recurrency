import { apiClient } from "./client";

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
    return apiClient.post<User>(`/api/auth/verify-email?token=${encodeURIComponent(token)}`, { code });
  },

  async resendVerification(token: string): Promise<void> {
    return apiClient.post(`/api/auth/resend-verification?token=${encodeURIComponent(token)}`);
  },
};