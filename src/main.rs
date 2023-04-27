use dotenv::dotenv;
use std::env;

use serenity::async_trait;
use serenity::model::application::interaction::{ Interaction, InteractionResponseType };
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;

mod commands;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(cmd) = interaction {
            println!("[EVT] Got command interaction: {:#?}", cmd);

            let content = match cmd.data.name.as_str() {
                "ping" => commands::ping::run(&cmd.data.options),
                _ => "[EVT] Command not implemented".to_string()
            };

            if let Err(reason) = cmd.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|msg| msg.content(content))
                }).await
            {
                println!("[EVT] Cannot reply to slash command: {}", reason);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId(env::var("GUILD_ID")
            .expect("[EVT] Could not find GUILD_ID env var")
            .parse()
            .expect("[EVT] GUILD_ID needs to be an integer")
        );

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |cmds| {
            cmds
                .create_application_command(|cmd| commands::ping::register(cmd))
        }).await;

        println!("[EVT] I now have access to the following slash commands: {:#?}", commands);
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("[ENV] Need to set DISCORD_TOKEN env var.");

    let mut client = Client::builder(&token, GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("[CLIENT] Could not create client.");

    if let Err(reason) = client.start().await {
        println!("[MAIN] Client error: {:?}", reason);
    }
}
