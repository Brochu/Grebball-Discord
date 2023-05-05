use std::env;
use mongodb::{ bson::doc, options::ClientOptions, Client };

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
