use log::*;
use twilight_gateway::{Intents, Shard, ShardId};

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    let secrets_file = include_bytes!("../conf.properties");
    let secrets = java_properties::read(secrets_file.as_slice()).unwrap();
    println!("{secrets:?}");

    let token = secrets["discord_token"].clone();
    let intents = Intents::all();

    let mut shard = Shard::new(ShardId::ONE, token, intents);

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
        debug!("received event: {event:?}");
    }
}
