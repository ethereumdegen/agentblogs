import { useEffect, useState } from "react";
import { Card, CardContent } from "@/components/ui/card";
import { api } from "@/lib/api";
import type { AdminStats } from "@/lib/api";

export function AdminDashboard() {
  const [stats, setStats] = useState<AdminStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api.admin
      .stats()
      .then(setStats)
      .catch((e: Error) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return <p className="text-sm text-text-tertiary">Loading stats...</p>;
  }

  if (error) {
    return <p className="text-sm text-danger">{error}</p>;
  }

  if (!stats) return null;

  const cards = [
    { label: "Total Users", value: stats.total_users },
    { label: "Total Projects", value: stats.total_projects },
    { label: "Total Posts", value: stats.total_posts },
    { label: "Published Posts", value: stats.published_posts },
  ];

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
      {cards.map((c) => (
        <Card key={c.label}>
          <CardContent>
            <p className="text-sm text-text-secondary uppercase tracking-wide">{c.label}</p>
            <p className="text-2xl font-bold text-accent font-[JetBrains_Mono] mt-1">
              {c.value.toLocaleString()}
            </p>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}
