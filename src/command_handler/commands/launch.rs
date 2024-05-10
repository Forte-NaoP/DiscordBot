use poise::serenity_prelude as serenity;
use serenity::{CreateCommand, Context, CommandInteraction, EditInteractionResponse};

use crate::command_handler::command_handler::COMMAND_LIST;

pub fn register() -> CreateCommand {
    CreateCommand::new("launch")
        .description("서버 명령어 초기화")
}

pub async fn run(
    ctx: &Context,
    command: CommandInteraction
) {
    command.defer(&ctx.http).await.unwrap();

    let guild_id = command.guild_id.unwrap();
    let local_commands = guild_id.get_commands(&ctx.http).await.unwrap();
    for cmd in local_commands.iter() {
        guild_id.delete_command(&ctx.http, cmd.id).await.unwrap();
    }

    COMMAND_LIST.register(guild_id, ctx).await;

    if let Err(why) = command
        .edit_response(&ctx.http, {
            EditInteractionResponse::new()
                .content("등록 완료")
        })
        .await
    {
        println!("{:#?}", why);
    }

}

