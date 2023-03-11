use crate::{db, Context, Error};

/// Restarts your tracking job, immediately checking for any updates
#[poise::command(track_edits, slash_command, rename = "restart")]
pub async fn restart(ctx: Context<'_>) -> Result<(), Error> {
    let guild = match ctx.guild() {
        Some(guild) => guild,
        None => {
            ctx.say("This command can only be used in a guild.").await?;
            return Ok(());
        }
    };

    let schedule = match db::servers::get_schedule(&ctx.data().pool, guild.id.0) {
        Ok(schedule) => schedule,
        Err(_) => {
            ctx.say("An error occurred while fetching the schedule.")
                .await?;
            return Ok(());
        }
    };

    if schedule.is_none() {
        ctx.say("Please set a schedule first.").await?;
        return Ok(());
    }

    match ctx.data().scheduler.start_schedule(guild.id.0).await {
        Ok(_) => (),
        Err(_) => {
            ctx.say("An error occurred while restarting the tracking job.")
                .await?;
            return Ok(());
        }
    }

    ctx.say("Restarted tracking job.").await?;
    Ok(())
}
