use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::{ CommandType, CommandOptionType };
use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::prelude::*;

use football;

const VS_EMOJI: &str = "<:VS:1102123108187525130>";

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction) {
    let week = command.data.options.first()
        .expect("[Week] No argument provided")
        .value.as_ref().unwrap()
        .as_str()
        .expect("[Week] Could not fetch week arg")
        .parse::<u64>()
        .expect("[Week] Could not parse week arg to int");

    let output = football::get_week(week)
        .await.expect("![Week] Could not fetch match data")
        .fold(String::new(), |mut out, m| {
            let ateam = football::get_short_name(m.away_team.as_str());
            let hteam = football::get_short_name(m.home_team.as_str());

            let aemoji = football::get_team_emoji(ateam.as_str());
            let hemoji = football::get_team_emoji(hteam.as_str());

            out.push_str(format!("<:{}:{}> {} <:{}:{}>\n",
                ateam, aemoji,
                VS_EMOJI,
                hteam, hemoji
            ).as_str());
            out
        });

    if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
        res
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|m| m
                .content(output)
                //TODO: Look for more options here
            )
    })
    .await {
        println!("![week] Cannot respond to slash command : {:?}", reason);
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("week")
        .description("Shows matches for a given week")
        .kind(CommandType::ChatInput)
        .create_option(|opt| {
            opt
                .name("week")
                .kind(CommandOptionType::String)
                .description("The week number to show the matches of")
                .required(true)
                .add_string_choice("1", 1)
                .add_string_choice("2", 2)
                .add_string_choice("3", 3)
                .add_string_choice("4", 4)
                .add_string_choice("5", 5)
                .add_string_choice("6", 6)
                .add_string_choice("7", 7)
                .add_string_choice("8", 8)
                .add_string_choice("9", 9)
                .add_string_choice("10", 10)
                .add_string_choice("11", 11)
                .add_string_choice("12", 12)
                .add_string_choice("13", 13)
                .add_string_choice("14", 14)
                .add_string_choice("15", 15)
                .add_string_choice("16", 16)
                .add_string_choice("17", 17)
                .add_string_choice("18", 18)
                .add_string_choice("WildCards", 160)
                .add_string_choice("Divisional", 125)
                .add_string_choice("Championship", 150)
                .add_string_choice("Super Bowl", 200)
        })
}
