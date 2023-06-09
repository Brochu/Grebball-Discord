use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::{CommandType, CommandOptionType};
use serenity::prelude::*;

use library::database::DB;

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction, db: &DB) {
    let week = command.data.options.first()
        .expect("[results] No argument provided").value.as_ref()
        .unwrap().as_str().unwrap().parse::<i64>()
        .expect("[results] Could not parse week arg to u64");

    let discordid = command.user.id.as_u64()
        .to_string().parse::<i64>()
        .unwrap();

    match db.fetch_picks(&discordid, &week).await {
        Ok(picks) => {
            //let results = get_week(&season, week).await
            //    .expect("[DB] Could not get week data to calculate results")
            //    .fold(Vec::<(String, u32)>::new(), |mut res, m| {
            //        picks.iter()
            //            .enumerate()
            //            .for_each(|(i, row)| {
            //                let name: String = row.get("name");
            //                if let Some(cached) = row.get::<Option<u32>, &str>("scorecache") {
            //                    res.push((name., cached));
            //                }
            //                else {
            //                    // TODO: Calculate match results here
            //                    println!("Match Id: {:?}", m.id_event);

            //                    if let Some(entry) = res.get_mut(i) {
            //                        entry.1 += 1;
            //                    }
            //                    else {
            //                        res.push((name, 1));
            //                    }
            //                }
            //            });
            //        res
            //    });

            println!("Query success:\n");
            picks.iter().for_each(|p| {
                println!("\t{}", p);
            });

            //TODO: Complete message formatting
            let message = picks.iter()
                .fold(String::new(), |mut m, w| {
                    m.push_str(format!("{}: 0\n", w.name).as_str());
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
