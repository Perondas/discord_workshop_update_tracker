use std::sync::Arc;

use itertools::Itertools;
use poise::serenity_prelude::{CacheAndHttp, GuildId};
use tracing::{info, warn};

use crate::{db, scheduler::Scheduler, steam, Error};

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
        let item_info = steam::get_item(&scheduler.pool, item_info.id).await?;

        if item_info.last_updated > last_notify {
            updated.push((item_info, last_notify, note));
        } else {
            unknown.push((last_notify, item_info, note));
        }
    }

    for (last_notify, item_info, note) in unknown {
        if let Ok(info) = steam::get_latest_item(&scheduler.pool, item_info.id).await {
            if info.last_updated > last_notify {
                updated.push((info, last_notify, note));
            }
        } else {
            failed.push((last_notify, item_info));
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
            send_in_chunks_updates(c, client, &updated).await?;
        } else {
            send_in_one_updates(c, client, &updated).await?;
        }

        for (item_info, _, _) in updated {
            db::subscriptions::update_last_notify(&scheduler.pool, guild_id, item_info.id)?;
        }
    }

    if !failed.is_empty() {
        c.send_message(&client, |d| {
            d.content("The following Items could not be updated:".to_string());

            for (_, item_info) in failed.iter() {
                d.add_embed(|e| {
                    e.title(format!("{}, Id: {}", item_info.name.clone(), item_info.id));
                    e.url(format!(
                        "https://steamcommunity.com/sharedfiles/filedetails/?id={}",
                        item_info.id
                    ));
                    e
                });
            }

            d
        })
        .await?;
    }

    Ok(())
}

async fn send_in_chunks_updates(
    c: &poise::serenity_prelude::GuildChannel,
    client: &CacheAndHttp,
    updated: &[(db::ItemInfo, u64, Option<String>)],
) -> Result<(), Error> {
    let chunks: Vec<Vec<(db::ItemInfo, Option<String>)>> = updated
        .iter()
        .chunks(5)
        .into_iter()
        .map(|c| c.map(|(m, _, n)| (m.clone(), n.clone())).collect())
        .collect();

    let parts = chunks.len();

    for (curr, chunk) in chunks.iter().enumerate() {
        c.send_message(&client, |d| {
            d.content(format!(
                "The following Items have been updated: Part {}/{}",
                curr + 1,
                parts
            ));

            for (item_info, note) in chunk.iter() {
                d.add_embed(|e| {
                    e.title(item_info.name.clone());
                    e.url(format!(
                        "https://steamcommunity.com/sharedfiles/filedetails/?id={}",
                        item_info.id
                    ));
                    if let Some(url) = item_info.preview_url.clone() {
                        e.image(url);
                    }

                    if let Some(note) = note {
                        e.field("Note:", note, false);
                    }

                    e
                });
            }

            d
        })
        .await?;
    }

    Ok(())
}

async fn send_in_one_updates(
    c: &poise::serenity_prelude::GuildChannel,
    client: &Arc<CacheAndHttp>,
    updated: &[(db::ItemInfo, u64, Option<String>)],
) -> Result<(), Error> {
    c.send_message(&client, |d| {
        d.content("The following Items have been updated:".to_string());

        for (item_info, _, note) in updated.iter() {
            d.add_embed(|e| {
                e.title(item_info.name.clone());
                e.url(format!(
                    "https://steamcommunity.com/sharedfiles/filedetails/?id={}",
                    item_info.id
                ));
                if let Some(url) = item_info.preview_url.clone() {
                    e.image(url);
                }

                if let Some(note) = note {
                    e.field("Note:", note, false);
                }

                e
            });
        }

        d
    })
    .await?;
    Ok(())
}
