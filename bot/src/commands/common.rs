macro_rules! get_guild {
    ($ctx:expr) => {
        match $ctx.guild() {
            Some(g) => g,
            None => {
                $ctx.say("This command can only be used in a guild.")
                    .await?;
                return Ok(());
            }
        }
    };
}

macro_rules! get_channel {
    ($ctx:expr, $id:expr) => {
        match crate::db::servers::get_update_channel(&$ctx.data().pool, $id) {
            Ok(c) => match c {
                Some(c) => c,
                None => {
                    $ctx.say("Please set an update channel first.").await?;
                    return Ok(());
                }
            },
            Err(_) => {
                $ctx.say("An error occurred while fetching the update channel.")
                    .await?;
                return Ok(());
            }
        }
    };
}

macro_rules! get_guild_channel {
    ($ctx:expr, $guild:expr, $item_channel:expr) => {
        match crate::commands::actions::get_guild_channel(&$guild, $item_channel) {
            Some(g) => g,
            None => {
                $ctx.say("The update channel is no longer available")
                    .await?;
                return Ok(());
            }
        }
    };
}

macro_rules! ok_or_respond {
    ($ctx:expr, $func:expr, $msg:expr) => {
        match $func {
            Ok(val) => val,
            Err(e) => {
                tracing::error!("Error while executing command: {e}");
                $ctx.say($msg).await?;
                return Ok(());
            }
        }
    };
}

macro_rules! get_by_name {
    ($ctx:expr, $item:expr) => {
        match $item.parse() {
            Ok(v) => {
                ok_or_respond!(
                    $ctx,
                    crate::steam::get_item(&$ctx.data().pool, v).await,
                    "Could not find the item."
                )
            }
            Err(_) => {
                ok_or_respond!(
                    $ctx,
                    crate::db::items::get_item_by_name(&$ctx.data().pool, &$item),
                    "Could not find the item by that name."
                )
            }
        }
    };
}

pub(crate) use get_by_name;
pub(crate) use get_channel;
pub(crate) use get_guild;
pub(crate) use get_guild_channel;
pub(crate) use ok_or_respond;
