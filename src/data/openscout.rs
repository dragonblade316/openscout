use anyhow::*;

use super::{
    super::{TeamMatchReport, TeamPitReport},
    MatchNumber,
};
use mongodb::{self, bson::doc, Collection};
use mongodb::{
    options::{ClientOptions, ServerApi, ServerApiVersion},
    Client,
};

#[derive(Clone)]
pub struct OpenScoutDB {
    db: mongodb::Client,
    match_collection: Collection<TeamMatchReport>,
    pit_collection: Collection<TeamPitReport>,
}
//TODO: going to need to figure out if the database will just be one big data base or per event db
impl OpenScoutDB {
    pub async fn new() -> Result<Self> {
        // Replace the placeholder with your Atlas connection string
        let uri = "mongodb://localhost:27017";
        let mut client_options = ClientOptions::parse(uri).await?;
        // Set the server_api field of the client_options object to Stable API version 1
        let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
        client_options.server_api = Some(server_api);
        // Create a new client and connect to the server
        let client = Client::with_options(client_options)?;
        // Send a ping to confirm a successful connection
        client
            .database("main")
            .run_command(doc! { "ping": 1 })
            .await?;
        println!("Pinged your deployment. You successfully connected to MongoDB!");

        let match_collection: Collection<TeamMatchReport> =
            client.database("main").collection("match");
        let pit_collection: Collection<TeamPitReport> = client.database("main").collection("pit");

        Ok(Self {
            db: client,
            match_collection,
            pit_collection,
        })
    }

    //ngl this was easier than expected
    pub async fn post_team_match_data(&self, data: TeamMatchReport) -> Result<()> {
        self.match_collection.insert_one(data).await?;
        Ok(())
    }

    pub async fn post_team_pit_data(&self, data: TeamPitReport) -> Result<()> {
        self.pit_collection.insert_one(data).await?;
        Ok(())
    }

    pub async fn get_team_match_data(
        &self,
        team: u32,
        match_number: MatchNumber,
        event: String,
    ) -> Result<TeamMatchReport> {
        let (name, num) = match match_number {
            MatchNumber::Practice(num) => ("Practice", num),
            MatchNumber::Qualifier(num) => ("Qualifier", num),
            MatchNumber::Semifinal(num) => ("Semifinal", num),
            MatchNumber::Final(num) => ("Final", num),
        };

        let data = self
            .match_collection
            .find(doc! {"$and": vec![
            doc! {"team_number": team},
            doc! {format!("match_number.{}", name): num},
            doc! {"event": event},
            ]})
            .await?;

        let close = data.deserialize_current()?;

        Ok(close)
    }

    pub async fn get_team_pit_data(&self, team: u32, event: String) -> Result<TeamPitReport> {
        let data = self
            .pit_collection
            .find(doc! {"$and": vec![
            doc! {"team_number": team},
            doc! {"event": event}
            ]})
            .await?;

        let close = data.deserialize_current()?;

        Ok(close)
    }
}
