use std::sync::OnceLock;

use twilight_http::{client::InteractionClient, Client};

use crate::config::config;

static CLIENT: OnceLock<Client> = OnceLock::new();
pub(crate) fn client() -> &'static Client {
    CLIENT.get_or_init(|| Client::new(config().discord_token.clone()))
}

pub(crate) fn interaction_client() -> InteractionClient<'static> {
    client().interaction(config().application_id)
}
