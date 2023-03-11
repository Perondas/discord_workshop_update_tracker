use mysql::{Opts, Pool};

use crate::Error;

pub mod items;
pub mod servers;
pub mod subscriptions;

#[derive(Debug, Clone)]
pub struct ItemInfo {
    pub id: u64,
    pub name: String,
    pub last_updated: u64,
    pub preview_url: Option<String>,
}

pub fn get_pool(url: &str) -> Result<Pool, Error> {
    let pool = Pool::new(Opts::from_url(url)?)?;

    Ok(pool)
}
