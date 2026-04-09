import { useEffect, useState, useCallback, useMemo } from "react";
import { toast } from "sonner";
import { Plus, Pencil, Trash2, Receipt, Tag, StickyNote } from "lucide-react";
import * as LucideIcons from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { format } from "date-fns";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useTransactionsStore } from "@/stores/transactionsStore";
import { useCategoriesStore } from "@/stores/categoriesStore";
import { useAuthStore } from "@/stores/authStore";
import { TransactionDialog } from "@/components/transaction-dialog";
import { NotesEditDialog } from "@/components/notes-edit-dialog";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import type { Transaction, CreateTransactionRequest } from "@/lib/api/transactions";

// Helper to get Lucide icon component from kebab-case name
const getIconComponent = (iconName: string): LucideIcon => {
  if (!iconName) return Tag;
  const icons = LucideIcons as unknown as Record<string, LucideIcon>;
  const pascalCase = iconName
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join("");
  return icons[pascalCase] || Tag;
};

// Safe toast wrapper
const showToast = (type: "success" | "error", message: string) => {
  setTimeout(() => {
    if (type === "success") {
      toast.success(message);
    } else {
      toast.error(message);
    }
  }, 0);
};

export function TransactionsPage() {
  const { user } = useAuthStore();
  const {
    transactions,
    isLoading,
    error,
    fetchTransactions,
    createTransaction,
    updateTransaction,
    deleteTransaction,
  } = useTransactionsStore();

  const { categories, fetchCategories } = useCategoriesStore();

  const getCategoryStyle = useCallback(
    (categoryId: string | null) => {
      if (!categoryId) return { color: "#6b7280", icon: "", name: "Uncategorized" };
      const cat = categories.find((c) => c.id === categoryId);
      if (!cat) return { color: "#6b7280", icon: "", name: "Uncategorized" };
      return { color: cat.color || "#6b7280", icon: cat.icon || "", name: cat.name };
    },
    [categories]
  );

  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingTransaction, setEditingTransaction] = useState<Transaction | undefined>();
  const [notesEditOpen, setNotesEditOpen] = useState(false);
  const [notesEditingTransaction, setNotesEditingTransaction] = useState<Transaction | undefined>();

  useEffect(() => {
    fetchTransactions();
    fetchCategories();
  }, [fetchTransactions, fetchCategories]);

  const handleCreate = () => {
    setEditingTransaction(undefined);
    setDialogOpen(true);
  };

  const handleEdit = (transaction: Transaction) => {
    setEditingTransaction(transaction);
    setDialogOpen(true);
  };

  const handleDelete = useCallback(
    async (transaction: Transaction) => {
      if (!confirm(`Delete "${transaction.name}"?`)) {
        return;
      }

      try {
        await deleteTransaction(transaction.id);
        showToast("success", `"${transaction.name}" deleted`);
      } catch {
        showToast("error", "Failed to delete transaction");
      }
    },
    [deleteTransaction]
  );

  const handleNotesClick = useCallback(
    (transaction: Transaction) => {
      setNotesEditingTransaction(transaction);
      setNotesEditOpen(true);
    },
    []
  );

  const handleNotesSave = useCallback(
    async (notes: string) => {
      if (!notesEditingTransaction) return;
      await updateTransaction(notesEditingTransaction.id, { notes });
      showToast("success", "Notes updated");
    },
    [notesEditingTransaction, updateTransaction]
  );

  const handleSubmit = useCallback(
    async (data: CreateTransactionRequest) => {
      try {
        if (editingTransaction) {
          await updateTransaction(editingTransaction.id, data);
          showToast("success", "Transaction updated");
        } else {
          await createTransaction(data);
          showToast("success", "Transaction added");
        }
      } catch {
        showToast("error", editingTransaction ? "Failed to update" : "Failed to create transaction");
      }
    },
    [editingTransaction, createTransaction, updateTransaction]
  );

  const formatCurrency = (amount: number, currency: string) => {
    return new Intl.NumberFormat("en-US", {
      style: "currency",
      currency,
    }).format(amount);
  };

  // Group transactions by month
  const groupedByMonth = useMemo(() => {
    const groups: Record<string, Transaction[]> = {};

    transactions.forEach((t) => {
      const date = new Date(t.date);
      const key = format(date, "yyyy-MM");
      if (!groups[key]) {
        groups[key] = [];
      }
      groups[key].push(t);
    });

    return Object.entries(groups)
      .sort(([a], [b]) => b.localeCompare(a))
      .map(([key, txns]) => {
        const date = new Date(key + "-01");
        const label = format(date, "MMMM yyyy");
        const total = txns.reduce((sum, t) => sum + parseFloat(t.amount.toString()), 0);
        return { key, label, transactions: txns, total };
      });
  }, [transactions]);

  if (error) {
    return (
      <div className="p-4 md:p-6">
        <div className="rounded-lg border border-destructive/50 bg-destructive/10 p-4 text-destructive">
          {error}
        </div>
      </div>
    );
  }

  return (
    <div className="p-4 md:p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Transactions</h1>
          <p className="text-muted-foreground">
            Track your one-time expenses
          </p>
        </div>
        <Button onClick={handleCreate}>
          <Plus className="mr-2 h-4 w-4" />
          Add Transaction
        </Button>
      </div>

      {isLoading ? (
        <div className="flex h-64 items-center justify-center">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
        </div>
      ) : transactions.length === 0 ? (
        <Card>
          <CardContent className="flex h-64 flex-col items-center justify-center gap-4">
            <Receipt className="h-12 w-12 text-muted-foreground" />
            <div className="text-center">
              <p className="text-lg font-medium">No transactions yet</p>
              <p className="text-sm text-muted-foreground">
                Add your first one-time expense
              </p>
            </div>
            <Button onClick={handleCreate}>
              <Plus className="mr-2 h-4 w-4" />
              Add Transaction
            </Button>
          </CardContent>
        </Card>
      ) : (
        <Tabs defaultValue={groupedByMonth[0]?.key} className="w-full">
          <TabsList className="mb-4 overflow-x-auto flex-nowrap">
            {groupedByMonth.map((group) => (
              <TabsTrigger key={group.key} value={group.key} className="flex-shrink-0">
                {group.label}
              </TabsTrigger>
            ))}
          </TabsList>

          {groupedByMonth.map((group) => (
            <TabsContent key={group.key} value={group.key} className="space-y-4">
              <div className="flex items-center justify-between rounded-lg bg-muted/50 px-4 py-2">
                <span className="text-sm font-medium">
                  {group.transactions.length} transactions
                </span>
                <span className="text-sm font-bold">
                  {formatCurrency(group.total, user?.currency || "USD")}
                </span>
              </div>

              <div className="space-y-2">
                {group.transactions.map((transaction) => (
                  <Card key={transaction.id} className="min-w-0">
                    <CardContent className="p-3 sm:p-4">
                      <div className="flex items-center gap-2 sm:gap-4">
                        <div className="flex h-9 w-9 sm:h-10 sm:w-10 shrink-0 items-center justify-center rounded-full bg-muted overflow-hidden">
                          {transaction.logo_url ? (
                            <img
                              src={transaction.logo_url}
                              alt={transaction.name}
                              className="h-full w-full object-contain p-1"
                              onError={(e) => {
                                (e.target as HTMLImageElement).style.display = "none";
                              }}
                            />
                          ) : (
                            <Receipt className="h-4 w-4 sm:h-5 sm:w-5 text-muted-foreground" />
                          )}
                        </div>
                        <div className="flex-1 min-w-0">
                          <p className="font-medium truncate text-sm sm:text-base">{transaction.name}</p>
                          <p className="text-xs text-muted-foreground">
                            {format(new Date(transaction.date), "MMM d, yyyy")}
                          </p>
                        </div>
                        <div className="flex items-center gap-2 sm:gap-4 shrink-0">
                          {(() => {
                            const { color, icon, name } = getCategoryStyle(transaction.category_id);
                            const IconComponent = getIconComponent(icon);
                            return (
                              <>
                                {transaction.notes && (
                                  <TooltipProvider delayDuration={200}>
                                    <Tooltip>
                                      <TooltipTrigger asChild>
                                        <Button variant="ghost" size="icon" className="h-8 w-8 sm:h-9 sm:w-9" onClick={() => handleNotesClick(transaction)}>
                                          <StickyNote className="h-4 w-4 text-muted-foreground" />
                                        </Button>
                                      </TooltipTrigger>
                                      <TooltipContent side="bottom" className="max-w-[280px] whitespace-pre-wrap text-xs">
                                        {transaction.notes}
                                      </TooltipContent>
                                    </Tooltip>
                                  </TooltipProvider>
                                )}
                                <Badge
                                  variant="secondary"
                                  className="font-normal text-xs hidden sm:inline-flex"
                                  style={{ color: color, backgroundColor: `${color}20` }}
                                >
                                  <IconComponent className="h-3 w-3 mr-1" />
                                  {name}
                                </Badge>
                              </>
                            );
                          })()}
                          <p className="font-medium text-sm sm:text-base">
                            {formatCurrency(parseFloat(transaction.amount.toString()), transaction.currency)}
                          </p>
                          <div className="flex items-center gap-0.5 sm:gap-1">
                            <Button variant="ghost" size="icon" className="h-8 w-8 sm:h-9 sm:w-9" onClick={() => handleEdit(transaction)}>
                              <Pencil className="h-4 w-4" />
                            </Button>
                            <Button variant="ghost" size="icon" className="h-8 w-8 sm:h-9 sm:w-9" onClick={() => handleDelete(transaction)}>
                              <Trash2 className="h-4 w-4 text-destructive" />
                            </Button>
                          </div>
                        </div>
                      </div>
                      <div className="mt-2 sm:hidden flex items-center gap-1">
                        {(() => {
                          const { color, icon, name } = getCategoryStyle(transaction.category_id);
                          const IconComponent = getIconComponent(icon);
                          return (
                            <>
                              {transaction.notes && (
                                <TooltipProvider delayDuration={200}>
                                  <Tooltip>
                                    <TooltipTrigger asChild>
                                      <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => handleNotesClick(transaction)}>
                                        <StickyNote className="h-3.5 w-3.5 text-muted-foreground" />
                                      </Button>
                                    </TooltipTrigger>
                                    <TooltipContent side="bottom" className="max-w-[280px] whitespace-pre-wrap text-xs">
                                      {transaction.notes}
                                    </TooltipContent>
                                  </Tooltip>
                                </TooltipProvider>
                              )}
                              <Badge variant="secondary" className="font-normal text-xs" style={{ color: color, backgroundColor: `${color}20` }}>
                                <IconComponent className="h-3 w-3 mr-1" />
                                {name}
                              </Badge>
                            </>
                          );
                        })()}
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </TabsContent>
          ))}
        </Tabs>
      )}

      <TransactionDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        transaction={editingTransaction}
        defaultCurrency={user?.currency}
        onSubmit={handleSubmit}
      />

      <NotesEditDialog
        open={notesEditOpen}
        onOpenChange={setNotesEditOpen}
        notes={notesEditingTransaction?.notes || ""}
        onSave={handleNotesSave}
        title={`Notes: ${notesEditingTransaction?.name || ""}`}
      />
    </div>
  );
}
