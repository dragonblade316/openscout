//This file will likely change with each releace
//
//data that is marked as changing should never be dirrectly called in code.
//instead it will only be interacted with though serde.
//I am not maintaining logic that relies on constantly changing data.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::MatchNumber;

#[derive(Serialize, Deserialize, Debug)]
pub struct TeamMatchReport {
    //unchanging
    pub team_number: u32,
    pub team_member: String,

    pub match_number: MatchNumber,

    pub notes: String,

    //changing yearly, do not access
    pub notes_speaker_auto: u32,
    pub notes_speaker_teleop: u32,
    pub notes_amp_teleop: u32,
    pub endgame: Endgame,

    //does not change but should still never be accessed
    pub team_spesific_data: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Deserialize, Serialize)]
pub struct TeamPitReport {
    team_number: u32,
    team_member: String,

    speaker: bool,
    amp: bool,
    posible_endgame: Endgame,

    drivebase: Drivebase,

    can_move_auto: bool,
    expected_notes_auto: bool,
}

// yearly support enums, do not use outside of team match report.
#[derive(Deserialize, Serialize, Debug)]
pub enum Endgame {
    ClimbAndTrap,
    Climb,
    Park,
    None,
}

#[derive(Deserialize, Serialize)]
pub enum Drivebase {
    Differential,
    Mecanum,
    Swerve,
    Other(String),
}
