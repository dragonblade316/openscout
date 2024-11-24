pub mod openscout;
pub mod season; //data structs
pub mod statbotics;
pub mod theblueallience;
use schemars::{schema_for, JsonSchema};

use std::collections::HashMap;

use anyhow::*;
use serde::{Deserialize, Serialize};
use statbotics::Statbotics;
use theblueallience::TheBlueAllience;

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
    pub async fn new(tba_key: String) -> Result<Self> {
        Ok(Self {
            openscoutdb: openscout::OpenScoutDB::new().await?,
            tba: TheBlueAllience::new(tba_key).await?,
            statbotics: Statbotics::new().await?,
        })
    }

    pub async fn getTeamData(&self, team_number: u32, event: String) -> Result<TeamData> {
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

    pub async fn postTeamMatchData(&self, data: TeamMatchReport) -> Result<()> {
        self.openscoutdb.post_team_match_data(data).await?;
        Ok(())
    }

    pub async fn post_team_pit_data(&self, data: TeamPitReport) -> Result<()> {
        self.openscoutdb.post_team_pit_data(data);
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

pub struct MatchData {}

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

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub enum MatchNumber {
    Practice(u32),
    Qualifier(u32),
    Semifinal(u32),
    Final(u32),
}

impl MatchNumber {
    pub fn get_tba_string(&self) -> Result<String> {
        match self {
            Self::Practice(_num) => {
                return Err(anyhow!("Practice matches are not recorded by tba"))
            }
            Self::Qualifier(num) => Ok(format!("q{}", num)),
            Self::Semifinal(num) => Ok(format!("sf{}m1", num)),
            Self::Final(num) => Ok(format!("f{}", num)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Allience {
    RED,
    BLUE,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Eventdata {
    key: String,
    name: String,
}
