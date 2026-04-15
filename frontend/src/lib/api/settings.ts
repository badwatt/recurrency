import { apiClient } from "./client";
import { getAuthParams } from "./auth-helpers";
import type { User } from "./auth";

export interface UpdateSettingsRequest {
  name?: string;
  currency?: string;
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
