use std::collections::HashMap;

use poise::serenity_prelude as serenity;
use serenity::{
    async_trait,
    GuildId,
    CreateCommand, CommandInteraction, CommandDataOption, EditInteractionResponse, Context
};

use lazy_static::lazy_static;

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

pub async fn execute_command(ctx: &serenity::Context, command: CommandInteraction) {

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
        CommandReturn::ControlInteraction(mut pages) => {          
            if let Err(why) = pages.control_interaction(ctx, command).await {
                println!("an error occured while handling embed pages.");
                println!("{:#?}", why);
            }
        }
        _ => command.delete_response(&ctx.http).await.unwrap(),
    }

}