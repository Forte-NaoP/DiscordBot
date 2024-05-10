use poise::serenity_prelude as serenity;
use serenity::{
    async_trait, CreateCommand, Context, CommandInteraction, CommandDataOption, VoiceState,
    UserId
};

use crate::command_handler::{
        command_handler::*,
        command_return::CommandReturn,
    };

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
        let (guild_id, channel_id) = {
            let guild_id = command.guild_id.unwrap();
            let voice_states: &HashMap<UserId, VoiceState> = &guild_id
                .to_guild_cached(ctx)
                .unwrap()
                .voice_states;
            let channel_id = voice_states
                .get(&command.user.id)
                .and_then(|voice_state| voice_state.channel_id);
            (guild_id, channel_id)
        };

        let connect_to = match channel_id {
            Some(channel) => channel,
            None => {
                return CommandReturn::String("음성채널에 먼저 접속해주세요.".to_owned())
            },
        };

        let manager = songbird::get(ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.")
            .clone();

        match manager.join(guild_id, connect_to).await {
            Ok(_) => CommandReturn::String("접속".to_owned()),
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