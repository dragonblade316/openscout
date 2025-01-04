use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};

use axum::{
    extract::{self, Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use clap::{Parser, Subcommand};
use data::{
    openscout::{Auth, AuthLevel, MongoAuth},
    Complevel, DataManager, Eventdata, MatchData, MatchNumber, TeamData, TeamMatchReport,
    TeamPitReport,
};
use log::error;
use serde::{Deserialize, Serialize};
use simplelog::Config;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

mod assignments;
mod data;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: PathBuf,

    #[command(subcommand)]
    cmd: Option<SubCommand>,
}

#[derive(Debug, Subcommand)]
enum SubCommand {
    version,
}

#[derive(Serialize, Deserialize, Debug)]
struct OSConfig {
    tba_key: String,
    enable_auth: Option<bool>,
    mongo_url: Option<String>,

    mongo_auth: Option<MongoAuth>,
    admin_auth: Option<Auth>,
}

#[derive(OpenApi)]
//#[openapi(
//    tags(
//        (name = CUSTOMER_TAG, description = "Customer API endpoints"),
//        (name = ORDER_TAG, description = "Order API endpoints")
//    )
//)]
struct ApiDoc;

#[tokio::main]
async fn main() -> () {
    let args = Args::parse();

    if let Some(cmd) = args.cmd {
        match cmd {
            SubCommand::version => {
                println!("{}", env!("CARGO_PKG_VERSION"));
                return;
            }
        }
    }

    let config: OSConfig = toml::from_str(
        fs::read_to_string(args.config)
            .expect("can't load args")
            .as_str(),
    )
    .expect("Can't parse config file");

    let dm = data::DataManager::new(config.tba_key, config.mongo_auth, config.enable_auth)
        .await
        .unwrap();

    if let Some(auth) = config.admin_auth {
        dm.add_user(auth)
            .await
            .unwrap_or(error!("Unable to set admin credentials"));
    }

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        //not sure I'm happy with how many time i typed route
        .routes(routes!(get_match_data))
        .routes(routes!(get_team_data))
        .routes(routes!(get_team_pit_data, post_team_pit_data))
        .routes(routes!(get_team_match_data, post_team_match_data))
        .routes(routes!(get_server_version))
        .routes(routes!(get_event_list))
        .routes(routes!(add_user))
        //.nest("/api/customer", customer::router())
        //.nest("/api/order", order::router())
        //.routes(routes!(
        //    inner::secret_handlers::get_secret,
        //    inner::secret_handlers::post_secret
        //)
        //)
        .split_for_parts();

    let app: Router<()> = router
        .merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api))
        .with_state(dm);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

//this will be the last thing implmented due to how painful it will be to write the query
//async fn get_event_data() {}

#[utoipa::path(get, path = "/matchdata/{event}/{complevel}/{match_num}", responses((status = 200, body = MatchData)), params(
        ("event" = String, Path, description = "The event id (blue allience format)"),
        ("complevel" = Complevel, Path, description = "The level of play"),
        ("match_num" = u32, Path, description = "the match number")
    )) ]
async fn get_match_data(
    Path(matchd): Path<MatchQuery>,
    headers: HeaderMap,
    State(dm): State<DataManager>,
) -> Result<Json<MatchData>, AppError> {
    dm.check_auth(&headers, AuthLevel::TEAM).await?;
    Ok(Json(
        dm.get_match_data(
            matchd.event,
            MatchNumber {
                number: matchd.match_num,
                level: matchd.complevel,
            },
        )
        .await?,
    ))
}

#[utoipa::path(get, path = "/teamdata/{team_number}/{event}", responses((status = OK, body = data::TeamData)), params(
    ("team_number" = u32, Path, description = "The team number"),
    ("event" = String, Path, description = "The event id (blue allience format)")
)) ]
async fn get_team_data(
    Path((team_number, event)): Path<(u32, String)>,
    headers: HeaderMap,
    State(dm): State<DataManager>,
) -> Result<Json<data::TeamData>, AppError> {
    dm.check_auth(&headers, AuthLevel::TEAM).await?;
    Ok(Json(dm.get_team_data(team_number, event).await?))
}

