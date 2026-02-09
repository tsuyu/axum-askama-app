use sqlx::mysql::MySqlPoolOptions;
use std::env;

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    println!("Connecting to database...");
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");

    println!("✓ Database connection successful\n");

    // Check if admins table exists
    let table_check: Result<(i64,), sqlx::Error> = sqlx::query_as(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = DATABASE() AND table_name = 'admins'"
    )
    .fetch_one(&pool)
    .await;

    match table_check {
        Ok((count,)) => {
            if count > 0 {
                println!("✓ Admins table exists");
            } else {
                println!("✗ Admins table does NOT exist!");
                println!("  Run the schema.sql file to create tables");
                return;
            }
        }
        Err(e) => {
            println!("✗ Error checking for admins table: {:?}", e);
            return;
        }
    }

    // Count admin users
    let admin_count: Result<(i64,), sqlx::Error> =
        sqlx::query_as("SELECT COUNT(*) FROM admins")
            .fetch_one(&pool)
            .await;

    match admin_count {
        Ok((count,)) => {
            println!("✓ Found {} admin user(s)\n", count);
            
            if count == 0 {
                println!("⚠ No admin users found!");
                println!("  To create an admin user, run:");
                println!("  cargo run --bin create_admin");
            }
        }
        Err(e) => {
            println!("✗ Error counting admins: {:?}", e);
            return;
        }
    }

    // List all admin users (without password hashes)
    let admins: Result<Vec<(i32, String, String)>, sqlx::Error> =
        sqlx::query_as("SELECT id, username, email FROM admins")
            .fetch_all(&pool)
            .await;

    match admins {
        Ok(admin_list) => {
            if !admin_list.is_empty() {
                println!("Admin users:");
                println!("ID | Username | Email");
                println!("---|----------|------");
                for (id, username, email) in admin_list {
                    println!("{:2} | {:<8} | {}", id, username, email);
                }
            }
        }
        Err(e) => {
            println!("✗ Error listing admins: {:?}", e);
        }
    }

    pool.close().await;
}
