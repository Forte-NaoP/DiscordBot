use poise::serenity_prelude as serenity;
use serenity::{
    async_trait, CreateCommand, Context, CommandInteraction, CommandDataOption, VoiceState,
    UserId
};

use crate::{command_handler::{
        command_handler::*,
        command_return::CommandReturn,
    }, connection_handler::establish_connection};

use std::collections::HashMap;

struct Connect;

pub fn command() -> Box<dyn CommandInterface + Sync + Send> {
    Box::new(Connect)
}

#[async_trait]
impl CommandInterface for Connect {
    async fn run(
        &self, 
        ctx: &Context, 
        command: &CommandInteraction, 
        _options: &[CommandDataOption]
    ) -> CommandReturn {
        match establish_connection(ctx, command).await {
            Ok(res) => CommandReturn::String("접속".to_owned()),
            Err(why) => CommandReturn::String(format!("{:?}", why))
        }
    }

    fn register(&self) -> CreateCommand {
        let command = CreateCommand::new("");
        command
            .name("join")
            .description("Join the voice channel")
    }
}