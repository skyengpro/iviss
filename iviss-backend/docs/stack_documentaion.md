# This documents is use to document all tools choose and use for the backend development.

## 1. Technology Stack

### 1.1 Backend

| Component                  | Technology         | Version | Justification                             |
| -------------------------- | ------------------ | ------- | ----------------------------------------- |
| **Language**         | Rust               | 1.75+   | Memory safety, performance, async/await   |
| **Web Framework**    | Axum               | 0.7     | Ergonomic, tower middleware, tokio-native |
| **Async Runtime**    | Tokio              | 1.x     | Industry standard for async Rust          |
| **Database Driver**  | sqlx               | 0.7     | Compile-time checked SQL, async           |
| **Password Hashing** | argon2             | 0.5     | Winner of Password Hashing Competition    |
|                            |                    |         |                                           |
| **Logging**          | tracing-subscriber | 0.3     | Structured logging                        |
| **CORS**             | tower-http         | 0.5     | CORS middleware                           |
|                            |                    |         |                                           |

### 1.2 Database

| Component             | Technology | Version | Justification                        |
| --------------------- | ---------- | ------- | ------------------------------------ |
| **Internal DB** | PostgreSQL | 15+     | ACID, full-text search, JSON support |
| **Migrations**  | sqlx-cli   | 0.7     | Version-controlled schema changes    |



### 1.3 developments tools

| Component | Technology | Version | Justification                     |
| --------- | ---------- | ------- | --------------------------------- |
| linker    | mold/lld   |         | Incrase linking phase performance |
|           |            |         |                                   |
