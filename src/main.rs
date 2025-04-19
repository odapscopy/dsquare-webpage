use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::net::SocketAddr;
// use tokio::signal;
use tower_http::{compression::CompressionLayer, services::ServeDir, trace::TraceLayer};

#[tokio::main]
async fn main() {
    // Initialize tracing for request logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Create our service router
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/index", get(serve_home))
        .route("/services", get(serve_services))
        .route("/contact-us", get(serve_contact))
        .route("/about-us", get(serve_about_us))
        .nest_service("/pages", ServeDir::new("pages").precompressed_gzip())
        .nest_service("/assets", ServeDir::new("assets").precompressed_gzip())
        .nest_service("/styles", ServeDir::new("styles").precompressed_gzip())
        .fallback(handle_404)
        // Add middleware for all routes
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("🚀 Server listening on {}", addr);

    // Build the server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    /*let server = axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .with_graceful_shutdown(shutdown_signal());*/
    axum::serve(listener, app).await.unwrap();

    // Start the server and handle any errors
    /*if let Err(e) = server.await {
        tracing::error!("Server error: {}", e);
    }*/
}

async fn serve_index() -> Result<Html<String>, AppError> {
    serve_file("index.html")
}

async fn serve_home() -> Result<Html<String>, AppError> {
    serve_file("pages/home.html")
}

async fn serve_services() -> Result<Html<String>, AppError> {
    serve_file("pages/services.html")
}

async fn serve_contact() -> Result<Html<String>, AppError> {
    serve_file("pages/contact-us.html")
}

async fn serve_about_us() -> Result<Html<String>, AppError> {
    serve_file("pages/about-us.html")
}

fn serve_file(path: &str) -> Result<Html<String>, AppError> {
    std::fs::read_to_string(path)
        .map(Html)
        .map_err(|_| AppError::NotFound)
}

async fn handle_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Html(
            std::fs::read_to_string("pages/404.html")
                .unwrap_or_else(|_| String::from("Page not found")),
        ),
    )
}

#[derive(Debug)]
enum AppError {
    NotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::NOT_FOUND,
            Html(
                std::fs::read_to_string("pages/404.html")
                    .unwrap_or_else(|_| String::from("Page not found")),
            ),
        )
            .into_response()
    }
}
