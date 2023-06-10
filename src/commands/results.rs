use std::env;

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::{CommandType, CommandOptionType};
use serenity::prelude::*;

use library::database::DB;
use library::football::{ Match, get_week };

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
            println!("[results] Number of matches: {}", matches.len());

            let results: Vec<(String, u32)> = picks.iter()
                .map(|p| {
                    println!("[results] Getting results for picks: {}", p.id);

                    match p.cached {
                        Some(cached) => (p.name.to_owned(), cached),
                        None => {
                            match &p.picks {
                                Some(pickstr) => {
                                    println!("[results] Need to correct picks: {}", pickstr);
                                    return (p.name.to_owned(), 0);
                                },
                                None => (p.name.to_owned(), 0),
                            }
                        }
                    }
                })
                .collect();

            //TODO: Complete message formatting
            let message = results.iter()
                .fold(String::new(), |mut m, (name, score)| {
                    m.push_str(format!("{name}: {score}\n").as_str());
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
