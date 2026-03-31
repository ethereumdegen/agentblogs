import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { api } from "@/lib/api";
import type { Project } from "@/lib/api";

const adjectives = [
  "swift", "bright", "dark", "silent", "cosmic", "neon", "rapid", "frozen",
  "lunar", "solar", "iron", "copper", "golden", "silver", "crystal", "amber",
  "crimson", "violet", "azure", "phantom", "quantum", "stellar", "primal",
];

const nouns = [
  "blog", "journal", "chronicle", "digest", "gazette", "herald", "ledger",
  "archive", "beacon", "channel", "dispatch", "folio", "ink", "lens",
  "muse", "note", "page", "quill", "scroll", "tome", "voice", "wire",
];

function randomName(): string {
  const adj = adjectives[Math.floor(Math.random() * adjectives.length)];
  const noun = nouns[Math.floor(Math.random() * nouns.length)];
  return `${adj}-${noun}`;
}

export function ProjectsPage() {
  const navigate = useNavigate();
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");

  const load = () => {
    setLoading(true);
    api.projects.list().then((list) => {
      setProjects(list);
      setLoading(false);
    }).catch(() => setLoading(false));
  };

  useEffect(() => {
    load();
  }, []);

  const handleCreate = async () => {
    try {
      const proj = await api.projects.create(randomName());
      navigate(`/projects/${proj.id}/dashboard`);
    } catch {
      // ignore
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm("Delete this project and all its data?")) return;
    await api.projects.delete(id);
    load();
  };

  const startRename = (project: Project, e: React.MouseEvent) => {
    e.stopPropagation();
    setRenamingId(project.id);
    setRenameValue(project.name);
  };

  const handleRename = async (id: string) => {
    const trimmed = renameValue.trim();
    if (!trimmed) {
      setRenamingId(null);
      return;
    }
    try {
      await api.projects.update(id, { name: trimmed });
      setProjects((prev) =>
        prev.map((p) => (p.id === id ? { ...p, name: trimmed } : p))
      );
    } catch {
      // ignore
    }
    setRenamingId(null);
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-20">
        <div className="text-sm text-text-tertiary">Loading...</div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold text-text-primary uppercase tracking-wide">Projects</h1>
          <p className="text-sm text-text-secondary mt-1">
            Each project has its own blog posts, API keys, and settings.
          </p>
        </div>
        <Button onClick={handleCreate}>New Project</Button>
      </div>

      {projects.length === 0 ? (
        <Card>
          <CardContent>
            <div className="text-center py-12">
              <p className="text-sm text-text-secondary mb-4">
                No projects yet. Create one to get started.
              </p>
              <Button onClick={handleCreate}>Create Project</Button>
            </div>
          </CardContent>
        </Card>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {projects.map((project) => (
            <Card
              key={project.id}
              className="cursor-pointer hover:border-accent/50 transition-colors"
              onClick={() => navigate(`/projects/${project.id}/dashboard`)}
            >
              <CardContent>
                <div className="flex items-center justify-between">
                  <div className="flex-1 min-w-0">
                    {renamingId === project.id ? (
                      <input
                        type="text"
                        value={renameValue}
                        onChange={(e) => setRenameValue(e.target.value)}
                        onKeyDown={(e) => {
                          if (e.key === "Enter") handleRename(project.id);
                          if (e.key === "Escape") setRenamingId(null);
                        }}
                        onBlur={() => handleRename(project.id)}
                        onClick={(e) => e.stopPropagation()}
                        className="w-full px-2 py-1 -mx-2 -my-1 border border-accent/50 rounded bg-surface-2 text-text-primary text-sm focus:outline-none focus:ring-2 focus:ring-accent/30"
                        autoFocus
                      />
                    ) : (
                      <h3 className="text-sm font-semibold text-text-primary truncate">
                        {project.name}
                      </h3>
                    )}
                    <p className="text-xs text-text-tertiary mt-1">
                      Created {new Date(project.created_at).toLocaleDateString()}
                    </p>
                  </div>
                  <div className="flex items-center gap-2 shrink-0 ml-3">
                    <button
                      onClick={(e) => startRename(project, e)}
                      className="text-xs text-text-tertiary hover:text-text-primary transition-colors"
                    >
                      Rename
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDelete(project.id);
                      }}
                      className="text-xs text-text-tertiary hover:text-danger transition-colors"
                    >
                      Delete
                    </button>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
