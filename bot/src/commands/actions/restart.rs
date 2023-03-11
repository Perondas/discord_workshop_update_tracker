use crate::{
    commands::common::{get_guild, ok_or_respond},
    db, Context, Error,
};

/// Restarts your tracking job, immediately checking for any updates
#[poise::command(slash_command, rename = "restart")]
pub async fn restart(ctx: Context<'_>) -> Result<(), Error> {
    let guild = get_guild!(ctx);

    let schedule = ok_or_respond!(
        ctx,
        db::servers::get_schedule(&ctx.data().pool, guild.id.0),
        "An error occurred while fetching the schedule."
    );

    if schedule.is_none() {
        ctx.say("Please set a schedule first.").await?;
        return Ok(());
    }

    ok_or_respond!(
        ctx,
        ctx.data().scheduler.start_schedule(guild.id.0).await,
        "An error occurred while restarting the tracking job."
    );

    ctx.say("Restarted tracking job.").await?;
    Ok(())
}
