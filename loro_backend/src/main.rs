use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use dotenv::dotenv;
use serde::Serialize;
use thiserror::Error;

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

#[derive(Error, Debug)]
enum ApiError {
    #[error("Repository not found")]
    NotFound,
    #[error("Rate limited by GitHub")]
    RateLimited,
    #[error("GitHub API error")]
    GitHubError,
    #[error("Internal server error")]
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, msg) = match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::RateLimited => {
                (StatusCode::TOO_MANY_REQUESTS, self.to_string())
            }
            ApiError::GitHubError => {
                (StatusCode::BAD_GATEWAY, self.to_string())
            }
            ApiError::Internal => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
        };
        (status, Json(ErrorMessage::new(&msg))).into_response()
    }
}

async fn get_repo_info(
    Path((owner, repo)): Path<(String, String)>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN is not set");
    let octocrab = octocrab::Octocrab::builder()
        .personal_token(token)
        .build()
        .unwrap();
    let repo_result = octocrab.repos(owner, repo).get().await;
    match repo_result {
        Ok(repo) => Ok((StatusCode::OK, Json(repo))),
        Err(error) => {
            if let octocrab::Error::GitHub { ref source, .. } = error {
                // Check for 404 Not Found
                if source.message.contains("Not Found") {
                    return Err(ApiError::NotFound);
                }
                // Check for 403 rate limit
                if source.message.contains("rate limit")
                    || source.message.contains("abuse detection")
                {
                    return Err(ApiError::RateLimited);
                }
                return Err(ApiError::GitHubError);
            }
            Err(ApiError::Internal)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().expect("Couldn't load .env file");
    let app = Router::new().route("/repo/{owner}/{repo}", get(get_repo_info));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
