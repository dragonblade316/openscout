use anyhow::*;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Allience, Eventdata, MatchNumber};

#[derive(Clone)]
pub struct TheBlueAllience {
    client: reqwest::Client,
    key: String,

    //TODO: replace this field with arguments
    tba_event_name: String,
}

impl TheBlueAllience {
    pub async fn new(key: String) -> Result<Self> {
        let tba = Self {
            client: reqwest::Client::new(),
            //why is this the idaho event key
            tba_event_name: "2024idbo".to_string(),
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

    pub async fn get_team_data(&self, team_num: u32) -> Result<TbaTeamdata> {
        info!("requesting opr data from tba");
        let opr_request = self
            .client
            .get(format!(
                "https://www.thebluealliance.com/api/v3/event/{}/oprs",
                self.tba_event_name
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
                .ok_or(anyhow!("team not here lol"))?
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

    pub async fn get_match_data(&self, match_number: MatchNumber) -> Result<TbaMatchData> {
        let match_key = format!(
            "{}{}",
            self.tba_event_name,
            match match_number {
                MatchNumber::Qualifier(num) => format!("_qm{}", num),
                _ => "".to_string(),
            }
        );

        let match_request = self
            .client
            .get(format!(
                "https://www.thebluealliance.com/api/v3/match/{}/",
                self.tba_event_name
            ))
            .header("X-TBA-Auth-Key", &self.key)
            .send()
            .await?
            .error_for_status()?
            .json::<TbaSerdeMatchBreakDown>()
            .await?;

        Ok(TbaMatchData {
            match_number,
            winning_allience: Some(Allience::RED),
            red_allience: [1, 2, 3],
            blue_allience: [1, 2, 3],
            red_score: match_request.alliences.red.score,
            blue_score: match_request.alliences.blue.score,
            red_score_breakdown: match_request.score_breakdown.red,
            blue_score_breakdown: match_request.score_breakdown.blue,
        })
    }

    pub async fn get_event_list(&self) -> Result<Vec<Eventdata>> {
        let event_request = self
            .client
            //TODO: set a single place to define the year
            .get(format!(
                "https://www.thebluealliance.com/api/v3/events/2024",
            ))
            .header("X-TBA-Auth-Key", &self.key)
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Eventdata>>()
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
    match_number: MatchNumber,
    //None if tie? TODO: figure this out
    winning_allience: Option<Allience>,
    red_allience: [u32; 3],
    blue_allience: [u32; 3],
    red_score: u32,
    blue_score: u32,
    red_score_breakdown: TbaScoreBreakdown,
    blue_score_breakdown: TbaScoreBreakdown,
}

#[derive(Debug, Serialize, Deserialize)]
struct Oprs {
    oprs: HashMap<String, f64>,
    dprs: HashMap<String, f64>,
    ccwms: HashMap<String, f64>,
}

///A intermidiary struct to
#[derive(Debug, Serialize, Deserialize)]
struct TbaSerdeMatchBreakDown {
    alliences: TbaSerdeAlliences,
    winning_alliance: String,
    score_breakdown: TbaSerdeScoreBreakdowns,
}

#[derive(Debug, Serialize, Deserialize)]
struct TbaSerdeAlliences {
    red: TbaSerdeAllience,
    blue: TbaSerdeAllience,
}

#[derive(Debug, Serialize, Deserialize)]
struct TbaSerdeAllience {
    score: u32,
    team_keys: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
struct TbaSerdeScoreBreakdowns {
    red: TbaScoreBreakdown,
    blue: TbaScoreBreakdown,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct TbaScoreBreakdown {
    auto_points: u32,
    teleop_points: u32,
    adjust_points: u32,
    foul_points: u32,
    //this is not every points field provided by the tba api but they seem to be the most useful
}
