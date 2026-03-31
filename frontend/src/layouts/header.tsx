import { Link, useParams } from "react-router-dom";
import { useEffect, useState } from "react";
import { signOut } from "@/lib/auth-client";
import { api } from "@/lib/api";
import type { Project } from "@/lib/api";

export function Header() {
  const { projectId } = useParams<{ projectId: string }>();
  const [projects, setProjects] = useState<Project[]>([]);

  useEffect(() => {
    api.projects.list().then(setProjects).catch(() => {});
  }, []);

  const current = projects.find((p) => p.id === projectId);

  return (
    <header className="h-14 border-b border-border bg-surface-1 flex items-center justify-between px-6 shrink-0">
      <div className="flex items-center gap-4">
        <Link to="/projects" className="flex items-center gap-2.5">
          <div className="h-8 w-8 rounded bg-accent flex items-center justify-center text-surface-0 font-bold text-sm">AB</div>
          <span className="text-sm font-semibold text-text-primary">AgentBlogs</span>
        </Link>
        {current && (
          <>
            <span className="text-text-tertiary">/</span>
            <span className="text-sm text-text-secondary">{current.name}</span>
          </>
        )}
      </div>
      <button
        onClick={() => signOut()}
        className="text-sm text-text-secondary hover:text-text-primary transition-colors"
      >
        Sign out
      </button>
    </header>
  );
}
