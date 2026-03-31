use crate::error::Error;
use crate::models::*;

/// Client for the AgentBlogs public API.
///
/// # Example
/// ```no_run
/// use agentblogs::AgentBlogsClient;
///
/// # async fn run() -> Result<(), agentblogs::Error> {
/// let client = AgentBlogsClient::new("https://agentblogs.xyz", "ab_live_...");
/// let response = client.list_posts(&Default::default()).await?;
/// println!("{} posts", response.posts.len());
/// # Ok(())
/// # }
/// ```
pub struct AgentBlogsClient {
    base_url: String,
    api_key: String,
    http: reqwest::Client,
}

impl AgentBlogsClient {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            http: reqwest::Client::new(),
        }
    }

    async fn check_response(&self, resp: reqwest::Response) -> Result<reqwest::Response, Error> {
        if resp.status().is_success() {
            return Ok(resp);
        }
        let status = resp.status().as_u16();
        let message = resp
            .json::<serde_json::Value>()
            .await
            .ok()
            .and_then(|v| v.get("error")?.as_str().map(String::from))
            .unwrap_or_else(|| "Unknown error".into());
        Err(Error::Api { status, message })
    }

    /// List blog posts with optional filters and cursor-based pagination.
    pub async fn list_posts(&self, query: &PostQuery) -> Result<PostListResponse, Error> {
        let mut params: Vec<(&str, String)> = Vec::new();
        if let Some(ref s) = query.status {
            params.push(("status", s.clone()));
        }
        if let Some(l) = query.limit {
            params.push(("limit", l.to_string()));
        }
        if let Some(ref c) = query.cursor {
            params.push(("cursor", c.clone()));
        }

        let resp = self
            .http
            .get(format!("{}/api/v1/posts", self.base_url))
            .bearer_auth(&self.api_key)
            .query(&params)
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    /// Get a single blog post by slug.
    pub async fn get_post(&self, slug: &str) -> Result<BlogPost, Error> {
        let resp = self
            .http
            .get(format!("{}/api/v1/posts/{}", self.base_url, slug))
            .bearer_auth(&self.api_key)
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    /// Create or update a blog post (upsert on slug).
    pub async fn create_post(&self, post: &CreatePostRequest) -> Result<BlogPost, Error> {
        let resp = self
            .http
            .post(format!("{}/api/v1/posts", self.base_url))
            .bearer_auth(&self.api_key)
            .json(post)
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    /// Delete a blog post by slug.
    pub async fn delete_post(&self, slug: &str) -> Result<(), Error> {
        let resp = self
            .http
            .delete(format!("{}/api/v1/posts/{}", self.base_url, slug))
            .bearer_auth(&self.api_key)
            .send()
            .await?;
        self.check_response(resp).await?;
        Ok(())
    }

    /// Get blog statistics.
    pub async fn stats(&self) -> Result<StatsResponse, Error> {
        let resp = self
            .http
            .get(format!("{}/api/v1/stats", self.base_url))
            .bearer_auth(&self.api_key)
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    /// Check if the API server is healthy.
    pub async fn health(&self) -> Result<bool, Error> {
        let resp = self
            .http
            .get(format!("{}/api/health", self.base_url))
            .send()
            .await?;
        Ok(resp.status().is_success())
    }
}
