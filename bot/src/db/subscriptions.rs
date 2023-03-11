use mysql::{params, prelude::Queryable, Pool};

use crate::Error;

use super::ModInfo;

pub fn get_all_subscriptions_of_guild(
    pool: &Pool,
    guild_id: u64,
) -> Result<Vec<(u64, ModInfo)>, Error> {
    let mut conn = pool.get_conn()?;

    let res: Vec<(u64, u64, String, u64, Option<String>)> = conn.query(format!(
        "SELECT Subscriptions.LastUpdate, Mods.ModId,  Mods.ModName, Mods.LastUpdate, Mods.PreviewUrl FROM Subscriptions INNER JOIN Mods ON Subscriptions.ModId = Mods.ModId WHERE Subscriptions.ServerId = {}",
        guild_id
    ))?;

    res.iter()
        .map(|(last_notified, id, name, last_updated, preview_url)| {
            Ok((
                *last_notified,
                ModInfo {
                    id: *id,
                    name: name.clone(),
                    last_updated: *last_updated,
                    preview_url: preview_url.clone(),
                },
            ))
        })
        .collect()
}

pub fn add_subscription(pool: &Pool, guild_id: u64, mod_id: u64) -> Result<(), Error> {
    let mut conn = pool.get_conn()?;

    conn.exec_drop(
        r"INSERT INTO Subscriptions (ServerId, ModId, LastUpdate) VALUES (:guild_id, :mod_id, UNIX_TIMESTAMP());",
        params! {
            "guild_id" => guild_id,
            "mod_id" => mod_id,
        },
    )?;
    Ok(())
}

pub fn remove_subscription(pool: &Pool, guild_id: u64, mod_id: u64) -> Result<(), Error> {
    let mut conn = pool.get_conn()?;

    conn.exec_drop(
        r"DELETE FROM Subscriptions WHERE ServerId = :guild_id AND ModId = :mod_id;",
        params! {
            "guild_id" => guild_id,
            "mod_id" => mod_id,
        },
    )?;
    Ok(())
}

pub fn update_last_notify(pool: &Pool, guild_id: u64, mod_id: u64) -> Result<(), Error> {
    let mut conn = pool.get_conn()?;

    conn.exec_drop(
        r"UPDATE Subscriptions SET LastUpdate = UNIX_TIMESTAMP() WHERE ServerId = :guild_id AND ModId = :mod_id;",
        params! {
            "guild_id" => guild_id,
            "mod_id" => mod_id,
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
