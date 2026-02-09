# Axum-Askama App with MySQL Authentication

A modern web application built with Axum, Askama templates, and MySQL authentication.

## Features

- ✨ Modern, responsive UI with gradient design
- 🔐 Secure authentication with bcrypt password hashing
- 🗄️ MySQL database integration
- 🧠 Redis-backed sessions via `tower-sessions`
- ⚡ Redis query cache (Laravel-style cache pattern)
- 🎨 Beautiful login and registration pages
- 🚀 Fast and efficient with Rust/Axum

## Prerequisites

- Rust (latest stable version)
- MySQL server (5.7+ or 8.0+)
- Redis server
- Cargo

## Setup Instructions

### 1. Install MySQL

Make sure MySQL is installed and running on your system.

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install mysql-server
sudo systemctl start mysql
```

**macOS:**
```bash
brew install mysql
brew services start mysql
```

### 2. Create Database

Log into MySQL and create the database:

```bash
mysql -u root -p
```

Then run:
```sql
CREATE DATABASE axum_app;
EXIT;
```

### 3. Set Up Database Schema

Run the schema file to create the users table:

```bash
mysql -u root -p axum_app < schema.sql
```

### 4. Install Redis

Make sure Redis is installed and running on your system.

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install redis-server
sudo systemctl start redis-server
```

**macOS:**
```bash
brew install redis
brew services start redis
```

### 5. Configure Environment Variables

Copy the example environment file and update with your MySQL credentials:

```bash
cp .env.example .env
```

Edit `.env` and update the `DATABASE_URL` with your MySQL credentials, and set `REDIS_URL`:

```env
DATABASE_URL=mysql://your_username:your_password@localhost:3306/axum_app
REDIS_URL=redis://127.0.0.1:6379
```

**Note:** If you're using MySQL root user without a password:
```env
DATABASE_URL=mysql://root:@localhost:3306/axum_app
REDIS_URL=redis://127.0.0.1:6379
```

### 6. Build and Run

```bash
cargo build
cargo run
```

The application will be available at: **http://127.0.0.1:3001**

## Usage

### Registration

1. Navigate to http://127.0.0.1:3000/register
2. Fill in username, email, and password (minimum 6 characters)
3. Click "Create Account"
4. You'll be automatically logged in and redirected to the home page

### Login

1. Navigate to http://127.0.0.1:3000/login
2. Enter your username and password
3. Click "Login"
4. You'll be redirected to the home page

### Protected Routes

- `/admin/users` - View all users (requires authentication)
- `/admin/users/:id` - View user details (requires authentication)

### Logout

Click the "Logout" link in the navigation bar.


## Security Features

- **Password Hashing:** Uses bcrypt with default cost factor
- **Session Management:** Redis-backed sessions with `tower-sessions`
- **SQL Injection Protection:** Uses parameterized queries via sqlx
- **CSRF Protection:** CSRF token stored in session and validated on forms

## Troubleshooting

### Database Connection Error

If you see "Access denied for user 'root'@'localhost'":
- Check your MySQL username and password in `.env`
- Ensure MySQL server is running: `sudo systemctl status mysql`
- Verify the database exists: `mysql -u root -p -e "SHOW DATABASES;"`

### Port Already in Use

If port 3001 is already in use, modify the port in `src/main.rs`:
```rust
let addr = SocketAddr::from(([127, 0, 0, 1], 3002)); // Change to 3002
```

### Redis Connection Error

If you see Redis connection errors:
- Ensure Redis is running: `sudo systemctl status redis-server`
- Verify `REDIS_URL` in `.env` (e.g. `redis://127.0.0.1:6379`)

## Development

To run in development mode with auto-reload, you can use `cargo-watch`:

```bash
cargo install cargo-watch
cargo watch -x run
```

## License

MIT
