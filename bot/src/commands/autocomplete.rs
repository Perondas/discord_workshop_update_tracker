use std::{sync::Arc, time};

use dashmap::DashMap;
use lazy_static::lazy_static;
use mysql::Pool;
use regex::Regex;
use tracing::{debug, error};

use crate::{Context, Error};

lazy_static! {
    static ref NAME_CACHE: NameCache = NameCache::new();
}

pub async fn autocomplete_name(
    ctx: Context<'_>,
    partial: &str,
) -> impl Iterator<Item = poise::AutocompleteChoice<String>> {
    let res = vec![];

    let to_choice = |(s, id): (String, u64)| poise::AutocompleteChoice {
        name: s,
        value: id.to_string(),
    };

    if partial.len() < 3 {
        return res.into_iter().map(to_choice);
    }

    let guild = match ctx.guild() {
        Some(g) => g,
        None => return res.into_iter().map(to_choice),
    };

    let names = match NAME_CACHE.get(guild.id.0, &ctx.data().pool, partial) {
        Ok(v) => v,
        Err(e) => {
            error!("Error getting names: {:?}", e);
            return res.into_iter().map(to_choice);
        }
    };

    let re = Regex::new(&format!(".*(?i){}.*", partial)).unwrap();

    let res: Vec<(String, u64)> = names.into_iter().filter(|(s, _)| re.is_match(s)).collect();

    res.into_iter().map(to_choice)
}

#[derive(Clone)]
struct NameCache {
    #[allow(clippy::type_complexity)]
    names: Arc<DashMap<(u64, String), (Vec<(String, u64)>, time::Instant)>>,
}

impl NameCache {
    fn new() -> Self {
        let s = Self {
            names: Arc::new(DashMap::new()),
        };

        let clone = s.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(time::Duration::from_secs(30));

            debug!("Name cache cleanup task started.");

            loop {
                interval.tick().await;

                clone
                    .names
                    .retain(|_, v| v.1.elapsed() < time::Duration::from_secs(60));

                clone.names.shrink_to_fit();
            }
        });
        s
    }

    fn get(&self, guild_id: u64, pool: &Pool, query: &str) -> Result<Vec<(String, u64)>, Error> {
        let first_three = query.chars().take(3).collect::<String>();
        match self.names.get(&(guild_id, first_three.clone())) {
            Some(v) => {
                let (names, _) = v.value();
                Ok(names.clone())
            }
            None => {
                let names = crate::db::items::get_subscribed_item_names(
                    pool,
                    guild_id,
                    Some(query.to_string()),
                )?;
                self.names.insert(
                    (guild_id, first_three),
                    (names.clone(), time::Instant::now()),
                );
                Ok(names)
            }
        }
    }
}
