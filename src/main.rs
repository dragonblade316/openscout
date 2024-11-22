use std::collections::HashMap;

use assignments::GameManager;
use axum::{
    extract,
    extract::Path,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use data::{
    os_data::{TeamMatchReport, TeamPitReport},
    DataManager, Eventdata, MatchNumber,
};
use log::warn;

mod assignments;
mod data;

#[tokio::main]
async fn main() -> () {
    let dm = data::DataManager::new(
        "fmgoCbQpFAu8myt5dOBOeBLWYRRJWRN49ByCMpLKpOR0Q9SeXo1g6SE1hMKHz6pL".to_string(),
    )
    .await
    .unwrap();

    let app: Router<()> = Router::new()
        .route("/matchdata/:matchnum", get(get_match_data))
        .route("/teamdata/:teamnum", get(get_team_data))
        .route("/teammatchdata", post(post_team_match_data))
        .route(
            "/teammatchdata/:teamnum/:matchnum/:event",
            get(get_team_match_data),
        )
        .route("/event_list", get(get_event_list))
        .with_state(dm)
        .route("/scoutassignment", get(get_scouting_assignment));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

//this will be the last thing implmented due to how painful it will be to write the query
//async fn get_event_data() {}

async fn get_match_data() {}

async fn get_team_data(
    Path(team): Path<u32>,
    State(dm): State<DataManager>,
) -> Result<Json<data::TeamData>, AppError> {
    Ok(Json(dm.getTeamData(team).await?))
}

async fn post_team_match_data(
    State(dm): State<DataManager>,
    extract::Json(data): extract::Json<TeamMatchReport>,
) -> Result<(), AppError> {
    dm.postTeamMatchData(data).await?;
    Ok(())
}
async fn post_team_pit_data(
    State(dm): State<DataManager>,
    extract::Json(data): extract::Json<TeamPitReport>,
) -> Result<(), AppError> {
    //dm.postTeamPitData(data).await?;
    Ok(())
}
async fn get_team_match_data() {}

async fn get_scouting_assignment() {}

#[axum::debug_handler]
async fn get_event_list(State(dm): State<DataManager>) -> Result<Json<Vec<Eventdata>>, AppError> {
    Ok(Json(dm.get_event_data().await?))
}

//figure out vergen
async fn get_server_spec() {}

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
