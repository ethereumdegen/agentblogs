import { createBrowserRouter, Navigate } from "react-router-dom";
import { DashboardLayout } from "@/layouts/dashboard-layout";
import { ProjectLayout } from "@/layouts/project-layout";
import { AdminLayout } from "@/layouts/admin-layout";
import { LandingPage } from "@/pages/landing";
import { SignInPage } from "@/pages/sign-in";
import { ProjectsPage } from "@/pages/projects";
import { DashboardHome } from "@/pages/dashboard";
import { BlogPostsPage } from "@/pages/blog-posts";
import { BlogEditorPage } from "@/pages/blog-editor";
import { ApiKeysPage } from "@/pages/api-keys";
import { ApiDocsPage } from "@/pages/api-docs";
import { SettingsPage } from "@/pages/settings";
import { PlatformSettingsPage } from "@/pages/admin/platform-settings";
import { AdminDashboard } from "@/pages/admin/dashboard";
import { AdminUsersPage } from "@/pages/admin/users";
import { ErrorBoundary } from "@/components/error-boundary";

export const router = createBrowserRouter([
  {
    path: "/",
    element: <LandingPage />,
    errorElement: <ErrorBoundary />,
  },
  {
    path: "/sign-in",
    element: <SignInPage />,
  },
  {
    element: <DashboardLayout />,
    errorElement: <ErrorBoundary />,
    children: [
      { path: "/projects", element: <ProjectsPage /> },
      { path: "/dashboard", element: <Navigate to="/projects" replace /> },
      { path: "/settings", element: <SettingsPage /> },
      {
        path: "/projects/:projectId",
        element: <ProjectLayout />,
        children: [
          { path: "dashboard", element: <DashboardHome /> },
          { path: "posts", element: <BlogPostsPage /> },
          { path: "posts/new", element: <BlogEditorPage /> },
          { path: "posts/:slug", element: <BlogEditorPage /> },
          { path: "api-keys", element: <ApiKeysPage /> },
          { path: "api-docs", element: <ApiDocsPage /> },
        ],
      },
      {
        path: "/admin",
        element: <AdminLayout />,
        children: [
          { index: true, element: <AdminDashboard /> },
          { path: "users", element: <AdminUsersPage /> },
          { path: "platforms", element: <PlatformSettingsPage /> },
        ],
      },
    ],
  },
]);
