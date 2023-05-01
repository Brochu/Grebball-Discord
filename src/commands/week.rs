use std::env;
use serde_json::Value;

use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::{ CommandType, CommandOptionType };
use serenity::model::prelude::interaction::application_command::CommandDataOption;

use football;

pub async fn run(options: &[CommandDataOption]) -> String {
    let league = env::var("CONF_LEAGUE")
        .expect("![Week] Could not find 'CONF_LEAGUE' env var")
        .parse::<u16>()
        .expect("![Week] Could not parse 'CONF_LEAGUE' to int");
    let season = env::var("CONF_SEASON")
        .expect("![Week] Could not find 'CONF_SEASON' env var")
        .parse::<u16>()
        .expect("![Week] Could not parse 'CONF_SEASON' to int");
    let week = options.first()
        .expect("[Week] No argument provided")
        .value.as_ref().unwrap()
        .as_u64()
        .expect("[Week] Could not parse week arg to int");

    let url = format!("https://www.thesportsdb.com/api/v1/json/3/eventsround.php?id={}&s={}&r={}",
        league, season, week);

    let res = reqwest
        ::get(url).await.expect("![Handler] Could not get reply")
        .text().await.unwrap();

    let val: Value = serde_json::from_str(res.as_str()).expect("![Handler] Could not parse response");
    if let Value::Object(o) = &val {
        let events = o.get("events").unwrap();

        let output = if let Value::Array(matches) = events {
            matches.iter().fold(String::new(), |mut out, m| {
                let ateam = football::get_short_name(m["strAwayTeam"].as_str().unwrap());
                let hteam = football::get_short_name(m["strHomeTeam"].as_str().unwrap());

                let aemoji = football::get_team_emoji(ateam.as_str());
                let hemoji = football::get_team_emoji(hteam.as_str());

                out.push_str(format!("<:{}:{}> <:VS:1102123108187525130> <:{}:{}>\n",
                    ateam, aemoji,
                    hteam, hemoji
                ).as_str());
                out
            })
        } else {
            String::new()
        };

        return output;
    }
    return String::new();
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
                .min_int_value(1)
                .max_int_value(18)
        })
}
