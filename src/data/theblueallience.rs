use anyhow::*;
use core::time;
use log::info;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, iter::zip};
use utoipa::ToSchema;

use crate::data::Complevel;

use super::{Allience, Eventdata, MatchNumber};

#[derive(Clone)]
pub struct TheBlueAllience {
    client: reqwest::Client,
    key: String,
}

impl TheBlueAllience {
    pub async fn new(key: String) -> Result<Self> {
        let tba = Self {
            client: reqwest::Client::new(),
            key,
        };
        tba.check().await?;

        Ok(tba)
    }

    ///Checks if the TBA api is working
    pub async fn check(&self) -> anyhow::Result<()> {
        let reqest = self
            .client
            .get("https://www.thebluealliance.com/api/v3/status")
            .header("X-TBA-Auth-Key", &self.key)
            .send()
            .await;

        reqest?.error_for_status()?;

        Ok(())
    }

    pub async fn get_team_data(&self, team_num: u32, event: String) -> Result<TbaTeamdata> {
        info!("requesting opr data from tba");
        let opr_request = self
            .client
            .get(format!(
                "https://www.thebluealliance.com/api/v3/event/{}/oprs",
                event
            ))
            .header("X-TBA-Auth-Key", &self.key)
            .send()
            .await?
            .error_for_status()?
            .json::<Oprs>()
            .await?;

        info!("recived opr data from tba");

        Ok(TbaTeamdata {
            team_num,
            opr: opr_request
                .oprs
                .get(&format!("frc{}", team_num))
                .ok_or(anyhow!("team not at this event"))?
                .clone(),
            dpr: opr_request
                .dprs
                .get(&format!("frc{}", team_num))
                .ok_or(anyhow!("team not here lol"))?
                .clone(),
            ccwm: opr_request
                .ccwms
                .get(&format!("frc{}", team_num))
                .ok_or(anyhow!("I dont think its possible to even trigger this"))?
                .clone(),
        })
    }

    pub async fn get_match_data(
        &self,
        match_number: MatchNumber,
        event: String,
    ) -> Result<TbaMatchData> {
        let match_key = format!("{}_{}", event, match_number.get_tba_string()?);

        let match_request = self
            .client
            .get(format!(
                "https://www.thebluealliance.com/api/v3/match/{}",
                match_key
            ))
            .header("X-TBA-Auth-Key", &self.key)
            .send()
            .await?
            .error_for_status()?
            .json::<TbaSerdeMatchBreakDown>()
            .await?;

        Ok(TbaMatchData {
            match_number,
            winning_allience: match match_request.winning_alliance.as_str() {
                "red" => Some(Allience::RED),
                "blue" => Some(Allience::BLUE),
                _ => None,
            },
            red_allience: match_request.alliances.red.get_team_nums(),
            blue_allience: match_request.alliances.blue.get_team_nums(),
            red_score: match_request.alliances.red.score,
            blue_score: match_request.alliances.blue.score,
            red_score_breakdown: match_request.score_breakdown.red,
            blue_score_breakdown: match_request.score_breakdown.blue,
            time: match_request.time,
            predicted_time: match_request.predicted_time,
            actual_time: match_request.actual_time,
        })
    }

    pub async fn get_match_data_list(&self, event: String) -> Result<Vec<TbaMatchData>> {
        let matches_request = self
            .client
            .get(format!(
                "https://www.thebluealliance.com/api/v3/{}/matches/",
                event
            ))
            .header("X-TBA-Auth-Key", &self.key)
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<TbaSerdeMatchBreakDown>>()
            .await?;

        let mut result = Vec::new();

        for i in matches_request {
            let match_number = MatchNumber {
                number: i.match_number,

                //TODO: I don't know if this method of getting the match number is correct
                level: match i.comp_level.as_str() {
                    "p" => crate::data::Complevel::Practice,
                    "qm" => Complevel::Qualifier,
                    "sf" => Complevel::Semifinal,
                    "f" => Complevel::Final,
                    _ => return Err(anyhow!("complevel pattern not matched")),
                },
            };

            result.push(TbaMatchData {
                match_number,
                winning_allience: match i.winning_alliance.as_str() {
                    "red" => Some(Allience::RED),
                    "blue" => Some(Allience::BLUE),
                    _ => None,
                },
                red_allience: i.alliances.red.get_team_nums(),
                blue_allience: i.alliances.blue.get_team_nums(),
                red_score: i.alliances.red.score,
                blue_score: i.alliances.blue.score,
                red_score_breakdown: i.score_breakdown.red,
                blue_score_breakdown: i.score_breakdown.blue,
                time: i.time,
                predicted_time: i.predicted_time,
                actual_time: i.actual_time,
            });
        }

        Ok(result)
    }

