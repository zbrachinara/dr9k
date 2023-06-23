use std::sync::OnceLock;

use log::*;
use twilight_gateway::{Event, Intents, Shard, ShardId};
use twilight_http::Client;

use crate::model::MessageModel;

mod model;

static CLIENT: OnceLock<Client> = OnceLock::new();

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    let secrets_file = include_bytes!("../conf.properties");
    let secrets = java_properties::read(secrets_file.as_slice()).unwrap();
    println!("{secrets:?}");

    let mut model = MessageModel::default();

    let token = &secrets["discord_token"];
    let intents = Intents::all();

    let mut shard = Shard::new(ShardId::ONE, token.clone(), intents);
    CLIENT
        .set(Client::new(token.clone()))
        .expect("Could not initialize http client to Discord.");
    // let client = Client::new(token.clone());

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

        // You'd normally want to spawn a new tokio task for each event and
        // handle the event there to not block the shard.
        // debug!("received event: {event:?}");
        #[allow(clippy::single_match)]
        match event {
            Event::MessageCreate(c) => {
                if model.insert_message(&c.0).is_err() {
                    tokio::spawn(async move {
                        let _ = CLIENT
                            .get()
                            .expect("The client has not initialized")
                            .delete_message(c.channel_id, c.id)
                            .await;
                    });
                }
            }
            _ => {}
        }
    }
}
