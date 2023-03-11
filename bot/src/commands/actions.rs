use poise::serenity_prelude::{Guild, GuildChannel};

pub mod add;
pub mod add_multiple;
pub mod list;
pub mod notes;
pub mod remove;
pub mod restart;

pub fn get_guild_channel(g: &Guild, channel_id: u64) -> Option<GuildChannel> {
    g.channels
        .iter()
        .find(|c| c.0 .0 == channel_id)
        .map(|c| c.1.clone())
        .and_then(|c| c.guild())
}
