use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::{env, net::SocketAddr, path::PathBuf, sync::Arc};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};

#[derive(Clone)]
struct AppState {
    content_dir: Arc<PathBuf>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing for request logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Get content directory from environment variable or use current directory
    let content_dir = env::var("CONTENT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::current_dir().expect("Failed to get current directory"));

    tracing::info!("Serving content from: {}", content_dir.display());

    let state = AppState {
        content_dir: Arc::new(content_dir.clone()),
    };

    // Create our service router
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/index", get(serve_home))
        .route("/services", get(serve_services))
        .route("/contact-us", get(serve_contact))
        .route("/about-us", get(serve_about_us))
        .nest_service(
            "/pages",
            ServeDir::new(content_dir.join("pages")).precompressed_gzip(),
        )
        .nest_service(
            "/assets",
            ServeDir::new(content_dir.join("assets")).precompressed_gzip(),
        )
        .nest_service(
            "/styles",
            ServeDir::new(content_dir.join("styles")).precompressed_gzip(),
        )
        .fallback(handle_404)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8090));
    tracing::info!("🚀 Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn serve_index(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    serve_file(state.content_dir.join("index.html"))
}

async fn serve_home(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    serve_file(state.content_dir.join("pages/home.html"))
}

async fn serve_services(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    serve_file(state.content_dir.join("pages/services.html"))
}

async fn serve_contact(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    serve_file(state.content_dir.join("pages/contact-us.html"))
}

async fn serve_about_us(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    serve_file(state.content_dir.join("pages/about-us.html"))
}

fn serve_file(path: PathBuf) -> Result<Html<String>, AppError> {
    std::fs::read_to_string(&path).map(Html).map_err(|e| {
        tracing::error!("Failed to read {}: {}", path.display(), e);
        AppError::NotFound
    })
}

async fn handle_404(State(state): State<AppState>) -> impl IntoResponse {
    let not_found_path = state.content_dir.join("pages/404.html");
    (
        StatusCode::NOT_FOUND,
        Html(
            std::fs::read_to_string(&not_found_path)
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
        (StatusCode::NOT_FOUND, Html(String::from("Page not found"))).into_response()
    }
}
