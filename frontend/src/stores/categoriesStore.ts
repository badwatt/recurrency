import { create } from "zustand";
import type { Category, CreateCategoryRequest, UpdateCategoryRequest } from "@/lib/api/categories";
import { categoriesApi } from "@/lib/api/categories";

interface CategoriesState {
  categories: Category[];
  isLoading: boolean;
  error: string | null;

  // Actions
  fetchCategories: () => Promise<void>;
  createCategory: (data: CreateCategoryRequest) => Promise<Category>;
  updateCategory: (id: string, data: UpdateCategoryRequest) => Promise<void>;
  deleteCategory: (id: string) => Promise<void>;
  setDefaultCategory: (id: string) => Promise<void>;
}

export const useCategoriesStore = create<CategoriesState>((set) => ({
  categories: [],
  isLoading: false,
  error: null,

  fetchCategories: async () => {
    set({ isLoading: true, error: null });
    try {
      const categories = await categoriesApi.list();
      set({ categories, isLoading: false });
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : "Failed to fetch categories",
        isLoading: false 
      });
    }
  },

  createCategory: async (data: CreateCategoryRequest) => {
    set({ isLoading: true, error: null });
    try {
      const category = await categoriesApi.create(data);
      set((state) => ({
        categories: [...state.categories, category].sort((a, b) => a.name.localeCompare(b.name)),
        isLoading: false,
      }));
      return category;
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Failed to create category",
        isLoading: false,
      });
      throw error;
    }
  },

  updateCategory: async (id: string, data: UpdateCategoryRequest) => {
    set({ isLoading: true, error: null });
    try {
      const updated = await categoriesApi.update(id, data);
      set((state) => {
        const newCategories = state.categories.map((c) => (c.id === id ? updated : c));
        // Force a new reference by spreading
        return { categories: [...newCategories], isLoading: false };
      });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Failed to update category",
        isLoading: false,
      });
      throw error;
    }
  },

  deleteCategory: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      await categoriesApi.delete(id);
      set((state) => ({
        categories: state.categories.filter((c) => c.id !== id),
        isLoading: false,
      }));
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Failed to delete category",
        isLoading: false,
      });
      throw error;
    }
  },

  setDefaultCategory: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      await categoriesApi.setDefault(id);
      set((state) => {
        // Remove default flag from all categories and set it only for the new default
        const newCategories = state.categories.map((c) => ({
          ...c,
          is_default: c.id === id,
        }));
        return { categories: newCategories, isLoading: false };
      });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Failed to set default category",
        isLoading: false,
      });
      throw error;
    }
  },
}));
