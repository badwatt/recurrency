import { useEffect, useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import { useAuthStore } from "@/stores/authStore";
import { settingsApi } from "@/lib/api/settings";
import { auth } from "@/lib/auth";
import { getApiBaseUrl, isTauriApp } from "@/lib/config";
import { notificationPreferencesApi, type NotificationPreference } from "@/lib/api/notificationPreferences";

const settingsSchema = z.object({
  name: z.string().min(2, "Name must be at least 2 characters"),
  currency: z.string().min(3, "Select a currency"),
});

type SettingsForm = z.infer<typeof settingsSchema>;

const currencies = [
  { code: "USD", name: "US Dollar" },
  { code: "EUR", name: "Euro" },
  { code: "GBP", name: "British Pound" },
  { code: "JPY", name: "Japanese Yen" },
  { code: "CAD", name: "Canadian Dollar" },
  { code: "AUD", name: "Australian Dollar" },
];

const notificationChannels = [
  { key: "email", label: "Email Notifications", description: "Receive email reminders before payments" },
  { key: "push", label: "Push Notifications", description: "Receive push notifications on your devices" },
  { key: "system", label: "System Notifications", description: "Receive desktop notifications" },
];

export function SettingsPage() {
  const [serverUrl, setServerUrl] = useState("");

  useEffect(() => {
    setServerUrl(getApiBaseUrl());
  }, []);

  const { user, setUser } = useAuthStore();
  const [isLoading, setIsLoading] = useState(false);
  const [message, setMessage] = useState<{ type: "success" | "error"; text: string } | null>(null);
  const [notificationPrefs, setNotificationPrefs] = useState<Record<string, boolean>>({
    email: true,
    push: true,
    system: true,
  });
  const [prefsLoading, setPrefsLoading] = useState(false);

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors, isSubmitting },
  } = useForm<SettingsForm>({
    resolver: zodResolver(settingsSchema),
    defaultValues: {
      name: user?.name || "",
      currency: user?.currency || "USD",
    },
  });

  useEffect(() => {
    if (user) {
      reset({
        name: user.name || "",
        currency: user.currency || "USD",
      });
    }
  }, [user, reset]);

  // Load notification preferences
  useEffect(() => {
    const loadPrefs = async () => {
      try {
        const response = await notificationPreferencesApi.get();
        const prefsMap: Record<string, boolean> = {};
        response.preferences.forEach((p: NotificationPreference) => {
          prefsMap[p.channel] = p.enabled;
        });
        setNotificationPrefs(prefsMap);
      } catch (error) {
        console.error("Failed to load notification preferences:", error);
      }
    };
    loadPrefs();
  }, []);

  const handleNotificationToggle = async (channel: string, enabled: boolean) => {
    setNotificationPrefs((prev) => ({ ...prev, [channel]: enabled }));
    setPrefsLoading(true);
    
    try {
      await notificationPreferencesApi.update([{ channel, enabled }]);
    } catch (error) {
      console.error("Failed to update notification preference:", error);
      // Revert on error
      setNotificationPrefs((prev) => ({ ...prev, [channel]: !enabled }));
    } finally {
      setPrefsLoading(false);
    }
  };

  const onSubmit = async (data: SettingsForm) => {
    setIsLoading(true);
    setMessage(null);
    
    try {
      const updatedUser = await settingsApi.update({
        name: data.name,
        currency: data.currency,
      });
      
      // Update local state and localStorage
      setUser(updatedUser);
      auth.updateStoredUser(updatedUser);
      reset({
        name: updatedUser.name || "",
        currency: updatedUser.currency,
      });
      
      setMessage({ type: "success", text: "Settings saved successfully!" });
    } catch (error) {
      console.error("Failed to save settings:", error);
      setMessage({ type: "error", text: "Failed to save settings. Please try again." });
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="p-4 md:p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Settings</h1>
        <p className="text-muted-foreground">
          Manage your account and preferences
        </p>
      </div>

      <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
        <Card>
          <CardHeader>
            <CardTitle>Profile</CardTitle>
            <CardDescription>
              Update your personal information
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="email">Email</Label>
              <Input
                id="email"
                type="email"
                value={user?.email || ""}
                disabled
                className="bg-muted"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="name">Name</Label>
              <Input
                id="name"
                type="text"
                {...register("name")}
              />
              {errors.name && (
                <p className="text-xs text-destructive">{errors.name.message}</p>
              )}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Preferences</CardTitle>
            <CardDescription>
              Customize your experience
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="currency">Default Currency</Label>
              <select
                id="currency"
                {...register("currency")}
                className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm focus:outline-none focus:ring-1 focus:ring-ring"
              >
                {currencies.map((currency) => (
                  <option key={currency.code} value={currency.code}>
                    {currency.code} - {currency.name}
                  </option>
                ))}
              </select>
              {errors.currency && (
                <p className="text-xs text-destructive">{errors.currency.message}</p>
              )}
            </div>
          </CardContent>
        </Card>

        {isTauriApp() && (
        <Card>
          <CardHeader>
            <CardTitle>Server</CardTitle>
            <CardDescription>
              Your Recurrency server connection
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="server-url">Server URL</Label>
              <Input
                id="server-url"
                type="text"
                value={serverUrl}
                disabled
                className="bg-muted"
              />
            </div>
          </CardContent>
        </Card>
        )}

        <Card>
          <CardHeader>
            <CardTitle>Notifications</CardTitle>
            <CardDescription>
              Configure how you receive notifications
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {notificationChannels.map((channel, index) => (
              <div key={channel.key}>
                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium">{channel.label}</p>
                    <p className="text-sm text-muted-foreground">
                      {channel.description}
                    </p>
                  </div>
                  <Switch
                    checked={notificationPrefs[channel.key] ?? true}
                    onCheckedChange={(enabled) => handleNotificationToggle(channel.key, enabled)}
                    disabled={prefsLoading}
                  />
                </div>
                {index < notificationChannels.length - 1 && <Separator className="mt-4" />}
              </div>
            ))}
          </CardContent>
        </Card>

        {message && (
          <div
            className={`p-4 rounded-md ${
              message.type === "success"
                ? "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-100"
                : "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-100"
            }`}
          >
            {message.text}
          </div>
        )}

        <div className="flex justify-end">
          <Button type="submit" disabled={isSubmitting || isLoading}>
            {isSubmitting || isLoading ? "Saving..." : "Save Changes"}
          </Button>
        </div>
      </form>
    </div>
  );
}
