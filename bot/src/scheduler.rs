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
    pub client: Arc<RwLock<Option<Arc<CacheAndHttp>>>>,
    pub jobs: Arc<DashMap<u64, JoinHandle<()>>>,
    pub pool: Arc<Pool>,
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
        interval.tick().await;

        if !db::servers::check_still_in_guild(&s.pool, guild_id)? {
            warn!(
                "Guild {} is no longer in the guild list, stopping tracking job",
                guild_id
            );
            break;
        }
        match crate::printer::notify_on_updates(s.clone(), guild_id).await {
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
    }

    Ok(())
}
