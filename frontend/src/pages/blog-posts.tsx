import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { api } from "@/lib/api";
import { useProject } from "@/contexts/project-context";
import type { BlogPost } from "@/lib/api";

export function BlogPostsPage() {
  const { project } = useProject();
  const navigate = useNavigate();
  const [posts, setPosts] = useState<BlogPost[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState("");
  const [statusFilter, setStatusFilter] = useState<string>("all");

  useEffect(() => {
    if (!project) return;
    setLoading(true);
    api.project(project.id).posts.list()
      .then(setPosts)
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [project?.id]);

  const filtered = posts.filter((p) => {
    if (statusFilter !== "all" && p.status !== statusFilter) return false;
    if (search) {
      const q = search.toLowerCase();
      return (
        p.title.toLowerCase().includes(q) ||
        p.slug.toLowerCase().includes(q) ||
        p.description.toLowerCase().includes(q)
      );
    }
    return true;
  });

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
          <h1 className="text-xl font-bold text-text-primary uppercase tracking-wide">Blog Posts</h1>
          <p className="text-sm text-text-secondary mt-1">
            Create and manage your blog content.
          </p>
        </div>
        <Button onClick={() => navigate("new")}>New Post</Button>
      </div>

      <div className="flex items-center gap-3">
        <div className="flex-1">
          <Input
            placeholder="Search posts..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>
        <select
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value)}
          className="bg-surface-2 border border-border rounded px-3 py-2 text-sm text-text-primary"
        >
          <option value="all">All</option>
          <option value="draft">Drafts</option>
          <option value="published">Published</option>
        </select>
      </div>

      <Card>
        {filtered.length === 0 ? (
          <div className="text-center py-12">
            <p className="text-sm text-text-secondary mb-4">
              {posts.length === 0
                ? "No blog posts yet. Create your first post!"
                : "No posts match your filters."}
            </p>
            {posts.length === 0 && (
              <Button onClick={() => navigate("new")}>Create Post</Button>
            )}
          </div>
        ) : (
          <div className="divide-y divide-border">
            {filtered.map((post) => (
              <div
                key={post.id}
                className="flex items-center justify-between px-5 py-4 cursor-pointer hover:bg-surface-2 transition-colors"
                onClick={() => navigate(post.slug)}
              >
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium text-text-primary">
                      {post.title || post.slug}
                    </span>
                    <Badge
                      variant={post.status === "published" ? "success" : "default"}
                    >
                      {post.status}
                    </Badge>
                  </div>
                  <div className="flex items-center gap-4 mt-1 text-xs text-text-tertiary">
                    <span className="font-[JetBrains_Mono] text-accent">/{post.slug}</span>
                    {post.author && <span>by {post.author}</span>}
                    {post.date && <span>{post.date}</span>}
                  </div>
                  {post.tags.length > 0 && (
                    <div className="flex items-center gap-1 mt-1.5">
                      {post.tags.map((tag) => (
                        <span
                          key={tag}
                          className="text-[10px] px-1.5 py-0.5 rounded bg-surface-2 text-text-tertiary"
                        >
                          {tag}
                        </span>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </Card>
    </div>
  );
}
