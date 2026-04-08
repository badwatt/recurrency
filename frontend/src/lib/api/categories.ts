import { apiClient } from "./client";
import { auth } from "@/lib/auth";

export interface Category {
  id: string;
  user_id: string;
  name: string;
  icon: string | null;
  color: string | null;
  is_system: boolean;
  is_default: boolean;
  created_at: string;
  updated_at: string;
}

export interface CategoryWithCounts extends Category {
  subscription_count: number;
  transaction_count: number;
}

export interface CreateCategoryRequest {
  name: string;
  icon?: string | null;
  color?: string | null;
}

export interface UpdateCategoryRequest {
  name?: string;
  icon?: string | null;
  color?: string | null;
}

async function getAuthHeaders() {
  const token = auth.getToken();
  if (!token) throw new Error("Not authenticated");
  return { token };
}

export const categoriesApi = {
  async list(): Promise<Category[]> {
    const { token } = await getAuthHeaders();
    return apiClient.get<Category[]>(`/api/categories?token=${encodeURIComponent(token)}`);
  },

  async listWithCounts(): Promise<CategoryWithCounts[]> {
    const { token } = await getAuthHeaders();
    return apiClient.get<CategoryWithCounts[]>(`/api/categories/with-counts?token=${encodeURIComponent(token)}`);
  },

  async get(id: string): Promise<Category> {
    const { token } = await getAuthHeaders();
    return apiClient.get<Category>(`/api/categories/${id}?token=${encodeURIComponent(token)}`);
  },

  async create(data: CreateCategoryRequest): Promise<Category> {
    const { token } = await getAuthHeaders();
    return apiClient.post<Category>(`/api/categories?token=${encodeURIComponent(token)}`, data);
  },

  async update(id: string, data: UpdateCategoryRequest): Promise<Category> {
    const { token } = await getAuthHeaders();
    return apiClient.put<Category>(`/api/categories/${id}?token=${encodeURIComponent(token)}`, data);
  },

  async delete(id: string): Promise<void> {
    const { token } = await getAuthHeaders();
    return apiClient.delete(`/api/categories/${id}?token=${encodeURIComponent(token)}`);
  },

  async setDefault(id: string): Promise<Category> {
    const { token } = await getAuthHeaders();
    return apiClient.put<Category>(`/api/categories/default?token=${encodeURIComponent(token)}`, { id });
  },
};

// Predefined category icons (Lucide icon names)
export const CATEGORY_ICONS = [
  "home",
  "shopping-bag",
  "coffee",
  "car",
  "heart",
  "film",
  "music",
  "gamepad-2",
  "briefcase",
  "graduation-cap",
  "plane",
  "zap",
  "wifi",
  "smartphone",
  "laptop",
  "monitor",
  "cloud",
  "database",
  "server",
  "lock",
  "shield",
  "credit-card",
  "wallet",
  "gift",
  "star",
  "sun",
  "moon",
  "cloud-rain",
  "thermometer",
  "activity",
  "health",
  "pill",
  "stethoscope",
  "baby",
  "pet",
  "flower",
  "tree-pine",
  "utensils",
  "beer",
  "wine",
];

// Predefined category colors
export const CATEGORY_COLORS = [
  "#EF4444", // Red
  "#F97316", // Orange
  "#F59E0B", // Amber
  "#EAB308", // Yellow
  "#84CC16", // Lime
  "#22C55E", // Green
  "#10B981", // Emerald
  "#14B8A6", // Teal
  "#06B6D4", // Cyan
  "#0EA5E9", // Sky
  "#3B82F6", // Blue
  "#6366F1", // Indigo
  "#8B5CF6", // Violet
  "#A855F7", // Purple
  "#D946EF", // Fuchsia
  "#EC4899", // Pink
  "#F43F5E", // Rose
  "#78716C", // Stone
];
