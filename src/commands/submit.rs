use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::{MessageId, ReactionType};
use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::model::prelude::command::CommandType;
use serenity::prelude::*;

use library::database::DB;

pub async fn run(ctx: Context, command: &ApplicationCommandInteraction, _db: &DB) {
    if let Some(id) = command.data.target_id {
        let message_id = MessageId(*id.as_u64());
        let message = command.data.resolved.messages.get(&message_id)
            .expect("![Submit] Could not find message with provided TargetId");

        println!("Message : {:#?}", message);
        //TODO: Parse metadata from the message
        //message.content.lines().for_each(|l| {
        //    let (first, second) = l.split_once(" ").unwrap();
        //    let (_, second) = second.split_once(" ").unwrap();

        //    println!("Match, {} vs. {}", first, second);
        //});

        println!("Reations :");
        message.reactions.iter().for_each(|r| {
            if let ReactionType::Custom { name, .. } = &r.reaction_type {
                let pick = name.as_ref().unwrap();
                println!("\t- {:#?}", pick);
            }
        });

        //TODO: How can we track match ids, season and week number to add to db
        // Handle adding picks to database first

        if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
            res
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|m| m
                    .content("Found the message with picks to submit")
                    //TODO: Look for more options here
                )
        })
        .await {
            println!("![submit] Cannot respond to slash command : {:?}", reason);
        }
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("submit")
        .kind(CommandType::Message)
}
