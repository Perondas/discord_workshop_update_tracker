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
            "preview_url" => info.preview_url.map(|s| sanitize_string(s)),
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
            "preview_url" => info.preview_url.map(|s| sanitize_string(s)),
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
