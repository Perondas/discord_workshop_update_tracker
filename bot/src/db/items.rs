use mysql::{params, prelude::Queryable, Pool};
use sql_lexer::sanitize_string;
use tracing::error;

use crate::Error;

use super::ItemInfo;

pub fn get_item(pool: &Pool, item_id: u64) -> Result<Option<ItemInfo>, Error> {
    let mut conn = pool.get_conn()?;

    let res: Option<(u64, String, u64, Option<String>)> =
        conn.query_first(format!("SELECT * FROM Items WHERE ItemId = {};", item_id))?;

    match res {
        Some((id, name, last_updated, preview_url)) => Ok(Some(ItemInfo {
            id,
            name,
            last_updated,
            preview_url,
        })),
        None => Ok(None),
    }
}

pub fn add_item(pool: &Pool, info: ItemInfo) -> Result<(), Error> {
    let mut conn = pool.get_conn()?;

    let res = conn.exec_drop(
        r"INSERT INTO Items (ItemId, ItemName, LastUpdate, PreviewUrl) VALUES (:id, :name, :last_update, :preview_url);",
        params! {
            "id" => info.id,
            "name" => sanitize_string(info.name),
            "last_update" => info.last_updated,
            // We don't sanitize the preview url because it is already sanitized by steam
            "preview_url" => info.preview_url,
        },
    );

    match res {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Error adding item: {:?}", e);
            Err(e.into())
        }
    }
}

pub fn update_item(pool: &Pool, info: ItemInfo) -> Result<(), Error> {
    let mut conn = pool.get_conn()?;

    let res = conn.exec_drop(
        r"UPDATE Items SET ItemName = :name, LastUpdate = :last_update, PreviewUrl = :preview_url WHERE ItemId = :id;",
        params! {
            "id" => info.id,
            "name" => sanitize_string(info.name),
            "last_update" => info.last_updated,
            // We don't sanitize the preview url because it is already sanitized by steam
            "preview_url" => info.preview_url,
        },
    );

    match res {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Error updating item: {:?}", e);
            Err(e.into())
        }
    }
}

pub fn get_item_by_name(pool: &Pool, name: &str) -> Result<ItemInfo, Error> {
    let mut conn = pool.get_conn()?;

    let name = sanitize_string(name.to_string());

    let res: Option<(u64, String, u64, Option<String>)> = conn.query_first(format!(
        "SELECT * FROM Items WHERE ItemName LIKE '%{}%';",
        name
    ))?;

    match res {
        Some((id, name, last_updated, preview_url)) => Ok(ItemInfo {
            id,
            name,
            last_updated,
            preview_url,
        }),
        None => Err("Item not found".into()),
    }
}

pub fn get_subscribed_item_names(
    pool: &Pool,
    guild_id: u64,
    query: Option<String>,
) -> Result<Vec<(String, u64)>, Error> {
    let mut conn = pool.get_conn()?;

    let query = query.map(sanitize_string);

    let res: Vec<(String, u64)> = match query {
        Some(query) => conn.query(format!(
            "SELECT ItemName, ItemId FROM Items WHERE ItemId IN (SELECT ItemId FROM Subscriptions WHERE ServerId = {}) AND ItemName LIKE '%{}%';",
            guild_id, query
        ))?,
        None => conn.query(format!(
            "SELECT ItemName, ItemId FROM Items WHERE ItemId IN (SELECT ItemId FROM Subscriptions WHERE ServerId = {});",
            guild_id
        ))?,
    };

    Ok(res)
}
