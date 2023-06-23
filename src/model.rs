use std::collections::{HashMap, HashSet};

use chrono::Utc;
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
    guilds: HashMap<Id<GuildMarker>, GuildInfo>,
}

#[derive(Default)]
struct GuildInfo {
    messages: HashMap<String, MessageMeta>,
    monitored_channels: HashSet<Id<ChannelMarker>>,
}

impl MessageModel {
    /// Attempt to insert the message into this model. Will return an error if the message does not
    /// comply with previous messages, otherwise will return `Ok`
    pub fn insert_message(&mut self, message: &Message) -> Result<MessageAccepted, MessageRejected> {
        let Some(guild_id) = message.guild_id else {return Ok(MessageAccepted::NotInGuild)};
        let guild_info = self.guilds.entry(guild_id).or_default();

        if !message.attachments.is_empty()
        {
            return Ok(MessageAccepted::HasAttachment);
        }
        if !guild_info.monitored_channels.contains(&message.channel_id) {
            return Ok(MessageAccepted::NotMonitored)
        }

        let content = message.content.clone();

        if guild_info
            .messages
            .insert(content, message.into())
            .is_some()
        {
            Err(MessageRejected::Text)
        } else {
            Ok(MessageAccepted::Nominal)
        }
    }
}
