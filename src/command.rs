use twilight_http::{client::InteractionClient, Client};
use twilight_model::{
    application::command::{Command, CommandType},
    id::{marker::GuildMarker, Id},
};

use crate::{get_client, APP_ID};

fn monitor_command() -> Command {
    Command {
        application_id: None,
        default_member_permissions: None,
        dm_permission: None,
        description: "Toggles r9k monitoring for the channel".to_string(),
        description_localizations: None,
        guild_id: None,
        id: None,
        kind: CommandType::ChatInput,
        name: "monitor".to_string(),
        name_localizations: None,
        nsfw: Some(false),
        options: vec![],
        version: Id::new(1),
    }
}

pub async fn init_commands_for_guild<'a>(
    interaction_builder: &InteractionClient<'a>,
    guild: Id<GuildMarker>,
) {
    let _ = interaction_builder
        .set_guild_commands(guild, &[monitor_command()])
        .await;
}

pub async fn init_commands(client: &Client) {
    let interaction_builder = client.interaction(APP_ID);
    if let Ok(list) = get_client().current_user_guilds().await {
        if let Ok(guilds) = list.model().await {
            for guild in guilds {
                init_commands_for_guild(&interaction_builder, guild.id).await;
            }
        }
    }
}
