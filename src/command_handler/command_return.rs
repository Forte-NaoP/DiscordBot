use poise::serenity_prelude as serenity;
use serenity::{
    CreateEmbed, Context, CommandInteraction, async_trait};

use std::any::Any;

pub enum CommandReturn {
    String(String),
    SingleEmbed(CreateEmbed),
    ControlInteraction(Box<dyn ControlInteraction + Send + Sync>),
    None,
}

#[async_trait]
pub trait ControlInteraction {
    async fn control_interaction(
        &mut self,
        ctx: &Context, 
        interaction: CommandInteraction, 
    ) -> Result<(), serenity::Error>;
    fn as_any(&self) -> &dyn Any;
}