#[utoipa::path(post, path = "/teammatchdata", responses((status = OK))) ]
async fn post_team_match_data(
    State(dm): State<DataManager>,
    headers: HeaderMap,
    extract::Json(data): extract::Json<TeamMatchReport>,
) -> Result<(), AppError> {
    dm.check_auth(&headers, AuthLevel::TEAM).await?;
    dm.post_team_match_data(data).await?;
    Ok(())
}

#[utoipa::path(post, path = "/teampitdata", responses((status = OK))) ]
async fn post_team_pit_data(
    State(dm): State<DataManager>,
    headers: HeaderMap,
    extract::Json(data): extract::Json<TeamPitReport>,
) -> Result<(), AppError> {
    dm.check_auth(&headers, AuthLevel::TEAM).await?;
    dm.post_team_pit_data(data).await?;
    Ok(())
}

#[utoipa::path(get, path = "/teammatchdata/last/{team_number}/{event}/{complevel}/{match_num}", responses((status = OK, body = TeamMatchReport)), params(
    ("team_number" = u32, Path, description = "the team number"),
    ("event" = String, Path, description = "The event id (blue alliance format)"),
    ("complevel" = Complevel, Path, description = "The level of competition"),
    ("match_num" = u32, Path, description = "The match number"),
)) ]
async fn get_team_match_data(
    Path(matchd): Path<TeamMatchQuery>,
    headers: HeaderMap,
    State(dm): State<DataManager>,
) -> Result<Json<TeamMatchReport>, AppError> {
    dm.check_auth(&headers, AuthLevel::TEAM).await?;
    Ok(Json(
        dm.get_team_match_data(
            matchd.team_num,
            MatchNumber {
                number: matchd.match_num,
                level: matchd.complevel,
            },
            matchd.event,
        )
        .await?,
    ))
}

#[utoipa::path(get, path = "/teampitdata/{team_num}/{event}", responses((status = OK, body = TeamPitReport)), params(
    ("team_num" = u32, Path, description = "The team number"),
    ("event" = String, Path, description = "The event id (blue alliance format)")
)) ]
async fn get_team_pit_data(
    State(dm): State<DataManager>,
    headers: HeaderMap,
    Path((team_num, event)): Path<(u32, String)>,
) -> Result<Json<TeamPitReport>, AppError> {
    dm.check_auth(&headers, AuthLevel::TEAM).await?;
    Ok(Json(dm.get_team_pit_data(team_num, event).await?))
}

async fn get_scouting_assignment() {}

#[axum::debug_handler]
#[utoipa::path(get, path = "/eventlist", responses((status = OK, body = Vec<Eventdata>))) ]
async fn get_event_list(State(dm): State<DataManager>) -> Result<Json<Vec<Eventdata>>, AppError> {
    Ok(Json(dm.get_event_data().await?))
}

#[utoipa::path(post, path = "/adduser", responses((status = OK))) ]
async fn add_user(
    State(dm): State<DataManager>,
    headers: HeaderMap,
    Json(auth): Json<Auth>,
) -> Result<(), AppError> {
    dm.check_auth(&headers, AuthLevel::ADMIN).await?;
    dm.add_user(auth).await?;
    Ok(())
}

//
//
#[derive(Debug, Serialize, Deserialize)]
struct TeamMatchQuery {
    team_num: u32,
    event: String,
    complevel: Complevel,
    match_num: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct MatchQuery {
    event: String,
    complevel: Complevel,
    match_num: u32,
}
#[utoipa::path(get, path = "/version", responses((status = OK, body = str))) ]
async fn get_server_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// Make our own error that wraps `anyhow::Error`.
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
