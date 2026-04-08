import { create } from "zustand";
import type { Transaction, CreateTransactionRequest, UpdateTransactionRequest } from "@/lib/api/transactions";
import { transactionsApi } from "@/lib/api/transactions";

interface TransactionsState {
  transactions: Transaction[];
  isLoading: boolean;
  error: string | null;

  // Actions
  fetchTransactions: () => Promise<void>;
  createTransaction: (data: CreateTransactionRequest) => Promise<Transaction>;
  updateTransaction: (id: string, data: UpdateTransactionRequest) => Promise<void>;
  deleteTransaction: (id: string) => Promise<void>;
}

export const useTransactionsStore = create<TransactionsState>((set) => ({
  transactions: [],
  isLoading: false,
  error: null,

  fetchTransactions: async () => {
    set({ isLoading: true, error: null });
    try {
      const transactions = await transactionsApi.list();
      set({ transactions, isLoading: false });
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : "Failed to fetch transactions",
        isLoading: false 
      });
    }
  },

  createTransaction: async (data: CreateTransactionRequest) => {
    set({ isLoading: true, error: null });
    try {
      const transaction = await transactionsApi.create(data);
      set((state) => ({
        transactions: [transaction, ...state.transactions],
        isLoading: false,
      }));
      return transaction;
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Failed to create transaction",
        isLoading: false,
      });
      throw error;
    }
  },

  updateTransaction: async (id: string, data: UpdateTransactionRequest) => {
    set({ isLoading: true, error: null });
    try {
      const updated = await transactionsApi.update(id, data);
      set((state) => ({
        transactions: state.transactions.map((t) => (t.id === id ? updated : t)),
        isLoading: false,
      }));
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Failed to update transaction",
        isLoading: false,
      });
      throw error;
    }
  },

  deleteTransaction: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      await transactionsApi.delete(id);
      set((state) => ({
        transactions: state.transactions.filter((t) => t.id !== id),
        isLoading: false,
        error: null,
      }));
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : "Failed to delete transaction";
      set({
        error: errorMessage,
        isLoading: false,
      });
      throw new Error(errorMessage);
    }
  },
}));
