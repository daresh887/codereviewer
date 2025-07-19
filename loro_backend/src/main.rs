use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use dotenv::dotenv;
use octocrab::Octocrab;
use serde::Serialize;
use std::{net::SocketAddr, sync::Arc};

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
    State(octocrab): State<Arc<Octocrab>>,
    Path((owner, repo)): Path<(String, String)>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    eprintln!("GET /repo/{}/{}", owner, repo);
    let repo_result = octocrab.repos(owner, repo).get().await;
    match repo_result {
        Ok(repo) => Ok((StatusCode::OK, Json(repo))),
        Err(error) => match error {
            octocrab::Error::GitHub { ref source, .. } => Err((
                source.status_code,
                Json(ErrorMessage::new(&source.message)),
            )),
            _ => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorMessage::new("Internal server error")),
            )),
        },
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN is not set");
    let octocrab = octocrab::Octocrab::builder()
        .personal_token(token)
        .build()?;

    let app = Router::new()
        .route("/repo/{owner}/{repo}", get(get_repo_info))
        .with_state(Arc::new(octocrab));

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3002".to_string())
        .parse::<u16>()
        .expect("PORT must be a number");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    eprintln!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
