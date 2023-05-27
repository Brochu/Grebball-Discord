use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::{CommandOptionType, CommandType};
use serenity::prelude::*;

use library::database::DB;

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction, db: &DB) {
    let week = command.data.options.first()
        .expect("[picks] No week arg given with the command").value.as_ref()
        .unwrap().as_str().unwrap().parse::<i64>()
        .expect("[picks] Could not parse week arg to u64");
    let week_name = command.data.options.first().unwrap().name.as_str();

    let discordid = command.user.id.as_u64()
        .to_string().parse::<i64>()
        .unwrap();

    match db.prime_picks(&discordid, &week).await {
        Ok(row_id) => {
            let url = format!("http://localhost:8080/{}/{}", discordid, row_id);

            if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
                res
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|m| m
                        .ephemeral(true)
                        .content(format!("Ready for {}'s pick to be completed here: {}", week_name, url))
                    )
            })
            .await {
                println!("![picks] Cannot respond to slash command : {:?}", reason);
            }
        },
        Err(_) => println!("[picks] Could not prime picks for week {}", week),
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("picks")
        .description("Make picks for a given week of the current season")
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
