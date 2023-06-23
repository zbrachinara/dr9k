use std::sync::OnceLock;

use log::*;
use twilight_gateway::{Event, Intents, Shard, ShardId};
use twilight_http::Client;

use crate::model::MessageModel;

mod model;
mod file;
mod command;

static CLIENT: OnceLock<Client> = OnceLock::new();
pub(crate) fn get_client() -> &'static Client{
    CLIENT.get().expect("Client was requested before client was initialized -- contact developer")
}

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    let mut model = MessageModel::default();
    
    let token = env!("discord_token");
    let mut shard = Shard::new(ShardId::ONE, token.to_string(), Intents::all());
    CLIENT
        .set(Client::new(token.to_string()))
        .expect("Could not initialize http client to Discord.");

    command::init_commands(get_client());

    loop {
        let event = match shard.next_event().await {
            Ok(event) => event,
            Err(source) => {
                warn!("error receiving event: {source}");

                // If the error is fatal, as may be the case for invalid
                // authentication or intents, then break out of the loop to
                // avoid constantly attempting to reconnect.
                if source.is_fatal() {
                    break;
                }

                continue;
            }
        };

        #[allow(clippy::single_match)]
        match event {
            Event::MessageCreate(c) => {
                if model.insert_message(&c.0).is_err() {
                    tokio::spawn(async move {
                        if let Err(e) = CLIENT
                            .get()
                            .expect("The client has not initialized")
                            .delete_message(c.channel_id, c.id)
                            .await
                        {
                            error!("Failure in deleting repeated message:\n{e}");
                        }
                    });
                }
            }
            _ => {}
        }
    }
}
