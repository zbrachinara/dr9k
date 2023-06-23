use std::collections::HashMap;

use chrono::Utc;
use twilight_model::id::marker::GuildMarker;

enum MessageRejected {

}

struct MessageMeta {
    timestamp: chrono::DateTime<Utc>,
}

#[derive(Default)]
struct MessageModel {
    guilds: HashMap<GuildMarker, HashMap<String, MessageMeta>>,
}

impl MessageModel {
    /// Attempt to insert the message into this model. Will return an error if the message does not
    /// comply with previous messages, otherwise will return `Ok`
    fn insert_message() -> Result<(), MessageRejected> {
        todo!()
    }
}