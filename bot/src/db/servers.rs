use std::sync::Arc;

use mysql::{params, prelude::Queryable, Pool};
use poise::serenity_prelude::Guild;

use crate::Error;

pub fn add_server(pool: Arc<Pool>, guild: &Guild) {
    let mut conn = pool.get_conn().unwrap();

    conn.exec_drop(
        r"INSERT INTO Servers (ServerId) VALUES (:id);",
        params! {
            "id" => guild.id.0,
        },
    )
    .unwrap();
}

pub fn remove_server(pool: Arc<Pool>, guild_id: u64) {
    let mut conn = pool.get_conn().unwrap();

    conn.exec_drop(
        r"DELETE FROM Servers WHERE ServerId = :id;",
        params! {
            "id" => guild_id,
        },
    )
    .unwrap();
}

pub fn set_update_channel(clone: Arc<Pool>, guild_id: u64, channel_id: u64) -> Result<(), Error> {
    let mut conn = clone.get_conn().unwrap();

    conn.exec_drop(
        r"UPDATE Servers SET ChannelId = :channel_id WHERE ServerId = :id;",
        params! {
            "channel_id" => channel_id,
            "id" => guild_id,
        },
    )?;
    Ok(())
}

pub fn get_update_channel(clone: Arc<Pool>, guild_id: u64) -> Result<Option<u64>, Error> {
    let mut conn = clone.get_conn().unwrap();

    let res: Option<Option<u64>> = conn.query_first(format!(
        "SELECT ChannelId FROM Servers WHERE ServerId = {};",
        guild_id
    ))?;

    Ok(res.flatten())
}

pub fn set_schedule(clone: Arc<Pool>, guild_id: u64, interval: u64) -> Result<(), Error> {
    let mut conn = clone.get_conn().unwrap();

    conn.exec_drop(
        r"UPDATE Servers SET Schedule = :interval WHERE ServerId = :id;",
        params! {
            "interval" => interval,
            "id" => guild_id,
        },
    )?;
    Ok(())
}

pub fn get_schedule(clone: Arc<Pool>, guild_id: u64) -> Result<Option<u64>, Error> {
    let mut conn = clone.get_conn().unwrap();

    let res: Option<Option<u64>> = conn.query_first(format!(
        "SELECT Schedule FROM Servers WHERE ServerId = {}",
        guild_id
    ))?;

    Ok(res.flatten())
}

pub fn get_all_schedules(clone: Arc<Pool>) -> Result<Vec<(u64, Option<u64>)>, Error> {
    let mut conn = clone.get_conn().unwrap();

    let res: Vec<(u64, Option<u64>)> = conn.query("SELECT ServerId, Schedule FROM Servers;")?;

    Ok(res)
}

pub(crate) fn check_still_in_guild(pool: Arc<Pool>, guild_id: u64) -> Result<bool, Error> {
    let mut conn = pool.get_conn().unwrap();

    let res: Option<Option<u64>> = conn.query_first(format!(
        "SELECT ServerId FROM Servers WHERE ServerId = {};",
        guild_id
    ))?;

    Ok(res.flatten().is_some())
}
