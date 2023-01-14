use crate::{db, Context, Error};

/// Restarts your tracking job, immediately checking for any updates
#[poise::command(track_edits, slash_command)]
pub async fn restart(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();

    let schedule = db::servers::get_schedule(ctx.data().pool.clone(), guild.id.0)?;

    if schedule.is_none() {
        ctx.say("Please set a schedule first.").await?;
        return Ok(());
    }

    ctx.data().scheduler.start_schedule(guild.id.0).await?;

    Ok(())
}
