use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::{ CommandType, CommandOptionType };
use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::prelude::*;

use library::football::{ get_week, get_team_emoji };

const VS_EMOJI: &str = "<:VS:1102123108187525130>";

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction) {
    let week = command.data.options.first().expect("[Week] No argument provided")
        .value.as_ref().unwrap()
        .as_str().expect("[Week] Could not fetch week arg")
        .parse::<u64>().expect("[Week] Could not parse week arg to int");

    let matches = get_week(week).await.expect("![Week] Could not fetch match data");

    let (output, embeds) = matches.fold((String::new(), Vec::<(String, String, bool)>::new()), |mut out, m| {
        let aemoji = get_team_emoji(m.away_team.as_str());
        let hemoji = get_team_emoji(m.home_team.as_str());

        out.0.push_str(format!("<:{}:{}> {} <:{}:{}>\n",
            m.away_team, aemoji,
            VS_EMOJI,
            m.home_team, hemoji
        ).as_str());

        out.1.push((format!("{}", m.id_event), format!("{}-{}", m.away_team, m.home_team), false));
        out
    });

    if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
        res
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|m|
                m
                    .content(output)
                    .embed(|e| {
                        e.fields(embeds)
                    })
                //TODO: Look for more options here
            )
    })
    .await {
        println!("![week] Cannot respond to slash command : {:?}", reason);
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("matches")
        .description("Shows matches for a given week")
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
