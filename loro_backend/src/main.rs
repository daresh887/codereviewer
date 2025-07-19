use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use dotenv::dotenv;
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
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

fn handle_octocrab_error(error: octocrab::Error) -> Response {
    match error {
        octocrab::Error::GitHub { ref source, .. } => {
            (source.status_code, Json(ErrorMessage::new(&source.message)))
                .into_response()
        }
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorMessage::new("Internal server error")),
        )
            .into_response(),
    }
}

async fn get_repo_info(
    State(octocrab): State<Arc<Octocrab>>,
    Path((owner, repo)): Path<(String, String)>,
) -> Response {
    eprintln!("GET /repo/{}/{}", owner, repo);

    let repo_result = octocrab.repos(&owner, &repo).get().await;

    if let Err(error) = repo_result {
        return handle_octocrab_error(error);
    }

    let repo = repo_result.unwrap();
    (StatusCode::OK, Json(repo)).into_response()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitTreeResponse {
    pub tree: Vec<GitTreeEntry>,
    pub truncated: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitTreeEntry {
    pub path: String,
    #[serde(rename = "type")]
    pub type_: String, // "blob" or "tree"
    pub mode: String,
    pub sha: String,
    pub size: Option<u64>, // Only for blobs
    pub url: String,
}

async fn get_git_tree(
    octocrab: Arc<Octocrab>,
    owner: &str,
    repo: &str,
) -> Result<GitTreeResponse, octocrab::Error> {
    octocrab
        .get::<GitTreeResponse, String, ()>(
            format!("/repos/{}/{}/git/trees/main?recursive=1", owner, repo),
            None,
        )
        .await
}

async fn get_repo_structure(
    State(octocrab): State<Arc<Octocrab>>,
    Path((owner, repo)): Path<(String, String)>,
) -> Response {
    eprintln!("GET /repo/{}/{}/structure", owner, repo);

    let tree = get_git_tree(Arc::clone(&octocrab), &owner, &repo).await;

    if let Err(error) = tree {
        return handle_octocrab_error(error);
    }

    let tree = tree.unwrap();
    (StatusCode::OK, Json(tree)).into_response()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().expect("Couldn't load .env file");

    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN is not set");
    let octocrab = octocrab::Octocrab::builder()
        .personal_token(token)
        .build()?;

    let app = Router::new()
        .route("/repo/{owner}/{repo}", get(get_repo_info))
        .route("/repo/{owner}/{repo}/structure", get(get_repo_structure))
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
