use std::env;

use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::{CommandOptionType, CommandType};
use serenity::prelude::*;

use library::database::DB;
use library::football::{BlameResult, calc_blame, get_team_id, get_schedule, get_long_name, get_team_emoji };

pub const ACCESS: i64 = 0;

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

fn calc_blame_score(blame_res: &BlameResult) -> i32 {
    return match blame_res {
        BlameResult::Bye | BlameResult::Tied | BlameResult::NoChoice => 0,
        BlameResult::WinUnique => 4,
        BlameResult::Win => 2,
        BlameResult::Loss => -2,
        BlameResult::LossUnique => -4,
    };
}

fn blame_score_emoji(blame_res: &BlameResult) -> &'static str{
    return match blame_res {
        BlameResult::Bye | BlameResult::Tied | BlameResult::NoChoice => ":white_large_square:",
        BlameResult::WinUnique => ":arrow_double_up:",
        BlameResult::Win => ":arrow_up:",
        BlameResult::Loss => ":arrow_down:",
        BlameResult::LossUnique => ":arrow_double_down:",
    };
}

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction, db: &DB) {
    let poolid = env::var("POOL_ID")
        .expect("![Handler] Could not find env var 'POOL_ID'").parse::<i64>()
        .expect("![Handler] Could not parse pool_id to int");
    let season = env::var("CONF_SEASON")
        .expect("[blame] Cannot find 'CONF_SEASON' in env").parse::<u16>()
        .expect("[blame] Could not parse 'CONF_SEASON' to u16");

    let team = command.data.options.first().unwrap().clone().value.unwrap();
    let team = team.as_str().unwrap();
    let teamid = get_team_id(team);
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
            println!("![blame] Cannot respond to slash command : {:?}", reason);
        }
        // We are done here, cannot handle invalid team name
        // Would like to create a list of possible values, but Discord caps it a 25 options
        return;
    }

    let discordid = command.user.id.as_u64()
        .to_string().parse::<i64>()
        .unwrap();
    let poolerid = match db.fetch_poolerid(&discordid).await {
        Ok(pid) => pid,
        Err(_) => {
            if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
                res
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|m| m
                        .ephemeral(true)
                        .content("Tu n'es pas inscrit au pool.")
                    )
            })
            .await {
                println!("![blame] Cannot respond to slash command : {:?}", reason);
            }
            return;
        },
    };

    let matches = get_schedule(&season, &teamid).await;
    let (seasondata, _) = db.fetch_season(&poolid, &season).await.unwrap();
    let pooler_picks: Vec<_> = seasondata.into_iter()
        .filter_map(|(_, _, picks)| picks.into_iter().find(|p| p.poolerid == poolerid))
        .collect();

    let outcomes = calc_blame(&matches, &pooler_picks, team);
    let emojis: String = outcomes.iter().enumerate().fold(String::new(), |mut acc, (i, o)| {
        acc.push_str(format!("`{:02}` {}\n", i+1, blame_score_emoji(o)).as_str());
        acc
    });
    let score: i32 = outcomes.iter().map(|o| calc_blame_score(o)).sum();

    if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
        res
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|m| m
                .ephemeral(true)
                    .content(format!("Blâmer <:{}:{}> {}\n{}\n Total pour la saison: {}", team, get_team_emoji(team), get_long_name(team), emojis, score))
            )
    })
    .await {
        println!("![blame] Cannot respond to slash command : {:?}", reason);
    }
}
