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
use tracing::info;

use crate::{
    command_handler::{
        command_handler::*,
        command_return::CommandReturn,
    }, connection_handler::*, global::*, utils::{guild_queue::get_guild_queue, url_checker::{self, url_checker}, youtube_dl::{ytdl_optioned, MetaData}}
};

use std::{collections::HashMap, io::Read, path::PathBuf, sync::Arc};

use chrono::Utc;

struct PlayList;

pub fn command() -> Box<dyn CommandInterface + Sync + Send> {
    Box::new(PlayList)
}

#[derive(Debug)]
struct ListOption {
    pub url: String,
    pub start: i64,
    pub duration: i64,
    pub skip: Option<String>,
}

impl TryFrom<&str> for ListOption {
    type Error = &'static str;
    
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut iter = value.split(',');
        let url = iter.next()
            .ok_or("missing url")
            .and_then(|url| url_checker(url).ok_or("invalid url"))
            .unwrap();
        let start = iter.next().and_then(|value| value.parse().ok()).unwrap_or(0);
        let duration = iter.next().and_then(|value| value.parse().ok()).unwrap_or(0);
        let skip = iter.next().map(|value| value.to_owned());

        Ok(ListOption {
            url,
            start,
            duration,
            skip,
        })
    }
}

#[async_trait]
impl CommandInterface for PlayList {
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

        let attachment_id = options.get(0)
            .and_then(|option| option.value.as_attachment_id())
            .unwrap();

        let attachment = command.data.resolved.attachments.get(&attachment_id).unwrap();
        let stream = attachment.download().await.unwrap();
        let (lines, failed) = String::from_utf8(stream)
            .unwrap()
            .lines()
            .enumerate()
            .fold((Vec::new(), Vec::new()), |(mut lines, mut failed), (index, line)| {
                match ListOption::try_from(line) {
                    Ok(option) => lines.push(option),
                    Err(_) => failed.push(index),
                }
                (lines, failed)
            });

        let guild_id = command.guild_id.unwrap();
        let guild_queue = get_guild_queue(ctx, guild_id).await;
        let manager = songbird::get(ctx).await.unwrap().clone();
        
        info!("{:?}", lines);

        for line in &lines {
            let manager_clone = manager.clone();
            if let Some(handler_lock) = manager_clone.get(guild_id) {
                let (path, output) = ytdl_optioned(&line.url, line.start, line.duration).await.unwrap();
                let src = File::new(path);
                let mut handler = handler_lock.lock().await;
                let mut meta: MetaData = output.into();
                meta.keyword = line.skip.clone();
                
                // guild_queue.add_source((INTERVAL.as_ref() as &[u8]).into(), INTERVAL_META.clone(), &mut handler).await;
                guild_queue.add_source(src.into(), meta, &mut handler, true).await;
            }
        }

        let mut result = String::new();
        if !failed.is_empty() {
            result.push_str(
                format!("총 {}개의 행이 무시되었습니다.\n무시된 행: {}\n",
                    failed.len(),
                    failed.iter()
                        .map(|index| index.to_string())
                        .collect::<Vec<String>>()
                        .join(", "))
                .as_str());
        }
        result.push_str(
            format!("총 {}개의 곡이 추가되었습니다.", lines.len())
            .as_str());
        CommandReturn::String(result)
    }

    fn register(&self) -> CreateCommand {
        let command = CreateCommand::new("");
        command
            .name("playlist")
            .description("play youtube from csv file")
            .add_option({
                let option = CreateCommandOption::new(
                    CommandOptionType::Attachment,
                    "csv",
                    "csv file"
                );
                option.required(true)
            })
    }
}