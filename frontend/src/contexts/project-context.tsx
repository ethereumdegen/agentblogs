import { createContext, useContext, useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { api } from "@/lib/api";
import type { Project } from "@/lib/api";

interface ProjectContextValue {
  project: Project | null;
  projects: Project[];
  loading: boolean;
  reload: () => void;
}

const ProjectContext = createContext<ProjectContextValue>({
  project: null,
  projects: [],
  loading: true,
  reload: () => {},
});

export function useProject() {
  return useContext(ProjectContext);
}

export function ProjectProvider({ children }: { children: React.ReactNode }) {
  const { projectId } = useParams<{ projectId: string }>();
  const navigate = useNavigate();
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);

  const load = () => {
    setLoading(true);
    api.projects.list().then((list) => {
      setProjects(list);
      setLoading(false);

      // If no projectId in URL but projects exist, redirect to first
      if (!projectId && list.length > 0) {
        navigate(`/projects/${list[0].id}/dashboard`, { replace: true });
      }
      // If projectId doesn't match any project, redirect to first
      if (projectId && list.length > 0 && !list.find((p) => p.id === projectId)) {
        navigate(`/projects/${list[0].id}/dashboard`, { replace: true });
      }
    }).catch(() => {
      setLoading(false);
    });
  };

  useEffect(() => {
    load();
  }, []);

  const project = projects.find((p) => p.id === projectId) ?? null;

  return (
    <ProjectContext.Provider value={{ project, projects, loading, reload: load }}>
      {children}
    </ProjectContext.Provider>
  );
}
