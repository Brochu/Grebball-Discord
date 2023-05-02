use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::{MessageId, ReactionType};
use serenity::model::prelude::command::CommandType;
use serenity::model::prelude::interaction::application_command::CommandData;

pub fn run(command_data: &CommandData) -> String {
    println!("Data sent with the submit command : {:#?}", command_data);

    if let Some(id) = command_data.target_id {
        let message_id = MessageId(*id.as_u64());
        let message = command_data.resolved.messages.get(&message_id)
            .expect("![Submit] Could not find message with provided TargetId");

        println!("Message : {:#?}", message);
        message.content.lines().for_each(|l| {
            let (first, second) = l.split_once(" ").unwrap();
            let (_, second) = second.split_once(" ").unwrap();

            println!("Match, {} vs. {}", first, second);
        });

        message.reactions.iter().for_each(|r| {
            if let ReactionType::Custom { name, .. } = &r.reaction_type {
                println!("reaction: {:#?}", name);
            }
        });
        return "Found the message with picks to submit".to_string();
    }

    return "![Submit] No target message sent with the command".to_string();
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("submit")
        .kind(CommandType::Message)
}
