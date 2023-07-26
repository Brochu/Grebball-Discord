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
use library::football::{ calc_results, get_week, get_team_emoji, Match };

mod commands;

const VS_EMOJI: &str = "<:VS:1102123108187525130>";

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

        let ascore = m.away_score.unwrap_or(0);
        let hscore = m.home_score.unwrap_or(0);
        let boldaway = ascore > hscore;
        let boldhome = ascore < hscore;

        out.push_str(format!("<:{}:{}> {} {} {} <:{}:{}>\n",
            m.away_team, aemoji,
            if boldaway { format!("__`{:02}`__", ascore) } else { format!("`{:02}`", ascore) },
            VS_EMOJI,
            if boldhome { format!("__`{:02}`__", hscore) } else { format!("`{:02}`", hscore) },
            m.home_team, hemoji
        ).as_str());
        out
    })
}

async fn weekly_results_message(db: &DB, poolid: &i64, season: &u16, week: &i64) -> String {
    let mut message = String::new();

    match db.fetch_picks(&poolid, &season, &week).await {
        Ok(picks) => {
            let matches: Vec<Match> = get_week(&season, &week).await
                .expect("![Main] Could not fetch matches for for automated message.")
                .collect();

            let results = calc_results(&week,&matches,&picks).await;

            for r in results {
                if r.cache {
                    if let Err(e) = db.cache_results(&r.pickid, &r.score).await {
                        println!("[results] Error while trying to cache score: {e}")
                    }
                }

                let pick = picks.iter().find(|p| p.pickid == r.pickid)
                    .expect("![results] Could not find pooler picks to fill icons");

                let icons = if let Some(poolerpicks) = &pick.picks {
                    matches.iter().fold(String::new(), |mut acc, m| {
                        let choice = poolerpicks.get(&m.id_event).unwrap()
                            .as_str().unwrap();

                        acc.push_str(format!("<:{}:{}>", choice, get_team_emoji(choice)).as_str());
                        acc
                    })
                }
                else {
                    String::new()
                };

                let width = 10 - r.name.len();
                message.push_str(format!("`{}{}` -> {} : {}\n",
                    r.name, " ".repeat(width), icons, r.score).as_str());
            }
        },
        Err(e) => {
            println!("![Handler] Could not fetch picks for poolid: {}; season: {}, week: {}\nerror: {}",
                poolid, season, week, e);
        },
    }

    message
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
