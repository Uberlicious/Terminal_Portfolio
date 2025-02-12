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

    let template = TerminalTemplate {
        init: true,
        modal: false,
        game: None,
    };
    HtmlTemplate(template)
}

#[derive(Template, Default)]
#[template(path = "pages/terminal.html")]
struct TerminalTemplate {
    init: bool,
    modal: bool,
    game: Option<u32>,
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
#[template(path = "menus/welcome.html")]
struct Welcome {
    init: bool,
}

#[derive(Template, Default)]
#[template(path = "menus/help.html")]
struct Help {
    init: bool,
}

#[derive(Template, Default)]
#[template(path = "menus/game.html")]
struct GameMenu {
    init: bool,
}

#[derive(Template, Default)]
#[template(path = "menus/projects.html")]
struct Projects {
    init: bool,
}

#[derive(Template, Default)]
#[template(path = "components/term-line.html")]
struct TermLine {
    init: bool,
}

#[derive(Template, Default)]
#[template(path = "components/modal.html", print = "all")]
struct Modal {
    init: bool,
    game: Option<u32>,
}

#[derive(Template, Default)]
#[template(path = "components/history.html")]
struct History {
    init: bool,
    commands: Vec<String>,
}

#[derive(Deserialize, Debug)]
enum Command {
    Welcome,
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

    let help = Help::default();

    if command.command.to_lowercase() == "help" {
        return HtmlTemplate(help).into_response();
    }

    if command.command.starts_with("games") {
        let game = GameMenu::default();
        let mut response = HtmlTemplate(game).into_response();

        let c: Vec<&str> = command.command.split_whitespace().collect();
        if c.len() > 1 {
            match c[1].to_lowercase().as_str() {
                "dieggle" | "1" => {
                    response = HtmlTemplate(Modal {
                        init: false,
                        game: Some(1),
                    })
                    .into_response();
                }
                "deathwalk" | "2" => {
                    response = HtmlTemplate(Modal {
                        init: false,
                        game: Some(2),
                    })
                    .into_response();
                }
                _ => {
                    println!("UNKNOWN")
                }
            }
        }

        return response;
    }

    if command.command.to_lowercase() == "projects" {
        return HtmlTemplate(Projects { init: false }).into_response();
    }

    if command.command.to_lowercase() == "history" {
        return HtmlTemplate(History {
            init: false,
            commands: lock.clone(),
        })
        .into_response();
    }

    let term = TermLine::default();
    HtmlTemplate(term).into_response()
}
