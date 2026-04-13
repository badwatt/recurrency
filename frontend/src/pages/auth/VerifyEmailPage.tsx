import { useState } from "react";
import { Link } from "react-router-dom";
import { toast } from "sonner";
import { ArrowLeft, Mail } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { authApi } from "@/lib/api/auth";

interface VerifyEmailPageProps {
  email: string;
  token: string;
  onVerified: () => void;
}

export function VerifyEmailPage({ email, token, onVerified }: VerifyEmailPageProps) {
  const [code, setCode] = useState("");
  const [loading, setLoading] = useState(false);
  const [resending, setResending] = useState(false);

  const handleVerify = async (e: React.FormEvent) => {
    e.preventDefault();
    if (code.length !== 6) {
      toast.error("Please enter a 6-digit code");
      return;
    }
    setLoading(true);
    try {
      await authApi.verifyEmail(token, code);
      toast.success("Email verified! You can now log in.");
      onVerified();
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : "Invalid code";
      toast.error(message);
    } finally {
      setLoading(false);
    }
  };

  const handleResend = async () => {
    setResending(true);
    try {
      await authApi.resendVerification(token);
      toast.success("New code sent!");
    } catch {
      toast.error("Failed to resend code");
    } finally {
      setResending(false);
    }
  };

  return (
    <div className="min-h-dvh flex items-center justify-center bg-gradient-to-br from-background to-muted p-4">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
            <Mail className="h-6 w-6 text-primary" />
          </div>
          <CardTitle className="text-2xl">Check your email</CardTitle>
          <p className="text-sm text-muted-foreground mt-2">
            We sent a 6-digit code to<br />
            <span className="font-medium text-foreground">{email}</span>
          </p>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleVerify} className="space-y-4">
            <Input
              type="text"
              placeholder="Enter 6-digit code"
              value={code}
              onChange={(e) => setCode(e.target.value.replace(/\D/g, "").slice(0, 6))}
              className="text-center text-2xl tracking-widest font-mono"
              maxLength={6}
              autoFocus
            />
            <Button type="submit" className="w-full" disabled={loading || code.length !== 6}>
              {loading ? "Verifying..." : "Verify Email"}
            </Button>
          </form>

          <div className="mt-4 flex items-center justify-between text-sm">
            <Link to="/login" className="flex items-center text-muted-foreground hover:text-foreground">
              <ArrowLeft className="mr-1 h-4 w-4" />
              Back to login
            </Link>
            <Button variant="ghost" size="sm" onClick={handleResend} disabled={resending}>
              {resending ? "Sending..." : "Resend code"}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
