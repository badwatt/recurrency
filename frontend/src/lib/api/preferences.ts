import { apiClient } from "./client";
import { getAuthParams } from "./auth-helpers";

export interface Preferences {
  theme: "light" | "dark" | "mocha" | "system";
  subscriptions_view: "grid" | "list";
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
