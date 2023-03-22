use mysql::{params, prelude::Queryable, Pool};
use sql_lexer::sanitize_string;

use crate::Error;

use super::ItemInfo;

#[allow(clippy::type_complexity)]
pub fn get_all_subscriptions_of_guild(
    pool: &Pool,
    guild_id: u64,
) -> Result<Vec<(u64, ItemInfo, Option<String>)>, Error> {
    let mut conn = pool.get_conn()?;

    let res: Vec<(u64, u64, String, u64, Option<String>, Option<String>)> = conn.query(format!(
        "SELECT Subscriptions.LastUpdate, Items.ItemId,  Items.ItemName, Items.LastUpdate, Items.PreviewUrl, Subscriptions.Note FROM Subscriptions INNER JOIN Items ON Subscriptions.ItemId = Items.ItemId WHERE Subscriptions.ServerId = {}",
        guild_id
    ))?;

    res.iter()
        .map(
            |(last_notified, id, name, last_updated, preview_url, note)| {
                Ok((
                    *last_notified,
                    ItemInfo {
                        id: *id,
                        name: name.clone(),
                        last_updated: *last_updated,
                        preview_url: preview_url.clone(),
                    },
                    note.clone(),
                ))
            },
        )
        .collect()
}

pub fn add_subscription(pool: &Pool, guild_id: u64, item_id: u64) -> Result<(), Error> {
    let mut conn = pool.get_conn()?;

    conn.exec_drop(
        r"INSERT INTO Subscriptions (ServerId, ItemId, LastUpdate) VALUES (:guild_id, :item_id, UNIX_TIMESTAMP());",
        params! {
            "guild_id" => guild_id,
            "item_id" => item_id,
        },
    )?;
    Ok(())
}

pub fn remove_subscription(pool: &Pool, guild_id: u64, item_id: u64) -> Result<(), Error> {
    let mut conn = pool.get_conn()?;

    conn.exec_drop(
        r"DELETE FROM Subscriptions WHERE ServerId = :guild_id AND ItemId = :item_id;",
        params! {
            "guild_id" => guild_id,
            "item_id" => item_id,
        },
    )?;
    Ok(())
}

pub fn update_last_notify(pool: &Pool, guild_id: u64, item_id: u64) -> Result<(), Error> {
    let mut conn = pool.get_conn()?;

    conn.exec_drop(
        r"UPDATE Subscriptions SET LastUpdate = UNIX_TIMESTAMP() WHERE ServerId = :guild_id AND ItemId = :item_id;",
        params! {
            "guild_id" => guild_id,
            "item_id" => item_id,
        },
    )?;
    Ok(())
}

pub fn count_guild_subscriptions(pool: &Pool, guild_id: u64) -> Result<u64, Error> {
    let mut conn = pool.get_conn()?;

    let res: Vec<(u64,)> = conn.query(format!(
        "SELECT COUNT(*) FROM Subscriptions WHERE ServerId = {}",
        guild_id
    ))?;

    Ok(res[0].0)
}

pub fn remove_all_subscriptions(pool: &Pool, guild_id: u64) -> Result<(), Error> {
    let mut conn = pool.get_conn()?;

    conn.exec_drop(
        r"DELETE FROM Subscriptions WHERE ServerId = :guild_id;",
        params! {
            "guild_id" => guild_id,
        },
    )?;
    Ok(())
}

pub fn get_note(pool: &Pool, guild_id: u64, item_id: u64) -> Result<Option<String>, Error> {
    let mut conn = pool.get_conn()?;

    let res: Option<Option<String>> = conn.query_first(format!(
        "SELECT Note FROM Subscriptions WHERE ServerId = {} AND ItemId = {}",
        guild_id, item_id
    ))?;

    match res {
        Some(note) => Ok(note),
        None => Err("Not subscribed".into()),
    }
}

pub fn update_subscription_note(
    pool: &Pool,
    guild_id: u64,
    item_id: u64,
    note: Option<String>,
) -> Result<(), Error> {
    let mut conn = pool.get_conn()?;

    if note.is_none() || note.as_ref().unwrap().is_empty() {
        conn.exec_drop(
            r"UPDATE Subscriptions SET Note = NULL WHERE ServerId = :guild_id AND ItemId = :item_id;",
            params! {
                "guild_id" => guild_id,
                "item_id" => item_id,
            },
        )?;
        return Ok(());
    }

    conn.exec_drop(
        r"UPDATE Subscriptions SET Note = :note WHERE ServerId = :guild_id AND ItemId = :item_id;",
        params! {
            "note" => note.map(sanitize_string),
            "guild_id" => guild_id,
            "item_id" => item_id,
        },
    )?;
    Ok(())
}

pub async fn get_changes_since(
    pool: &Pool,
    guild_id: u64,
    since: u64,
) -> Result<Vec<(ItemInfo, Option<String>)>, Error> {
    let mut conn = pool.get_conn()?;

    let res: Vec<(u64, String, u64, Option<String>, Option<String>)> = conn.query(format!(
        "SELECT Items.ItemId, Items.ItemName, Items.LastUpdate, Items.PreviewUrl, Subscriptions.Note FROM Subscriptions INNER JOIN Items ON Subscriptions.ItemId = Items.ItemId WHERE Subscriptions.ServerId = {} AND Subscriptions.LastUpdate > {}",
        guild_id,
        since
    ))?;

    Ok(res
        .iter()
        .map(|(id, name, last_updated, preview_url, note)| {
            (
                ItemInfo {
                    id: *id,
                    name: name.clone(),
                    last_updated: *last_updated,
                    preview_url: preview_url.clone(),
                },
                note.clone(),
            )
        })
        .collect())
}
