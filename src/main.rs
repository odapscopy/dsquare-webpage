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

/// Application state that holds the content directories
#[derive(Clone)]
struct AppState {
    content_dirs: Arc<Vec<PathBuf>>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing for request logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Get multiple content directories from CONTENT_DIRS environment variable
    let content_dirs: Vec<PathBuf> = env::var("CONTENT_DIRS")
        .unwrap_or_else(|_| String::from("./pages,./assets,./styles,."))
        .split(',')
        .map(PathBuf::from)
        .collect();

    tracing::info!("Serving content from: {:?}", content_dirs);

    let state = AppState {
        content_dirs: Arc::new(content_dirs.clone()),
    };

    // Create our service router
    let mut app_router = Router::new()
        .route("/", get(serve_index))
        .route("/index", get(serve_home))
        .route("/services", get(serve_services))
        .route("/contact-us", get(serve_contact))
        .route("/about-us", get(serve_about_us))
        .fallback(handle_404)
        // Add middleware for all routes
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .with_state(state);

    // Add static file serving for each content directory
    for dir in &content_dirs {
        let dir_name = dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_else(|| {
                tracing::warn!(
                    "Could not determine directory name for {:?}, using default",
                    dir
                );
                "static"
            });

        tracing::info!(
            "Setting up nested service for {}: {}",
            dir_name,
            dir.display()
        );

        app_router = app_router.nest_service(
            &format!("/{}", dir_name),
            ServeDir::new(dir.clone()).precompressed_gzip(),
        );
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], 8090));
    tracing::info!("🚀 Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app_router).await.unwrap();
}

async fn serve_index(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    serve_file("index.html", &state.content_dirs)
}

async fn serve_home(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    serve_file("pages/home.html", &state.content_dirs)
}

async fn serve_services(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    serve_file("pages/services.html", &state.content_dirs)
}

async fn serve_contact(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    serve_file("pages/contact-us.html", &state.content_dirs)
}

async fn serve_about_us(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    serve_file("pages/about-us.html", &state.content_dirs)
}

/// Attempt to serve a file from any of the content directories
fn serve_file(filename: &str, content_dirs: &[PathBuf]) -> Result<Html<String>, AppError> {
    for dir in content_dirs {
        let file_path = dir.join(filename);
        tracing::debug!("Checking file: {}", file_path.display());

        if file_path.exists() {
            return std::fs::read_to_string(&file_path)
                .map(Html)
                .map_err(|err| {
                    tracing::error!("Failed to read {}: {}", file_path.display(), err);
                    AppError::NotFound
                });
        }
    }

    tracing::warn!("File not found: {}", filename);
    Err(AppError::NotFound)
}

/// Handler for 404 responses
async fn handle_404(State(state): State<AppState>) -> impl IntoResponse {
    // First try to find the custom 404 page
    let not_found_path = state.content_dirs.iter().find_map(|dir| {
        let path = dir.join("pages/404.html");
        if path.exists() {
            Some(path)
        } else {
            None
        }
    });

    match not_found_path {
        Some(path) => {
            tracing::info!("Serving custom 404 page: {}", path.display());
            match std::fs::read_to_string(&path) {
                Ok(content) => (StatusCode::NOT_FOUND, Html(content)).into_response(),
                Err(err) => {
                    tracing::error!("Error reading 404 page: {}", err);
                    (StatusCode::NOT_FOUND, Html("Page not found".to_string())).into_response()
                }
            }
        }
        None => {
            tracing::warn!("Custom 404 page not found, using fallback.");
            (StatusCode::NOT_FOUND, Html("Page not found".to_string())).into_response()
        }
    }
}

/// Application error types
#[derive(Debug)]
enum AppError {
    NotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::NotFound => {
                (StatusCode::NOT_FOUND, Html("Page not found".to_string())).into_response()
            }
        }
    }
}
