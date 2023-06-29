use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use chrono::offset::Utc;
use tokio::sync::{Mutex, RwLock, RwLockReadGuard};
use twilight_model::channel::message::Message;
use twilight_model::id::marker::{ChannelMarker, GuildMarker};
use twilight_model::id::Id;

#[derive(Debug)]
pub enum MessageRejected {
    Text,
}

#[derive(Debug)]
pub enum MessageAccepted {
    /// The message is not part of a guild, which this bot does not check
    NotInGuild,
    /// The message has an attachment, which the r9k system lets pass for now
    HasAttachment,
    /// The message was sent in a guild, but not in a monitored channel
    NotMonitored,
    /// The message is sent by a bot, so we don't care too much
    Bot,
    /// The message is accepted by the r9k system
    Nominal,
}

struct MessageMeta {
    timestamp: i64,
}

impl<'a> From<&'a Message> for MessageMeta {
    fn from(value: &'a Message) -> Self {
        Self {
            timestamp: value.timestamp.as_micros(),
        }
    }
}

type GuildsMap = HashMap<Id<GuildMarker>, Mutex<GuildMeta>>;
#[derive(Default, Clone)]
pub struct MessageModel {
    guilds: Arc<RwLock<GuildsMap>>,
}

struct GuildMeta {
    messages: HashMap<String, MessageMeta>,
    monitored_channels: HashSet<Id<ChannelMarker>>,
    /// Amount of time a message will be guarded against
    ttl: i64,
}

impl Default for GuildMeta {
    fn default() -> Self {
        Self {
            messages: Default::default(),
            monitored_channels: Default::default(),
            ttl: 864_000_000_000,
        }
    }
}

impl MessageModel {
    pub async fn init_guild(&self, guild: Id<GuildMarker>) {
        let mut guild_map = self.guilds.write().await;
        guild_map.insert(guild, Default::default());
    }

    /// Gets a reference to the [GuildMeta] related to the guild with the given id, or creates a new
    /// one and assigns to it, only acquiring a write lock to the guild map if necessary. To bypass
    /// lifetime restrictions, this function requires the caller to hold a guard, which you can
    /// create by declaring an [Option::None] in the callee scope and passing a reference to it
    /// here.
    async fn get_or_init<'slf, 'grd>(
        &'slf self,
        guard: &'grd mut Option<RwLockReadGuard<'slf, GuildsMap>>,
        guild_id: Id<GuildMarker>,
    ) -> &'grd Mutex<GuildMeta>
    where
        'slf: 'grd,
    {
        *guard = Some(self.guilds.read().await);
        if unsafe { guard.as_ref().unwrap_unchecked() }
            .get(&guild_id)
            .is_none()
        {
            *guard = None;
            log::debug!("An entry was not found for guild {guild_id}, creating a default");
            let mut guild_map = self.guilds.write().await;
            guild_map.insert(guild_id, Default::default());
            *guard = Some(guild_map.downgrade());
        }
        unsafe {
            guard
                .as_ref()
                .unwrap_unchecked()
                .get(&guild_id)
                .unwrap_unchecked()
        }
    }

    /// Attempt to insert the message into this model. Will return an error if the message does not
    /// comply with previous messages, otherwise will return `Ok`
    pub async fn insert_message(
        &self,
        message: &Message,
    ) -> Result<MessageAccepted, MessageRejected> {
        let Some(guild_id) = message.guild_id else {return Ok(MessageAccepted::NotInGuild)};
        if !message.attachments.is_empty() {
            return Ok(MessageAccepted::HasAttachment);
        }

        let mut guild_map = None;
        let guild_meta_mx = self.get_or_init(&mut guild_map, guild_id).await;
        let mut guild_info = guild_meta_mx.lock().await;

        if !guild_info.monitored_channels.contains(&message.channel_id) {
            return Ok(MessageAccepted::NotMonitored);
        }
        if message.author.bot {
            return Ok(MessageAccepted::Bot);
        }

        let content = message.content.clone();
        if_chain::if_chain! {
            if let Some(meta) = guild_info.messages.insert(content, message.into());
            if Utc::now().timestamp_micros() < guild_info.ttl + meta.timestamp;
            then {
                Err(MessageRejected::Text)
            } else {
                Ok(MessageAccepted::Nominal)
            }
        }
    }

    pub async fn toggle_monitor(&self, guild: Id<GuildMarker>, channel: Id<ChannelMarker>) -> bool {
        let mut guild_map = None;
        let mut guild_info = self.get_or_init(&mut guild_map, guild).await.lock().await;

        if guild_info.monitored_channels.remove(&channel) {
            true
        } else {
            guild_info.monitored_channels.insert(channel);
            false
        }
    }
}
