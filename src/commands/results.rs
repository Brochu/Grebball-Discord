use std::env;

use serde_json::{Value, Map};
use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::{CommandType, CommandOptionType};
use serenity::prelude::*;

use library::database::{ DB, WeekPicks };
use library::football::{ Match, get_week, get_team_emoji };

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
    let season = env::var("CONF_SEASON")
        .expect("[results] Cannot find 'CONF_SEASON' in env").parse::<u16>()
        .expect("[results] Could not parse 'CONF_SEASON' to u16");
    let week = command.data.options.first()
        .expect("[results] No argument provided").value.as_ref()
        .unwrap().as_str().unwrap().parse::<i64>()
        .expect("[results] Could not parse week arg to u64");

    let discordid = command.user.id.as_u64()
        .to_string().parse::<i64>()
        .unwrap();

    match db.fetch_picks(&discordid, &season, &week).await {
        Ok(picks) => {
            let matches: Vec<Match> = get_week(&season, &week).await
                .expect("[results] Could not fetch week data")
                .collect();
            let results: Vec<(String, u32, String)> = picks.iter()
                .map(|p| {
                    let (name, score) = match p.cached {
                        Some(cached) => (p.name.to_owned(), cached),
                        None => {
                            match &p.picks {
                                Some(poolerpicks) => (
                                    p.name.to_owned(),
                                    calc_results(&matches, &picks, &poolerpicks, p.pickid, p.poolerid)
                                ),
                                None => (p.name.to_owned(), 0),
                            }
                        }
                    };

                    let icons = match &p.picks {
                        Some(poolerpicks) => {
                            matches.iter().fold(String::new(), |mut str, m| {
                                let choice = poolerpicks.get(&m.id_event)
                                    .unwrap().as_str()
                                    .unwrap();
                                str.push_str(format!("<:{}:{}>", choice, get_team_emoji(choice)).as_str());

                                str.push(' ');
                                str
                            })
                        },
                        None => String::new(),
                    };

                    (name, score, icons)
                })
                .collect();

            //TODO: Complete message formatting
            let message = results.iter()
                .fold(String::new(), |mut m, (name, score, icons)| {
                    let width = 10 - name.len();
                    m.push_str(format!("`{name}{}`->", " ".repeat(width)).as_str());

                    m.push_str(format!("{icons} |").as_str());
                    m.push_str(format!(" {score}\n").as_str());
                    m
                });

            if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
                res
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|m| m
                        .content(format!("Results:\n{}", message))
                    )
            })
            .await {
                println!("![results] Cannot respond to slash command : {:?}", reason);
            }
        }
        Err(e) => println!("Query error: {:?}", e),
    };
}

fn calc_results(
    matches: &[Match],
    poolpicks: &[WeekPicks],
    picks: &Map<String, Value>,
    pickid: i64,
    poolerid: i64) -> u32 {

    //TODO: Finish implementation
    println!("[results] Calculating for pooler id {}; pick id {}: ", poolerid, pickid);

    matches.iter().fold(0, |total, m| {
        if let Some(pick) = picks.get(&m.id_event) {
            let pick = pick.as_str()
                .expect("[results] Could not get match pick as str");

            //TODO: Better handling of errors here in case a pooler did not make picks yet
            let pool: Vec<&str> = poolpicks.iter()
                .filter(|&pp| pp.poolerid != poolerid)
                .map(|pp| { 
                    match &pp.picks {
                        Some(p) => p.get(&m.id_event).unwrap().as_str().unwrap(),
                        None => "",
                    }
                })
                .collect();

            println!("{}\n\tmy pick: {}; pool picks: {:?}", m, pick, pool);
            total
        }
        else {
            total
        }
    })
}
