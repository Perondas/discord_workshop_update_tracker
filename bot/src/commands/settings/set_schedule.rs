use crate::{
    commands::common::{get_guild, ok_or_respond},
    db, Context, Error,
};

/// Set how often the bot should look for updates
#[poise::command(track_edits, slash_command, rename = "set_schedule")]
pub async fn set_schedule(
    ctx: Context<'_>,
    #[description = "Updates will bec checked every x hours"] interval: u64,
) -> Result<(), Error> {
    let guild = get_guild!(ctx);

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

    ok_or_respond!(
        ctx,
        db::servers::set_schedule(&state.pool, guild.id.0, interval),
        "An error occurred while updating the schedule."
    );

    state.scheduler.start_schedule(guild.id.0).await?;

    ctx.say("Schedule set.").await?;

    Ok(())
}
