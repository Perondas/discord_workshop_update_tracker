use std::collections::HashMap;

use lazy_static::lazy_static;
use mysql::Pool;
use tokio::sync::Semaphore;
use tracing::debug;

lazy_static! {
    // Used for rate limiting on the steam API
    static ref SEMAPHORE: Semaphore = Semaphore::new(3);
}

use crate::{
    db::{self, ModInfo},
    Error,
};

pub async fn get_mod(pool: &Pool, mod_id: u64) -> Result<ModInfo, Error> {
    if let Ok(Some(mod_info)) = db::mods::get_mod(pool, mod_id) {
        debug!("Found mod in db: {:?}", mod_info);
        return Ok(mod_info);
    }
    let mod_info = get_mod_from_steam(mod_id).await?;

    db::mods::add_mod(pool, mod_info.clone())?;

    Ok(mod_info)
}

pub async fn get_latest_mod(pool: &Pool, mod_id: u64) -> Result<ModInfo, Error> {
    let mod_info = get_mod_from_steam(mod_id).await?;

    db::mods::update_mod(pool, mod_info.clone())?;

    Ok(mod_info)
}

async fn get_mod_from_steam(mod_id: u64) -> Result<ModInfo, Error> {
    let permit = SEMAPHORE.acquire().await?;
    let c = reqwest::Client::new();

    let url = "https://api.steampowered.com/ISteamRemoteStorage/GetPublishedFileDetails/v1/";

    let mut params = HashMap::new();
    let id = mod_id.to_string();
    params.insert("itemcount", "1");
    params.insert("publishedfileids[0]", &id);

    let res = c.post(url).form(&params).send().await?;

    std::mem::drop(permit);

    let parse = json::parse(std::str::from_utf8(&res.bytes().await?)?)?;

    let name = parse["response"]["publishedfiledetails"][0]["title"].to_string();
    let last_updated = if parse["response"]["publishedfiledetails"][0]["time_updated"].is_number() {
        parse["response"]["publishedfiledetails"][0]["time_updated"]
            .to_string()
            .parse()?
    } else {
        parse["response"]["publishedfiledetails"][0]["time_created"]
            .to_string()
            .parse()?
    };
    let preview_url = if parse["response"]["publishedfiledetails"][0]["preview_url"].is_string() {
        Some(parse["response"]["publishedfiledetails"][0]["preview_url"].to_string())
    } else {
        None
    };

    Ok(ModInfo {
        id: mod_id,
        name,
        last_updated,
        preview_url,
    })
}
