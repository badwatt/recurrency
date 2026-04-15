import { apiClient } from "./client";
import { getAuthParams } from "./auth-helpers";

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
