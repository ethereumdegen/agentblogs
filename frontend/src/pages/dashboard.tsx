import { useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { api } from "@/lib/api";
import { useProject } from "@/contexts/project-context";
import type { BlogPost } from "@/lib/api";

export function DashboardHome() {
  const { project, reload } = useProject();
  const navigate = useNavigate();
  const [posts, setPosts] = useState<BlogPost[]>([]);
  const [loading, setLoading] = useState(true);
  const [isRenaming, setIsRenaming] = useState(false);
  const [renameValue, setRenameValue] = useState("");
  const renameInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!project) return;
    setLoading(true);
    api.project(project.id).posts.list()
      .then(setPosts)
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [project?.id]);

  if (!project || loading) {
    return (
      <div className="flex items-center justify-center py-20">
        <div className="text-sm text-text-tertiary">Loading...</div>
      </div>
    );
  }

  const published = posts.filter((p) => p.status === "published").length;
  const drafts = posts.filter((p) => p.status === "draft").length;

  return (
    <div className="space-y-8">
      <div>
        {isRenaming ? (
          <input
            ref={renameInputRef}
            value={renameValue}
            onChange={(e) => setRenameValue(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.currentTarget.blur();
              } else if (e.key === "Escape") {
                setIsRenaming(false);
              }
            }}
            onBlur={async () => {
              const trimmed = renameValue.trim();
              if (trimmed && trimmed !== project.name) {
                await api.projects.update(project.id, { name: trimmed });
                reload();
              }
              setIsRenaming(false);
            }}
            className="text-xl font-bold text-text-primary uppercase tracking-wide bg-transparent border-b-2 border-accent outline-none w-full"
          />
        ) : (
          <h1
            className="text-xl font-bold text-text-primary uppercase tracking-wide cursor-pointer"
            onDoubleClick={() => {
              setRenameValue(project.name);
              setIsRenaming(true);
              setTimeout(() => renameInputRef.current?.select(), 0);
            }}
          >
            {project.name}
          </h1>
        )}
        <p className="text-sm text-text-secondary mt-1">
          Overview of your blog.
        </p>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card>
          <CardContent>
            <p className="text-sm text-text-secondary uppercase tracking-wide">Total Posts</p>
            <p className="text-2xl font-bold text-accent font-[JetBrains_Mono] mt-1">
              {posts.length}
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardContent>
            <p className="text-sm text-text-secondary uppercase tracking-wide">Published</p>
            <p className="text-2xl font-bold text-accent font-[JetBrains_Mono] mt-1">
              {published}
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardContent>
            <p className="text-sm text-text-secondary uppercase tracking-wide">Drafts</p>
            <p className="text-2xl font-bold text-accent font-[JetBrains_Mono] mt-1">
              {drafts}
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Quick Actions */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <Button
          variant="secondary"
          className="h-auto py-4"
          onClick={() => navigate("../posts/new")}
        >
          New Blog Post
        </Button>
        <Button
          variant="secondary"
          className="h-auto py-4"
          onClick={() => navigate("../posts")}
        >
          View All Posts
        </Button>
      </div>

      {/* Recent Posts */}
      <div>
        <h2 className="text-sm font-bold text-text-primary uppercase tracking-wide mb-3">
          Recent Posts
        </h2>
        <Card>
          {posts.length === 0 ? (
            <CardContent>
              <p className="text-sm text-text-tertiary text-center py-6">
                No blog posts yet. Create your first post!
              </p>
            </CardContent>
          ) : (
            <div className="divide-y divide-border">
              {posts.slice(0, 5).map((post) => (
                <div
                  key={post.id}
                  className="flex items-center gap-3 px-5 py-3 cursor-pointer hover:bg-surface-2 transition-colors"
                  onClick={() => navigate(`../posts/${post.slug}`)}
                >
                  <div className="flex-1 min-w-0">
                    <span className="text-sm font-medium text-text-primary">
                      {post.title || post.slug}
                    </span>
                    {post.description && (
                      <p className="text-xs text-text-tertiary mt-0.5 truncate">
                        {post.description}
                      </p>
                    )}
                  </div>
                  <Badge
                    variant={post.status === "published" ? "success" : "default"}
                  >
                    {post.status}
                  </Badge>
                </div>
              ))}
            </div>
          )}
        </Card>
      </div>
    </div>
  );
}
