use anyhow::Context;
use askama::Template;
use axum::{
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Form, Router,
};
use serde::Deserialize;
use tower_http::{cors::CorsLayer, services::ServeDir, set_header::SetResponseHeaderLayer};
use tower_sessions::{Expiry, MemoryStore, Session, SessionManagerLayer};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const HISTORY_KEY: &str = "terminal_history";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "terminal_portfolio=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("initializing router...");

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Set to true in production with HTTPS
        .with_expiry(Expiry::OnInactivity(time::Duration::hours(1)));

    let api_router = Router::new().route("/commands", post(commands));

    let assets_path = std::env::current_dir().unwrap();
    let router = Router::new()
        .nest("/api", api_router)
        .route("/", get(terminal))
        .nest_service(
            "/assets",
            ServeDir::new(format!("{}/assets", assets_path.to_str().unwrap())),
        )
        .layer(session_layer)
        .layer(CorsLayer::permissive())
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            header::HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            header::HeaderValue::from_static("SAMEORIGIN"),
        ));

    let port = 8000_u16;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    info!("router initialized, now listening on port {}", port);
    axum::serve(listener, router)
        .await
        .context("error while starting server")?;
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
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
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
#[template(path = "menus/info_cmd.html")]
struct Info {
    init: bool,
}

#[derive(Template, Default)]
#[template(path = "components/neofetch.html")]
struct Neofetch;

#[derive(Template, Default)]
#[template(path = "components/term-line.html")]
struct TermLine {
    init: bool,
}

#[derive(Template, Default)]
#[template(path = "components/modal.html")]
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
struct CommandRequest {
    command: String,
}

#[axum::debug_handler]
async fn commands(session: Session, Form(request): Form<CommandRequest>) -> Response {
    let raw_command = request.command.trim();
    let command_lowercase = raw_command.to_lowercase();
    let parts: Vec<&str> = command_lowercase.split_whitespace().collect();

    if parts.is_empty() {
        return HtmlTemplate(TermLine::default()).into_response();
    }

    info!("->> {:<12} - commands - {raw_command}", "HANDLER");

    let mut history: Vec<String> = match session.get(HISTORY_KEY).await {
        Ok(Some(h)) => h,
        _ => Vec::new(),
    };

    match parts[0] {
        "clear" => {
            session
                .remove::<Vec<String>>(HISTORY_KEY)
                .await
                .unwrap_or_default();
            return HtmlTemplate(TermLine::default()).into_response();
        }
        _ => {
            history.push(raw_command.to_string());
            if let Err(e) = session.insert(HISTORY_KEY, &history).await {
                error!("Failed to update session history: {e}");
            }
        }
    }

    match parts[0] {
        "welcome" => HtmlTemplate(Welcome::default()).into_response(),
        "help" => HtmlTemplate(Help::default()).into_response(),
        "projects" => HtmlTemplate(Projects::default()).into_response(),
        "info" => HtmlTemplate(Info::default()).into_response(),
        "neofetch" => HtmlTemplate(Neofetch::default()).into_response(),
        "history" => {
            let history_limit = 25;
            let start_idx = history.len().saturating_sub(history_limit);
            let recent_history = &history[start_idx..];

            let formatted_history: Vec<String> = recent_history
                .iter()
                .enumerate()
                .map(|(i, s)| format!("{}: {s}", i + 1 + start_idx))
                .collect();

            HtmlTemplate(History {
                init: false,
                commands: formatted_history,
            })
            .into_response()
        }
        "games" => {
            if parts.len() > 1 {
                match parts[1] {
                    "dieggle" | "1" => HtmlTemplate(Modal {
                        init: false,
                        game: Some(1),
                    })
                    .into_response(),
                    "deathwalk" | "2" => HtmlTemplate(Modal {
                        init: false,
                        game: Some(2),
                    })
                    .into_response(),
                    _ => HtmlTemplate(GameMenu::default()).into_response(),
                }
            } else {
                HtmlTemplate(GameMenu::default()).into_response()
            }
        }
        _ => HtmlTemplate(TermLine::default()).into_response(),
    }
}
