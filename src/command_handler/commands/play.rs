use lazy_static::lazy_static;
use poise::serenity_prelude as serenity;
use serenity::{
    async_trait, CreateCommand, Context, CommandInteraction, CommandDataOption, 
    CreateCommandOption, CommandOptionType, GuildId, CreateEmbed
};
use songbird::{
    input::{File, Input},
    tracks::TrackQueue,
};

use crate::{
    global::*,
    command_handler::{
        command_handler::*,
        command_return::CommandReturn,
    }, 
    connection_handler::*, 
    utils::{url_checker::url_checker, youtube_dl::ytdl_optioned, guild_queue::get_guild_queue},
};

use std::{collections::HashMap, io::Read, path::PathBuf, sync::Arc};

use chrono::Utc;

struct Play;

pub fn command() -> Box<dyn CommandInterface + Sync + Send> {
    Box::new(Play)
}

lazy_static! {
    static ref INTERVAL: Vec<u8> = {
        let mut f = std::fs::File::open(format!("{TARGET}{NO_SOUND}")).unwrap();
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).unwrap();
        buffer
    };
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

        let mut start = 0;
        let mut duration = 0;
        let mut skip = None;

        for option in &options[1..] {
            match option.name.as_str() {
                "start" => {
                    start = match option.value.as_i64() {
                        Some(value) if value > 0 => value,
                        _ => 0,
                    }
                },
                "duration" => {
                    duration = match option.value.as_i64() {
                        Some(value) if value > 0 => value,
                        _ => 0,
                    }
                },
                "skip" => {
                    skip = match option.value.as_str() {
                        Some(value) => Some(value.to_owned()),
                        _ => None,
                    }
                },
                _ => {}
            }
        }


        let guild_id = command.guild_id.unwrap();
        let guild_queue = get_guild_queue(ctx, guild_id).await;

        let manager = songbird::get(ctx).await.unwrap().clone();
        if let Some(handler_lock) = manager.get(guild_id) {
            let (path, meta) = ytdl_optioned(&url, start, duration).await.unwrap();
            let src = File::new(path);
            let mut handler = handler_lock.lock().await;
            let handle = match skip {
                Some(skip) => guild_queue.add_source_with_word(src.into(), skip, &mut handler).await,
                None => guild_queue.add_source(src.into(), &mut handler).await,
            };
            guild_queue.add_source((INTERVAL.as_ref() as &[u8]).into(), &mut handler).await;
            
            let after = Utc::now().timestamp() + meta.duration.unwrap();
            let embed = CreateEmbed::default()
                .title(meta.title.unwrap().to_owned())
                .description(format!("<t:{}:R>", after));
            CommandReturn::SingleEmbed(embed)
            // CommandReturn::SongInfoEmbed(handle, meta)
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
            .add_option({
                let option = CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "start",
                    "start from"
                );
                option
            })
            .add_option({
                let option = CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "duration",
                    "play how long"
                );
                option
            })
            .add_option({
                let option = CreateCommandOption::new(
                    CommandOptionType::String,
                    "skip",
                    "keyword for skip the song"
                );
                option
            })
    }
}