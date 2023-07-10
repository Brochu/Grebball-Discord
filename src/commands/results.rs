use std::{ env, cmp::Ordering };

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::{CommandType, CommandOptionType};
use serenity::prelude::*;

use library::database::DB;
use library::football::{ calc_results, get_week, get_team_emoji, Match };

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("results")
        .description("Request the results all current pool members for a given week")
        .kind(CommandType::ChatInput)
        .create_option(|opt| {
            opt
                .name("week")
                .kind(CommandOptionType::String)
                .description("The week number to show the matches of")
                .required(true)
                .add_string_choice("week 1", 1)
                .add_string_choice("week 2", 2)
                .add_string_choice("week 3", 3)
                .add_string_choice("week 4", 4)
                .add_string_choice("week 5", 5)
                .add_string_choice("week 6", 6)
                .add_string_choice("week 7", 7)
                .add_string_choice("week 8", 8)
                .add_string_choice("week 9", 9)
                .add_string_choice("week 10", 10)
                .add_string_choice("week 11", 11)
                .add_string_choice("week 12", 12)
                .add_string_choice("week 13", 13)
                .add_string_choice("week 14", 14)
                .add_string_choice("week 15", 15)
                .add_string_choice("week 16", 16)
                .add_string_choice("week 17", 17)
                .add_string_choice("week 18", 18)
                .add_string_choice("WildCards", 160)
                .add_string_choice("Divisional", 125)
                .add_string_choice("Championship", 150)
                .add_string_choice("Super Bowl", 200)
        })
}

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction, db: &DB) {
    let poolid = env::var("POOL_ID")
        .expect("![results] Could not find env var 'POOL_ID'").parse()
        .expect("![results] Could not parse pool_id to int");
    let season = env::var("CONF_SEASON")
        .expect("[results] Cannot find 'CONF_SEASON' in env").parse::<u16>()
        .expect("[results] Could not parse 'CONF_SEASON' to u16");
    let week = command.data.options.first()
        .expect("[results] No argument provided").value.as_ref()
        .unwrap().as_str().unwrap().parse::<i64>()
        .expect("[results] Could not parse week arg to u64");

    match db.fetch_picks(&poolid, &season, &week).await {
        Ok(picks) => {
            let matches: Vec<Match> = get_week(&season, &week).await
                .expect("[results] Could not fetch week data")
                .collect();

            let results = calc_results(&week, &matches, &picks).await;

            let mut message = String::new();

            let _header = matches.iter().map(|m| {
                match (m.away_score, m.home_score) {
                    (Some(a), Some(h)) => {
                        match a.cmp(&h) {
                            Ordering::Less => m.home_team.as_str(),
                            Ordering::Greater => m.away_team.as_str(),
                            Ordering::Equal => "",
                        }
                    }
                    _ => {
                        ""
                    }
                }
            })
            .fold(String::new(), |header, _winner| {
                header
            });

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

            if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
                res
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|m| m
                        .content(format!("Results:\n{message}"))
                    )
            })
            .await {
                println!("![results] Cannot respond to slash command : {:?}", reason);
            }
        },
        Err(e) => {
            println!("![results] Could not fetch picks for poolid: {}; season: {}, week: {}\nerror: {}",
                poolid, season, week, e);
        },
    }
}
