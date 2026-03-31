import { useRouteError, isRouteErrorResponse, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";

export function ErrorBoundary() {
  const error = useRouteError();
  const navigate = useNavigate();

  let status = 500;
  let title = "Something went wrong";
  let message = "An unexpected error occurred. Please try again.";

  if (isRouteErrorResponse(error)) {
    status = error.status;
    if (status === 404) {
      title = "Page not found";
      message = "The page you're looking for doesn't exist or has been moved.";
    } else if (status === 403) {
      title = "Access denied";
      message = "You don't have permission to view this page.";
    } else {
      message = error.statusText || message;
    }
  } else if (error instanceof Error) {
    message = error.message;
  }

  return (
    <div className="min-h-screen bg-surface-0 flex items-center justify-center p-6">
      <div className="max-w-md w-full text-center space-y-6">
        <div className="text-6xl font-bold text-text-tertiary">{status}</div>
        <h1 className="text-xl font-semibold text-text-primary">{title}</h1>
        <p className="text-sm text-text-secondary">{message}</p>
        <div className="flex items-center justify-center gap-3">
          <Button variant="secondary" size="sm" onClick={() => navigate(-1)}>
            Go Back
          </Button>
          <Button size="sm" onClick={() => navigate("/projects")}>
            Go to Projects
          </Button>
        </div>
      </div>
    </div>
  );
}