    pub async fn get_event_list(&self) -> Result<Vec<Eventdata>> {
        let event_request = self
            .client
            .get(format!(
                "https://www.thebluealliance.com/api/v3/events/{}",
                env!("CARGO_PKG_VERSION")
                    .split(".")
                    .into_iter()
                    .next()
                    .expect("This error will never trigger since the first number if the version will always be the year")
            ))
            .header("X-TBA-Auth-Key", &self.key)
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Eventdata>>()
            .await?;

        Ok(event_request)
    }

    pub async fn get_event_keys(&self) -> Result<Vec<String>> {
        let event_request = self
            .client
            .get(format!(
                "https://www.thebluealliance.com/api/v3/events/{}/keys",
                env!("CARGO_PKG_VERSION")
                    .split(".")
                    .into_iter()
                    .next()
                    .expect("This error will never trigger since the first number if the version will always be the year")
            ))
            .header("X-TBA-Auth-Key", &self.key)
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<String>>()
            .await?;

        Ok(event_request)
    }
}

pub struct TbaTeamdata {
    pub team_num: u32,
    pub opr: f64,
    pub dpr: f64,
    pub ccwm: f64,
}

pub struct TbaMatchData {
    pub match_number: MatchNumber,
    pub winning_allience: Option<Allience>,
    pub red_allience: [u32; 3],
    pub blue_allience: [u32; 3],
    pub red_score: u32,
    pub blue_score: u32,
    pub red_score_breakdown: TbaScoreBreakdown,
    pub blue_score_breakdown: TbaScoreBreakdown,
    pub time: u64,
    pub actual_time: u64,
    pub predicted_time: u64,
}
#[allow(nonstandard_style)]
#[derive(Debug, Serialize, Deserialize)]
struct Oprs {
    oprs: HashMap<String, f64>,
    dprs: HashMap<String, f64>,
    ccwms: HashMap<String, f64>,
}
#[allow(nonstandard_style)]
///A intermidiary struct to
#[derive(Debug, Serialize, Deserialize)]
struct TbaSerdeMatchBreakDown {
    match_number: u32,
    comp_level: String,
    alliances: TbaSerdeAlliences,
    winning_alliance: String,
    score_breakdown: TbaSerdeScoreBreakdowns,
    time: u64,
    actual_time: u64,
    predicted_time: u64,
}
#[allow(nonstandard_style)]
#[derive(Debug, Serialize, Deserialize)]
struct TbaSerdeAlliences {
    red: TbaSerdeAllience,
    blue: TbaSerdeAllience,
}
#[allow(nonstandard_style)]
#[derive(Debug, Serialize, Deserialize)]
struct TbaSerdeAllience {
    score: u32,
    team_keys: Vec<String>,
}

impl TbaSerdeAllience {
    fn get_team_nums(&self) -> [u32; 3] {
        let mut numbers: [u32; 3] = [0; 3];

        //why is this 0..3???
        //idk but it works so who am i to question how the iterator works
        for (i, j) in zip(0..3, &self.team_keys) {
            numbers[i] = j
                .chars()
                .into_iter()
                .filter(|&n| n.is_numeric())
                .collect::<String>()
                .parse::<u32>()
                .expect("this should be a number")
        }

        numbers
    }
}
#[allow(nonstandard_style)]
#[derive(Debug, Serialize, Deserialize)]
struct TbaSerdeScoreBreakdowns {
    red: TbaScoreBreakdown,
    blue: TbaScoreBreakdown,
}

#[allow(nonstandard_style)]
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TbaScoreBreakdown {
    autoPoints: u32,
    teleopPoints: u32,
    adjustPoints: u32,
    foulPoints: u32,
    //this is not every points field provided by the tba api but they seem to be the most useful
}
