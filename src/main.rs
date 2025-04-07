use chrono::{Local, Datelike, Timelike, Weekday};
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

use library::database::DB;
use library::football::{ get_week, get_team_emoji };

mod commands;

const VS_EMOJI: &str = "<:VS:1102123108187525130>";

struct Bot {
    database: DB,
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!allo" {
            let reply = format!("Salut, {}!", msg.author.name);
            if let Err(reason) = msg.channel_id.say(&ctx.http, reply).await {
                println!("![Handler] Handler message error : {:?}", reason);
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(cmd) = interaction {
            match cmd.data.name.as_str() {
                "semaine"  => commands::matches::run(ctx, &cmd, &self.database).await,
                "choix"    => commands::picks::run(ctx, &cmd, &self.database).await,
                //"ping"   => commands::ping::run(ctx, &cmd).await,
                "resultat" => commands::results::run(ctx, &cmd, &self.database).await,
                "saison"   => commands::season::run(ctx, &cmd, &self.database).await,
                "stats"    => commands::stats::run(ctx, &cmd, &self.database).await,
                "equipe"   => commands::team::run(ctx, &cmd, &self.database).await,
                "blame"    => commands::blame::run(ctx, &cmd, &self.database).await,
                "features" => commands::features::run(ctx, &cmd, &self.database).await,
                _          => println!("![Handler] Command not implemented!"),
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
                //.create_application_command(|cmd| commands::ping::register(cmd))
                .create_application_command(|cmd| commands::results::register(cmd))
                .create_application_command(|cmd| commands::season::register(cmd))
                .create_application_command(|cmd| commands::stats::register(cmd))
                .create_application_command(|cmd| commands::team::register(cmd))
                .create_application_command(|cmd| commands::blame::register(cmd))
                .create_application_command(|cmd| commands::features::register(cmd))
        }).await.expect("![Handler] Could not set application commands in Discord Guild");

        println!("[Handler] Here are the available commands:");
        commands.iter()
            .for_each(|c| println!("\t-{}", c.name));

        spawn(async move {
            let db = DB::new().await;

            if let Ok(hook_url) = env::var("WEEKLY_WEBHOOK") {
                let hook = Webhook::from_url(&ctx.http, hook_url.as_str()).await.unwrap();
                println!("[Handler] Webhook created and ready to fire");

                loop {
                    let now = Local::now();

                    if now.weekday() == Weekday::Tue && now.hour() == 13 {
                        let poolid = env::var("POOL_ID")
                            .expect("![Handler] Could not find env var 'POOL_ID'").parse()
                            .expect("![Handler] Could not parse pool_id to int");
                        let season = env::var("CONF_SEASON")
                            .expect("![Handler] Cannot find 'CONF_SEASON' in env").parse()
                            .expect("![Handler] Could not parse 'CONF_SEASON' to u16");
                        let week = db.find_week(&poolid, &season).await
                            .expect("![Handler] Could not find current week from DB");

                        let matches = weekly_matches_message(&season, &week).await;
                        let results = weekly_results_message(&db, &poolid, &season, &week).await;

                        hook.execute(&ctx.http, false, |m| {
                            m.content(matches)
                        }).await.unwrap();

                        hook.execute(&ctx.http, false, |m| {
                            m.content(results)
                        }).await.unwrap();

                        tokio::time::sleep(Duration::from_secs(3600)).await;
                    }
                    else {
                        tokio::time::sleep(Duration::from_secs(600)).await;
                    }
                }
            }
        });
    }
}

async fn weekly_matches_message(season: &u16, week: &i64) -> String {
    let matches = get_week(&season, &week).await
        .expect("![Week] Could not fetch match data");

    matches.fold(String::new(), |mut out, m| {
        let aemoji = get_team_emoji(m.away_team.as_str());
        let hemoji = get_team_emoji(m.home_team.as_str());

            let (ascore, hscore, aline, hline) = if let (Some(a), Some(h)) = (m.away_score, m.home_score) {
                (a.to_string(), h.to_string(),
                    a > h, h > a)
            }
            else {
                ("--".to_string(), "--".to_string(),
                    false, false)
            };

        out.push_str(format!("<:{}:{}> {} {} {} <:{}:{}>\n",
            m.away_team, aemoji,
            if aline { format!("__`{:02}`__", ascore) } else { format!("`{:02}`", ascore) },
            VS_EMOJI,
            if hline { format!("__`{:02}`__", hscore) } else { format!("`{:02}`", hscore) },
            m.home_team, hemoji
        ).as_str());
        out
    })
}

async fn weekly_results_message(_db: &DB, _poolid: &i64, _season: &u16, _week: &i64) -> String {
    //TODO: Bring changes from results.rs
    String::new()
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
