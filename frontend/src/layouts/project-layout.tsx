import { Outlet } from "react-router-dom";
import { ProjectProvider } from "@/contexts/project-context";

export function ProjectLayout() {
  return (
    <ProjectProvider>
      <Outlet />
    </ProjectProvider>
  );
}
