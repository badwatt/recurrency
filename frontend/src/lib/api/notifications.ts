import { apiClient } from "./client";
import { auth } from "@/lib/auth";

export interface Notification {
  id: string;
  user_id: string;
  type: string;
  title: string;
  message: string | null;
  is_read: boolean;
  created_at: string;
}

export interface NotificationResponse {
  notifications: Notification[];
  unread_count: number;
}

async function getAuthParams() {
  const token = auth.getToken();
  if (!token) throw new Error("Not authenticated");
  return { token };
}

export const notificationsApi = {
  async list(): Promise<NotificationResponse> {
    const { token } = await getAuthParams();
    return apiClient.get<NotificationResponse>(
      `/api/notifications?token=${encodeURIComponent(token)}`
    );
  },

  async markRead(id: string): Promise<{ success: boolean }> {
    const { token } = await getAuthParams();
    return apiClient.post<{ success: boolean }>(
      `/api/notifications/${id}/read?token=${encodeURIComponent(token)}`
    );
  },

  async markAllRead(): Promise<{ marked_count: number }> {
    const { token } = await getAuthParams();
    return apiClient.post<{ marked_count: number }>(
      `/api/notifications/read-all?token=${encodeURIComponent(token)}`
    );
  },

  async delete(id: string): Promise<{ success: boolean }> {
    const { token } = await getAuthParams();
    return apiClient.delete<{ success: boolean }>(
      `/api/notifications/${id}?token=${encodeURIComponent(token)}`
    );
  },
};
