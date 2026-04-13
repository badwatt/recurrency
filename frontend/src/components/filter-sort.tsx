import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import { ArrowUp, ArrowDown, X, Filter } from "lucide-react";

export interface SortOption {
  value: string;
  label: string;
}

export interface FilterConfig {
  status?: string;
  frequency?: string;
  categoryId?: string;
  priceMin?: string;
  priceMax?: string;
}

export interface SortConfig {
  field: string;
  direction: "asc" | "desc";
}

interface FilterSortProps {
  filters: FilterConfig;
  sort: SortConfig;
  onFiltersChange: (filters: FilterConfig) => void;
  onSortChange: (sort: SortConfig) => void;
  onClear: () => void;
  statusOptions?: { value: string; label: string }[];
  frequencyOptions?: { value: string; label: string }[];
  categories: { id: string; name: string }[];
  sortOptions: SortOption[];
}

export function FilterSort({
  filters,
  sort,
  onFiltersChange,
  onSortChange,
  onClear,
  statusOptions,
  frequencyOptions,
  categories,
  sortOptions,
}: FilterSortProps) {
  const activeCount = countActiveFilters(filters);

  return (
    <div className="space-y-3">
      <div className="flex items-center gap-2 flex-wrap">
        <div className="flex items-center gap-1.5">
          <Filter className="h-4 w-4 text-muted-foreground" />
          <span className="text-sm font-medium">Filters</span>
          {activeCount > 0 && (
            <Badge variant="secondary" className="h-5 px-1.5 text-xs">
              {activeCount}
            </Badge>
          )}
        </div>

        {statusOptions && (
          <Select
            value={filters.status || "all"}
            onValueChange={(v) =>
              onFiltersChange({ ...filters, status: v === "all" ? undefined : v })
            }
          >
            <SelectTrigger className="w-[120px]">
              <SelectValue placeholder="Status" />
            </SelectTrigger>
            <SelectContent>
              {statusOptions.map((o) => (
                <SelectItem key={o.value} value={o.value}>
                  {o.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}

        {frequencyOptions && (
          <Select
            value={filters.frequency || "all"}
            onValueChange={(v) =>
              onFiltersChange({ ...filters, frequency: v === "all" ? undefined : v })
            }
          >
            <SelectTrigger className="w-[130px]">
              <SelectValue placeholder="Frequency" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All frequencies</SelectItem>
              {frequencyOptions.map((o) => (
                <SelectItem key={o.value} value={o.value}>
                  {o.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}

        <Select
          value={filters.categoryId || "all"}
          onValueChange={(v) =>
            onFiltersChange({ ...filters, categoryId: v === "all" ? undefined : v })
          }
        >
          <SelectTrigger className="w-[140px]">
            <SelectValue placeholder="Category" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All categories</SelectItem>
            {categories.map((c) => (
              <SelectItem key={c.id} value={c.id}>
                {c.name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>

        <div className="flex items-center gap-1">
          <Input
            type="number"
            placeholder="Min"
            className="w-[80px] h-9 text-sm"
            value={filters.priceMin || ""}
            onChange={(e) =>
              onFiltersChange({ ...filters, priceMin: e.target.value || undefined })
            }
          />
          <span className="text-muted-foreground text-xs">–</span>
          <Input
            type="number"
            placeholder="Max"
            className="w-[80px] h-9 text-sm"
            value={filters.priceMax || ""}
            onChange={(e) =>
              onFiltersChange({ ...filters, priceMax: e.target.value || undefined })
            }
          />
        </div>

        {activeCount > 0 && (
          <Button variant="ghost" size="sm" onClick={onClear} className="h-8 px-2">
            <X className="h-3.5 w-3.5 mr-1" />
            Clear
          </Button>
        )}
      </div>

      <div className="flex items-center gap-2">
        <span className="text-sm text-muted-foreground">Sort by</span>
        <Select
          value={sort.field}
          onValueChange={(v) => onSortChange({ ...sort, field: v })}
        >
          <SelectTrigger className="w-[160px] h-8">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {sortOptions.map((o) => (
              <SelectItem key={o.value} value={o.value}>
                {o.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8"
          onClick={() =>
            onSortChange({ ...sort, direction: sort.direction === "asc" ? "desc" : "asc" })
          }
          title={sort.direction === "asc" ? "Ascending" : "Descending"}
        >
          {sort.direction === "asc" ? (
            <ArrowUp className="h-4 w-4" />
          ) : (
            <ArrowDown className="h-4 w-4" />
          )}
        </Button>
      </div>
    </div>
  );
}

function countActiveFilters(f: FilterConfig): number {
  let n = 0;
  if (f.status) n++;
  if (f.frequency) n++;
  if (f.categoryId) n++;
  if (f.priceMin) n++;
  if (f.priceMax) n++;
  return n;
}