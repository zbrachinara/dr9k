use std::sync::OnceLock;

use config::config;
use log::*;
use twilight_gateway::{Event, Intents, Shard, ShardId};
use twilight_http::{client::InteractionClient, Client};
use twilight_model::{
    application::interaction::InteractionData,
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder as IRDB;

use crate::model::MessageModel;

mod command;
mod config;
mod file;
mod model;

static CLIENT: OnceLock<Client> = OnceLock::new();
pub(crate) fn get_client() -> &'static Client {
    CLIENT
        .get()
        .expect("Client was requested before client was initialized -- contact developer")
}

pub(crate) fn interaction_client() -> InteractionClient<'static> {
    get_client().interaction(config().application_id)
}

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    let mut model = MessageModel::default();

    let token = config().discord_token.clone();
    let mut shard = Shard::new(ShardId::ONE, token.clone(), Intents::all());
    CLIENT
        .set(Client::new(token))
        .expect("Could not initialize http client to Discord.");

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
            Event::InteractionCreate(interaction) => {
                let (guild, channel) = if_chain::if_chain! {
                    if let Some(InteractionData::ApplicationCommand(ref command))
                        = interaction.data;
                    if command.name == "monitor";
                    if let Some(ref guild) = interaction.guild_id;
                    if let Some(ref channel) = interaction.channel;
                    then {
                        (guild, channel)
                    } else {
                        continue;
                    }
                };

                let _ = interaction_client()
                    .create_response(
                        interaction.id,
                        &interaction.token,
                        &InteractionResponse {
                            kind: InteractionResponseType::ChannelMessageWithSource,
                            data: Some(
                                IRDB::new()
                                    .content(if model.toggle_monitor(*guild, channel.id) {
                                        "Stopping monitoring of this channel"
                                    } else {
                                        "Beginning to monitor this channel"
                                    })
                                    .build(),
                            ),
                        },
                    )
                    .await;
            }
            _ => {}
        }
    }
}
