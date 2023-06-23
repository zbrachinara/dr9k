use std::collections::{HashMap, HashSet};

use chrono::Utc;
use twilight_model::channel::message::Message;
use twilight_model::channel::message::component::ComponentType;
use twilight_model::id::marker::{GuildMarker, ChannelMarker};
use twilight_model::id::Id;

#[derive(Debug)]
pub enum MessageRejected {
    Text,
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
    monitored_channels: HashSet<Id<ChannelMarker>>
}

impl MessageModel {
    /// Attempt to insert the message into this model. Will return an error if the message does not
    /// comply with previous messages, otherwise will return `Ok`
    pub fn insert_message(&mut self, message: &Message) -> Result<(), MessageRejected> {
        if !message.attachments.is_empty() {
            return Ok(()) // for now, we will escape images
        }

        let content = message.content.clone();
        let Some(guild_id) = message.guild_id else {return Ok(())}; // this is not from a guild, no reason to reject
        let guild_info = self.guilds.entry(guild_id).or_default();

        if guild_info.messages.insert(content, message.into()).is_some() {
            Err(MessageRejected::Text)
        } else {
            Ok(())
        }
    }
}
