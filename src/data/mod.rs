pub mod openscout;
pub mod season; //data structs
pub mod statbotics;
pub mod theblueallience;
use axum::{http::HeaderMap, response::IntoResponse};
use log::warn;
use openscout::{Auth, AuthLevel, MongoAuth};
use rand::{prelude::Distribution, seq::IteratorRandom};
use reqwest::StatusCode;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use utoipa::ToSchema;

use std::{
    collections::{binary_heap::Iter, HashMap},
    thread::current,
};

use anyhow::*;
use serde::{Deserialize, Serialize};
use statbotics::Statbotics;
use theblueallience::{TbaScoreBreakdown, TheBlueAllience};

use crate::{assignments, get_team_pit_data};

//TODO: set a client here so that the connection pool is shared by all there services (or not, I
//don't think there would be a benifit to this)

#[derive(Clone)]
pub struct DataManager {
    openscoutdb: openscout::OpenScoutDB,
    tba: theblueallience::TheBlueAllience,
    statbotics: statbotics::Statbotics,
    event_list: Vec<String>,
    //TODO: I may want to add a flag that enables event checks. This would be for scenarios where
    //the event is not defined such as scrimiges
    enable_auth: bool,
    enable_event_check: bool,
    global_match_assignment: HashMap<String, MatchScoutAssignments>,
    team_match_assignments: HashMap<(u32, String), MatchScoutAssignments>,
}

