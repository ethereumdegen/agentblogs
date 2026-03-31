import { useEffect, useState } from "react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Modal } from "@/components/ui/modal";
import { api } from "@/lib/api";
import { useProject } from "@/contexts/project-context";
import type { ApiKey, CreateApiKeyResponse } from "@/lib/api";

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

export function ApiKeysPage() {
  const { project } = useProject();
  const [keys, setKeys] = useState<ApiKey[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [createOpen, setCreateOpen] = useState(false);
  const [name, setName] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [newKey, setNewKey] = useState<string | null>(null);
  const [showRevoked, setShowRevoked] = useState(false);
  const [copied, setCopied] = useState(false);

  const p = project ? api.project(project.id) : null;

  const loadKeys = () => {
    if (!p) return;
    setLoading(true);
    setError(null);
    p.apiKeys.list()
      .then(setKeys)
      .catch((err) => setError(err?.message ?? "Failed to load API keys"))
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    loadKeys();
  }, [project?.id]);

  const activeKeys = keys.filter((k) => !k.revoked);
  const revokedKeys = keys.filter((k) => k.revoked);

  const handleCreate = async () => {
    if (!name.trim() || !p) return;
    setSubmitting(true);
    try {
      const result: CreateApiKeyResponse = await p.apiKeys.create(name.trim());
      setKeys((prev) => [result as ApiKey, ...prev]);
      setNewKey(result.secret ?? null);
      setName("");
      if (!result.secret) setCreateOpen(false);
    } catch {
      // noop
    } finally {
      setSubmitting(false);
    }
  };

  const handleRevoke = async (id: string) => {
    if (!p) return;
    try {
      await p.apiKeys.revoke(id);
      loadKeys();
    } catch {
      // noop
    }
  };

  const handleCopy = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // noop
    }
  };

  const handleCloseCreate = () => {
    setCreateOpen(false);
    setNewKey(null);
    setName("");
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-20">
        <div className="text-sm text-text-tertiary">Loading...</div>
      </div>
    );
  }

  return (
    <div className="space-y-8">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold text-text-primary uppercase tracking-wide">API Keys</h1>
          <p className="text-sm text-text-secondary mt-1">
            Manage API keys to access AgentBlogs programmatically.
          </p>
        </div>
        <Button size="sm" onClick={() => setCreateOpen(true)}>
          Create API Key
        </Button>
      </div>

      {error && (
        <Card className="border-danger/50 bg-danger-dim">
          <CardContent>
            <div className="flex items-center justify-between">
              <p className="text-sm text-danger">{error}</p>
              <Button variant="secondary" size="sm" onClick={loadKeys}>Retry</Button>
            </div>
          </CardContent>
        </Card>
      )}

      <Card>
        {activeKeys.length === 0 ? (
          <div className="text-center py-12">
            <p className="text-sm font-medium text-text-primary">No API keys</p>
            <p className="text-sm text-text-secondary mt-1">
              Create an API key to start using the AgentBlogs API.
            </p>
          </div>
        ) : (
          <div className="divide-y divide-border">
            {activeKeys.map((key) => (
              <div key={key.id} className="flex items-center justify-between px-5 py-4">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium text-text-primary">{key.name}</span>
                    <Badge variant="success">Active</Badge>
                  </div>
                  <div className="flex items-center gap-4 mt-1 text-xs text-text-tertiary">
                    <span className="font-[JetBrains_Mono] text-accent">{key.key_prefix}...</span>
                    <span>Created {formatDate(key.created_at)}</span>
                    <span>
                      {key.last_used_at
                        ? `Last used ${formatDate(key.last_used_at)}`
                        : "Never used"}
                    </span>
                  </div>
                </div>
                <Button variant="danger" size="sm" onClick={() => handleRevoke(key.id)}>
                  Revoke
                </Button>
              </div>
            ))}
          </div>
        )}
      </Card>

      {revokedKeys.length > 0 && (
        <div>
          <button
            onClick={() => setShowRevoked(!showRevoked)}
            className="text-sm text-text-secondary hover:text-text-primary transition-colors flex items-center gap-1 cursor-pointer"
          >
            <svg
              className={`w-4 h-4 transition-transform ${showRevoked ? "rotate-90" : ""}`}
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              strokeWidth={2}
            >
              <path strokeLinecap="round" strokeLinejoin="round" d="M9 5l7 7-7 7" />
            </svg>
            Revoked keys ({revokedKeys.length})
          </button>
          {showRevoked && (
            <Card className="mt-3">
              <div className="divide-y divide-border">
                {revokedKeys.map((key) => (
                  <div key={key.id} className="px-5 py-3 opacity-60">
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-medium text-text-primary">{key.name}</span>
                      <Badge variant="error">Revoked</Badge>
                    </div>
                    <div className="flex items-center gap-4 mt-1 text-xs text-text-tertiary">
                      <span className="font-[JetBrains_Mono]">{key.key_prefix}...</span>
                    </div>
                  </div>
                ))}
              </div>
            </Card>
          )}
        </div>
      )}

      <Modal open={createOpen} onClose={handleCloseCreate} title={newKey ? "API Key Created" : "Create API Key"}>
        {newKey ? (
          <div className="space-y-4">
            <p className="text-sm text-text-secondary">
              Copy your API key now. You will not be able to see it again.
            </p>
            <div className="flex items-center gap-2">
              <code className="flex-1 bg-surface-2 rounded px-3 py-2 text-sm text-accent font-[JetBrains_Mono] break-all">
                {newKey}
              </code>
              <Button variant="secondary" size="sm" onClick={() => handleCopy(newKey)}>
                {copied ? "Copied!" : "Copy"}
              </Button>
            </div>
            <div className="flex justify-end">
              <Button size="sm" onClick={handleCloseCreate}>Done</Button>
            </div>
          </div>
        ) : (
          <div className="space-y-4">
            <Input
              label="Key Name"
              placeholder="e.g., Production, Development"
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
            <div className="flex justify-end gap-2">
              <Button variant="secondary" size="sm" onClick={handleCloseCreate}>Cancel</Button>
              <Button size="sm" onClick={handleCreate} disabled={!name.trim() || submitting}>
                {submitting ? "Creating..." : "Create"}
              </Button>
            </div>
          </div>
        )}
      </Modal>
    </div>
  );
}
