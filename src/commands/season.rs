use std::env;

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandType;
use serenity::prelude::*;

use library::database::DB;
use library::football::{Match, get_week, calc_results};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("saison")
        .description("Montre les résultats de toutes les semaines de la saison courante")
        .kind(CommandType::ChatInput)
}

struct SeasonResult {
    //poolerid: i64,
    name: String,
    scores: Vec<u32>,
    total: u32,
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

    let (weeks, _week_count) = db.fetch_season(&poolid, &season).await.unwrap();
    let mut season_data = Vec::<SeasonResult>::new();

    for (_, picks) in weeks.iter() {
        for pick in picks {
            let score = if pick.cached.is_some() {
                pick.cached.unwrap()
            } else {
                let matches: Vec<Match> = get_week(&season, &pick.week).await.unwrap().collect();
                let results = calc_results(&pick.week, &matches, &picks).await;
                let result = results.iter()
                    .find(|res| res.poolerid == pick.poolerid)
                    .unwrap();

                if result.cache {
                    db.cache_results(&result.pickid.unwrap(), &result.score).await.unwrap();
                }
                result.score
            };

            if let Some(data) = season_data.iter_mut().find(|d| d.name.eq(&pick.name)) {
                data.scores.push(score);
                data.total += score;
            }
            else {
                season_data.push(SeasonResult{ name: pick.name.clone(), scores: vec![score], total: score });
            }
        }
    }

    season_data.sort_unstable_by(|l, r| { r.total.cmp(&l.total) });
    let header = (1..=_week_count).fold(String::new(), |m, i| {
        if i <= 18 {
            format!("{}| `{:02}` ", m, i)
        }
        else {
            match i {
                19 => format!("{}| `{:02}` ", m, "WC"),
                20 => format!("{}| `{:02}` ", m, "DV"),
                21 => format!("{}| `{:02}` ", m, "CF"),
                22 => format!("{}| `{:02}` ", m, "SB"),
                _ => unreachable!(),
            }
        }
    });
    let header = format!("`{}{}`: {}", "Semaines", " ".repeat(15-8), header);
    let message = season_data.iter()
        .fold(String::new(), |m, entry| {
            let width = 10 - entry.name.len();
            let grid = entry.scores.iter().fold(String::new(), |g, s| { format!("{}| `{:02}` ", g, s) });

            format!("{}\n`{}{}[{:03}]`: {}", m, entry.name, " ".repeat(width), entry.total, grid)
        });

    if let Err(reason) = command.edit_original_interaction_response(&ctx.http, |res| {
        res.content(format!("Saison {}\n{}\n{}", season, header, message))
    })
    .await {
        println!("![results] Cannot respond to slash command : {:?}", reason);
    }
}
