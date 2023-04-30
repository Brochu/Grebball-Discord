use dotenv::dotenv;
use std::env;

use serde_json::Value;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
//use serenity::model::id::GuildId;
//use serenity::model::prelude::interaction::{ Interaction, InteractionResponseType };
use serenity::prelude::*;

mod commands;
mod football;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!hello" {
            let reply = format!("Hi, {}!", msg.author.name);
            if let Err(reason) = msg.channel_id.say(&ctx.http, reply).await {
                println!("![Handler] Handler message error : {:?}", reason);
            }
        }

        if msg.content.starts_with("!week") {
            let (_, arg) = msg.content.split_once(" ").unwrap();

            let mut url = "https://www.thesportsdb.com/api/v1/json/3/eventsround.php?id=4391&s=2022".to_string();
            url.push_str("&r=");
            url.push_str(arg);

            let res = reqwest
                ::get(url).await.expect("[Handler] Could not get reply")
                .text().await.unwrap();

            let val: Value = serde_json::from_str(res.as_str()).expect("[Handler] Could not parse response");
            if let Value::Object(o) = &val {
                let events = o.get("events").unwrap();

                let output = if let Value::Array(matches) = events {
                    matches.iter().fold(String::new(), |mut out, m| {
                        let ateam = football::get_short_name(m["strAwayTeam"].as_str().unwrap());
                        let hteam = football::get_short_name(m["strHomeTeam"].as_str().unwrap());

                        let aemoji = football::get_team_emoji(ateam.as_str());
                        let hemoji = football::get_team_emoji(hteam.as_str());

                        out.push_str(format!("<:{}:{}> <:VS:1102123108187525130> <:{}:{}>\n",
                            ateam, aemoji,
                            hteam, hemoji
                        ).as_str());
                        out
                    })
                } else {
                    String::new()
                };

                if let Err(reason) = msg.channel_id.say(&ctx.http, output.as_str()).await {
                    println!("![Handler] Handler message error : {:?}", reason);
                }
            }
        }
    }

    //TODO: Handle slash commands
    //async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    //}

    async fn ready(&self, _: Context, ready: Ready) {
        println!("[Handler] ({}) {} is connected w/ version: {}",
            ready.user.id,
            ready.user.name,
            ready.version
        );

        //TODO: Create the commands needed
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok(); // Include .env file to environment

    let token = env::var("DISCORD_TOKEN").expect("[MAIN] Cannot find 'DISCORD_TOKEN' in env");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("[MAIN] Could not create client");

    if let Err(reason) = client.start().await {
        println!("![MAIN] Client error : {:?}", reason);
    }
}
