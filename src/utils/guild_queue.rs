use songbird::{
    driver::Driver,
    events::{Event, EventContext, EventData, EventHandler, TrackEvent},
    input::Input,
    tracks::{Track, TrackHandle, TrackResult},
    id::ChannelId,
};

use poise::serenity_prelude as serenity;
use serenity::{async_trait, Context, GuildId};

use crate::GuildQueueKey;

use tracing::{info, warn};
use parking_lot::Mutex;
use std::{collections::VecDeque, ops::Deref, sync::Arc, time::Duration};

// Modified copy from songbird/src/tracks/queue.rs

#[derive(Clone, Debug, Default)]
pub struct GuildQueue {
    inner: Arc<Mutex<GuildQueueCore>>,
}

pub async fn get_guild_queue(ctx: &Context, guild_id: GuildId) -> GuildQueue {
    let guild_queue_map = {
        let data_read = ctx.data.read().await;
        data_read.get::<GuildQueueKey>().unwrap().clone()
    };
    let guild_queue = guild_queue_map
        .entry(guild_id)
        .or_insert_with(|| GuildQueue::new())
        .clone();
    guild_queue
}

#[derive(Debug)]
pub struct Queued(TrackHandle);

impl Deref for Queued {
    type Target = TrackHandle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Queued {
    /// Clones the inner handle
    #[must_use]
    pub fn handle(&self) -> TrackHandle {
        self.0.clone()
    }
}

#[derive(Debug, Default)]
pub struct GuildQueueCore {
    tracks: VecDeque<(Queued, Option<String>)>,
    channel: Option<ChannelId>
}

struct QueueHandler {
    remote_lock: Arc<Mutex<GuildQueueCore>>,
}

#[async_trait]
impl EventHandler for QueueHandler {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let mut inner = self.remote_lock.lock();

        // Due to possibility that users might remove, reorder,
        // or dequeue+stop tracks, we need to verify that the FIRST
        // track is the one who has ended.
        match ctx {
            EventContext::Track(ts) => {
                // This slice should have exactly one entry.
                // If the ended track has same id as the queue head, then
                // we can progress the queue.
                if inner.tracks.front()?.0.uuid() != ts.first()?.1.uuid() {
                    return None;
                }
            },
            _ => return None,
        }

        let _old = inner.tracks.pop_front();

        info!("Queued track ended: {:?}.", ctx);
        info!("{} tracks remain.", inner.tracks.len());

        // Keep going until we find one track which works, or we run out.
        while let Some(new) = inner.tracks.front() {
            if new.0.play().is_err() {
                // Discard files which cannot be used for whatever reason.
                warn!("Track in Queue couldn't be played...");
                inner.tracks.pop_front();
            } else {
                break;
            }
        }

        None
    }
}

struct SongPreloader {
    remote_lock: Arc<Mutex<GuildQueueCore>>,
}

#[async_trait]
impl EventHandler for SongPreloader {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let inner = self.remote_lock.lock();

        if let Some(track) = inner.tracks.get(1) {
            // This is the sync-version so that we can fire and ignore
            // the request ASAP.
            drop(track.0.make_playable());
        }

        None
    }
}

