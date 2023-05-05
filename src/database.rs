use std::env;
use std::fmt::Display;

use mongodb::{ bson::oid::ObjectId, bson::doc, options::ClientOptions, Client };

pub struct PoolerResult {
    pooler_id: ObjectId,
    pooler_name: String,

    match_ids: Vec<String>,
    results: Vec<i8>,
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

pub async fn fetch_results() -> Option<impl Iterator<Item=PoolerResult>> {
    //TODO: Need to find the actual data, mocked for now
    let temp = vec![ PoolerResult {
        pooler_id: ObjectId::new(),
        pooler_name: "".to_string(),
        match_ids: vec![],
        results: vec![]
    }];

    Some(temp.into_iter())
}
