use std::{sync::Arc, time::Duration};

use dashmap::DashMap;
use itertools::Itertools;
use mysql::Pool;
use poise::serenity_prelude::{CacheAndHttp, GuildId};
use tokio::{sync::RwLock, task::JoinHandle, time::sleep};
use tracing::{debug, error, info, warn};

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
        info!("Removing tracking job for guild: {}", guild_id);
        if let Some((_, job)) = self.jobs.remove(&guild_id) {
            job.abort();
        }
    }

    pub async fn start_cron(&self, client: Arc<CacheAndHttp>) -> Result<(), Error> {
        *self.client.write().await = Some(client.clone());

        let schedules = db::servers::get_all_schedules(&self.pool)?;
        let count = schedules.len();

        info!(
            "Starting {} tracking jobs",
            schedules.iter().filter_map(|j| j.1).count()
        );

        for (guild_id, schedule) in schedules {
            if let Some(hours) = schedule {
                let s = self.clone();
                s.start_job(guild_id, hours);

                // Spread out the registrations so we don't hit any rate limits
                sleep(Duration::from_secs(((60 * 30) / count) as u64)).await;
            }
        }

        info!("Started all tracking jobs");
        Ok(())
    }

    pub async fn start_schedule(&self, guild_id: u64) -> Result<(), Error> {
        let hours = db::servers::get_schedule(&self.pool, guild_id)?
            .ok_or("No schedule set for this server.")?;

        let s = self.clone();
        s.start_job(guild_id, hours);

        Ok(())
    }

    fn start_job(self, guild_id: u64, hours: u64) {
        let s = self.clone();
        let h = tokio::spawn(async move {
            match work_loop(s.clone(), guild_id, hours).await {
                Ok(_) => {}
                Err(e) => {
                    error!(
                        "Tracking job for guild: {} failed with error: {}",
                        guild_id, e
                    );
                }
            }

            // No matter what happens, we remove the job from the list
            s.remove(guild_id);
        });

        if let Some(old) = self.jobs.insert(guild_id, h) {
            // Abort the old job if it exists
            old.abort();
        }
    }

    pub fn is_running(&self, guild_id: u64) -> bool {
        debug!(guild_id, "Checking if tracking job is running");
        self.jobs.contains_key(&guild_id)
    }
}

async fn work_loop(s: Scheduler, guild_id: u64, hours: u64) -> Result<(), Error> {
    info!("Starting tracking job for guild: {}", guild_id);

    let mut interval = tokio::time::interval(Duration::from_secs(60 * 60 * hours));

    // We use delay as we don't care about precision, we just want to tick every so often
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        if !db::servers::check_still_in_guild(&s.pool, guild_id)? {
            warn!(
                "Guild {} is no longer in the guild list, stopping tracking job",
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

        interval.tick().await;
    }

    Ok(())
}

async fn notify_on_updates(scheduler: Scheduler, guild_id: u64) -> Result<(), Error> {
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

    for (last_notify, item_info) in subscriptions {
        let item_info = steam::get_item(&scheduler.pool, item_info.id).await?;

        if item_info.last_updated > last_notify {
            updated.push((item_info, last_notify));
        } else {
            unknown.push((last_notify, item_info));
        }
    }

    for (last_notify, item_info) in unknown {
        if let Ok(info) = steam::get_latest_item(&scheduler.pool, item_info.id).await {
            if info.last_updated > last_notify {
                updated.push((info, last_notify));
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

        for (item_info, _) in updated {
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
    updated: &[(db::ItemInfo, u64)],
) -> Result<(), Error> {
    let chunks: Vec<Vec<db::ItemInfo>> = updated
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

            for item_info in chunk.iter() {
                d.add_embed(|e| {
                    e.title(item_info.name.clone());
                    e.url(format!(
                        "https://steamcommunity.com/sharedfiles/filedetails/?id={}",
                        item_info.id
                    ));
                    if let Some(url) = item_info.preview_url.clone() {
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
    updated: &[(db::ItemInfo, u64)],
) -> Result<(), Error> {
    c.send_message(&client, |d| {
        d.content("The following Items have been updated:".to_string());

        for (item_info, _) in updated.iter() {
            d.add_embed(|e| {
                e.title(item_info.name.clone());
                e.url(format!(
                    "https://steamcommunity.com/sharedfiles/filedetails/?id={}",
                    item_info.id
                ));
                if let Some(url) = item_info.preview_url.clone() {
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
