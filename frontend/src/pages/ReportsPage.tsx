import { useEffect, useState, useCallback } from "react";
import { Download, TrendingUp, DollarSign, CreditCard, Receipt } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, PieChart, Pie, Cell, LineChart, Line } from "recharts";
import { reportsApi, type ReportResponse } from "@/lib/api/reports";
import { useAuthStore } from "@/stores/authStore";
import { useCategoriesStore } from "@/stores/categoriesStore";
import { formatCurrency } from "@/lib/format";
import { getIcon } from "@/lib/icons";

const MONTHS = [
  { value: "1", label: "January" },
  { value: "2", label: "February" },
  { value: "3", label: "March" },
  { value: "4", label: "April" },
  { value: "5", label: "May" },
  { value: "6", label: "June" },
  { value: "7", label: "July" },
  { value: "8", label: "August" },
  { value: "9", label: "September" },
  { value: "10", label: "October" },
  { value: "11", label: "November" },
  { value: "12", label: "December" },
];

export function ReportsPage() {
  const { user } = useAuthStore();
  const { categories, fetchCategories } = useCategoriesStore();
  const [report, setReport] = useState<ReportResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [selectedYear, setSelectedYear] = useState(new Date().getFullYear().toString());
  const [selectedMonth, setSelectedMonth] = useState((new Date().getMonth() + 1).toString());

  const getCategoryStyleByName = useCallback(
    (categoryName: string) => {
      const cat = categories.find((c) => c.name === categoryName);
      if (!cat) return { color: "#6b7280", icon: "" };
      return { color: cat.color || "#6b7280", icon: cat.icon || "" };
    },
    [categories]
  );

  useEffect(() => {
    fetchReport();
    fetchCategories();
  }, [selectedYear, selectedMonth]);

  const fetchReport = async () => {
    setIsLoading(true);
    try {
      const data = await reportsApi.getSummary(
        parseInt(selectedYear),
        parseInt(selectedMonth)
      );
      setReport(data);
    } catch (error) {
      console.error("Failed to fetch report:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const fc = (amount: string | number) => formatCurrency(amount, user?.currency || "EUR");

  const handleExportCSV = () => {
    if (!report) return;

    const headers = ["Category", "Amount", "Count", "Percentage"];
    const rows = report.category_breakdown.map((cat) => [
      cat.category_name,
      cat.total,
      cat.count.toString(),
      cat.percentage.toFixed(1) + "%",
    ]);

    const csv = [headers.join(","), ...rows.map((r) => r.join(","))].join("\n");
    const blob = new Blob([csv], { type: "text/csv" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `recurrency-report-${selectedYear}-${selectedMonth}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  };

  if (!report && !isLoading) {
    return (
      <div className="p-6">
        <div className="text-center text-muted-foreground">
          No data available for this period
        </div>
      </div>
    );
  }

  return (
    <div className="p-4 md:p-6 space-y-6 overflow-x-hidden">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Reports</h1>
          <p className="text-muted-foreground">
            Analyze your spending patterns
          </p>
        </div>
        <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:gap-3">
          <div className="flex items-center gap-2">
            <div className="flex-1 min-w-0">
              <Select value={selectedMonth} onValueChange={setSelectedMonth}>
                <SelectTrigger className="w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {MONTHS.map((m) => (
                    <SelectItem key={m.value} value={m.value}>
                      {m.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="w-20 shrink-0">
              <Select value={selectedYear} onValueChange={setSelectedYear}>
                <SelectTrigger className="w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {[2024, 2025, 2026].map((y) => (
                    <SelectItem key={y} value={y.toString()}>
                      {y}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
          <button
            onClick={handleExportCSV}
            className="flex items-center justify-center gap-1.5 px-3 py-2 text-sm border rounded-md hover:bg-muted"
          >
            <Download className="h-4 w-4" />
            <span className="sm:hidden">Export</span>
            <span className="hidden sm:inline">Export CSV</span>
          </button>
        </div>
      </div>

      {isLoading ? (
        <div className="flex h-64 items-center justify-center">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
        </div>
      ) : report ? (
        <>
          {/* Summary Cards */}
          <div className="grid gap-4 md:grid-cols-4">
            <Card>
              <CardHeader className="flex flex-row items-center justify-between pb-2">
                <CardTitle className="text-sm font-medium">Total Spending</CardTitle>
                <DollarSign className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {fc(report.monthly_summary.overall_total)}
                </div>
                <p className="text-xs text-muted-foreground">
                  {MONTHS.find((m) => m.value === selectedMonth)?.label} {selectedYear}
                </p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between pb-2">
                <CardTitle className="text-sm font-medium">Subscriptions</CardTitle>
                <CreditCard className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {fc(report.monthly_summary.subscriptions_total)}
                </div>
                <p className="text-xs text-muted-foreground">
                  {report.monthly_summary.subscriptions_count} active
                </p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between pb-2">
                <CardTitle className="text-sm font-medium">One-time</CardTitle>
                <Receipt className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {fc(report.monthly_summary.transactions_total)}
                </div>
                <p className="text-xs text-muted-foreground">
                  {report.monthly_summary.transactions_count} transactions
                </p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between pb-2">
                <CardTitle className="text-sm font-medium">Categories</CardTitle>
                <TrendingUp className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {report.category_breakdown.length}
                </div>
                <p className="text-xs text-muted-foreground">
                  with expenses
                </p>
              </CardContent>
            </Card>
          </div>

          {/* Charts */}
          <div className="grid gap-6 md:grid-cols-2">
            {/* Spending Trends */}
            <Card>
              <CardHeader>
                <CardTitle>Spending Trends</CardTitle>
              </CardHeader>
              <CardContent>
                {report.recent_trends.length > 0 ? (
                  <ResponsiveContainer width="100%" height={300}>
                    <LineChart data={report.recent_trends}>
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis dataKey="period" />
                      <YAxis tickFormatter={(value) => `$${value}`} />
                      <Tooltip formatter={(value: number) => fc(value)} />
                      <Line
                        type="monotone"
                        dataKey="total"
                        stroke="#3B82F6"
                        strokeWidth={2}
                        dot={{ fill: "#3B82F6" }}
                      />
                    </LineChart>
                  </ResponsiveContainer>
                ) : (
                  <div className="h-[300px] flex items-center justify-center text-muted-foreground">
                    No trend data available
                  </div>
                )}
              </CardContent>
            </Card>

            {/* Category Breakdown */}
            <Card>
              <CardHeader>
                <CardTitle>By Category</CardTitle>
              </CardHeader>
              <CardContent>
                {report.category_breakdown.length > 0 ? (
                  <div className="flex flex-col sm:flex-row items-center gap-4 sm:gap-8">
                    <ResponsiveContainer width="100%" height={300} className="sm:max-w-[50%]">
                      <PieChart>
                        <Pie
                          data={report.category_breakdown}
                          cx="50%"
                          cy="50%"
                          innerRadius={60}
                          outerRadius={100}
                          paddingAngle={2}
                          dataKey="total"
                          nameKey="category_name"
                        >
                          {report.category_breakdown.map((cat, index) => {
                            const { color } = getCategoryStyleByName(cat.category_name);
                            return <Cell key={`cell-${index}`} fill={color} />;
                          })}
                        </Pie>
                        <Tooltip formatter={(value: number) => fc(value)} />
                      </PieChart>
                    </ResponsiveContainer>
                    <div className="flex-1 space-y-2">
                      {report.category_breakdown.slice(0, 5).map((cat) => {
                        const { color, icon } = getCategoryStyleByName(cat.category_name);
                        const Icon = getIcon(icon);
                        return (
                          <div key={cat.category_name} className="flex items-center gap-2">
                            <div
                              className="w-6 h-6 rounded-full flex items-center justify-center"
                              style={{ backgroundColor: color + "20" }}
                            >
                              <Icon className="h-3 w-3" style={{ color }} />
                            </div>
                            <span className="text-sm flex-1 truncate">{cat.category_name}</span>
                            <span className="text-sm font-medium">{cat.percentage.toFixed(0)}%</span>
                          </div>
                        );
                      })}
                    </div>
                  </div>
                ) : (
                  <div className="h-[300px] flex items-center justify-center text-muted-foreground">
                    No category data available
                  </div>
                )}
              </CardContent>
            </Card>
          </div>

          {/* Category Details Table */}
          <Card>
            <CardHeader>
              <CardTitle>Category Breakdown</CardTitle>
            </CardHeader>
            <CardContent>
              {report.category_breakdown.length > 0 ? (
                <div className="space-y-4">
                  {report.category_breakdown.map((cat) => {
                    const { color, icon } = getCategoryStyleByName(cat.category_name);
                    const Icon = getIcon(icon);
                    return (
                      <div key={cat.category_name} className="space-y-2">
                        <div className="flex items-center justify-between">
                          <div className="flex items-center gap-3">
                            <div
                              className="w-8 h-8 rounded-full flex items-center justify-center"
                              style={{ backgroundColor: color + "20" }}
                            >
                              <Icon className="h-4 w-4" style={{ color }} />
                            </div>
                            <span className="font-medium">{cat.category_name}</span>
                          </div>
                          <div className="flex items-center gap-4">
                            <span className="text-muted-foreground">{cat.count} items</span>
                            <span className="font-bold">{fc(cat.total)}</span>
                          </div>
                        </div>
                        <div className="h-2 bg-muted rounded-full overflow-hidden">
                          <div
                            className="h-full rounded-full"
                            style={{ width: `${cat.percentage}%`, backgroundColor: color }}
                          />
                        </div>
                      </div>
                    );
                  })}
                </div>
              ) : (
                <div className="text-center text-muted-foreground py-8">
                  No category data available
                </div>
              )}
            </CardContent>
          </Card>
        </>
      ) : null}
    </div>
  );
}
