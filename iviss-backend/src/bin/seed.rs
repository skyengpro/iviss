use anyhow::{Context, Result};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::fs;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Load environment variables
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set in .env")?;

    println!("🌱 Connecting to database...");

    // 2. Connect to the database
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("Failed to connect to the database")?;

    println!("✅ Connected to database.");

    // 3. Find the seed migration file
    // We look for the file we created earlier
    let seed_file_path = "seeds/seed_data.sql";

    println!("📖 Reading seed data from {}...", seed_file_path);
    let sql = fs::read_to_string(seed_file_path)
        .context(format!("Failed to read seed file: {}", seed_file_path))?;

    // 4. Execute the SQL
    println!("🚀 Executing seed SQL...");
    use sqlx::Executor;
    pool.execute(sql.as_str())
        .await
        .context("Failed to execute seed SQL")?;

    println!("✨ Database seeded successfully with 6 car plates and associated data!");

    Ok(())
}
