use std::env;

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::{CommandOptionType, CommandType};
use serenity::prelude::*;

use library::database::DB;
use library::football::{calc_blame, get_team_id, get_week, Match};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("blame")
        .description("Blamer une équipe - Calcule la difference de points gagné/perdu par une équipe")
        .kind(CommandType::ChatInput)
        .create_option(|opt| {
            opt
                .name("équipe")
                .kind(CommandOptionType::String)
                .description("L'équipe à blamer")
                .required(true)
        })
}

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction, db: &DB) {
    let poolid = env::var("POOL_ID")
        .expect("![Handler] Could not find env var 'POOL_ID'").parse::<i64>()
        .expect("![Handler] Could not parse pool_id to int");
    let season = env::var("CONF_SEASON")
        .expect("[picks] Cannot find 'CONF_SEASON' in env").parse::<u16>()
        .expect("[picks] Could not parse 'CONF_SEASON' to u16");

    let team = command.data.options.first().unwrap().clone().value.unwrap();
    let teamid = get_team_id(team.as_str().unwrap());
    if teamid == -1 {
        if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
            res
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|m| m
                    .ephemeral(true)
                    .content(format!("ERROR: Cannot blame invalid team '{}'", team))
                )
        })
        .await {
            println!("![picks] Cannot respond to slash command : {:?}", reason);
        }
        return; // We are done here, cannot handle invalid team name
    }

    let discordid = command.user.id.as_u64()
        .to_string().parse::<i64>()
        .unwrap();
    let poolerid = db.fetch_poolerid(&discordid).await.unwrap();
    println!("poolerid = {}", poolerid);

    let (seasondata, _) = db.fetch_season(&poolid, &season).await.unwrap();
    //seasondata.iter().for_each(|(id, w)| {
    //    println!("[{}] - {}", id, w.len())
    //});

    for (week, allpicks) in seasondata {
        //TODO: Change this get week for get_schedule with only the teams matches
        let matches: Vec<Match> = get_week(&season, &week).await.unwrap().collect();
        let blame_score = calc_blame(&week, &matches, &*allpicks, &poolerid, "").await;

        println!("[{}] - score = {}", week, blame_score);
    }

    if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
        res
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|m| m
                .ephemeral(true)
                    .content(format!("[{}]{} -> blame `{}`", discordid, season, team))
            )
    })
    .await {
        println!("![picks] Cannot respond to slash command : {:?}", reason);
    }
}
