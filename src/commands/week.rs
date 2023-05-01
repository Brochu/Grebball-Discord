use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::{ CommandType, CommandOptionType };
use serenity::model::prelude::interaction::application_command::CommandDataOption;

use football;

const VS_EMOJI: &str = "<:VS:1102123108187525130>";

pub async fn run(options: &[CommandDataOption]) -> String {
    let week = options.first()
        .expect("[Week] No argument provided")
        .value.as_ref().unwrap()
        .as_u64()
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

    return output;
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("week")
        .description("Shows matches for a given week")
        .kind(CommandType::ChatInput)
        .create_option(|opt| {
            opt
                .name("week")
                .kind(CommandOptionType::Integer)
                .description("The week number to show the matches of")
                .required(true)
                //TODO: Will need to handle post season weeks, probably with a set list of string options
                .min_int_value(1)
                .max_int_value(18)
        })
}
