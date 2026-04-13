import { useState, useEffect } from "react";
import { useNavigate, Link } from "react-router-dom";
import { toast } from "sonner";
import { useAuthStore } from "@/stores/authStore";
import { demoApi, type DemoStatus } from "@/lib/api/demo";
import { getApiBaseUrl, setApiBaseUrl, isTauriApp } from "@/lib/config";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

export function LoginPage() {
  const navigate = useNavigate();
  const { login, isLoading } = useAuthStore();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [demoStatus, setDemoStatus] = useState<DemoStatus | null>(null);
  const [isDemoLoading, setIsDemoLoading] = useState(false);
  const [serverUrl, setServerUrl] = useState("");
  const [serverError, setServerError] = useState("");
  const [serverSaving, setServerSaving] = useState(false);

  useEffect(() => {
    demoApi.getStatus().then(setDemoStatus).catch(console.error);
  }, []);

  useEffect(() => {
    if (isTauriApp()) {
      try {
        setServerUrl(getApiBaseUrl());
      } catch {
        // config not initialized yet
      }
    }
  }, []);

  const handleSaveServer = async () => {
    setServerSaving(true);
    setServerError("");
    let url = serverUrl.trim();
    if (!url) {
      setServerError("URL cannot be empty");
      setServerSaving(false);
      return;
    }
    if (!/^https?:\/\//i.test(url)) url = `https://${url}`;
    try {
      new URL(url);
    } catch {
      setServerError("Invalid URL format");
      setServerSaving(false);
      return;
    }
    url = url.replace(/\/+$/, "");
    try {
      const res = await fetch(`${url}/health`);
      if (!res.ok) throw new Error(`Server responded with ${res.status}`);
      await setApiBaseUrl(url);
      setServerUrl(url);
      toast.success("Server connected");
    } catch (err) {
      setServerError(err instanceof Error ? err.message : "Could not connect");
    } finally {
      setServerSaving(false);
    }
  };

  const handleDemoLogin = async () => {
    setError("");
    setIsDemoLoading(true);
    try {
      // Init demo (creates/resets user) and get credentials
      const { email, password } = await demoApi.init();
      // Login with those credentials
      await login(email, password);
      // Reset demo data
      await demoApi.reset();
      navigate("/");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Demo login failed");
    } finally {
      setIsDemoLoading(false);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    try {
      await login(email, password);
      toast.success("Welcome back!");
      navigate("/");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Login failed");
    }
  };

  return (
    <div className="flex min-h-dvh items-center justify-center bg-background p-4">
      <Card className="w-full max-w-md">
        <CardHeader className="space-y-1">
          <CardTitle className="text-2xl font-bold">Sign in</CardTitle>
          <CardDescription>
            Enter your email and password to access your account
          </CardDescription>
        </CardHeader>
        <form onSubmit={handleSubmit}>
          <CardContent className="space-y-4">
            {error && (
              <div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
                {error}
              </div>
            )}
            {isTauriApp() && (
              <div className="space-y-2">
                <Label htmlFor="server-url">Server URL</Label>
                <div className="flex gap-2">
                  <Input
                    id="server-url"
                    type="text"
                    placeholder="https://myserver.example.com"
                    value={serverUrl}
                    onChange={(e) => setServerUrl(e.target.value)}
                    disabled={serverSaving}
                  />
                  <Button
                    type="button"
                    variant="outline"
                    onClick={handleSaveServer}
                    disabled={serverSaving}
                  >
                    {serverSaving ? "..." : "Save"}
                  </Button>
                </div>
                {serverError && (
                  <p className="text-xs text-destructive">{serverError}</p>
                )}
              </div>
            )}
            <div className="space-y-2">
              <Label htmlFor="email">Email</Label>
              <Input
                id="email"
                type="email"
                placeholder="m@example.com"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="password">Password</Label>
              <Input
                id="password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
                minLength={8}
              />
            </div>
          </CardContent>
          <CardFooter className="flex flex-col space-y-4">
            <Button type="submit" className="w-full" disabled={isLoading}>
              {isLoading ? "Signing in..." : "Sign in"}
            </Button>
            {demoStatus?.enabled && (
              <Button
                type="button"
                variant="outline"
                className="w-full"
                onClick={handleDemoLogin}
                disabled={isDemoLoading}
              >
                {isDemoLoading ? "Loading..." : "Demo account"}
              </Button>
            )}
            <p className="text-sm text-muted-foreground">
              Don't have an account?{" "}
              <Link to="/register" className="text-primary hover:underline">
                Sign up
              </Link>
            </p>
          </CardFooter>
        </form>
      </Card>
    </div>
  );
}
