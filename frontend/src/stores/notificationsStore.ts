import { create } from "zustand";
import { notificationsApi, type Notification, type NotificationResponse } from "@/lib/api/notifications";

interface NotificationsState {
  notifications: Notification[];
  unreadCount: number;
  isLoading: boolean;
  error: string | null;
  isOpen: boolean;
  fetchNotifications: () => Promise<void>;
  markAsRead: (id: string) => Promise<void>;
  markAllAsRead: () => Promise<void>;
  deleteNotification: (id: string) => Promise<void>;
  toggle: () => void;
  setOpen: (open: boolean) => void;
}

export const useNotificationsStore = create<NotificationsState>((set, get) => ({
  notifications: [],
  unreadCount: 0,
  isLoading: false,
  error: null,
  isOpen: false,

  fetchNotifications: async () => {
    set({ isLoading: true, error: null });
    try {
      const response: NotificationResponse = await notificationsApi.list();
      set({
        notifications: response.notifications,
        unreadCount: response.unread_count,
        isLoading: false,
      });
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  markAsRead: async (id: string) => {
    try {
      await notificationsApi.markRead(id);
      set((state) => ({
        notifications: state.notifications.map((n) =>
          n.id === id ? { ...n, is_read: true } : n
        ),
        unreadCount: Math.max(0, state.unreadCount - 1),
      }));
    } catch (error) {
      console.error("Failed to mark notification as read:", error);
    }
  },

  markAllAsRead: async () => {
    try {
      await notificationsApi.markAllRead();
      set((state) => ({
        notifications: state.notifications.map((n) => ({ ...n, is_read: true })),
        unreadCount: 0,
      }));
    } catch (error) {
      console.error("Failed to mark all as read:", error);
    }
  },

  deleteNotification: async (id: string) => {
    try {
      await notificationsApi.delete(id);
      const notification = get().notifications.find((n) => n.id === id);
      set((state) => ({
        notifications: state.notifications.filter((n) => n.id !== id),
        unreadCount: notification && !notification.is_read
          ? Math.max(0, state.unreadCount - 1)
          : state.unreadCount,
      }));
    } catch (error) {
      console.error("Failed to delete notification:", error);
    }
  },

  toggle: () => set((state) => ({ isOpen: !state.isOpen })),
  setOpen: (open: boolean) => set({ isOpen: open }),
}));
