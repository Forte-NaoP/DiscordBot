use poise::serenity_prelude as serenity;
use serenity::{
    async_trait, CreateCommand, Context, CommandInteraction, CommandDataOption, 
    CreateCommandOption, CommandOptionType, Channel, ChannelType, Mentionable
};

use crate::{
    command_handler::{
        command_handler::*,
        command_return::CommandReturn,
    }, 
    utils::{guild_queue::get_guild_queue, board::get_board}
};

use std::{any::Any, collections::HashMap, io::Read, path::PathBuf, sync::Arc};

struct Register;

pub fn command() -> Box<dyn CommandInterface + Sync + Send> {
    Box::new(Register)
}

#[async_trait]
impl CommandInterface for Register {
    async fn run(
        &self, 
        ctx: &Context, 
        command: &CommandInteraction, 
        options: &[CommandDataOption]
    ) -> CommandReturn {

        let channel_id = options.get(0)
            .and_then(|option| option.value.as_channel_id())
            .unwrap();

        let channel = match channel_id.to_channel(&ctx.http).await {
            Ok(Channel::Guild(channel)) if channel.kind == ChannelType::Text => channel,
            Ok(_) | Err(_)=> return CommandReturn::String("텍스트 채널을 등록해 주세요".to_owned()),
        };

        let guild_id = command.guild_id.unwrap();
        let guild_queue = get_guild_queue(ctx, guild_id).await;
        guild_queue.register_channel(channel.id.into());

        let mut board = get_board(ctx, guild_id).await;
        board.set(&ctx, channel.clone()).await;

        CommandReturn::String(format!("{} 등록됨", channel.mention()))
    }

    fn register(&self) -> CreateCommand {
        let command = CreateCommand::new("");
        command
            .name("regchan")
            .description("Register chat channel to check")
            .add_option({
                let option = CreateCommandOption::new(
                    CommandOptionType::Channel,
                    "channel",
                    "chat channel"
                );
                option.required(true)
            })
    }
}