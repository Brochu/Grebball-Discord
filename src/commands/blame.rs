use std::env;

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::{CommandOptionType, CommandType};
use serenity::prelude::*;

use library::database::DB;
use library::football::{calc_blame, get_team_id, get_schedule};

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
        // We are done here, cannot handle invalid team name
        // Would like to create a list of possible values, but Discord caps it a 25 options
        return;
    }

    let discordid = command.user.id.as_u64()
        .to_string().parse::<i64>()
        .unwrap();
    let poolerid = db.fetch_poolerid(&discordid).await.unwrap();

    let matches = get_schedule(&season, &teamid).await;
    let (seasondata, week_count) = db.fetch_season(&poolid, &season).await.unwrap();
    (0..week_count).for_each(|i| {
        println!("{:02} -- {:?}", i+1, matches[i]);
    });
    let _blame_score = calc_blame(&seasondata[0].0, &Vec::new(), &seasondata[0].1, &poolerid, team.as_str().unwrap());

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
