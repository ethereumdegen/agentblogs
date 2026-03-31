import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { api } from "@/lib/api";
import { useProject } from "@/contexts/project-context";

export function BlogEditorPage() {
  const { project } = useProject();
  const { slug: existingSlug } = useParams<{ slug: string }>();
  const navigate = useNavigate();
  const isNew = !existingSlug || existingSlug === "new";

  const [slug, setSlug] = useState("");
  const [title, setTitle] = useState("");
  const [date, setDate] = useState("");
  const [description, setDescription] = useState("");
  const [author, setAuthor] = useState("");
  const [cover, setCover] = useState("");
  const [markdown, setMarkdown] = useState("");
  const [status, setStatus] = useState("draft");
  const [tagsInput, setTagsInput] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(!isNew);
  const [imagePrompt, setImagePrompt] = useState("");
  const [generatingImage, setGeneratingImage] = useState(false);

  useEffect(() => {
    if (isNew || !project) return;
    setLoading(true);
    api.project(project.id).posts.get(existingSlug!)
      .then((post) => {
        setSlug(post.slug);
        setTitle(post.title);
        setDate(post.date);
        setDescription(post.description);
        setAuthor(post.author);
        setCover(post.cover);
        setMarkdown(post.markdown);
        setStatus(post.status);
        setTagsInput(post.tags.join(", "));
      })
      .catch((e) => setError(e?.message ?? "Failed to load post"))
      .finally(() => setLoading(false));
  }, [existingSlug, project?.id]);

  const handleSave = async () => {
    if (!project || !slug.trim()) return;
    setSaving(true);
    setError(null);
    try {
      const tags = tagsInput
        .split(",")
        .map((t) => t.trim())
        .filter(Boolean);
      await api.project(project.id).posts.upsert({
        slug: slug.trim(),
        title,
        date,
        description,
        author,
        cover,
        markdown,
        status,
        tags,
      });
      navigate(`/projects/${project.id}/posts`);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "Failed to save post");
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async () => {
    if (!project || isNew) return;
    if (!confirm("Delete this post?")) return;
    try {
      await api.project(project.id).posts.delete(existingSlug!);
      navigate(`/projects/${project.id}/posts`);
    } catch {
      // ignore
    }
  };

  const handleGenerateImage = async () => {
    if (!project || !imagePrompt.trim() || isNew) return;
    setGeneratingImage(true);
    setError(null);
    try {
      const result = await api.project(project.id).posts.generateImage(existingSlug!, imagePrompt);
      setCover(result.url);
      setImagePrompt("");
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : "Image generation failed");
    } finally {
      setGeneratingImage(false);
    }
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
          <h1 className="text-xl font-bold text-text-primary uppercase tracking-wide">
            {isNew ? "New Post" : "Edit Post"}
          </h1>
        </div>
        <div className="flex items-center gap-2">
          {!isNew && (
            <Button variant="danger" size="sm" onClick={handleDelete}>
              Delete
            </Button>
          )}
          <Button onClick={handleSave} disabled={saving || !slug.trim()}>
            {saving ? "Saving..." : "Save"}
          </Button>
        </div>
      </div>

      {error && (
        <Card className="border-danger/50 bg-danger-dim">
          <CardContent>
            <p className="text-sm text-danger">{error}</p>
          </CardContent>
        </Card>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Main content */}
        <div className="lg:col-span-2 space-y-4">
          <Input
            label="Title"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="My awesome blog post"
          />
          <Input
            label="Slug"
            value={slug}
            onChange={(e) => setSlug(e.target.value)}
            placeholder="my-awesome-post"
            disabled={!isNew}
          />
          <div>
            <label className="block text-sm font-medium text-text-primary mb-1.5">
              Content (Markdown)
            </label>
            <textarea
              value={markdown}
              onChange={(e) => setMarkdown(e.target.value)}
              placeholder="Write your blog post in markdown..."
              rows={20}
              className="w-full bg-surface-2 border border-border rounded px-3 py-2 text-sm text-text-primary placeholder-text-tertiary focus:outline-none focus:ring-2 focus:ring-accent/30 font-[JetBrains_Mono] resize-y"
            />
          </div>
        </div>

        {/* Sidebar */}
        <div className="space-y-4">
          <Card>
            <CardContent className="space-y-4">
              <h3 className="text-sm font-bold text-text-primary uppercase tracking-wide">Metadata</h3>
              <div>
                <label className="block text-sm font-medium text-text-primary mb-1.5">Status</label>
                <select
                  value={status}
                  onChange={(e) => setStatus(e.target.value)}
                  className="w-full bg-surface-2 border border-border rounded px-3 py-2 text-sm text-text-primary"
                >
                  <option value="draft">Draft</option>
                  <option value="published">Published</option>
                </select>
              </div>
              <Input
                label="Date"
                value={date}
                onChange={(e) => setDate(e.target.value)}
                placeholder="2025-01-15"
              />
              <Input
                label="Author"
                value={author}
                onChange={(e) => setAuthor(e.target.value)}
                placeholder="John Doe"
              />
              <Input
                label="Description"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="A short description..."
              />
              <Input
                label="Tags (comma-separated)"
                value={tagsInput}
                onChange={(e) => setTagsInput(e.target.value)}
                placeholder="ai, rust, blog"
              />
              {tagsInput && (
                <div className="flex flex-wrap gap-1">
                  {tagsInput.split(",").map((t) => t.trim()).filter(Boolean).map((tag) => (
                    <Badge key={tag} variant="info">{tag}</Badge>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardContent className="space-y-4">
              <h3 className="text-sm font-bold text-text-primary uppercase tracking-wide">Cover Image</h3>
              <Input
                label="Cover URL"
                value={cover}
                onChange={(e) => setCover(e.target.value)}
                placeholder="https://..."
              />
              {cover && (
                <img
                  src={cover}
                  alt="Cover preview"
                  className="w-full rounded border border-border"
                />
              )}
              {!isNew && (
                <div className="space-y-2">
                  <Input
                    label="AI Image Prompt"
                    value={imagePrompt}
                    onChange={(e) => setImagePrompt(e.target.value)}
                    placeholder="A futuristic blog header..."
                  />
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={handleGenerateImage}
                    disabled={generatingImage || !imagePrompt.trim()}
                    className="w-full"
                  >
                    {generatingImage ? "Generating..." : "Generate Cover Image"}
                  </Button>
                  <p className="text-xs text-text-tertiary">Pro plan required</p>
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}
