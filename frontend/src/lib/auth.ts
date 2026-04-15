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

// Auth functions that work with our custom Rust backend
export const auth = {
  async signUp(data: { email: string; password: string; name: string }): Promise<AuthResponse> {
    const response = await authApi.register({
      email: data.email,
      password: data.password,
      name: data.name,
    });

    // Store token for later use
    storage.setItem("recurrency-token", response.token);
    storage.setItem("recurrency-user", JSON.stringify(response.user));

    return response;
  },

  async signIn(data: { email: string; password: string }): Promise<AuthResponse> {
    const response = await authApi.login({
      email: data.email,
      password: data.password,
    });

    // Store token for later use
    storage.setItem("recurrency-token", response.token);
    storage.setItem("recurrency-user", JSON.stringify(response.user));

    return response;
  },

  async signOut(): Promise<void> {
    storage.removeItem("recurrency-token");
    storage.removeItem("recurrency-user");
  },

  async getCurrentUser(): Promise<User | null> {
    const token = storage.getItem("recurrency-token");
    if (!token) return null;

    const userStr = storage.getItem("recurrency-user");
    if (!userStr) return null;

    try {
      return JSON.parse(userStr) as User;
    } catch {
      return null;
    }
  },

  updateStoredUser(user: User): void {
    storage.setItem("recurrency-user", JSON.stringify(user));
  },

  getToken(): string | null {
    return storage.getItem("recurrency-token");
  },
};