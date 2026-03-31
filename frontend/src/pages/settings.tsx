import { useSession } from "@/lib/auth-client";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

export function SettingsPage() {
  const { data: session } = useSession();

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-xl font-bold text-text-primary uppercase tracking-wide">Settings</h1>
        <p className="text-sm text-text-secondary mt-1">
          Manage your account.
        </p>
      </div>

      <Card>
        <CardContent>
          <h2 className="text-sm font-bold text-text-primary uppercase tracking-wide mb-4">Account</h2>
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-sm text-text-secondary">Name</span>
              <span className="text-sm text-text-primary">
                {session?.user?.name || "---"}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-text-secondary">Email</span>
              <span className="text-sm text-text-primary">
                {session?.user?.email || "---"}
              </span>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardContent>
          <h2 className="text-sm font-bold text-text-primary uppercase tracking-wide mb-4">Plan</h2>
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium text-text-primary">Free Plan</span>
            <Badge variant="default">Current</Badge>
          </div>
          <p className="text-sm text-text-secondary mt-1">
            3 projects, unlimited posts, 1 API key per project.
          </p>
        </CardContent>
      </Card>

      <Card className="border-danger/30">
        <CardContent>
          <h2 className="text-sm font-bold text-danger uppercase tracking-wide mb-4">
            Danger Zone
          </h2>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-text-primary">Delete Account</p>
              <p className="text-sm text-text-secondary mt-0.5">
                Permanently delete your account and all associated data.
              </p>
            </div>
            <Button variant="danger" size="sm">Delete Account</Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
