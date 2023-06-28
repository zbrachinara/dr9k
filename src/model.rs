use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

use chrono::offset::Utc;
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

#[derive(Default)]
pub struct MessageModel {
    guilds: HashMap<Id<GuildMarker>, GuildMeta>,
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
    /// Attempt to insert the message into this model. Will return an error if the message does not
    /// comply with previous messages, otherwise will return `Ok`
    pub fn insert_message(
        &mut self,
        message: &Message,
    ) -> Result<MessageAccepted, MessageRejected> {
        let Some(guild_id) = message.guild_id else {return Ok(MessageAccepted::NotInGuild)};
        let guild_info = self.guilds.entry(guild_id).or_default();

        if !message.attachments.is_empty() {
            return Ok(MessageAccepted::HasAttachment);
        }
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

    pub fn toggle_monitor(&mut self, guild: Id<GuildMarker>, channel: Id<ChannelMarker>) -> bool {
        let guild_info = self.guilds.entry(guild).or_default();
        if guild_info.monitored_channels.remove(&channel) {
            true
        } else {
            guild_info.monitored_channels.insert(channel);
            false
        }
    }
}
