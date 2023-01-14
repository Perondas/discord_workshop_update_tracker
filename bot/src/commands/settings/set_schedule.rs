use crate::{db, Context, Error};

/// Set how often the bot should look for updates
#[poise::command(track_edits, slash_command)]
pub async fn set_schedule(
    ctx: Context<'_>,
    #[description = "Updates will bec checked every x hours"] interval: u64,
) -> Result<(), Error> {
    let state = ctx.data().clone();
    if db::servers::get_update_channel(state.pool.clone(), ctx.guild_id().unwrap().0).is_err() {
        ctx.say("Please set an update channel first.").await?;
        return Ok(());
    }

    match db::servers::set_schedule(state.pool.clone(), ctx.guild_id().unwrap().0, interval) {
        Ok(_) => (),
        Err(_) => {
            ctx.say("An error occurred while updating the schedule.")
                .await?;
            return Ok(());
        }
    }

    state
        .scheduler
        .start_schedule(ctx.guild_id().unwrap().0)
        .await?;

    ctx.say("Schedule set.").await?;

    Ok(())
}
