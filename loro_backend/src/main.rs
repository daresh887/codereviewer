use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde::Serialize;

#[derive(Serialize)]
struct ErrorMessage {
    error: String,
}

impl ErrorMessage {
    fn new(error: &str) -> Self {
        Self {
            error: error.to_string(),
        }
    }
}

async fn get_repo_info(
    Path((owner, repo)): Path<(String, String)>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let repo = octocrab::instance().repos(owner, repo).get().await;
    match repo {
        Ok(repo) => Ok(Json(repo)),
        Err(error) => {
            let (status_code, error_response) = match error {
                octocrab::Error::GitHub { .. } => (
                    StatusCode::NOT_FOUND,
                    Json(ErrorMessage::new("Repository not found")),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorMessage::new("GitHub error")),
                ),
            };
            Err((status_code, error_response))
        }
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/repo/{owner}/{repo}", get(get_repo_info));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
