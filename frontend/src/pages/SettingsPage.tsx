import { useEffect, useState } from "react";
import { z } from "zod";
import { toast } from "sonner";
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

const nameSchema = z.string().min(2, "Name must be at least 2 characters");

import { CURRENCIES } from "@/lib/constants";

const notificationChannels = [
  { key: "email", label: "Email Notifications", description: "Receive email reminders before payments" },
  { key: "push", label: "Push Notifications", description: "Receive push notifications on your devices" },
  { key: "system", label: "System Notifications", description: "Receive desktop notifications" },
];

export function SettingsPage() {
  const { user, setUser } = useAuthStore();
  const [name, setName] = useState(user?.name || "");
  const [currency, setCurrency] = useState(user?.currency || "EUR");
  const [notificationPrefs, setNotificationPrefs] = useState<Record<string, boolean>>({
    email: false,
    push: false,
    system: false,
  });
  const [prefsLoading, setPrefsLoading] = useState(false);
  const [serverUrl, setServerUrl] = useState("");

  useEffect(() => {
    setServerUrl(getApiBaseUrl());
  }, []);

  useEffect(() => {
    if (user) {
      setName(user.name || "");
      setCurrency(user.currency || "EUR");
    }
  }, [user]);

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

  const saveName = async () => {
    const result = nameSchema.safeParse(name);
    if (!result.success) {
      toast.error(result.error.errors[0].message);
      return;
    }
    try {
      const updatedUser = await settingsApi.update({ name });
      setUser(updatedUser);
      auth.updateStoredUser(updatedUser);
      toast.success("Name saved");
    } catch {
      toast.error("Failed to save name");
    }
  };

  const saveCurrency = async (newCurrency: string) => {
    setCurrency(newCurrency);
    try {
      const updatedUser = await settingsApi.update({ currency: newCurrency });
      setUser(updatedUser);
      auth.updateStoredUser(updatedUser);
      toast.success("Currency saved");
    } catch {
      toast.error("Failed to save currency");
    }
  };

  const handleNotificationToggle = async (channel: string, enabled: boolean) => {
    setNotificationPrefs((prev) => ({ ...prev, [channel]: enabled }));
    setPrefsLoading(true);

    try {
      await notificationPreferencesApi.update([{ channel, enabled }]);
      toast.success(`${channel} notifications ${enabled ? "enabled" : "disabled"}`);
    } catch {
      toast.error("Failed to update notification preference");
      setNotificationPrefs((prev) => ({ ...prev, [channel]: !enabled }));
    } finally {
      setPrefsLoading(false);
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
              value={name}
              onChange={(e) => setName(e.target.value)}
              onBlur={saveName}
            />
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
              value={currency}
              onChange={(e) => saveCurrency(e.target.value)}
              className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm focus:outline-none focus:ring-1 focus:ring-ring"
            >
              {CURRENCIES.map((c) => (
                <option key={c.value} value={c.value}>
                  {c.value} - {c.label}
                </option>
              ))}
            </select>
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
                  checked={notificationPrefs[channel.key] ?? false}
                  onCheckedChange={(enabled) => handleNotificationToggle(channel.key, enabled)}
                  disabled={prefsLoading}
                />
              </div>
              {index < notificationChannels.length - 1 && <Separator className="mt-4" />}
            </div>
          ))}
        </CardContent>
      </Card>
    </div>
  );
}