use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::model::prelude::interaction::{ Interaction, InteractionResponseType };
use serenity::prelude::*;

mod commands;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
    }

    //TODO: Handle slash commands
    //async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    //}

    async fn ready(&self, ctx: Context, ready: Ready) {
    }
}

fn main() {
}
