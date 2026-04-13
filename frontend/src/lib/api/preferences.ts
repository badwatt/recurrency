import { apiClient } from "./client";
import { auth } from "@/lib/auth";

export interface Preferences {
  theme: "light" | "dark" | "mocha" | "system";
  subscriptions_view: "grid" | "list";
}

async function getAuthParams() {
  const token = auth.getToken();
  if (!token) throw new Error("Not authenticated");
  return { token };
}

export const preferencesApi = {
  async get(): Promise<Preferences> {
    const { token } = await getAuthParams();
    return apiClient.get<Preferences>(
      `/api/preferences?token=${encodeURIComponent(token)}`
    );
  },

  async update(data: Partial<Preferences>): Promise<Preferences> {
    const { token } = await getAuthParams();
    return apiClient.put<Preferences>(
      `/api/preferences?token=${encodeURIComponent(token)}`,
      data
    );
  },
};
