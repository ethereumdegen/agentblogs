use std::env;
use std::path::Path;

use sqlx::migrate::{Migrate, Migrator};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let connect_options: sqlx::postgres::PgConnectOptions = database_url
        .parse::<sqlx::postgres::PgConnectOptions>()?
        .statement_cache_capacity(0);

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(connect_options)
        .await?;

    let migrations_dir = if Path::new("migrations").exists() {
        "migrations"
    } else if Path::new("/app/migrations").exists() {
        "/app/migrations"
    } else {
        panic!("Could not find migrations directory");
    };

    println!("Migrations from {}", migrations_dir);

    let mut migrator = Migrator::new(Path::new(migrations_dir)).await?;
    migrator.set_ignore_missing(true);

    let mut conn = pool.acquire().await?;
    conn.ensure_migrations_table().await.ok();
    let applied: std::collections::HashSet<i64> = conn
        .list_applied_migrations()
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|m| m.version)
        .collect();
    drop(conn);

    let mut pending = 0;
    for migration in migrator.iter() {
        if applied.contains(&migration.version) {
            println!("  {:>3} {}  SKIP", migration.version, migration.description);
        } else {
            println!("  {:>3} {}  PENDING", migration.version, migration.description);
            pending += 1;
        }
    }

    if pending == 0 {
        println!("All {} migrations up to date", applied.len());
        return Ok(());
    }

    sqlx::query("SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = current_database() AND pid != pg_backend_pid()")
        .execute(&pool)
        .await
        .ok();

    migrator.run(&pool).await?;
    println!("{} migrations applied", pending);

    Ok(())
}
