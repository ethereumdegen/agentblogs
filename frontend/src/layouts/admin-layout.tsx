import { Link, Navigate, Outlet, useLocation } from "react-router-dom";
import { useEffect, useState } from "react";
import { api } from "@/lib/api";

const adminTabs = [
  { label: "Dashboard", path: "/admin" },
  { label: "Users", path: "/admin/users" },
  { label: "Platforms", path: "/admin/platforms" },
];

export function AdminLayout() {
  const location = useLocation();
  const [status, setStatus] = useState<"loading" | "admin" | "denied">("loading");

  useEffect(() => {
    api.admin
      .status()
      .then(() => setStatus("admin"))
      .catch(() => setStatus("denied"));
  }, []);

  if (status === "loading") {
    return (
      <div className="flex items-center justify-center py-20 text-text-secondary">
        Loading...
      </div>
    );
  }

  if (status === "denied") {
    return <Navigate to="/projects" replace />;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <h1 className="text-xl font-bold text-text-primary uppercase tracking-wide">Admin</h1>
          <Link
            to="/projects"
            className="text-sm text-text-secondary hover:text-text-primary"
          >
            &larr; Back to portal
          </Link>
        </div>
      </div>

      <nav className="flex gap-1 border-b border-border">
        {adminTabs.map((tab) => {
          const active =
            tab.path === "/admin"
              ? location.pathname === "/admin"
              : location.pathname.startsWith(tab.path);
          return (
            <Link
              key={tab.path}
              to={tab.path}
              className={`px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors ${
                active
                  ? "border-accent text-accent"
                  : "border-transparent text-text-secondary hover:text-text-primary hover:border-border"
              }`}
            >
              {tab.label}
            </Link>
          );
        })}
      </nav>

      <Outlet />
    </div>
  );
}
