import { apiClient } from "./client";
import { auth } from "@/lib/auth";

export interface NotificationPreference {
  channel: string;
  enabled: boolean;
  reminder_days_before: number;
}

export interface NotificationPreferencesResponse {
  preferences: NotificationPreference[];
}

export interface UpdatePreferenceRequest {
  channel: string;
  enabled: boolean;
}

async function getAuthParams() {
  const token = auth.getToken();
  if (!token) throw new Error("Not authenticated");
  return { token };
}

export const notificationPreferencesApi = {
  async get(): Promise<NotificationPreferencesResponse> {
    const { token } = await getAuthParams();
    return apiClient.get<NotificationPreferencesResponse>(
      `/api/settings/notifications?token=${encodeURIComponent(token)}`
    );
  },

  async update(preferences: UpdatePreferenceRequest[]): Promise<NotificationPreferencesResponse> {
    const { token } = await getAuthParams();
    return apiClient.put<NotificationPreferencesResponse>(
      `/api/settings/notifications?token=${encodeURIComponent(token)}`,
      { preferences }
    );
  },
};
