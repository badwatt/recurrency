import { useEffect, useState, useCallback } from "react";
import { toast } from "sonner";
import { Plus, Pencil, Trash2, Tag, Star } from "lucide-react";
import * as LucideIcons from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { useCategoriesStore } from "@/stores/categoriesStore";
import { CategoryDialog } from "@/components/category-dialog";
import type { Category, CreateCategoryRequest } from "@/lib/api/categories";

// Helper to get icon component safely from kebab-case name
const getIcon = (name: string | null | undefined): LucideIcon => {
  if (!name) return Tag;
  const icons = LucideIcons as unknown as Record<string, LucideIcon>;
  // Convert kebab-case to PascalCase
  const pascalCase = name
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join("");
  return icons[pascalCase] || Tag;
};

// Safe toast wrapper that schedules after render
const showToast = (type: "success" | "error", message: string) => {
  setTimeout(() => {
    if (type === "success") {
      toast.success(message);
    } else {
      toast.error(message);
    }
  }, 0);
};

export function CategoriesPage() {
  const {
    categories,
    isLoading,
    error,
    fetchCategories,
    createCategory,
    updateCategory,
    deleteCategory,
    setDefaultCategory,
  } = useCategoriesStore();

  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingCategory, setEditingCategory] = useState<Category | undefined>();
  const [categoriesKey, setCategoriesKey] = useState(0);

  useEffect(() => {
    fetchCategories().then(() => setCategoriesKey((k) => k + 1));
  }, [fetchCategories]);

  const handleCreate = () => {
    setEditingCategory(undefined);
    setDialogOpen(true);
  };

  const handleEdit = (category: Category) => {
    setEditingCategory(category);
    setDialogOpen(true);
  };

  const handleDelete = useCallback(async (category: Category) => {
    if (!confirm(`Delete "${category.name}"? Subscriptions and transactions using this category will not be deleted.`)) {
      return;
    }

    try {
      await deleteCategory(category.id);
      showToast("success", `"${category.name}" deleted`);
    } catch {
      showToast("error", "Failed to delete category");
    }
  }, [deleteCategory]);

  const handleSetDefault = useCallback(async (category: Category) => {
    try {
      await setDefaultCategory(category.id);
      showToast("success", `"${category.name}" is now the default category`);
    } catch {
      showToast("error", "Failed to set default category");
    }
  }, [setDefaultCategory]);

  const handleSubmit = useCallback(async (data: CreateCategoryRequest) => {
    try {
      if (editingCategory) {
        await updateCategory(editingCategory.id, data);
        showToast("success", "Category updated");
      } else {
        await createCategory(data);
        showToast("success", "Category created");
      }
      // Force re-render by changing key
      setCategoriesKey((k) => k + 1);
      setDialogOpen(false);
    } catch {
      showToast("error", editingCategory ? "Failed to update" : "Failed to create category");
    }
  }, [editingCategory, createCategory, updateCategory]);

  if (error) {
    return (
      <div className="p-4 md:p-6">
        <div className="rounded-lg border border-destructive bg-destructive/10 p-4 text-destructive">
          {error}
        </div>
      </div>
    );
  }

  return (
    <div className="p-4 md:p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Categories</h1>
          <p className="text-muted-foreground">
            Organize your expenses with categories
          </p>
        </div>
        <Button onClick={handleCreate}>
          <Plus className="mr-2 h-4 w-4" />
          Add Category
        </Button>
      </div>

      {isLoading && categories.length === 0 ? (
        <div className="flex h-64 items-center justify-center">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
        </div>
      ) : categories.length === 0 ? (
        <Card>
          <CardContent className="flex h-64 flex-col items-center justify-center gap-4">
            <Tag className="h-12 w-12 text-muted-foreground" />
            <div className="text-center">
              <p className="text-lg font-medium">No categories yet</p>
              <p className="text-sm text-muted-foreground">
                Create categories to organize your subscriptions and transactions
              </p>
            </div>
            <Button onClick={handleCreate}>
              <Plus className="mr-2 h-4 w-4" />
              Add Category
            </Button>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4" key={categoriesKey}>
          {categories.map((category) => {
            const Icon = getIcon(category.icon);
            return (
              <Card key={category.id} className="min-w-0">
                <CardContent className="p-3 sm:p-4">
                  <div className="flex items-center gap-2 sm:gap-3">
                    <div
                      className="flex h-10 w-10 sm:h-12 sm:w-12 shrink-0 items-center justify-center rounded-full"
                      style={{ backgroundColor: (category.color || "#888") + "20" }}
                    >
                      <Icon
                        className="h-5 w-5 sm:h-6 sm:w-6"
                        style={{ color: category.color || "#888" }}
                      />
                    </div>
                    <div className="flex-1 min-w-0">
                      <p className="font-medium truncate text-sm sm:text-base">
                        {category.name}
                      </p>
                      {category.is_default && (
                        <span className="text-xs text-muted-foreground">Default</span>
                      )}
                    </div>
                    <div className="flex gap-0.5 sm:gap-1 shrink-0">
                      {!category.is_default && (
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8 sm:h-9 sm:w-9"
                          onClick={() => handleSetDefault(category)}
                          title="Set as default"
                        >
                          <Star className="h-4 w-4" />
                        </Button>
                      )}
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8 sm:h-9 sm:w-9"
                        onClick={() => handleEdit(category)}
                      >
                        <Pencil className="h-4 w-4" />
                      </Button>
                      {!category.is_default && (
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8 sm:h-9 sm:w-9"
                          onClick={() => handleDelete(category)}
                        >
                          <Trash2 className="h-4 w-4 text-destructive" />
                        </Button>
                      )}
                    </div>
                  </div>
                </CardContent>
              </Card>
            );
          })}
        </div>
      )}

      <CategoryDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        category={editingCategory}
        onSubmit={handleSubmit}
      />
    </div>
  );
}
