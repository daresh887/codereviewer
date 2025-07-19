use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use dotenv::dotenv;
use futures::{future::BoxFuture, FutureExt};
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

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum Node {
    #[serde(rename = "file")]
    File { name: String },
    #[serde(rename = "directory")]
    Directory { name: String, children: Vec<Node> },
}

fn get_structure<'a>(
    owner: &'a str,
    repo: &'a str,
    octocrab: Arc<Octocrab>,
    path: String,
) -> BoxFuture<'a, Result<Vec<Node>, Response>> {
    async move {
        let contents_result = octocrab
            .repos(owner, repo)
            .get_content()
            .path(&path)
            .send()
            .await;

        if let Err(error) = contents_result {
            return Err(handle_octocrab_error(error));
        }

        let items = contents_result.unwrap().items;
        let mut ret = Vec::new();
        for item in items {
            match item.r#type.as_str() {
                "file" => ret.push(Node::File { name: item.name }),
                "dir" => {
                    let mut path = path.clone();
                    path.push_str("/");
                    path.push_str(item.name.as_str());
                    ret.push(Node::Directory {
                        name: item.name,
                        children: get_structure(
                            owner,
                            repo,
                            Arc::clone(&octocrab),
                            path.clone(),
                        )
                        .await?,
                    })
                }
                _ => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorMessage::new(
                            "Internal server error: Unknown item type",
                        )),
                    )
                        .into_response())
                }
            }
        }

        Ok(ret)
    }
    .boxed()
}

async fn get_repo_structure(
    State(octocrab): State<Arc<Octocrab>>,
    Path((owner, repo)): Path<(String, String)>,
) -> Response {
    eprintln!("GET /repo/{}/{}/structure", owner, repo);

    let structure = get_structure(&owner, &repo, octocrab, String::new()).await;

    if let Err(error) = structure {
        return error;
    }

    (StatusCode::OK, Json(structure.unwrap())).into_response()
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
