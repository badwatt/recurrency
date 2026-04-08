import { apiClient } from "./client";
import { auth } from "@/lib/auth";
import type { User } from "./auth";

export interface UpdateSettingsRequest {
  name?: string;
  currency?: string;
}

async function getAuthParams() {
  const token = auth.getToken();
  if (!token) throw new Error("Not authenticated");
  return { token };
}

export const settingsApi = {
  async get(): Promise<User> {
    const { token } = await getAuthParams();
    return apiClient.get<User>(
      `/api/settings?token=${encodeURIComponent(token)}`
    );
  },

  async update(data: UpdateSettingsRequest): Promise<User> {
    const { token } = await getAuthParams();
    return apiClient.put<User>(
      `/api/settings?token=${encodeURIComponent(token)}`,
      data
    );
  },
};
