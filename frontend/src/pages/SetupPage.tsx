import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { setApiBaseUrl } from "@/lib/config";

export function SetupPage() {
  const [url, setUrl] = useState("");
  const [error, setError] = useState("");
  const [testing, setTesting] = useState(false);

  const handleConnect = async () => {
    setError("");

    // Validate URL format
    let parsedUrl: string = url.trim();
    if (!parsedUrl) {
      setError("Please enter a server URL");
      return;
    }

    // Add https:// if no protocol specified
    if (!/^https?:\/\//i.test(parsedUrl)) {
      parsedUrl = `https://${parsedUrl}`;
    }

    try {
      new URL(parsedUrl);
    } catch {
      setError("Invalid URL format");
      return;
    }

    // Remove trailing slash
    parsedUrl = parsedUrl.replace(/\/+$/, "");

    // Test connectivity
    setTesting(true);
    try {
      const response = await fetch(`${parsedUrl}/health`);
      if (!response.ok) {
        throw new Error(`Server responded with ${response.status}`);
      }
      await setApiBaseUrl(parsedUrl);
      // Reload to reinitialize with new URL
      window.location.href = "/";
    } catch (err) {
      setError(
        err instanceof Error
          ? `Could not connect: ${err.message}`
          : "Could not connect to server. Check the URL and try again."
      );
    } finally {
      setTesting(false);
    }
  };

  return (
    <div className="flex items-center justify-center min-h-screen bg-background p-4">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <CardTitle className="text-2xl">Welcome to Recur</CardTitle>
          <CardDescription>
            Enter your server address to get started
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="server-url">Server URL</Label>
              <Input
                id="server-url"
                type="text"
                placeholder="https://myserver.example.com"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleConnect();
                }}
                disabled={testing}
              />
              {error && (
                <p className="text-sm text-destructive">{error}</p>
              )}
              <p className="text-xs text-muted-foreground">
                Enter the URL of your Recur server (e.g., https://recur.example.com)
              </p>
            </div>
            <Button
              className="w-full"
              onClick={handleConnect}
              disabled={testing || !url.trim()}
            >
              {testing ? "Connecting..." : "Connect"}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}