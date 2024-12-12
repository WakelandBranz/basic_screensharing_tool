use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde_json::json;

#[derive(Serialize)]
pub(crate) struct DiscordWebhook<'a> {
    username: Option<&'a str>,
    avatar_url: Option<&'a str>,
    content: Option<&'a str>,
    embeds: Option<Vec<DiscordEmbed>>,
}

#[derive(Serialize)]
pub(crate) struct DiscordEmbed {
    pub(crate) title: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) color: Option<u32>,
    pub(crate) fields: Option<Vec<EmbedField>>,
}

#[derive(Serialize)]
pub(crate) struct EmbedField {
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) inline: Option<bool>,
}

pub(crate) async fn send_discord_webhook(
    url: String,
    username: Option<&str>,
    content: Option<&str>,
    embeds: Option<Vec<DiscordEmbed>>
) -> anyhow::Result<()> {
    let webhook = DiscordWebhook {
        username,
        avatar_url: None,
        content,
        embeds,
    };

    let client = Client::new();
    client.post(url)
        .json(&webhook)
        .send()
        .await?;

    Ok(())
}