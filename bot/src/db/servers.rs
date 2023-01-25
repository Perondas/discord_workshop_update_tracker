use mysql::{params, prelude::Queryable, Pool};
use poise::serenity_prelude::Guild;

use crate::Error;

pub fn add_server(pool: &Pool, guild: &Guild) -> Result<(), Error> {
    let mut conn = pool.get_conn()?;

    match conn.exec_drop(
        r"INSERT INTO Servers (ServerId) VALUES (:id);",
        params! {
            "id" => guild.id.0,
        },
    ) {
        Ok(_) => Ok(()),
        Err(e) => {
            if e.to_string().contains("Duplicate entry") {
                Ok(())
            } else {
                Err(e.into())
            }
        }
    }
}

pub fn remove_server(pool: &Pool, guild_id: u64) -> Result<(), Error> {
    let mut conn = pool.get_conn()?;

    conn.exec_drop(
        r"DELETE FROM Servers WHERE ServerId = :id;",
        params! {
            "id" => guild_id,
        },
    )?;
    Ok(())
}

pub fn set_update_channel(pool: &Pool, guild_id: u64, channel_id: u64) -> Result<(), Error> {
    let mut conn = pool.get_conn()?;

    conn.exec_drop(
        r"UPDATE Servers SET ChannelId = :channel_id WHERE ServerId = :id;",
        params! {
            "channel_id" => channel_id,
            "id" => guild_id,
        },
    )?;
    Ok(())
}

pub fn get_update_channel(pool: &Pool, guild_id: u64) -> Result<Option<u64>, Error> {
    let mut conn = pool.get_conn()?;

    let res: Option<Option<u64>> = conn.query_first(format!(
        "SELECT ChannelId FROM Servers WHERE ServerId = {};",
        guild_id
    ))?;

    Ok(res.flatten())
}

pub fn set_schedule(pool: &Pool, guild_id: u64, interval: u64) -> Result<(), Error> {
    let mut conn = pool.get_conn()?;

    conn.exec_drop(
        r"UPDATE Servers SET Schedule = :interval WHERE ServerId = :id;",
        params! {
            "interval" => interval,
            "id" => guild_id,
        },
    )?;
    Ok(())
}

pub fn get_schedule(pool: &Pool, guild_id: u64) -> Result<Option<u64>, Error> {
    let mut conn = pool.get_conn()?;

    let res: Option<Option<u64>> = conn.query_first(format!(
        "SELECT Schedule FROM Servers WHERE ServerId = {}",
        guild_id
    ))?;

    Ok(res.flatten())
}

pub fn get_all_schedules(pool: &Pool) -> Result<Vec<(u64, Option<u64>)>, Error> {
    let mut conn = pool.get_conn()?;

    let res: Vec<(u64, Option<u64>)> = conn.query("SELECT ServerId, Schedule FROM Servers;")?;

    Ok(res)
}

pub fn check_still_in_guild(pool: &Pool, guild_id: u64) -> Result<bool, Error> {
    let mut conn = pool.get_conn()?;

    let res: Option<Option<u64>> = conn.query_first(format!(
        "SELECT ServerId FROM Servers WHERE ServerId = {};",
        guild_id
    ))?;

    Ok(res.flatten().is_some())
}

pub fn update_last_update_timestamp(pool: &Pool, guild_id: u64) -> Result<(), Error> {
    let mut conn = pool.get_conn()?;

    conn.exec_drop(
        r"UPDATE Servers SET LastUpdate =  UNIX_TIMESTAMP() WHERE ServerId = :id;",
        params! {
            "id" => guild_id,
        },
    )?;

    Ok(())
}

pub fn get_last_update(pool: &Pool, guild_id: u64) -> Result<Option<u64>, Error> {
    let mut conn = pool.get_conn()?;

    let res: Option<Option<u64>> = conn.query_first(format!(
        "SELECT LastUpdate FROM Servers WHERE ServerId = {};",
        guild_id
    ))?;

    Ok(res.flatten())
}
