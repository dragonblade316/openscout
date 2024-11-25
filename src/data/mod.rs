pub mod openscout;
pub mod season; //data structs
pub mod statbotics;
pub mod theblueallience;
use axum::{http::HeaderMap, response::IntoResponse};
use log::warn;
use openscout::{Auth, AuthLevel, MongoAuth};
use reqwest::StatusCode;
use schemars::{schema_for, JsonSchema};

use std::collections::HashMap;

use anyhow::*;
use serde::{Deserialize, Serialize};
use statbotics::Statbotics;
use theblueallience::{TbaScoreBreakdown, TheBlueAllience};

//TODO: set a client here so that the connection pool is shared by all there services (or not, I
//don't think there would be a benifit to this)

#[derive(Clone)]
pub struct DataManager {
    openscoutdb: openscout::OpenScoutDB,
    tba: theblueallience::TheBlueAllience,
    statbotics: statbotics::Statbotics,
    //TODO: event list
}

impl DataManager {
    pub async fn new(tba_key: String, mongo_auth: Option<MongoAuth>) -> Result<Self> {
        Ok(Self {
            openscoutdb: openscout::OpenScoutDB::new(None, mongo_auth).await?,
            tba: TheBlueAllience::new(tba_key).await?,
            statbotics: Statbotics::new().await?,
        })
    }

    pub async fn get_team_data(&self, team_number: u32, event: String) -> Result<TeamData> {
        let tba_data = self.tba.get_team_data(team_number.clone(), event).await?;
        let statbotics_data = self.statbotics.get_team_data(team_number.clone()).await?;

        Ok(TeamData {
            team_number,

            opr: tba_data.opr,
            dpr: tba_data.dpr,
            ccwm: tba_data.ccwm,

            unitless_epa: statbotics_data.epa.unitless,
            norm_epa: statbotics_data.epa.norm,
        })
    }

    pub async fn get_match_data(&self, event: String, match_num: MatchNumber) -> Result<MatchData> {
        let tba_data = self
            .tba
            .get_match_data(match_num.clone(), event.clone())
            .await?;
        let statbotics_data = self
            .statbotics
            .get_match_data(event.clone(), match_num.clone())
            .await?;

        MatchData {
            winner: tba_data.winning_allience,
            predicted_winner: statbotics_data.pred.winner,
            red_win_prob: statbotics_data.pred.red_win_prob,
            red_allience: tba_data.red_allience,
            blue_allience: tba_data.blue_allience,
            red_score: tba_data.red_score,
            blue_score: tba_data.blue_score,
            red_score_breakdown: tba_data.red_score_breakdown,
            blue_score_breakdown: tba_data.blue_score_breakdown,
            predicted_red_score: statbotics_data.pred.red_score,
            predicted_blue_score: statbotics_data.pred.blue_score,
            event,
            match_number: match_num,
        };

        todo!()
    }

    pub async fn post_team_match_data(&self, data: TeamMatchReport) -> Result<()> {
        self.openscoutdb.post_team_match_data(data).await?;
        Ok(())
    }

    pub async fn post_team_pit_data(&self, data: TeamPitReport) -> Result<()> {
        self.openscoutdb.post_team_pit_data(data).await?;
        Ok(())
    }

    pub async fn get_team_match_data(
        &self,
        team_number: u32,
        match_number: MatchNumber,
        event: String,
    ) -> Result<TeamMatchReport> {
        self.openscoutdb
            .get_team_match_data(team_number, match_number, event)
            .await
    }

    pub async fn get_team_pit_data(
        &self,
        team_number: u32,
        event: String,
    ) -> Result<TeamPitReport> {
        self.openscoutdb.get_team_pit_data(team_number, event).await
    }

    pub async fn get_event_data(&self) -> Result<Vec<Eventdata>> {
        self.tba.get_event_list().await
    }

    pub async fn check_auth(&self, headers: &HeaderMap, required_auth: AuthLevel) -> Result<()> {
        //it is the destiny of all my codebases to have some annoying ugly as crap code to convert
        //things to the correct datatype
        //TODO: give this actual errors
        let team: u32 = headers
            .get("id")
            .unwrap_or(return Err(anyhow!("")))
            .to_str()?
            .parse()?;
        let key: String = headers
            .get("key")
            .unwrap_or(return Err(anyhow!("")))
            .to_str()?
            .to_string();

        let auth = self.openscoutdb.check_auth(team).await?;

        if key != auth.key || auth.auth < required_auth {
            return Err(anyhow!(StatusCode::UNAUTHORIZED));
        }

        Ok(())
    }

    pub async fn add_user(&self, auth: Auth) -> Result<()> {
        self.openscoutdb.add_auth(auth).await?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamData {
    team_number: u32,

    opr: f64,
    dpr: f64,
    ccwm: f64,

    unitless_epa: f64,
    norm_epa: f64,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct MatchData {
    winner: Option<Allience>,
    predicted_winner: Option<Allience>,
    red_win_prob: f64,
    red_allience: [u32; 3],
    blue_allience: [u32; 3],
    red_score: u32,
    blue_score: u32,
    red_score_breakdown: TbaScoreBreakdown,
    blue_score_breakdown: TbaScoreBreakdown,
    predicted_red_score: f64,
    predicted_blue_score: f64,
    event: String,
    match_number: MatchNumber,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct TeamMatchReport {
    //unchanging
    pub team_number: u32,
    pub team_member: String,

    pub event: String,
    pub match_number: MatchNumber,

    pub notes: String,

    pub data: season::MatchData2024,

    //does not change but should still never be accessed
    pub team_spesific_data: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Deserialize, Serialize)]
pub struct TeamPitReport {
    team_number: u32,
    team_member: String,
    event: String,

    data: season::PitData2024,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum Complevel {
    Practice,
    Qualifier,
    Semifinal,
    Final,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MatchNumber {
    pub number: u32,
    pub level: Complevel,
}

impl MatchNumber {
    pub fn get_tba_string(&self) -> Result<String> {
        match self.level {
            Complevel::Practice => return Err(anyhow!("Practice matches are not recorded by tba")),
            Complevel::Qualifier => Ok(format!("q{}", self.number)),
            Complevel::Semifinal => Ok(format!("sf{}m1", self.number)),
            Complevel::Final => Ok(format!("f1m{}", self.number)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub enum Allience {
    RED,
    BLUE,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Eventdata {
    key: String,
    name: String,
}
