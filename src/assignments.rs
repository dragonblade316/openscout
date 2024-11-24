use anyhow::*;
use log::*;

pub struct Assignment {
    position: Position,
    team_number: u32,
}

pub enum Position {
    RED1,
    RED2,
    RED3,
    BLUE1,
    BLUE2,
    BLUE3,
}

pub struct GameManager {
    event: String,

    //idk if this will be used or if it will just be included in the event string
    field: String,

    current_match: Match,

    client: reqwest::Client,
    apikey: String,
}

impl GameManager {
    ///constructs the game manager
    pub async fn new(event: String, apikey: String) -> Result<Self> {
        //check that the first api works and that the event exists.

        let gm = Self {
            event,
            field: "".to_string(),
            current_match: Match::new(0),
            client: reqwest::Client::new(),
            apikey: format!("Basic {}", apikey),
        };
        gm.client
            .get("https://frc-api.firstinspires.org/status")
            .header("Authorization", &gm.apikey)
            .send()
            .await?
            .error_for_status()?;

        info!("Gamemanager started");

        Ok(gm)
    }
    ///internal function to check if current match is still valid
    async fn check_match(&mut self) {
        //gets a list of the matches and searches for the last sceduled match with a start time.
        //Then check how long that match has been running for. If it has been running longer then a
        //spesified time, then consider it dead and move to the next match.
        //
        //
        //https://frc-api.firstinspires.org/v3.0/:season/matches/:eventCode?tournamentLevel=&teamNumber=&matchNumber=&start=&end=
        //
        //https://frc-api-docs.firstinspires.org/#733f4607-ab40-4e00-b3e1-36cfb1a2e77e
        //
        //keep a cached match start time for 15 seconds so there is no risk of being rate limited
        //when all the scouts start new rounds.
    }
    pub async fn get_assignment(&self) {}
}

///This struct will be used to track what positions still need to be filled
struct Match {
    match_number: i32,
}

impl Match {
    fn new(match_number: i32) -> Self {
        Self { match_number }
    }
}
