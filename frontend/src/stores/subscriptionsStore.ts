import { create } from "zustand";
import type { Subscription, CreateSubscriptionRequest, UpdateSubscriptionRequest } from "@/lib/api/subscriptions";
import { subscriptionsApi } from "@/lib/api/subscriptions";

interface SubscriptionsState {
  subscriptions: Subscription[];
  isLoading: boolean;
  error: string | null;

  // Actions
  fetchSubscriptions: () => Promise<void>;
  createSubscription: (data: CreateSubscriptionRequest) => Promise<Subscription>;
  updateSubscription: (id: string, data: UpdateSubscriptionRequest) => Promise<void>;
  deleteSubscription: (id: string) => Promise<void>;
  getUpcoming: () => Promise<Subscription[]>;
}

export const useSubscriptionsStore = create<SubscriptionsState>((set) => ({
  subscriptions: [],
  isLoading: false,
  error: null,

  fetchSubscriptions: async () => {
    set({ isLoading: true, error: null });
    try {
      const subscriptions = await subscriptionsApi.list();
      set({ subscriptions, isLoading: false });
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : "Failed to fetch subscriptions",
        isLoading: false 
      });
    }
  },

  createSubscription: async (data: CreateSubscriptionRequest) => {
    set({ isLoading: true, error: null });
    try {
      const subscription = await subscriptionsApi.create(data);
      set((state) => ({
        subscriptions: [subscription, ...state.subscriptions],
        isLoading: false,
      }));
      return subscription;
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Failed to create subscription",
        isLoading: false,
      });
      throw error;
    }
  },

  updateSubscription: async (id: string, data: UpdateSubscriptionRequest) => {
    set({ isLoading: true, error: null });
    try {
      const updated = await subscriptionsApi.update(id, data);
      set((state) => ({
        subscriptions: state.subscriptions.map((s) => (s.id === id ? updated : s)),
        isLoading: false,
      }));
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Failed to update subscription",
        isLoading: false,
      });
      throw error;
    }
  },

  deleteSubscription: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      await subscriptionsApi.delete(id);
      set((state) => ({
        subscriptions: state.subscriptions.filter((s) => s.id !== id),
        isLoading: false,
        error: null,
      }));
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : "Failed to delete subscription";
      set({
        error: errorMessage,
        isLoading: false,
      });
      throw new Error(errorMessage);
    }
  },

  getUpcoming: async () => {
    return subscriptionsApi.upcoming();
  },
}));
