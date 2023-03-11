use crate::{db, Context, Error};

/// Set how often the bot should look for updates
#[poise::command(track_edits, slash_command, rename = "set_schedule")]
pub async fn set_schedule(
    ctx: Context<'_>,
    #[description = "Updates will bec checked every x hours"] interval: u64,
) -> Result<(), Error> {
    let guild = match ctx.guild() {
        Some(guild) => guild,
        None => {
            ctx.say("This command can only be used in a guild.").await?;
            return Ok(());
        }
    };

    let state = ctx.data().clone();
    match db::servers::get_update_channel(&state.pool, guild.id.0) {
        Ok(channel) => {
            if channel.is_none() {
                ctx.say("Please set an update channel first.").await?;
                return Ok(());
            }
        }
        Err(_) => {
            ctx.say("An error occurred while fetching the update channel.")
                .await?;
            return Ok(());
        }
    }

    match db::servers::set_schedule(&state.pool, guild.id.0, interval) {
        Ok(_) => (),
        Err(_) => {
            ctx.say("An error occurred while updating the schedule.")
                .await?;
            return Ok(());
        }
    }

    state.scheduler.start_schedule(guild.id.0).await?;

    ctx.say("Schedule set.").await?;

    Ok(())
}
