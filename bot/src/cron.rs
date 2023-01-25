use std::{sync::Arc, time::Duration};

use dashmap::DashMap;
use itertools::Itertools;
use mysql::Pool;
use poise::serenity_prelude::{CacheAndHttp, GuildId};
use tokio::{sync::RwLock, task::JoinHandle, time::sleep};
use tracing::{info, warn};

use crate::{db, steam, Error};

#[derive(Clone)]
pub struct Scheduler {
    client: Arc<RwLock<Option<Arc<CacheAndHttp>>>>,
    jobs: Arc<DashMap<u64, JoinHandle<()>>>,
    pool: Arc<Pool>,
}

impl Scheduler {
    pub fn new(pool: Arc<Pool>) -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            jobs: Arc::new(DashMap::new()),
            pool,
        }
    }

    pub fn remove(&self, guild_id: u64) {
        info!("Removing cron job for guild: {}", guild_id);
        if let Some((_, job)) = self.jobs.remove(&guild_id) {
            job.abort();
        }
    }

    pub async fn start_cron(&self, client: Arc<CacheAndHttp>) -> Result<(), Error> {
        *self.client.write().await = Some(client.clone());

        let schedules = db::servers::get_all_schedules(&self.pool)?;
        let count = schedules.len();

        info!(
            "Starting {} cron jobs",
            schedules.iter().filter_map(|j| j.1).count()
        );

        for (guild_id, schedule) in schedules {
            if let Some(hours) = schedule {
                let s = self.clone();
                let h = tokio::spawn(async move {
                    work_loop(s, guild_id, hours)
                        .await
                        // TODO: Handle a failed cron job
                        // Maybe restart it?
                        .expect("Cron job failed");
                });
                if let Some(old) = self.jobs.insert(guild_id, h) {
                    old.abort();
                }
                // Spread out the registrations so we don't hit any rate limits
                sleep(Duration::from_secs(((60 * 60) / count) as u64)).await;
            }
        }

        info!("Started all cron jobs");
        Ok(())
    }

    pub async fn start_schedule(&self, guild_id: u64) -> Result<(), Error> {
        let hours = db::servers::get_schedule(&self.pool, guild_id)?
            .ok_or("No schedule set for this server.")?;

        let s = self.clone();
        let h = tokio::spawn(async move {
            work_loop(s, guild_id, hours)
                .await
                .expect("Cron job failed");
        });
        if let Some(old) = self.jobs.insert(guild_id, h) {
            // If there was already a job running, abort it
            old.abort();
        }

        Ok(())
    }

    pub fn is_running(&self, guild_id: u64) -> bool {
        self.jobs.contains_key(&guild_id)
    }
}

async fn work_loop(s: Scheduler, guild_id: u64, hours: u64) -> Result<(), Error> {
    loop {
        if !db::servers::check_still_in_guild(&s.pool, guild_id)? {
            warn!(
                "Guild {} is no longer in the guild list, stopping cron job",
                guild_id
            );
            break;
        }
        match notify_on_updates(s.clone(), guild_id).await {
            Ok(_) => {}
            Err(e) => {
                tracing::error!(
                    "Error while notifying on updates: {}, for server: {}",
                    e,
                    guild_id
                );
                break;
            }
        }
        db::servers::update_last_update_timestamp(&s.pool, guild_id)?;
        sleep(Duration::from_secs(hours * 60 * 60)).await;
    }

    Ok(())
}

async fn notify_on_updates(scheduler: Scheduler, guild_id: u64) -> Result<(), Error> {
    let client = scheduler.client.read().await;
    if let Some(client) = &*client {
        let subscriptions =
            db::subscriptions::get_all_subscriptions_of_guild(&scheduler.pool, guild_id)?;

        let mut updated = Vec::new();
        let mut unknown = Vec::new();
        let mut failed = Vec::new();

        for (last_notify, mod_info) in subscriptions {
            let mod_info = steam::get_mod(&scheduler.pool, mod_info.id).await?;

            if mod_info.last_updated > last_notify {
                updated.push((mod_info, last_notify));
            } else {
                unknown.push((last_notify, mod_info));
            }
        }

        for (last_notify, mod_info) in unknown {
            if let Ok(info) = steam::get_latest_mod(&scheduler.pool, mod_info.id).await {
                if info.last_updated > last_notify {
                    updated.push((info, last_notify));
                }
            } else {
                failed.push((last_notify, mod_info));
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

            for (mod_info, _) in updated {
                db::subscriptions::update_last_notify(&scheduler.pool, guild_id, mod_info.id)?;
            }
        }

        if !failed.is_empty() {
            c.send_message(&client, |d| {
                d.content("The following Items could not be updated:".to_string());

                for (_, mod_info) in failed.iter() {
                    d.add_embed(|e| {
                        e.title(format!("{}, Id: {}", mod_info.name.clone(), mod_info.id));
                        e.url(format!(
                            "https://steamcommunity.com/sharedfiles/filedetails/?id={}",
                            mod_info.id
                        ));
                        e
                    });
                }

                d
            })
            .await?;
        }
    } else {
        warn!("Client not set for scheduler");
    }

    Ok(())
}

async fn send_in_chunks_updates(
    c: &poise::serenity_prelude::GuildChannel,
    client: &CacheAndHttp,
    updated: &[(db::ModInfo, u64)],
) -> Result<(), Error> {
    let chunks: Vec<Vec<db::ModInfo>> = updated
        .iter()
        .chunks(5)
        .into_iter()
        .map(|c| c.map(|(m, _)| m.clone()).collect())
        .collect();

    let parts = chunks.len();

    for (curr, chunk) in chunks.iter().enumerate() {
        c.send_message(&client, |d| {
            d.content(format!(
                "The following Items have been updated: Part {}/{}",
                curr + 1,
                parts
            ));

            for mod_info in chunk.iter() {
                d.add_embed(|e| {
                    e.title(mod_info.name.clone());
                    e.url(format!(
                        "https://steamcommunity.com/sharedfiles/filedetails/?id={}",
                        mod_info.id
                    ));
                    if let Some(url) = mod_info.preview_url.clone() {
                        e.image(url);
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
    updated: &[(db::ModInfo, u64)],
) -> Result<(), Error> {
    c.send_message(&client, |d| {
        d.content("The following Items have been updated:".to_string());

        for (mod_info, _) in updated.iter() {
            d.add_embed(|e| {
                e.title(mod_info.name.clone());
                e.url(format!(
                    "https://steamcommunity.com/sharedfiles/filedetails/?id={}",
                    mod_info.id
                ));
                if let Some(url) = mod_info.preview_url.clone() {
                    e.image(url);
                }

                e
            });
        }

        d
    })
    .await?;
    Ok(())
}
