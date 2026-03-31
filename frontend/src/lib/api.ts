export interface Project {
  id: string;
  user_id: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface BlogPost {
  id: string;
  project_id: string;
  slug: string;
  title: string;
  date: string;
  description: string;
  author: string;
  cover: string;
  markdown: string;
  status: string;
  tags: string[];
  created_at: string;
  updated_at: string;
}

export interface BlogPostSummary {
  id: string;
  slug: string;
  title: string;
  date: string;
  description: string;
  author: string;
  cover: string;
  status: string;
  tags: string[];
  created_at: string;
  updated_at: string;
}

export interface ApiKey {
  id: string;
  name: string;
  key_prefix: string;
  key?: string;
  scopes: string[];
  created_at: string;
  last_used_at: string | null;
  expires_at: string | null;
  revoked: boolean;
}

export interface PlatformSetting {
  platform: string;
  key_name: string;
  key_value: string;
  is_secret: boolean;
}

export interface AdminStatus {
  is_admin: boolean;
  platforms: Record<string, boolean>;
}

export interface AdminStats {
  total_users: number;
  total_projects: number;
  total_posts: number;
  published_posts: number;
}

export interface AdminUser {
  id: string;
  futureauth_user_id: string;
  email: string;
  name: string | null;
  plan: string;
  is_admin: boolean;
  created_at: string;
  post_count: number;
  project_count: number;
}

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(options?.headers as Record<string, string>),
  };
  const res = await fetch(path, { ...options, headers, credentials: "include" });
  if (!res.ok) {
    const text = await res.text().catch(() => "Request failed");
    throw new Error(text);
  }
  return res.json();
}

export interface CreateApiKeyResponse extends ApiKey {
  secret: string;
}

function projectApi(projectId: string) {
  const base = `/api/internal/projects/${projectId}`;
  return {
    posts: {
      list: () => request<BlogPost[]>(`${base}/posts`),
      get: (slug: string) => request<BlogPost>(`${base}/posts/${slug}`),
      upsert: (post: {
        slug: string;
        title?: string;
        date?: string;
        description?: string;
        author?: string;
        cover?: string;
        markdown?: string;
        status?: string;
        tags?: string[];
      }) =>
        request<BlogPost>(`${base}/posts`, {
          method: "POST",
          body: JSON.stringify(post),
        }),
      delete: (slug: string) =>
        request<void>(`${base}/posts/${slug}`, { method: "DELETE" }),
      generateImage: (slug: string, prompt: string) =>
        request<{ url: string }>(`${base}/posts/${slug}/generate-image`, {
          method: "POST",
          body: JSON.stringify({ prompt }),
        }),
    },
    apiKeys: {
      list: () => request<ApiKey[]>(`${base}/api-keys`),
      create: (name: string) =>
        request<CreateApiKeyResponse>(`${base}/api-keys`, {
          method: "POST",
          body: JSON.stringify({ name }),
        }),
      revoke: (id: string) =>
        request<void>(`${base}/api-keys/${id}`, { method: "DELETE" }),
    },
  };
}

export const api = {
  project: projectApi,
  projects: {
    list: () => request<Project[]>("/api/internal/projects"),
    create: (name: string) =>
      request<Project>("/api/internal/projects", {
        method: "POST",
        body: JSON.stringify({ name }),
      }),
    update: (id: string, data: { name?: string }) =>
      request<Project>(`/api/internal/projects/${id}`, {
        method: "PATCH",
        body: JSON.stringify(data),
      }),
    delete: (id: string) =>
      request<void>(`/api/internal/projects/${id}`, { method: "DELETE" }),
  },
  admin: {
    status: () => request<AdminStatus>("/api/internal/admin/status"),
    stats: () => request<AdminStats>("/api/internal/admin/stats"),
    users: {
      list: () => request<AdminUser[]>("/api/internal/admin/users"),
      update: (id: string, data: { is_admin?: boolean; plan?: string }) =>
        request<AdminUser>(`/api/internal/admin/users/${id}`, {
          method: "PATCH",
          body: JSON.stringify(data),
        }),
    },
    platforms: {
      get: (platform: string) =>
        request<PlatformSetting[]>(`/api/internal/admin/platforms/${platform}`),
      set: (platform: string, keyName: string, keyValue: string) =>
        request<{ ok: boolean }>(`/api/internal/admin/platforms/${platform}`, {
          method: "PUT",
          body: JSON.stringify({ key_name: keyName, key_value: keyValue }),
        }),
      deleteSetting: (platform: string, key: string) =>
        request<void>(`/api/internal/admin/platforms/${platform}/${key}`, {
          method: "DELETE",
        }),
    },
  },
};
