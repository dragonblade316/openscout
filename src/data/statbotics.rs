use anyhow::*;
use serde::*;

use super::{Allience, MatchNumber};

#[derive(Debug, Clone)]
pub struct Statbotics {
    client: reqwest::Client,
}

impl Statbotics {
    pub async fn new() -> Result<Self> {
        let client = reqwest::Client::new();
        client
            .get("https://api.statbotics.io/v3/")
            .send()
            .await?
            .error_for_status()?;

        Ok(Self { client })
    }

    pub async fn get_team_data(&self, team_num: u32) -> Result<StatboticsTeamData> {
        let epa_request = self
            .client
            .get(format!("https://api.statbotics.io/v3/team_year/{}/2024", {
                team_num
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<StatboticsTeamData>()
            .await?;

        Ok(epa_request)
    }

    pub async fn get_match_data(
        &self,
        event: String,
        match_number: MatchNumber,
    ) -> Result<StatboticsMatchData> {
        let event_match = format!("{}_{}", event, match_number.get_tba_string()?);

        let request = self
            .client
            .get(format!(
                "https://api.statbotics.io/v3/match/{}",
                event_match
            ))
            .send()
            .await?
            .error_for_status()?
            .json::<StatboticsMatchData>()
            .await?;

        Ok(request)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatboticsTeamData {
    pub team: String,
    pub epa: EPA,
}

#[allow(nonstandard_style)]
#[derive(Debug, Serialize, Deserialize)]
pub struct StatboticsMatchData {
    pub pred: StatboticsPrediction,
    pub result: StatboticsResult,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EPA {
    //total points needed maybe
    pub unitless: f64,
    pub norm: f64,
    //conf needed maybe

    //I am NOT doing the point breakdown. not only is it season spesific, but the tba
    //scorebreakdown was already hard enough.
}
#[allow(nonstandard_style)]
#[derive(Debug, Serialize, Deserialize)]
pub struct StatboticsPrediction {
    pub winner: Option<Allience>, //might need to figure this out.
    pub red_win_prob: f64,
    pub red_score: f64,
    pub blue_score: f64,
}

#[allow(nonstandard_style)]
#[derive(Debug, Serialize, Deserialize)]
pub struct StatboticsResult {
    pub winner: Option<Allience>,
    pub red_score: f64,
    pub blue_score: f64,
    pub red_no_foul: f64,
    pub blue_no_foul: f64,
    //there are more fields but they are mostly handled by the tba api so I'm going to ignore them
}
