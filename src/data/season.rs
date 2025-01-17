//! This File will change every season to match the game.
//! This is a collection of tracked values that every scouting app really should have.
//! Values contained in the sturcts of these files should never be accessed by anything other than
//! serde.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub struct MatchData2024 {
    pub notes_speaker_auto: u32,
    pub notes_speaker_teleop: u32,
    pub notes_amp_teleop: u32,
    pub endgame: Endgame,
}

impl MatchData2024 {
    pub fn avg(data: Vec<MatchData2024>) -> MatchData2024 {
        todo!()
    }
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct PitData2024 {
    speaker: bool,
    amp: bool,
    posible_endgame: Endgame,

    drivebase: Drivebase,

    can_move_auto: bool,
    expected_notes_auto: bool,
}

// yearly support enums, do not use outside of team match report.
#[derive(Deserialize, Serialize, ToSchema, Debug)]
pub enum Endgame {
    ClimbAndTrap,
    Climb,
    Park,
    None,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub enum Drivebase {
    Differential,
    Mecanum,
    Swerve,
    Other(String),
}
