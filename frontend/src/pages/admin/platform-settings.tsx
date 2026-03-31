import { useEffect, useState } from "react";
import { Card, CardHeader, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { api } from "@/lib/api";
import type { PlatformSetting } from "@/lib/api";

const PLATFORMS = [
  {
    id: "openai",
    label: "OpenAI",
    keys: [{ name: "api_key", label: "API Key", secret: true }],
  },
  {
    id: "fal",
    label: "fal.ai",
    keys: [{ name: "api_key", label: "API Key", secret: true }],
  },
];

export function PlatformSettingsPage() {
  const [settings, setSettings] = useState<Record<string, PlatformSetting[]>>({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = async () => {
    try {
      const result: Record<string, PlatformSetting[]> = {};
      for (const platform of PLATFORMS) {
        try {
          const data = await api.admin.platforms.get(platform.id);
          result[platform.id] = data;
        } catch {
          result[platform.id] = [];
        }
      }
      setSettings(result);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "Failed to load");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
  }, []);

  if (loading) {
    return <p className="text-text-tertiary">Loading platform settings...</p>;
  }

  if (error) {
    return <p className="text-danger">{error}</p>;
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-bold text-text-primary uppercase tracking-wide">
          Platform Settings
        </h1>
        <p className="text-sm text-text-secondary mt-1">
          Configure API keys for AI image generation. These are stored securely in the database.
        </p>
      </div>

      <div className="grid gap-6">
        {PLATFORMS.map((platform) => (
          <PlatformCard
            key={platform.id}
            platform={platform}
            existing={settings[platform.id] || []}
            onSaved={load}
          />
        ))}
      </div>
    </div>
  );
}

function PlatformCard({
  platform,
  existing,
  onSaved,
}: {
  platform: (typeof PLATFORMS)[number];
  existing: PlatformSetting[];
  onSaved: () => void;
}) {
  const [values, setValues] = useState<Record<string, string>>({});
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  const isConfigured = platform.keys.every((k) =>
    existing.some((e) => e.key_name === k.name)
  );

  const handleSave = async () => {
    setSaving(true);
    setMessage(null);
    try {
      for (const [keyName, value] of Object.entries(values)) {
        if (!value.trim()) continue;
        await api.admin.platforms.set(platform.id, keyName, value.trim());
      }
      setValues({});
      setMessage("Saved");
      onSaved();
    } catch (e: unknown) {
      setMessage(e instanceof Error ? e.message : "Save failed");
    } finally {
      setSaving(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <h2 className="text-base font-semibold text-text-primary">{platform.label}</h2>
          <Badge variant={isConfigured ? "success" : "warning"}>
            {isConfigured ? "Configured" : "Not configured"}
          </Badge>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          {platform.keys.map((key) => {
            const existingKey = existing.find((e) => e.key_name === key.name);
            return (
              <div key={key.name} className="flex items-end gap-3">
                <div className="flex-1">
                  <Input
                    label={key.label}
                    type={key.secret ? "password" : "text"}
                    placeholder={
                      existingKey
                        ? "(already set \u2014 leave blank to keep)"
                        : "Enter value..."
                    }
                    value={values[key.name] || ""}
                    onChange={(e) =>
                      setValues((prev) => ({ ...prev, [key.name]: e.target.value }))
                    }
                  />
                </div>
                {existingKey && (
                  <Badge variant="success" className="mb-1">Set</Badge>
                )}
              </div>
            );
          })}

          <div className="flex items-center gap-3 pt-2">
            <Button size="sm" onClick={handleSave} disabled={saving}>
              {saving ? "Saving..." : "Save"}
            </Button>
            {message && (
              <span className="text-sm text-text-secondary">{message}</span>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
