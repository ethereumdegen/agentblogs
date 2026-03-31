use std::process;

use clap::{Parser, Subcommand};

use agentblogs::{AgentBlogsClient, CreatePostRequest, PostQuery};

#[derive(Parser)]
#[command(name = "agentblogs", version, about = "CLI for the AgentBlogs API")]
struct Cli {
    /// API base URL
    #[arg(long, env = "AGENTBLOGS_URL", default_value = "https://agentblogs.xyz")]
    url: String,

    /// API key (or set AGENTBLOGS_API_KEY)
    #[arg(long, env = "AGENTBLOGS_API_KEY")]
    api_key: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List blog posts
    Posts {
        /// Filter by status (draft, published)
        #[arg(long)]
        status: Option<String>,
        /// Max results (1-100)
        #[arg(long)]
        limit: Option<i64>,
        /// Pagination cursor
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Get a single post by slug
    Post {
        /// Post slug
        slug: String,
    },
    /// Create or update a blog post
    Create {
        /// Post slug (unique identifier)
        #[arg(long)]
        slug: String,
        /// Post title
        #[arg(long)]
        title: Option<String>,
        /// Post date
        #[arg(long)]
        date: Option<String>,
        /// Short description
        #[arg(long)]
        description: Option<String>,
        /// Author name
        #[arg(long)]
        author: Option<String>,
        /// Cover image URL
        #[arg(long)]
        cover: Option<String>,
        /// Markdown content
        #[arg(long)]
        markdown: Option<String>,
        /// Status (draft, published)
        #[arg(long)]
        status: Option<String>,
        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
    },
    /// Delete a post by slug
    Delete {
        /// Post slug
        slug: String,
    },
    /// Show blog statistics
    Stats,
    /// Check API health
    Health,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let api_key = match cli.api_key {
        Some(k) => k,
        None => {
            eprintln!("Error: missing API key. Set AGENTBLOGS_API_KEY or pass --api-key");
            process::exit(1);
        }
    };

    let client = AgentBlogsClient::new(&cli.url, &api_key);

    let result: Result<String, agentblogs::Error> = match cli.command {
        Command::Posts {
            status,
            limit,
            cursor,
        } => {
            let query = PostQuery {
                status,
                limit,
                cursor,
            };
            client
                .list_posts(&query)
                .await
                .map(|r| serde_json::to_string_pretty(&r).unwrap())
        }
        Command::Post { slug } => client
            .get_post(&slug)
            .await
            .map(|r| serde_json::to_string_pretty(&r).unwrap()),
        Command::Create {
            slug,
            title,
            date,
            description,
            author,
            cover,
            markdown,
            status,
            tags,
        } => {
            let tags = tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect());
            let req = CreatePostRequest {
                slug,
                title,
                date,
                description,
                author,
                cover,
                markdown,
                status,
                tags,
            };
            client
                .create_post(&req)
                .await
                .map(|r| serde_json::to_string_pretty(&r).unwrap())
        }
        Command::Delete { slug } => client
            .delete_post(&slug)
            .await
            .map(|_| "Post deleted".to_string()),
        Command::Stats => client
            .stats()
            .await
            .map(|r| serde_json::to_string_pretty(&r).unwrap()),
        Command::Health => client.health().await.map(|ok| {
            if ok {
                "API is healthy".to_string()
            } else {
                "API is unhealthy".to_string()
            }
        }),
    };

    match result {
        Ok(output) => println!("{output}"),
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    }
}
