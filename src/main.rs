use anyhow::anyhow;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::model::prelude::interaction::{ Interaction, InteractionResponseType };
use serenity::prelude::*;

use shuttle_secrets::SecretStore;
use tracing::{error, info};

mod commands;

struct Handler {
    guild_id: GuildId,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!hello" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "world!").await {
                error!("Error sending message: {:?}", e);
            }
        }
    }

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
        info!("{} is connected!", ready.user.name);

        let commands = GuildId::set_application_commands(&self.guild_id, &ctx.http, |cmds| {
            cmds
                .create_application_command(|cmd| commands::ping::register(cmd))
        }).await;

        println!("[EVT] I now have access to the following slash commands: {:#?}", commands);
    }
}

//TODO: Try and figure out the issue with Shuttle?
// If we can't figure this one, test running a web server lib + Docker on home server
// Last option is to handle everything just with messages, w/o slash commands
#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };
    
    let gid = if let Some(gid) = secret_store.get("GUILD_ID") {
        gid
    } else {
        return Err(anyhow!("'GUILD_ID' was not found").into());
    };

    let client = Client::builder(&token, GatewayIntents::empty())
        .event_handler(Handler { guild_id: GuildId(gid.parse().expect("'GUILD_ID' needs to be an integer")) })
        .await
        .expect("Err creating client");

    Ok(client.into())
}
