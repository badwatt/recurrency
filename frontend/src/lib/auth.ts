import { createAuthClient } from "better-auth/react";
import { authApi, type User, type AuthResponse } from "./api/auth";

// Custom storage adapter for our backend
const storage = {
  getItem: (key: string): string | null => {
    return localStorage.getItem(key);
  },
  setItem: (key: string, value: string): void => {
    localStorage.setItem(key, value);
  },
  removeItem: (key: string): void => {
    localStorage.removeItem(key);
  },
};

// Create Better Auth client with custom JWT backend
export const authClient = createAuthClient({
  baseURL: import.meta.env.VITE_API_URL || "http://localhost:8080",
  storage,
  // Disable built-in endpoints since we use custom backend
  disableInternalHandlers: true,
});

// Auth functions that work with our custom Rust backend
export const auth = {
  async signUp(data: { email: string; password: string; name: string }): Promise<AuthResponse> {
    const response = await authApi.register({
      email: data.email,
      password: data.password,
      name: data.name,
    });

    // Store token for later use
    storage.setItem("recur-token", response.token);
    storage.setItem("recur-user", JSON.stringify(response.user));

    return response;
  },

  async signIn(data: { email: string; password: string }): Promise<AuthResponse> {
    const response = await authApi.login({
      email: data.email,
      password: data.password,
    });

    // Store token for later use
    storage.setItem("recur-token", response.token);
    storage.setItem("recur-user", JSON.stringify(response.user));

    return response;
  },

  async signOut(): Promise<void> {
    storage.removeItem("recur-token");
    storage.removeItem("recur-user");
  },

  async getCurrentUser(): Promise<User | null> {
    const token = storage.getItem("recur-token");
    if (!token) return null;

    const userStr = storage.getItem("recur-user");
    if (!userStr) return null;

    try {
      return JSON.parse(userStr) as User;
    } catch {
      return null;
    }
  },

  updateStoredUser(user: User): void {
    storage.setItem("recur-user", JSON.stringify(user));
  },

  async isAuthenticated(): Promise<boolean> {
    const token = storage.getItem("recur-token");
    if (!token) return false;

    try {
      await authApi.verifyToken(token);
      return true;
    } catch {
      // Token invalid, clear it
      this.signOut();
      return false;
    }
  },

  getToken(): string | null {
    return storage.getItem("recur-token");
  },
};
