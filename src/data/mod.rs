pub mod openscout;
pub mod os_data; //data structs
pub mod statbotics;
pub mod theblueallience;

use anyhow::*;
use log::warn;
use os_data::TeamMatchReport;
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
}

impl DataManager {
    pub async fn new(tba_key: String) -> Result<Self> {
        Ok(Self {
            openscoutdb: openscout::OpenScoutDB::new().await?,
            tba: TheBlueAllience::new(tba_key).await?,
            statbotics: Statbotics::new().await?,
        })
    }

    pub async fn getTeamData(&self, team_number: u32) -> Result<TeamData> {
        let tba_data = self.tba.get_team_data(team_number.clone()).await?;
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

    pub async fn get_team_match_data(
        &self,
        team_number: u32,
        match_number: MatchNumber,
    ) -> Result<TeamMatchReport> {
        self.openscoutdb
            .get_team_match_data(team_number, match_number)
            .await
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

#[derive(Debug, Serialize, Deserialize)]
pub enum MatchNumber {
    Practice(u32),
    Qualifier(u32),
    Playoff(u32),
    Final(u32),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Allience {
    RED,
    BLUE,
}

pub enum WinningAllience {
    RED,
    BLUE,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Eventdata {
    key: String,
    name: String,
}
