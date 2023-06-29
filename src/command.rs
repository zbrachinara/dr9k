use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::Interaction,
    http::interaction::{InteractionResponse, InteractionResponseType},
    id::{marker::GuildMarker, Id},
};
use twilight_util::builder::InteractionResponseDataBuilder as IRDB;

use crate::{
    clients::{client, interaction_client},
    model::MessageModel,
};

#[derive(CommandModel, CreateCommand)]
#[command(name = "monitor")]
/// Controls r9k monitoring for this channel
pub enum Monitor {
    #[command(name = "enable")]
    Enable(EnableCommand),
    #[command(name = "check")]
    Check(CheckCommand),
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "enable")]
/// Toggles r9k monitoring for this channel
pub struct EnableCommand;

#[derive(CommandModel, CreateCommand)]
#[command(name = "check")]
/// Toggles r9k monitoring for this channel
pub struct CheckCommand;

impl Monitor {
    pub async fn handle(self, interaction: &Interaction, model: &MessageModel) {
        match self {
            Self::Enable(_) => {
                if let Some((guild, channel)) = interaction
                    .guild_id
                    .as_ref()
                    .zip(interaction.channel.as_ref())
                {
                    let _ = interaction_respond(
                        if model.toggle_monitor(*guild, channel.id).await {
                            "Stopping monitoring of this channel"
                        } else {
                            "Beginning to monitor this channel"
                        },
                        interaction,
                    )
                    .await;
                }
            }
            Self::Check(_) => {
                if let Some((guild, channel)) = interaction
                    .guild_id
                    .as_ref()
                    .zip(interaction.channel.as_ref())
                {
                    let _ = interaction_respond(
                        if model.is_monitored(*guild, channel.id).await {
                            "This channel is being monitored"
                        } else {
                            "This channel is not being monitored"
                        },
                        interaction,
                    )
                    .await;
                }
            }
        }
    }
}

async fn interaction_respond(
    message: &str,
    interaction: &Interaction,
) -> Result<(), twilight_http::Error> {
    interaction_client()
        .create_response(
            interaction.id,
            &interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(IRDB::new().content(message).build()),
            },
        )
        .await
        .map(|_| ())
}

pub async fn init_commands_for_guild<'a>(guild: Id<GuildMarker>) {
    let _ = interaction_client()
        .set_guild_commands(guild, &[Monitor::create_command().into()])
        .await;
}

pub async fn init_commands() {
    if let Ok(list) = client().current_user_guilds().await {
        if let Ok(guilds) = list.model().await {
            for guild in guilds {
                init_commands_for_guild(guild.id).await;
            }
        }
    }
}
