use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::{MessageId, ReactionType};
//use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::model::prelude::command::CommandType;
use serenity::prelude::*;

pub async fn run(_: Context, command: &ApplicationCommandInteraction) {
    println!("Data sent with the submit command : {:#?}", command.data);

    if let Some(id) = command.data.target_id {
        let message_id = MessageId(*id.as_u64());
        let message = command.data.resolved.messages.get(&message_id)
            .expect("![Submit] Could not find message with provided TargetId");

        println!("Message : {:#?}", message);
        message.content.lines().for_each(|l| {
            let (first, second) = l.split_once(" ").unwrap();
            let (_, second) = second.split_once(" ").unwrap();

            println!("Match, {} vs. {}", first, second);
        });

        message.reactions.iter().for_each(|r| {
            if let ReactionType::Custom { name, .. } = &r.reaction_type {
                let pick = name.as_ref().unwrap();
                println!("reaction: {:#?}", pick);
            }
        });

        //TODO: How can we track match ids, season and week number to add to db

        //TODO: Maybe send a message in case of error later
        // Handle adding picks to database first
        //if let Err(reason) = command.create_interaction_response(&ctx.http, |res| {
        //    res
        //        .kind(InteractionResponseType::ChannelMessageWithSource)
        //        .interaction_response_data(|m| m
        //            .content("Found the message with picks to submit")
        //            //TODO: Look for more options here
        //        )
        //})
        //.await {
        //    println!("![week] Cannot respond to slash command : {:?}", reason);
        //}
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("submit")
        .kind(CommandType::Message)
}
