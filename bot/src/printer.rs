use std::time;

use itertools::Itertools;
use poise::serenity_prelude::{CacheHttp, CreateEmbed, GuildId};
use tracing::{info, warn};

use crate::{
    db::{self, ItemInfo},
    scheduler::Scheduler,
    steam, Error,
};

pub async fn notify_on_updates(scheduler: Scheduler, guild_id: u64) -> Result<(), Error> {
    let client = scheduler.client.read().await;

    let client = match &*client {
        Some(c) => c,
        None => {
            warn!("Client not set, skipping update check");
            return Ok(());
        }
    };

    let subscriptions =
        db::subscriptions::get_all_subscriptions_of_guild(&scheduler.pool, guild_id)?;

    let mut updated = Vec::new();
    let mut unknown = Vec::new();
    let mut failed = Vec::new();

    for (last_notify, item_info, note) in subscriptions {
        // Only notify once per hour
        if last_notify + 60 * 60
            > time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)?
                .as_secs()
        {
            continue;
        }

        let item_info = match steam::get_item(&scheduler.pool, item_info.id).await {
            Ok(info) => info,
            Err(e) => {
                warn!("Failed to get item info: {}", e);
                failed.push((item_info, note));
                continue;
            }
        };

        if item_info.last_updated > last_notify {
            updated.push((item_info, note));
        } else {
            unknown.push((last_notify, item_info, note));
        }
    }

    for (last_notify, item_info, note) in unknown {
        if let Ok(info) = steam::get_latest_item(&scheduler.pool, item_info.id).await {
            if info.last_updated > last_notify {
                updated.push((info, note));
            }
        } else {
            failed.push((item_info, note));
        }
    }

    let update_channel = db::servers::get_update_channel(&scheduler.pool, guild_id)?
        .ok_or("No update channel set")?;

    let id = GuildId(guild_id);
    let g = id.to_partial_guild(&client.http).await?;

    let channels = g.channels(&client.http).await?;

    let c = match channels.iter().find(|c| c.0 .0 == update_channel) {
        Some(c) => c.1,
        None => {
            return Err(format!(
                "Update channel {} not found in guild {}",
                update_channel, guild_id
            )
            .into())
        }
    };

    if updated.is_empty() {
        info!("No updates for guild: {}", guild_id);
    } else {
        info!("Found {} updates for guild: {}", updated.len(), guild_id);
        if updated.len() > 5 {
            send_in_chunks("The following items were updated:", c, client, &updated).await?;
        } else {
            send_in_one("The following items were updated:", c, client, &updated).await?;
        }

        for (item_info, _) in updated {
            db::subscriptions::update_last_notify(&scheduler.pool, guild_id, item_info.id)?;
        }
    }

    if !failed.is_empty() {
        if failed.len() > 5 {
            send_in_chunks(
                "The following Items could not be updated:",
                c,
                client,
                &failed,
            )
            .await?;
        } else {
            send_in_one(
                "The following Items could not be updated:",
                c,
                client,
                &failed,
            )
            .await?
        }
    }

    Ok(())
}

pub async fn send_in_chunks(
    msg: &str,
    c: &poise::serenity_prelude::GuildChannel,
    client: impl CacheHttp,
    updated: &[(db::ItemInfo, Option<String>)],
) -> Result<(), Error> {
    let chunks: Vec<Vec<(db::ItemInfo, Option<String>)>> = updated
        .iter()
        .chunks(5)
        .into_iter()
        .map(|c| c.map(|(m, n)| (m.clone(), n.clone())).collect())
        .collect();

    let parts = chunks.len();

    for (curr, chunk) in chunks.iter().enumerate() {
        c.send_message(&client, |d| {
            d.content(format!("{}\nPart {}/{}", msg, curr + 1, parts));

            for (item_info, note) in chunk.iter() {
                d.add_embed(|e| {
                    item_to_embed(e, item_info, note);
                    e
                });
            }

            d
        })
        .await?;
    }

    Ok(())
}

pub async fn send_in_one(
    msg: &str,
    c: &poise::serenity_prelude::GuildChannel,
    client: impl CacheHttp,
    updated: &[(db::ItemInfo, Option<String>)],
) -> Result<(), Error> {
    c.send_message(&client, |d| {
        d.content(msg);

        for (item_info, note) in updated.iter() {
            d.add_embed(|e| {
                item_to_embed(e, item_info, note);
                e
            });
        }

        d
    })
    .await?;
    Ok(())
}

fn item_to_embed(e: &mut CreateEmbed, item_info: &ItemInfo, note: &Option<String>) {
    e.title(&item_info.name);
    e.url(format!(
        "https://steamcommunity.com/sharedfiles/filedetails/?id={}",
        item_info.id
    ));

    if let Some(url) = item_info.preview_url.as_ref() {
        e.image(url);
    }

    if let Some(note) = note {
        e.footer(|f| {
            f.text(format!("Note: {}", note));
            f
        });
    }
}
