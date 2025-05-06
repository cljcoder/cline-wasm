use axum::{
    http::StatusCode,
    routing::post,
    Json, // Import Json extractor
    Router,
};
use serde::Deserialize; // Import Deserialize
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

// Define a struct to deserialize the incoming JSON data
#[derive(Deserialize, Debug)] // Add Debug for easy printing
struct UserData {
    name: String,
    age: String,
}

#[tokio::main]
async fn main() {
    // Initialize tracing/logging
    tracing_subscriber::fmt::init();

    // Define CORS layer to allow requests from the frontend (likely localhost:8080)
    let cors = CorsLayer::new()
        .allow_origin(Any) // Allow any origin for simplicity, restrict in production
        .allow_methods(Any)
        .allow_headers(Any);

    // Build our application with a route
    let app = Router::new()
        // `POST /api/log` will be handled by the `log_message` handler
        .route("/api/log", post(log_message))
        .layer(cors); // Apply the CORS middleware

    // Run our app with hyper, listening globally on port 3000
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Handler for POST /api/log, now accepting JSON data
async fn log_message(Json(payload): Json<UserData>) -> StatusCode {
    // Print the received data to the server's console
    println!("Received data: Name = {}, Age = {}", payload.name, payload.age);
    StatusCode::OK // Respond with HTTP 200 OK
}
