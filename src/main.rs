use config::config;
use log::*;
use twilight_gateway::{Event, Intents, Shard, ShardId};
use twilight_interactions::command::{CommandInputData, CommandModel};
use twilight_model::application::interaction::InteractionData;

use crate::{clients::client, command::Monitor, model::MessageModel};

mod clients;
mod command;
mod config;
mod file;
mod model;

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    let mut model = MessageModel::default();

    let token = config().discord_token.clone();
    let mut shard = Shard::new(ShardId::ONE, token.clone(), Intents::all());

    command::init_commands().await;

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

        debug!("{event:?}");
        #[allow(clippy::single_match)]
        match event {
            Event::MessageCreate(c) => {
                if model.insert_message(&c.0).is_err() {
                    tokio::spawn(async move {
                        if let Err(e) = client().delete_message(c.channel_id, c.id).await {
                            error!("Failure in deleting repeated message:\n{e}");
                        }
                    });
                }
            }
            Event::InteractionCreate(interaction) => {
                if let Some(InteractionData::ApplicationCommand(ref command)) = interaction.data {
                    let command_data = CommandInputData::from((**command).clone());
                    match command.name.as_str() {
                        "monitor" => {
                            Monitor::from_interaction(command_data)
                                .unwrap()
                                .handle(&interaction, &mut model)
                                .await;
                        }
                        _ => continue,
                    }
                }
            }
            _ => {}
        }
    }
}
