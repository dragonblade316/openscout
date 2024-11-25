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
    Complevel, DataManager, Eventdata, MatchData, MatchNumber, TeamMatchReport, TeamPitReport,
};
use log::error;
use serde::{Deserialize, Serialize};
use simplelog::Config;

mod assignments;
mod data;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: PathBuf,

    #[command(subcommand)]
    cmd: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    version,
}

#[derive(Serialize, Deserialize, Debug)]
struct OSConfig {
    tba_key: String,
    mongo_url: String,

    mongo_auth: Option<MongoAuth>,
    admin_auth: Option<Auth>,
}

#[tokio::main]
async fn main() -> () {
    let args = Args::parse();
    let config: OSConfig = toml::from_str(
        fs::read_to_string(args.config)
            .expect("can't load args")
            .as_str(),
    )
    .expect("Can't parse config file");

    let dm = data::DataManager::new(config.tba_key, config.mongo_auth)
        .await
        .unwrap();

    if let Some(auth) = config.admin_auth {
        dm.add_user(auth)
            .await
            .unwrap_or(error!("Unable to set admin credentials"));
    }

    let app: Router<()> = Router::new()
        .route(
            "/matchdata/:event/:complevel/:match_num",
            get(get_match_data),
        )
        .route("/teamdata/:teamnum/:event", get(get_team_data))
        .route("/teammatchdata", post(post_team_match_data))
        .route(
            "/teammatchdata/:team_num/:event/:complevel/:match_num",
            get(get_team_match_data),
        )
        .route("/teampitdata", post(post_team_pit_data))
        .route("/teampitdata/:teamnum/:event", get(get_team_pit_data))
        .route("/event_list", get(get_event_list))
        .route("/adduser", post(add_user))
        .with_state(dm)
        //.route("/scoutassignment", get(get_scouting_assignment))
        .route("/version", get(get_server_version));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

//this will be the last thing implmented due to how painful it will be to write the query
//async fn get_event_data() {}

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

async fn get_team_data(
    Path(team): Path<u32>,
    Path(event): Path<String>,
    headers: HeaderMap,
    State(dm): State<DataManager>,
) -> Result<Json<data::TeamData>, AppError> {
    dm.check_auth(&headers, AuthLevel::TEAM).await?;
    Ok(Json(dm.get_team_data(team, event).await?))
}

async fn post_team_match_data(
    State(dm): State<DataManager>,
    headers: HeaderMap,
    extract::Json(data): extract::Json<TeamMatchReport>,
) -> Result<(), AppError> {
    dm.check_auth(&headers, AuthLevel::TEAM).await?;
    dm.post_team_match_data(data).await?;
    Ok(())
}
async fn post_team_pit_data(
    State(dm): State<DataManager>,
    headers: HeaderMap,
    extract::Json(data): extract::Json<TeamPitReport>,
) -> Result<(), AppError> {
    dm.check_auth(&headers, AuthLevel::TEAM).await?;
    dm.post_team_pit_data(data).await?;
    Ok(())
}
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

#[axum::debug_handler]
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
async fn get_event_list(State(dm): State<DataManager>) -> Result<Json<Vec<Eventdata>>, AppError> {
    Ok(Json(dm.get_event_data().await?))
}

async fn add_user(State(dm): State<DataManager>, headers: HeaderMap) -> Result<(), AppError> {
    dm.check_auth(&headers, AuthLevel::ADMIN).await?;
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
