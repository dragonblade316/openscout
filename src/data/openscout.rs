use anyhow::*;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{
    super::{TeamMatchReport, TeamPitReport},
    Complevel, MatchNumber,
};
use mongodb::{self, bson::doc, options::Credential, Collection};
use mongodb::{
    options::{ClientOptions, ServerApi, ServerApiVersion},
    Client,
};

#[derive(Clone)]
pub struct OpenScoutDB {
    db: mongodb::Client,
    match_collection: Collection<TeamMatchReport>,
    pit_collection: Collection<TeamPitReport>,
    auth_collection: Collection<Auth>,
}

impl OpenScoutDB {
    pub async fn new(url: Option<String>, auth: Option<MongoAuth>) -> Result<Self> {
        // Replace the placeholder with your Atlas connection string
        let uri = match url {
            Some(url) => url,
            None => "mongodb://localhost:27017".to_string(),
        };
        let mut client_options = ClientOptions::parse(uri).await?;

        if let Some(auth) = auth {
            let default_cred = Credential::builder()
                .username(auth.username)
                .password(auth.password)
                .source("main".to_string())
                .build();
            client_options.credential = Some(default_cred);
        }
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
        let auth_collection: Collection<Auth> = client.database("main").collection("auth");

        Ok(Self {
            db: client,
            match_collection,
            pit_collection,
            auth_collection,
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
        let (name, num) = match match_number.level {
            Complevel::Practice => ("Practice", match_number.number),
            Complevel::Qualifier => ("Qualifier", match_number.number),
            Complevel::Semifinal => ("Semifinal", match_number.number),
            Complevel::Final => ("Final", match_number.number),
        };

        let data = self
            .match_collection
            .find_one(doc! {"$and": vec![
            doc! {"team_number": team},
            doc! {"match_number.number": num},
            doc! {"match_number.level": name},
            doc! {"event": event},
            ]})
            .await?;

        match data {
            Some(data) => Ok(data),
            None => Err(anyhow!(StatusCode::NO_CONTENT)),
        }
    }

    //TODO: check if there is data here and return the appropriet status code if not
    pub async fn get_team_pit_data(&self, team: u32, event: String) -> Result<TeamPitReport> {
        let data = self
            .pit_collection
            .find_one(doc! {"$and": vec![
            doc! {"team_number": team},
            doc! {"event": event}
            ]})
            .await?;

        match data {
            Some(data) => Ok(data),
            None => Err(anyhow!(StatusCode::NO_CONTENT)),
        }
    }

    pub async fn check_auth(&self, team: u32) -> Result<Auth> {
        let data = self.auth_collection.find(doc! {"_id": team}).await?;

        Ok(data.deserialize_current()?)
    }

    pub async fn add_auth(&self, auth: Auth) -> Result<()> {
        self.auth_collection.insert_one(auth).await?;
        Ok(())
    }
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Auth {
    pub _id: u32,
    pub key: String,
    pub auth: AuthLevel,
}
#[derive(PartialEq, Eq, PartialOrd, Debug, Serialize, Deserialize, ToSchema)]
pub enum AuthLevel {
    ADMIN,
    TEAM,
}

impl AuthLevel {
    fn index(&self) -> u8 {
        match self {
            AuthLevel::ADMIN => 0,
            AuthLevel::TEAM => 1,
        }
    }
}

impl Ord for AuthLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index().cmp(&other.index())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MongoAuth {
    username: String,
    password: String,
}
