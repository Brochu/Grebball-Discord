use dotenv::dotenv;
use std::env;

use serenity::async_trait;
use serenity::model::application::interaction::Interaction;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;

use sqlx::sqlite::SqlitePool;

mod commands;

struct Bot {
    database: sqlx::SqlitePool,
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!hello" {
            let test = sqlx::query!(
            r#"
            SELECT 'Hello' AS 'greeting'
            "#
            )
            .fetch_all(&self.database).await
            .unwrap();

            println!("{:?}", test);

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
                "ping"    => commands::ping::run(ctx, &cmd).await,
                "results" => commands::results::run(ctx, &cmd).await,
                "submit"  => commands::submit::run(ctx, &cmd).await,
                _         => println!("![Handler] Command not implemented!"),
            };
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
                .create_application_command(|cmd| commands::matches::register(cmd))
                .create_application_command(|cmd| commands::ping::register(cmd))
                .create_application_command(|cmd| commands::results::register(cmd))
                .create_application_command(|cmd| commands::submit::register(cmd))
        }).await;

        println!("Here are the available commands: {:#?}", commands);
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok(); // Include .env file to environment

    let db_url = env::var("DATABASE_URL")
        .expect("![MAIN] Cannot find 'DATABASE_URL' in env");
    let database = SqlitePool::connect(db_url.as_str()).await.unwrap();
    let bot = Bot { database };

    let token = env::var("DISCORD_TOKEN")
        .expect("![MAIN] Cannot find 'DISCORD_TOKEN' in env");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(bot)
        .await
        .expect("![MAIN] Could not create client");

    if let Err(reason) = client.start().await {
        println!("![MAIN] Client error : {:?}", reason);
    }
}
