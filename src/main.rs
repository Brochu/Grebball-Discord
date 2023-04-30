use dotenv::dotenv;
use std::env;

use serde_json::Value;

use serenity::async_trait;
use serenity::model::application::interaction::{ Interaction, InteractionResponseType };
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
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
                ::get(url).await.expect("![Handler] Could not get reply")
                .text().await.unwrap();

            let val: Value = serde_json::from_str(res.as_str()).expect("![Handler] Could not parse response");
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

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(cmd) = interaction {
            println!("[Handler] Got command interaction: {:#?}", cmd);

            let reply = match cmd.data.name.as_str() {
                "ping" => commands::ping::run(&cmd.data.options),
                "week" => commands::week::run(&cmd.data.options),
                _ => "Command not implemented!".to_string(),
            };

            if let Err(reason) = cmd.create_interaction_response(&ctx.http, |res| {
                res
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|m| m.content(reply))
            })
            .await {
                println!("![Handler] Cannot respond to slash command : {:?}", reason);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("[Handler] ({}) {} is connected w/ version: {}",
            ready.user.id,
            ready.user.name,
            ready.version
        );

        let guild_id = GuildId(env::var("GUILD_ID")
            .expect("![Handler] Could not find env var 'GUILD_ID'")
            .parse()
            .expect("![Handler] Could not parse guild_id to int")
        );

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |cmds| {
            cmds
                .create_application_command(|cmd| commands::ping::register(cmd))
                .create_application_command(|cmd| commands::week::register(cmd))
        }).await;

        println!("Here are the available commands: {:#?}", commands);
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok(); // Include .env file to environment

    let token = env::var("DISCORD_TOKEN").expect("![MAIN] Cannot find 'DISCORD_TOKEN' in env");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("![MAIN] Could not create client");

    if let Err(reason) = client.start().await {
        println!("![MAIN] Client error : {:?}", reason);
    }
}
