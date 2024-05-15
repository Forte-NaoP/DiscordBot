use std::collections::HashMap;
use std::sync::Arc;

use poise::serenity_prelude as serenity;
use serenity::{
    async_trait,
    GuildId,
    builder::*,
    CreateCommand, CommandInteraction, CommandDataOption, EditInteractionResponse, Context,
    CreateInteractionResponse, CreateEmbed
};

use lazy_static::lazy_static;
use songbird::tracks::PlayMode;

use crate::command_handler::{ command_return::*, commands::* };

#[async_trait]
pub trait CommandInterface {
    async fn run(
        &self, 
        ctx: &Context, 
        command: &CommandInteraction, 
        options: &[CommandDataOption]
    ) -> CommandReturn;
    fn register(&self) -> CreateCommand;
}

pub struct CommandList {
    pub commands: HashMap<&'static str, Box<dyn CommandInterface + Send + Sync>>,
}

impl CommandList {
    pub async fn register(&'static self, gid: GuildId, ctx: &Context) {
        for (_, command) in &self.commands {
            if let Err(why) = gid
                .create_command(&ctx.http, command.register())
                .await
            {
                println!("Cannot create application command: {:#?}", why);
            }
        }
    }
}

lazy_static! {
    pub static ref COMMAND_LIST: CommandList = CommandList {
        commands: HashMap::from([
            ("join", connect::command()),
            ("leave", disconnect::command()),
            ("play", play::command())
        ])
    };
}

pub async fn execute_command(ctx: Arc<serenity::Context>, command: CommandInteraction) {

    command.defer(&ctx.http).await.unwrap();
    let cmd_result = match COMMAND_LIST.commands.get(command.data.name.as_str()) {
        Some(result) => result.run(&ctx, &command, &command.data.options).await,
        None => CommandReturn::String("등록되지않은 명령어입니다.".to_string()),
    };

    match cmd_result {
        CommandReturn::String(content) => {
            if let Err(why) = command
                .edit_response(&ctx.http, {
                    EditInteractionResponse::new()
                        .content(content.clone())
                })
                .await
            {
                println!(
                    "Failed to send Single-string \"{:?}\" from command \"{}\".",
                    content, command.data.name
                );
                println!("{:#?}", why);
            }
        }
        CommandReturn::SingleEmbed(embed) => {
            if let Err(why) = command
                .edit_response(&ctx.http, {
                    EditInteractionResponse::new()
                        .add_embed(embed.clone())
                })
                .await
            {
                println!(
                    "Failed to send single-embed \"{:#?}\" from command \"{}\".",
                    embed, command.data.name
                );
                println!("{:#?}", why);
            }
        }
        CommandReturn::SongInfoEmbed(handle, meta) => {
            if let Err(why) = command
                .edit_response(&ctx.http, {
                    EditInteractionResponse::new()
                        .content("재생 시작")
                })
                .await
            {
                println!(
                    "Failed to send Single-string from command \"{}\".",
                    command.data.name
                );
                println!("{:#?}", why);
            };
            let title = meta.title.clone().unwrap_or("제목 없음".to_owned());
            let duration = meta.duration.clone().unwrap_or(0);
            let info = handle.get_info().await.unwrap();
            let embed = CreateEmbed::new()
                .title(&title)
                .description(format!("{}:{} / {}:{}", 
                    info.play_time.as_secs() / 60, info.play_time.as_secs() % 60,
                    duration / 60, duration % 60));
            let mut msg = command.channel_id.send_message(
                &ctx.http,
                CreateMessage::new().embed(embed.clone()))
                .await
                .unwrap();

            tokio::spawn(async move {
                while let Ok(info) = handle.get_info().await {
                    if info.playing == PlayMode::Stop {
                        println!("{} stopped", &title);
                        break;
                    }
                    let embed = CreateEmbed::new()
                        .title(title.as_str())
                        .description(format!("{}:{} / {}:{}", 
                            info.play_time.as_secs() / 60, info.play_time.as_secs() % 60,
                            duration / 60, duration % 60));
                    if let Err(e) = msg.edit(&ctx.http, EditMessage::new().embed(embed)).await {
                        println!("Failed to update message: {:?}", e);
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                } 
            });
            
            
        }
        CommandReturn::ControlInteraction(mut pages) => {          
            if let Err(why) = pages.control_interaction(ctx, command).await {
                println!("an error occured while handling embed pages.");
                println!("{:#?}", why);
            }
        }
        _ => command.delete_response(&ctx.http).await.unwrap(),
    }

}