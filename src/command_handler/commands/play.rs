use poise::serenity_prelude as serenity;
use serenity::{
    async_trait, CreateCommand, Context, CommandInteraction, CommandDataOption, 
    CreateCommandOption, CommandOptionType
};
use songbird::input::{Compose, YoutubeDl};

use crate::{
    command_handler::{
        command_handler::*,
        command_return::CommandReturn,
    }, 
    connection_handler::*, 
    utils::url_checker::url_checker, HttpKey,
};

use std::collections::HashMap;

struct Play;

pub fn command() -> Box<dyn CommandInterface + Sync + Send> {
    Box::new(Play)
}

#[async_trait]
impl CommandInterface for Play {
    async fn run(
        &self, 
        ctx: &Context, 
        command: &CommandInteraction, 
        options: &[CommandDataOption]
    ) -> CommandReturn {

        match establish_connection(ctx, command).await {
            Ok(_) => (),
            Err(why) => match why {
                ConnectionErrorCode::JoinVoiceChannelFirst => return CommandReturn::String("음성채널에 먼저 접속해주세요.".to_owned()),
                ConnectionErrorCode::AlreadyInUse => return CommandReturn::String("다른 채널에서 사용중입니다.".to_owned()),
                _ => return CommandReturn::String("연결에 실패했습니다.".to_owned()),
            },
        };

        let url = options.get(0)
            .and_then(|option| option.value.as_str())
            .and_then(url_checker)
            .unwrap();

        let guild_id = command.guild_id.unwrap();
        let manager = songbird::get(ctx).await.unwrap();

        let http_client = {
            let data = ctx.data.read().await;
            data.get::<HttpKey>()
                .cloned()
                .unwrap()
        };
        
        if let Some(handler_lock) = manager.get(guild_id) {
            let mut handler = handler_lock.lock().await;

            let mut src = YoutubeDl::new(http_client, url);
            let title = {
                let metadata = src.aux_metadata().await.unwrap();
                metadata.title.unwrap()
            };    
            let _ = handler.play_input(src.into());

            CommandReturn::String(format!("{} 재생중", title))
        } else {
            CommandReturn::String("재생 실패".to_owned())
        }

    }

    fn register(&self) -> CreateCommand {
        let command = CreateCommand::new("");
        command
            .name("play")
            .description("Join the voice channel")
            .add_option({
                let option = CreateCommandOption::new(
                    CommandOptionType::String,
                    "url",
                    "youtube url"
                );
                option.required(true)
            })
    }
}