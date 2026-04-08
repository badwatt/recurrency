import { useState, useEffect } from "react";
import * as LucideIcons from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { X } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { Category, CreateCategoryRequest } from "@/lib/api/categories";
import { CATEGORY_ICONS, CATEGORY_COLORS } from "@/lib/api/categories";

interface CategoryDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  category?: Category;
  onSubmit: (data: CreateCategoryRequest) => Promise<void>;
}

// Convert kebab-case to PascalCase and get icon component
const getIcon = (name: string | null | undefined): LucideIcon => {
  if (!name) return LucideIcons.Tag;
  const icons = LucideIcons as unknown as Record<string, LucideIcon>;
  const pascalCase = name
    .split("-")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join("");
  return icons[pascalCase] || LucideIcons.Tag;
};

export function CategoryDialog({
  open,
  onOpenChange,
  category,
  onSubmit,
}: CategoryDialogProps) {
  const [name, setName] = useState("");
  const [selectedIcon, setSelectedIcon] = useState<string | null>(null);
  const [selectedColor, setSelectedColor] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Sync state when dialog opens or category changes
  useEffect(() => {
    if (open) {
      setName(category?.name || "");
      setSelectedIcon(category?.icon || null);
      setSelectedColor(category?.color || null);
    }
  }, [open, category]);

  const handleOpenChange = (newOpen: boolean) => {
    if (!newOpen) {
      setName("");
      setSelectedIcon(null);
      setSelectedColor(null);
    }
    onOpenChange(newOpen);
  };

  const handleFormSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;

    setIsSubmitting(true);
    try {
      await onSubmit({
        name: name.trim(),
        icon: selectedIcon,
        color: selectedColor,
      });
      handleOpenChange(false);
    } finally {
      setIsSubmitting(false);
    }
  };

  const IconComponent = getIcon(selectedIcon);

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-[500px]" onOpenAutoFocus={(e) => e.preventDefault()}>
        <DialogHeader>
          <DialogTitle>
            {category ? "Edit Category" : "Add Category"}
          </DialogTitle>
          <DialogDescription>
            {category
              ? "Make changes to your category here."
              : "Create a new category to organize your expenses."}
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleFormSubmit} className="space-y-6">
          <div className="space-y-2">
            <Label htmlFor="name">Name</Label>
            <Input
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Entertainment, Utilities, etc."
            />
          </div>

          {/* Icon Picker */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label>Icon</Label>
              {selectedIcon && (
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={() => setSelectedIcon(null)}
                  className="h-6 px-2 text-xs"
                >
                  <X className="h-3 w-3 mr-1" />
                  Clear
                </Button>
              )}
            </div>
            <div className="grid grid-cols-10 gap-2 max-h-40 overflow-y-auto p-2 border rounded-md">
              {CATEGORY_ICONS.map((iconName) => {
                const Icon = getIcon(iconName);
                return (
                  <button
                    key={iconName}
                    type="button"
                    onClick={() => setSelectedIcon(iconName)}
                    className={`p-2 rounded-md flex items-center justify-center transition-colors ${
                      selectedIcon === iconName
                        ? "bg-primary text-primary-foreground"
                        : "bg-muted hover:bg-muted/80"
                    }`}
                    title={iconName}
                  >
                    <Icon className="h-5 w-5" />
                  </button>
                );
              })}
            </div>
          </div>

          {/* Color Picker */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label>Color</Label>
              {selectedColor && (
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={() => setSelectedColor(null)}
                  className="h-6 px-2 text-xs"
                >
                  <X className="h-3 w-3 mr-1" />
                  Clear
                </Button>
              )}
            </div>
            <div className="flex flex-wrap gap-2">
              {CATEGORY_COLORS.map((color) => (
                <button
                  key={color}
                  type="button"
                  onClick={() => setSelectedColor(color)}
                  className={`w-8 h-8 rounded-full transition-transform ${
                    selectedColor === color ? "ring-2 ring-offset-2 ring-primary scale-110" : ""
                  }`}
                  style={{ backgroundColor: color }}
                />
              ))}
            </div>
          </div>

          {/* Preview */}
          <div className="flex items-center gap-3 p-4 border rounded-lg bg-muted/50">
            <div
              className="flex h-12 w-12 items-center justify-center rounded-full"
              style={{ backgroundColor: (selectedColor || "#6b7280") + "20" }}
            >
              <IconComponent
                className="h-6 w-6"
                style={{ color: selectedColor || "#6b7280" }}
              />
            </div>
            <div>
              <p className="font-medium">{name || "Category Name"}</p>
              <p className="text-sm text-muted-foreground">Preview</p>
            </div>
          </div>

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => handleOpenChange(false)}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={isSubmitting || !name.trim()}>
              {isSubmitting
                ? "Saving..."
                : category
                ? "Save Changes"
                : "Create Category"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
