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
    db::{self, ItemInfo},
    Error,
};

pub async fn get_item(pool: &Pool, item_id: u64) -> Result<ItemInfo, Error> {
    if let Ok(Some(item_info)) = db::items::get_item(pool, item_id) {
        debug!("Found item in db: {:?}", item_info);
        return Ok(item_info);
    }
    let item_info = get_item_from_steam(item_id).await?;

    db::items::add_item(pool, item_info.clone())?;

    Ok(item_info)
}

pub async fn get_collection_ids(_pool: &Pool, collection_id: u64) -> Result<Vec<u64>, Error> {
    get_collection_members_from_steam(collection_id).await
}

pub async fn get_latest_item(pool: &Pool, item_id: u64) -> Result<ItemInfo, Error> {
    let item_info = get_item_from_steam(item_id).await?;

    db::items::update_item(pool, item_info.clone())?;

    Ok(item_info)
}

async fn get_item_from_steam(item_id: u64) -> Result<ItemInfo, Error> {
    let permit = SEMAPHORE.acquire().await?;
    let c = reqwest::Client::new();

    let url = "https://api.steampowered.com/ISteamRemoteStorage/GetPublishedFileDetails/v1/";

    let mut params = HashMap::new();
    let id = item_id.to_string();
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

    Ok(ItemInfo {
        id: item_id,
        name,
        last_updated,
        preview_url,
    })
}

async fn get_collection_members_from_steam(collection_id: u64) -> Result<Vec<u64>, Error> {
    let permit = SEMAPHORE.acquire().await?;
    let c = reqwest::Client::new();

    let url = "https://api.steampowered.com/ISteamRemoteStorage/GetCollectionDetails/v1/";

    let mut params = HashMap::new();
    let id = collection_id.to_string();
    params.insert("collectioncount", "1");
    params.insert("publishedfileids[0]", &id);

    let res = c.post(url).form(&params).send().await?;

    std::mem::drop(permit);

    let parse = json::parse(std::str::from_utf8(&res.bytes().await?)?)?;

    let mut members = Vec::new();

    for member in parse["response"]["collectiondetails"][0]["children"].members() {
        members.push(
            member["publishedfileid"]
                .as_str()
                .ok_or("No member id")?
                .parse()?,
        );
    }

    Ok(members)
}
