use std::sync::Arc;

use mysql::{params, prelude::Queryable, Pool};
use tracing::error;

use crate::Error;

use super::ModInfo;

pub fn get_mod(clone: Arc<Pool>, mod_id: u64) -> Result<Option<ModInfo>, Error> {
    let mut conn = clone.get_conn().unwrap();

    let res: Option<(u64, String, u64, Option<String>)> =
        conn.query_first(format!("SELECT * FROM Mods WHERE ModId = {};", mod_id))?;

    match res {
        Some((id, name, last_updated, preview_url)) => Ok(Some(ModInfo {
            id,
            name,
            last_updated,
            preview_url,
        })),
        None => Ok(None),
    }
}

pub fn add_mod(pool: Arc<Pool>, info: ModInfo) -> Result<(), Error> {
    let mut conn = pool.get_conn().unwrap();

    let res = conn.exec_drop(
        r"INSERT INTO Mods (ModId, ModName, LastUpdate, PreviewUrl) VALUES (:id, :name, :last_update, :preview_url);",
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
            error!("Error adding mod: {:?}", e);
            Err(e.into())
        }
    }
}

pub fn update_mod(pool: Arc<Pool>, info: ModInfo) -> Result<(), Error> {
    let mut conn = pool.get_conn().unwrap();

    let res = conn.exec_drop(
        r"UPDATE Mods SET ModName = :name, LastUpdate = :last_update, PreviewUrl = :preview_url WHERE ModId = :id;",
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
            error!("Error updating mod: {:?}", e);
            Err(e.into())
        }
    }
}
