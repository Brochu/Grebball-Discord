use dotenv::dotenv;
use std::env;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
//use serenity::model::id::GuildId;
//use serenity::model::prelude::interaction::{ Interaction, InteractionResponseType };
use serenity::prelude::*;

mod commands;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            let mut reply = "pong!".to_string();
            reply.push_str(" to ");
            reply.push_str(msg.author.name.as_str());

            if let Err(reason) = msg.channel_id.say(&ctx.http, reply.as_str()).await {
                println!("![Handler] Handler message error : {:?}", reason);
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
