use core::fmt;
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
    command_history: Mutex<Vec<String>>,
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
        command_history: Mutex::new(vec![]),
    });

    let api_router = Router::new()
        .route("/commands", post(commands))
        .with_state(app_state);

    let assets_path = std::env::current_dir().unwrap();
    let router = Router::new()
        .nest("/api", api_router)
        .route("/", get(terminal))
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

async fn terminal() -> impl IntoResponse {
    info!("->> {:<12} - terminal", "HANDLER");

    let template = TerminalTemplate { init: true };
    HtmlTemplate(template)
}

#[derive(Template, Default)]
#[template(path = "pages/terminal.html")]
struct TerminalTemplate {
    init: bool,
}

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

async fn _hello_from_the_server() -> &'static str {
    "Hello!"
}

#[derive(Template, Default)]
#[template(path = "components/welcome.html")]
struct Welcome {
    init: bool,
}

#[derive(Template, Default)]
#[template(path = "components/term-line.html")]
struct TermLine {
    init: bool,
}

#[derive(Deserialize, Debug)]
enum Command {
    Welcome(String),
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Deserialize, Debug)]
struct CommandRequest {
    command: String,
}

async fn commands(
    State(state): State<Arc<AppState>>,
    Form(command): Form<CommandRequest>,
) -> Response {
    info!("->> {:<12} - commands - {command:?}", "HANDLER");

    let mut lock = state.command_history.lock().unwrap();
    lock.push(command.command.clone());

    let welcome = Welcome::default();

    if command.command.to_lowercase() == "welcome" {
        return HtmlTemplate(welcome).into_response();
    }

    let term = TermLine::default();
    HtmlTemplate(term).into_response()
}
