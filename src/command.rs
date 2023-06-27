use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::Interaction,
    http::interaction::{InteractionResponse, InteractionResponseType},
    id::{marker::GuildMarker, Id},
};
use twilight_util::builder::InteractionResponseDataBuilder as IRDB;

use crate::{get_client, interaction_client, model::MessageModel};

#[derive(CommandModel, CreateCommand)]
#[command(name = "monitor")]
/// Toggles r9k monitoring for this channel
pub struct Monitor {}

impl Monitor {
    pub async fn handle(self, interaction: &Interaction, model: &mut MessageModel) {
        if let Some((guild, channel)) = interaction
            .guild_id
            .as_ref()
            .zip(interaction.channel.as_ref())
        {
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
    }
}

pub async fn init_commands_for_guild<'a>(guild: Id<GuildMarker>) {
    let _ = interaction_client()
        .set_guild_commands(guild, &[Monitor::create_command().into()])
        .await;
}

pub async fn init_commands() {
    if let Ok(list) = get_client().current_user_guilds().await {
        if let Ok(guilds) = list.model().await {
            for guild in guilds {
                init_commands_for_guild(guild.id).await;
            }
        }
    }
}
