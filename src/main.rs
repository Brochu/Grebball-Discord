use dotenv::dotenv;
use std::env;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {

            if let Err(reason) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("[EVT] Error sending response: {:?}", reason);
            }
        }

        if msg.content.starts_with("!week") {
            let (_, arg) = msg.content.split_once(" ").expect("[EVT] Could not split");

            if let Err(reason) = msg.channel_id.say(&ctx.http, arg).await {
                println!("[EVT] Error sending response: {:?}", reason);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("[ENV] Need to set DISCORD_TOKEN env var.");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("[CLIENT] Could not create client.");

    if let Err(reason) = client.start().await {
        println!("[MAIN] Client error: {:?}", reason);
    }
}
