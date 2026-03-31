import { Outlet, Navigate } from "react-router-dom";
import { useSession } from "@/lib/auth-client";
import { Header } from "./header";
import { Sidebar } from "./sidebar";

export function DashboardLayout() {
  const { data: session, isPending } = useSession();

  if (isPending) {
    return (
      <div className="h-screen flex items-center justify-center bg-surface-0">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-accent" />
      </div>
    );
  }

  if (!session) {
    return <Navigate to="/sign-in" replace />;
  }

  return (
    <div className="h-screen flex flex-col bg-surface-0">
      <Header />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar />
        <main className="flex-1 overflow-y-auto">
          <div className="max-w-5xl mx-auto px-8 py-8">
            <Outlet />
          </div>
        </main>
      </div>
    </div>
  );
}
