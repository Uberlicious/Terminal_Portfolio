use std::sync::{Arc, Mutex};

use anyhow::Context;
use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Form, Router,
};
use serde::Deserialize;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct AppState {
    todos: Mutex<Vec<String>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "axum_htmx_askama=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("initializing router...");

    let app_state = Arc::new(AppState {
        todos: Mutex::new(vec![]),
    });

    let api_router = Router::new()
        .route("/hello", get(hello_from_the_server))
        .route("/todos", post(add_todo))
        .with_state(app_state);

    let assets_path = std::env::current_dir().unwrap();
    let router = Router::new()
        .nest("/api", api_router)
        .route("/", get(hello))
        .route("/another-page", get(another_page))
        .nest_service(
            "/assets",
            ServeDir::new(format!("{}/assets", assets_path.to_str().unwrap())),
        );
    let port = 8000_u16;
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .unwrap();

    info!("router initialized, now listening on port {}", port);

    axum::serve(listener, router)
        .await
        .context("error while starting serer")?;
    Ok(())
}

async fn hello() -> impl IntoResponse {
    info!("->> {:<12} - hello", "HANDLER");

    let template = HelloTemplate {};
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "pages/hello.html")]
struct HelloTemplate;

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render tempalte. Error: {}", err),
            )
                .into_response(),
        }
    }
}

async fn another_page() -> impl IntoResponse {
    info!("->> {:<12} - another-page", "HANDLER");
    let template = AnotherPageTemplate {};
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "pages/another-page.html")]
struct AnotherPageTemplate;

async fn hello_from_the_server() -> &'static str {
    "Hello!"
}

#[derive(Template)]
#[template(path = "components/todo-list.html")]
struct TodoList {
    todos: Vec<String>,
}

#[derive(Deserialize)]
struct TodoRequest {
    todo: String,
}

async fn add_todo(
    State(state): State<Arc<AppState>>,
    Form(todo): Form<TodoRequest>,
) -> impl IntoResponse {
    info!("->> {:<12} - add-todo", "HANDLER");

    let mut lock = state.todos.lock().unwrap();
    lock.push(todo.todo);

    let template = TodoList {
        todos: lock.clone(),
    };

    HtmlTemplate(template)
}