impl GuildQueue {
    /// Create a new, empty, track queue.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(GuildQueueCore {
                tracks: VecDeque::new(),
                channel: None
            })),
        }
    }

    pub fn register_channel(&self, channel: ChannelId) {
        let mut inner = self.inner.lock();
        inner.channel = Some(channel);
    }

    pub async fn add_source_with_word(&self, input: Input, word: String, driver: &mut Driver) -> TrackHandle {
        self.add(input.into(), Some(word), driver).await
    }

    /// Adds an audio source to the queue, to be played in the channel managed by `driver`.
    ///
    /// This method will preload the next track 5 seconds before the current track ends, if
    /// the [`AuxMetadata`] can be successfully queried for a [`Duration`].
    ///
    /// [`AuxMetadata`]: crate::input::AuxMetadata
    pub async fn add_source(&self, input: Input, driver: &mut Driver) -> TrackHandle {
        self.add(input.into(), None, driver).await
    }

    /// Adds a [`Track`] object to the queue, to be played in the channel managed by `driver`.
    ///
    /// This allows additional configuration or event handlers to be added
    /// before enqueueing the audio track. [`Track`]s will be paused pre-emptively.
    ///
    /// This method will preload the next track 5 seconds before the current track ends, if
    /// the [`AuxMetadata`] can be successfully queried for a [`Duration`].
    ///
    /// [`AuxMetadata`]: crate::input::AuxMetadata
    pub async fn add(&self, mut track: Track, word: Option<String>, driver: &mut Driver) -> TrackHandle {
        let preload_time = Self::get_preload_time(&mut track).await;
        self.add_with_preload(track, word, driver, preload_time)
    }

    pub(crate) async fn get_preload_time(track: &mut Track) -> Option<Duration> {
        let meta = match track.input {
            Input::Lazy(ref mut rec) | Input::Live(_, Some(ref mut rec)) =>
                rec.aux_metadata().await.ok(),
            Input::Live(_, None) => None,
        };

        meta.and_then(|meta| meta.duration)
            .map(|d| d.saturating_sub(Duration::from_secs(5)))
    }

    /// Add an existing [`Track`] to the queue, using a known time to preload the next track.
    ///
    /// `preload_time` can be specified to enable gapless playback: this is the
    /// playback position *in this track* when the the driver will begin to load the next track.
    /// The standard [`Self::add`] method use [`AuxMetadata`] to set this to 5 seconds before
    /// a track ends.
    ///
    /// A `None` value will not ready the next track until this track ends, disabling preload.
    ///
    /// [`AuxMetadata`]: crate::input::AuxMetadata
    #[inline]
    pub fn add_with_preload(
        &self,
        mut track: Track,
        word: Option<String>,
        driver: &mut Driver,
        preload_time: Option<Duration>,
    ) -> TrackHandle {
        // Attempts to start loading the next track before this one ends.
        // Idea is to provide as close to gapless playback as possible,
        // while minimising memory use.
        info!("Track added to queue.");

        let remote_lock = self.inner.clone();
        track.events.add_event(
            EventData::new(Event::Track(TrackEvent::End), QueueHandler { remote_lock }),
            Duration::ZERO,
        );

        if let Some(time) = preload_time {
            let remote_lock = self.inner.clone();
            track.events.add_event(
                EventData::new(Event::Delayed(time), SongPreloader { remote_lock }),
                Duration::ZERO,
            );
        }

        let (should_play, handle) = {
            let mut inner = self.inner.lock();

            let handle = driver.play(track.pause());
            inner.tracks.push_back((Queued(handle.clone()), word));

            (inner.tracks.len() == 1, handle)
        };

        if should_play {
            drop(handle.play());
        }

        handle
    }

    /// Returns a handle to the currently playing track.
    #[must_use]
    pub fn current(&self) -> Option<(TrackHandle, Option<String>)> {
        let inner = self.inner.lock();

        inner.tracks.front().map(|f| (f.0.handle(), f.1.clone()))
    }

    /// Attempts to remove a track from the specified index.
    ///
    /// The returned entry can be readded to *this* queue via [`modify_queue`].
    ///
    /// [`modify_queue`]: TrackQueue::modify_queue
    #[must_use]
    pub fn dequeue(&self, index: usize) -> Option<Queued> {
        self.modify_queue(|vq| vq.remove(index)).map(|(q, _)| q)
    }

    /// Returns the number of tracks currently in the queue.
    #[must_use]
    pub fn len(&self) -> usize {
        let inner = self.inner.lock();

        inner.tracks.len()
    }

    /// Returns whether there are no tracks currently in the queue.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        let inner = self.inner.lock();

        inner.tracks.is_empty()
    }

    /// Allows modification of the inner queue (i.e., deletion, reordering).
    ///
    /// Users must be careful to `stop` removed tracks, so as to prevent
    /// resource leaks.
    pub fn modify_queue<F, O>(&self, func: F) -> O
    where
        F: FnOnce(&mut VecDeque<(Queued, Option<String>)>) -> O,
    {
        let mut inner = self.inner.lock();
        func(&mut inner.tracks)
    }

    /// Pause the track at the head of the queue.
    pub fn pause(&self) -> TrackResult<()> {
        let inner = self.inner.lock();

        if let Some(handle) = inner.tracks.front() {
            handle.0.pause()
        } else {
            Ok(())
        }
    }

    /// Resume the track at the head of the queue.
    pub fn resume(&self) -> TrackResult<()> {
        let inner = self.inner.lock();

        if let Some(handle) = inner.tracks.front() {
            handle.0.play()
        } else {
            Ok(())
        }
    }

    /// Stop the currently playing track, and clears the queue.
    pub fn stop(&self) {
        let mut inner = self.inner.lock();

        for track in inner.tracks.drain(..) {
            // Errors when removing tracks don't really make
            // a difference: an error just implies it's already gone.
            drop(track.0.stop());
        }
    }

    /// Skip to the next track in the queue, if it exists.
    pub fn skip(&self) -> TrackResult<()> {
        let inner = self.inner.lock();

        inner.stop_current()
    }

    /// Returns a list of currently queued tracks.
    ///
    /// Does not allow for modification of the queue, instead returns a snapshot of the queue at the time of calling.
    ///
    /// Use [`modify_queue`] for direct modification of the queue.
    ///
    /// [`modify_queue`]: TrackQueue::modify_queue
    #[must_use]
    pub fn current_queue(&self) -> Vec<(TrackHandle, Option<String>)> {
        let inner = self.inner.lock();

        inner.tracks.iter().map(|e| (e.0.handle(), e.1.clone())).collect()
    }

    pub fn channel(&self) -> Option<ChannelId> {
        let inner = self.inner.lock();
        inner.channel.clone()
    }

}

impl GuildQueueCore {
    /// Skip to the next track in the queue, if it exists.
    fn stop_current(&self) -> TrackResult<()> {
        if let Some(handle) = self.tracks.front() {
            handle.0.stop()
        } else {
            Ok(())
        }
    }
}