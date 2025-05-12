use axum::{
    http::StatusCode,
    routing::{get, post}, // Add get
    Json, // Import Json extractor
    Router, extract::State,
};
use chrono::Local; // For getting current time
use serde::Deserialize; // Import Deserialize
use std::{net::SocketAddr, env, sync::Arc};
use dotenvy::dotenv;
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};

// Define a struct to deserialize the incoming JSON data
#[derive(Deserialize, Debug)] // Add Debug for easy printing
struct UserData {
    name: String,
    age: String,
}

#[derive(Clone)] // Add Clone to allow AppState to be used with Axum's with_state
struct AppState {
    db_pool: PgPool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing/logging
    tracing_subscriber::fmt::init();

    // Load environment variables from .env file
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Create a PostgreSQL connection pool
    let db_pool = PgPool::connect(&database_url).await?;
    tracing::info!("Successfully connected to the database.");

    // Create table if it doesn't exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_logs (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            age INTEGER,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );
        "#,
    )
    .execute(&db_pool)
    .await?;
    tracing::info!("'user_logs' table checked/created successfully.");

    let app_state = AppState { db_pool };

    // Define CORS layer to allow requests from the frontend (likely localhost:8080)
    let cors = CorsLayer::new()
        .allow_origin(Any) // Allow any origin for simplicity, restrict in production
        .allow_methods(Any)
        .allow_headers(Any);

    // Build our application with a route
    let app = Router::new()
        // `POST /api/log` will be handled by the `log_message` handler
        .route("/api/log", post(log_message))
        // `GET /api/time` will be handled by the `get_time` handler
        .route("/api/time", get(get_time))
        .layer(cors) // Apply the CORS middleware
        .with_state(app_state); // Add the application state

    // Run our app with hyper, listening globally on port 3000
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

// Handler for POST /api/log, now accepting JSON data and AppState
async fn log_message(
    State(state): State<AppState>,
    Json(payload): Json<UserData>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Print the received data to the server's console
    tracing::info!("Received data: Name = {}, Age = {}", payload.name, payload.age);

    // Attempt to parse age string to i32. Handle potential errors.
    let age_as_i32 = match payload.age.parse::<i32>() {
        Ok(age_val) => Some(age_val),
        Err(_) => {
            tracing::warn!("Could not parse age '{}' to i32. Storing as NULL.", payload.age);
            None // Store as NULL if parsing fails
        }
    };

    match sqlx::query("INSERT INTO user_logs (name, age) VALUES ($1, $2)")
        .bind(&payload.name)
        .bind(age_as_i32) // This will be None if parsing failed, resulting in NULL in DB
        .execute(&state.db_pool)
        .await
    {
        Ok(_) => {
            tracing::info!("Successfully inserted data into user_logs table.");
            Ok(StatusCode::OK)
        }
        Err(e) => {
            tracing::error!("Failed to insert data into database: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to log message: {}", e),
            ))
        }
    }
}

// Handler for GET /api/time
async fn get_time() -> (StatusCode, String) {
    let now = Local::now();
    let time_str = now.format("%H:%M:%S").to_string();
    (StatusCode::OK, time_str)
}
