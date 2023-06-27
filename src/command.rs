use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::id::{marker::GuildMarker, Id};

use crate::{get_client, interaction_client};

#[derive(CommandModel, CreateCommand)]
#[command(name = "monitor")]
/// Toggles r9k monitoring for this channel
struct Monitor {}

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
