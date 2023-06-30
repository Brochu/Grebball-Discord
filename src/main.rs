use dotenv::dotenv;
use std::{env, time::Duration};

use serenity::async_trait;
use serenity::model::application::interaction::Interaction;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::model::webhook::Webhook;
use serenity::prelude::*;

use tokio::spawn;
use tokio::time::{ interval, MissedTickBehavior };

use library::database::DB;

mod commands;

struct Bot {
    database: DB,
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!hello" {
            let reply = format!("Hi, {}!", msg.author.name);
            if let Err(reason) = msg.channel_id.say(&ctx.http, reply).await {
                println!("![Handler] Handler message error : {:?}", reason);
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(cmd) = interaction {
            match cmd.data.name.as_str() {
                "matches" => commands::matches::run(ctx, &cmd).await,
                "picks"   => commands::picks::run(ctx, &cmd, &self.database).await,
                "ping"    => commands::ping::run(ctx, &cmd).await,
                "results" => commands::results::run(ctx, &cmd, &self.database).await,
                "season" => commands::season::run(ctx, &cmd, &self.database).await,
                _         => println!("![Handler] Command not implemented!"),
            };
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("[Handler] ({}) {} is connected w/ version: {}\n",
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
                .create_application_command(|cmd| commands::matches::register(cmd))
                .create_application_command(|cmd| commands::picks::register(cmd))
                .create_application_command(|cmd| commands::ping::register(cmd))
                .create_application_command(|cmd| commands::results::register(cmd))
                .create_application_command(|cmd| commands::season::register(cmd))
        }).await.expect("![Handler] Could not set application commands in Discord Guild");

        println!("[Handler] Here are the available commands:");
        commands.iter()
            .for_each(|c| println!("\t-{}", c.name));

        spawn(async move {
            let mut timer = interval(Duration::from_secs_f64(5.0));
            timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
            let db = DB::new().await;

            if let Ok(hook_url) = env::var("RESULTS_WEBHOOK") {
                let hook = Webhook::from_url(&ctx.http, hook_url.as_str()).await.unwrap();
                println!("[Handler] Webhook created and ready to fire");

                loop {
                    timer.tick().await;

                    let message = weekly_message(&db).await;
                    hook.execute(&ctx.http, false, |m| {
                        m.content(message)
                    }).await.unwrap();

                }
            }
        });
    }
}

async fn weekly_message(db: &DB) -> String {
    let pool_id = env::var("POOL_ID")
        .expect("![Handler] Could not find env var 'POOL_ID'").parse()
        .expect("![Handler] Could not parse pool_id to int");
    let season = env::var("CONF_SEASON")
        .expect("![Handler] Cannot find 'CONF_SEASON' in env").parse()
        .expect("![Handler] Could not parse 'CONF_SEASON' to u16");

    let week = db.find_week(&pool_id).await
        .expect("![Handler] Could not find current week from DB");

    let _picks = db.fetch_picks(&pool_id, &season, &week);

    //TODO: Create right message here with database results
    format!("Building results message for current week {}", week)
}

#[tokio::main]
async fn main() {
    dotenv().ok(); // Include .env file to environment

    let token = env::var("DISCORD_TOKEN")
        .expect("![MAIN] Cannot find 'DISCORD_TOKEN' in env");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Bot { database: DB::new().await })
        .await
        .expect("![MAIN] Could not create client");

    if let Err(reason) = client.start().await {
        println!("![MAIN] Client error : {:?}", reason);
    }
}
