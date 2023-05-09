use std::env;
use std::fmt::Display;

use mongodb::{ bson::oid::ObjectId, bson::doc, options::ClientOptions, Client };
use serde::Deserialize;
//use crate::football;

pub struct PoolerResult {
    pub pooler_id: ObjectId,
    pub pooler_name: String,

    pub match_ids: Vec<String>,
    pub results: Vec<i8>,
}

impl Display for PoolerResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = format!("[{}] {}\n", self.pooler_id, self.pooler_name);

        self.match_ids.iter().enumerate().for_each(|(idx, mid)| {
            out.push_str(format!("\t{}: {}\n", mid, self.results[idx]).as_str());
        });

        writeln!(f, "{out}")
    }
}

#[derive(Deserialize)]
struct Pooler {
    _id: ObjectId,
    name: String,
    favTeam: String,
    pool_id: ObjectId,
    user_id: ObjectId,
}

impl Display for Pooler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[{}] {} - ({})\n\tpool: {}; user: {}\n",
            self._id, self.name, self.favTeam,
            self.pool_id, self.user_id
        )
    }
}

pub async fn ping() {
    let uri = env::var("MONGDO_URI")
        .expect("![Results] Could not find 'MONGDO_URI' env var");
    let client_opts = ClientOptions::parse(uri).await
        .expect("![Results] Could not parse MongoDB connect info");

    let client = Client::with_options(client_opts)
        .expect("![Results] Could not connect to MongoDB");
    let result = client.database("pool_football_app_dev")
        .run_command(doc! { "ping": 1 }, None).await.unwrap();

    println!("Ping result: {:#?}", result);
}

pub async fn fetch_results(week: u64) -> Option<impl Iterator<Item=PoolerResult>> {
    //let matches = football::get_week(week);
    println!("For week: {}", week);

    let uri = env::var("MONGDO_URI")
        .expect("![Results] Could not find 'MONGDO_URI' env var");
    let client_opts = ClientOptions::parse(uri).await
        .expect("![Results] Could not parse MongoDB connect info");

    let client = Client::with_options(client_opts)
        .expect("![Results] Could not connect to MongoDB");
    let result = client.database("pool_football_app_dev")
        .collection::<Pooler>("poolers").find_one(None, None)
        .await.expect("![Results] Could not find all poolers");

    if let Some(pooler) = result {
        println!("{}", pooler);
    }

    let temp = vec![ PoolerResult {
        pooler_id: ObjectId::new(),
        pooler_name: "".to_string(),
        match_ids: vec![],
        results: vec![]
    }];

    Some(temp.into_iter())
}
