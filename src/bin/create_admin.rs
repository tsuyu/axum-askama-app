use sqlx::mysql::MySqlPoolOptions;
use std::env;

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenvy::dotenv().ok();

    let args: Vec<String> = env::args().collect();
    
    // Simple argument parsing
    let mut username = String::new();
    let mut email = String::new();
    let mut password = String::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--username" | "-u" => {
                if i + 1 < args.len() {
                    username = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --username requires a value");
                    print_usage();
                    return;
                }
            }
            "--email" | "-e" => {
                if i + 1 < args.len() {
                    email = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --email requires a value");
                    print_usage();
                    return;
                }
            }
            "--password" | "-p" => {
                if i + 1 < args.len() {
                    password = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --password requires a value");
                    print_usage();
                    return;
                }
            }
            "--help" | "-h" => {
                print_usage();
                return;
            }
            _ => {
                eprintln!("Error: Unknown argument: {}", args[i]);
                print_usage();
                return;
            }
        }
    }

    // Validate required arguments
    if username.is_empty() || email.is_empty() || password.is_empty() {
        eprintln!("Error: All arguments are required");
        print_usage();
        return;
    }

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    println!("Creating admin user...");
    println!("  Username: {}", username);
    println!("  Email: {}", email);
    println!("  Password: {}", "*".repeat(password.len()));

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");

    // Hash the password using bcrypt
    let password_hash = bcrypt::hash(&password, bcrypt::DEFAULT_COST)
        .expect("Failed to hash password");

    // Insert the admin user
    let result = sqlx::query(
        "INSERT INTO admins (username, email, password_hash) VALUES (?, ?, ?)"
    )
    .bind(&username)
    .bind(&email)
    .bind(&password_hash)
    .execute(&pool)
    .await;

    match result {
        Ok(query_result) => {
            println!("\n✓ Admin user created successfully!");
            println!("  User ID: {}", query_result.last_insert_id());
            println!("\nYou can now login at: http://127.0.0.1:3001/loginadmin");
            println!("  Username: {}", username);
            println!("  Password: {}", password);
        }
        Err(e) => {
            eprintln!("\n✗ Failed to create admin user: {:?}", e);
            if e.to_string().contains("Duplicate entry") {
                eprintln!("  This username or email already exists");
            }
        }
    }

    pool.close().await;
}

fn print_usage() {
    println!("Usage: cargo run --bin create_admin -- --username <username> --email <email> --password <password>");
    println!();
    println!("Options:");
    println!("  --username, -u <username>  Admin username");
    println!("  --email, -e <email>        Admin email");
    println!("  --password, -p <password>  Admin password");
    println!("  --help, -h                 Show this help message");
    println!();
    println!("Example:");
    println!("  cargo run --bin create_admin -- -u admin -e admin@example.com -p admin123");
}
