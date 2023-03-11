use mysql::{params, prelude::Queryable, Pool};
use tracing::error;

use crate::Error;

use super::ItemInfo;

pub fn get_item(pool: &Pool, item_id: u64) -> Result<Option<ItemInfo>, Error> {
    let mut conn = pool.get_conn()?;

    let res: Option<(u64, String, u64, Option<String>)> =
        conn.query_first(format!("SELECT * FROM items WHERE itemId = {};", item_id))?;

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
        r"INSERT INTO items (itemId, itemName, LastUpdate, PreviewUrl) VALUES (:id, :name, :last_update, :preview_url);",
        params! {
            "id" => info.id,
            "name" => info.name,
            "last_update" => info.last_updated,
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
        r"UPDATE items SET itemName = :name, LastUpdate = :last_update, PreviewUrl = :preview_url WHERE itemId = :id;",
        params! {
            "id" => info.id,
            "name" => info.name,
            "last_update" => info.last_updated,
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