impl DataManager {
    pub async fn new(
        tba_key: String,
        mongo_auth: Option<MongoAuth>,
        enable_auth: Option<bool>,
    ) -> Result<Self> {
        let tba = TheBlueAllience::new(tba_key).await?;
        let event_keys = tba.get_event_keys().await?;

        Ok(Self {
            openscoutdb: openscout::OpenScoutDB::new(None, mongo_auth).await?,
            tba,
            statbotics: Statbotics::new().await?,
            event_list: event_keys,
            enable_auth: enable_auth.unwrap_or(true),
            enable_event_check: true, //TODO: put this in the config
            global_match_assignment: HashMap::new(),
            team_match_assignments: HashMap::new(),
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

        Ok(MatchData {
            winner: tba_data.winning_allience,
            predicted_winner: statbotics_data.pred.winner,
            red_win_prob: statbotics_data.pred.red_win_prob,
            //TODO: there appears to be a bug where the thrid alliance is 0
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
        })
    }

    pub async fn post_team_match_data(&self, data: TeamMatchReport) -> Result<()> {
        self.check_event_key(&data.event)?;
        self.openscoutdb.post_team_match_data(data).await?;
        Ok(())
    }

    pub async fn post_team_pit_data(&self, data: TeamPitReport) -> Result<()> {
        self.check_event_key(&data.event)?;
        self.openscoutdb.post_team_pit_data(data).await?;
        Ok(())
    }

    ///Gives the last recorded team match report
    pub async fn get_last_team_match_data(
        &self,
        team_number: u32,
        match_number: MatchNumber,
        event: String,
    ) -> Result<TeamMatchReport> {
        self.openscoutdb
            .get_team_match_data(team_number, match_number, event)
            .await
    }

    ///
    pub async fn get_avg_team_match_data(
        &self,
        team_number: u32,
        match_number: MatchNumber,
        event: String,
    ) {
    }

    pub async fn get_all_team_match_data_by_team(
        &self,
        team_number: u32,
        recording_team: u32,
        match_number: MatchNumber,
        event: String,
    ) {
        self.openscoutdb.get_all_team_match_data_by_team(
            team_number,
            recording_team,
            match_number,
            event,
        );
    }

    pub async fn get_team_match_data_by_induvidual(
        &self,
        team_number: u32,
        recording_team: u32,
        recording_induvidual: u32,
        match_number: MatchNumber,
        event: String,
    ) {
    }

    pub async fn get_last_team_match_data_by_team(
        &self,
        team_number: u32,
        recording_team: u32,
        match_number: MatchNumber,
        event: String,
    ) {
    }

    pub async fn get_all_team_match_data(
        team_number: u32,
        match_number: MatchNumber,
        event: String,
    ) {
    }

    pub async fn get_last_team_pit_data(
        &self,
        team_number: u32,
        event: String,
    ) -> Result<TeamPitReport> {
        self.openscoutdb
            .get_last_team_pit_data(team_number, event)
            .await
    }

    ///
    pub async fn get_avg_team_pit_data(
        &self,
        team_number: u32,
        event: String,
    ) -> Result<TeamPitReport> {
        self.openscoutdb
            .get_avg_team_pit_data(team_number, event)
            .await
    }

    pub async fn get_all_team_pit_data_by_team(
        &self,
        team_number: u32,
        recording_team: u32,
        event: String,
    ) -> Result<Vec<TeamPitReport>> {
        todo!()
    }

    pub async fn get_team_pit_data_by_induvidual(
        &self,
        team_number: u32,
        recording_team: u32,
        recording_induvidual: u32,
        event: String,
    ) -> Result<TeamPitReport> {
        todo!()
    }

    pub async fn get_last_team_pit_data_by_team(
        &self,
        team_number: u32,
        recording_team: u32,
        event: String,
    ) -> Result<TeamPitReport> {
        todo!()
    }

    pub async fn get_all_team_pit_data(
        &self,
        team_number: u32,
        event: String,
    ) -> Result<Vec<TeamPitReport>> {
        todo!()
    }

    pub async fn get_event_data(&self) -> Result<Vec<Eventdata>> {
        self.tba.get_event_list().await
    }

    pub async fn check_auth(&self, headers: &HeaderMap, required_auth: AuthLevel) -> Result<()> {
        if !self.enable_auth {
            return Ok(());
        }

        //it is the destiny of all my codebases to have some annoying ugly as crap code to convert
        //things to the correct datatype
        //TODO: give this actual errors
        let team: u32 = headers
            .get("id")
            .unwrap_or(return Err(anyhow!(StatusCode::BAD_REQUEST)))
            .to_str()?
            .parse()?;
        let key: String = headers
            .get("key")
            .unwrap_or(return Err(anyhow!(StatusCode::BAD_REQUEST)))
            .to_str()?
            .to_string();

        let auth = self.openscoutdb.check_auth(team).await?;

        if key != auth.key || auth.auth < required_auth {
            return Err(anyhow!(StatusCode::UNAUTHORIZED));
        }

        Ok(())
    }

    ///This will be used on methods that write to the database to prevent data being uploaded with
    ///a nonexistant event (typos happen).
    fn check_event_key(&self, key: &String) -> Result<()> {
        if !self.event_list.iter().any(|k| *key == *k) && self.enable_event_check {
            return Err(anyhow!("The given event does not exist"));
        }
        Ok(())
    }

    pub async fn add_user(&self, auth: Auth) -> Result<()> {
        self.openscoutdb.add_auth(auth).await?;
        Ok(())
    }

    pub async fn get_current_match(&self, event: String) -> Result<MatchNumber> {
        todo!()
    }
    pub async fn get_global_scouting_assignment(event: String) {}

    pub async fn get_team_scouting_assignment(
        &mut self,
        event: String,
        team_number: u32,
    ) -> Result<ScoutingAssignment> {
        let mut assignments = match self
            .team_match_assignments
            .get_mut(&(team_number, event.clone()))
        {
            Some(data) => data,
            None => {
                self.team_match_assignments
                    .insert((team_number, event.clone()), MatchScoutAssignments::new());
                self.team_match_assignments
                    .get_mut(&(team_number, event.clone()))
                    .expect("I litterally just created this entry")
            }
        };

        todo!()
    }
}

#[derive(Debug, Serialize, ToSchema, Deserialize)]
pub struct TeamData {
    team_number: u32,

    opr: f64,
    dpr: f64,
    ccwm: f64,

    unitless_epa: f64,
    norm_epa: f64,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
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

#[derive(Serialize, Deserialize, ToSchema, Debug)]
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

//impl TeamMatchReport

#[derive(Deserialize, Serialize, ToSchema)]
pub struct TeamPitReport {
    team_number: u32,
    team_member: String,
    event: String,

    data: season::PitData2024,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum Complevel {
    Practice,
    Qualifier,
    Semifinal,
    Final,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MatchNumber {
    pub number: u32,
    pub level: Complevel,
}

impl MatchNumber {
    pub fn get_tba_string(&self) -> Result<String> {
        match self.level {
            Complevel::Practice => return Err(anyhow!("Practice matches are not recorded by tba")),
            Complevel::Qualifier => Ok(format!("qm{}", self.number)),
            Complevel::Semifinal => Ok(format!("sf{}m1", self.number)),
            Complevel::Final => Ok(format!("f1m{}", self.number)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub enum Allience {
    //TODO: may want to fix this patchwork solution
    #[serde(alias = "red")]
    RED,
    #[serde(alias = "blue")]
    BLUE,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Eventdata {
    key: String,
    name: String,
}

#[derive(Debug, Clone)]
struct MatchScoutAssignments {
    current_match: u32,
    matches: HashMap<u32, Match>,
}

//TODO: finish this
impl MatchScoutAssignments {
    pub fn new() -> Self {
        Self {
            current_match: 0,
            matches: HashMap::new(),
        }
    }

    pub fn get_assignment(&mut self, match_num: u32) -> Slot {
        if match_num > self.current_match {
            self.current_match = match_num;
        }

        self.matches
            .get_mut(&match_num)
            .get_or_insert(&mut Match::new())
            .get_assignment()
    }
}

#[derive(Debug, Clone)]
struct Match {
    slots: HashMap<Slot, bool>,
}

impl Match {
    fn new() -> Self {
        Self {
            slots: Slot::iter()
                .map(|s| {
                    return (s, false);
                })
                .collect(),
        }
    }

    fn get_assignment(&self) -> Slot {
        //well this is mildly cursed
        self.slots
            .iter()
            .find(|(slot, isfilled)| !isfilled.clone())
            .unwrap_or((
                &Slot::iter()
                    .choose(&mut rand::thread_rng())
                    .expect("This can not happen bc the iter is from the enum"),
                &false,
            ))
            .0
            .clone()
    }
}

#[derive(Debug, Clone, EnumIter, Hash, PartialEq, Eq)]
enum Slot {
    RED1,
    RED2,
    RED3,
    BLUE1,
    BLUE2,
    BLUE3,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
struct ScoutingAssignment {
    event: String,
    match_number: MatchNumber,
    slot: i32, //TODO: replace this with an enum
    team_number: u32,
}
