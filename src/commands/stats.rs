use std::env;
use std::fmt::Display;

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandType;
use serenity::prelude::*;

use library::database::DB;
use library::football::{ Match, get_week };

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("stats")
        .description("Montre les statistiques de toutes les semaines de la saison courante")
        .kind(CommandType::ChatInput)
}

struct PoolStats {
    uni_count: u32,
    uni_hits: u32,
}
impl Display for PoolStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[POOL] unilateral rate: {} / {}", self.uni_hits, self.uni_count)
    }
}

struct PoolerStats {
    name: String,
    pick_count: u32,
    hit_count: u32,
    unique_count: u32,
    unique_hits: u32,
}
impl Display for PoolerStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[{}] hit rate: {} / {}; unique hit rate: {} / {}",
            self.name, self.hit_count, self.pick_count, self.unique_hits, self.unique_count)
    }
}

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction, db: &DB) {
    let poolid = env::var("POOL_ID")
        .expect("![Handler] Could not find env var 'POOL_ID'").parse::<i64>()
        .expect("![Handler] Could not parse pool_id to int");
    let season = env::var("CONF_SEASON")
        .expect("[results] Cannot find 'CONF_SEASON' in env").parse::<u16>()
        .expect("[results] Could not parse 'CONF_SEASON' to u16");

    if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
        res
            .kind(InteractionResponseType::DeferredChannelMessageWithSource)
            .interaction_response_data(|m| m
                .content("Calcul ...")
            )
    })
    .await {
        println!("![results] Cannot respond to slash command : {:?}", reason);
    }

    let mut _pool = PoolStats{ uni_count: 0, uni_hits: 0 };
    let mut _poolers = Vec::<PoolerStats>::new();

    let (weeks, _) = db.fetch_season(&poolid, &season).await.unwrap();
    for (w, week) in &weeks[0..1] {
        let matches: Vec<Match> = get_week(&season, &w).await.unwrap().collect();
        let _pickstring: Vec<String> = matches.iter().map(|m| {
            week.iter().fold(String::new(), |mut acc, pooler| {
                acc.push_str(pooler.picks.as_ref().unwrap().get(&m.id_event).unwrap().as_str().unwrap());
                acc.push_str(", ");
                acc
            })
        })
        .inspect(|s| println!(" - {}", s))
        .collect();
    }

    if let Err(reason) = command.edit_original_interaction_response(&ctx.http, |res| {
        res.content(format!("Statistiques {}-{}\n{}", season, poolid, "STATS HERE"))
    })
    .await {
        println!("![results] Cannot respond to slash command : {:?}", reason);
    }
}
