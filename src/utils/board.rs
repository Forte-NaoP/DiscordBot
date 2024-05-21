use poise::serenity_prelude as serenity;
use serenity::{
    Context, ChannelType, Message, GuildChannel, GuildId, CreateThread, CreateMessage, CreateEmbed
};

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::BoardKey;

#[derive(Clone, Debug, Default)]
pub struct Board {
    inner: Arc<Mutex<BoardCore>>
}

#[derive(Debug, Default)]
pub struct BoardCore {
    thread: Option<GuildChannel>,
    score: Option<Message>,
    status: Option<Message>,
}

pub async fn get_board(ctx: &Context, guild_id: GuildId) -> Board {
    let board_map = {
        let data_read = ctx.data.read().await;
        data_read.get::<BoardKey>().unwrap().clone()
    };
    let board = board_map
        .entry(guild_id)
        .or_insert_with(|| Board::new())
        .clone();
    board
}

impl Board {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new( BoardCore {
                thread: None,
                score: None,
                status: None
            })),
        }
    }

    pub async fn delete(&mut self, ctx: &Context) {
        let mut inner = self.inner.lock().await;
        if let Some(thread) = inner.thread.take() {
            thread.delete(&ctx.http).await.unwrap();
            inner.score = None;
            inner.status = None;
        }
    }

    async fn reset(&mut self, thread: GuildChannel, score: Message, status: Message) {
        let mut inner = self.inner.lock().await;
        inner.thread = Some(thread);
        inner.score = Some(score);
        inner.status = Some(status);
    }

    pub async fn set(&mut self, ctx: &Context, channel: GuildChannel) {
        self.delete(ctx).await;

        let thread = CreateThread::new("Board").kind(ChannelType::PublicThread);
        let thread_channel = channel
            .create_thread(&ctx.http, thread)
            .await
            .unwrap();
        
        let score_embed = CreateEmbed::new().title("Score");
        let score = thread_channel
            .send_message(&ctx.http, CreateMessage::new().add_embed(score_embed))
            .await
            .unwrap();

        let status_embed = CreateEmbed::new().title("재생중인 곡");
        let status = thread_channel
            .send_message(&ctx.http, CreateMessage::new().add_embed(status_embed))
            .await
            .unwrap();

        self.reset(thread_channel, score, status).await;
    }
}