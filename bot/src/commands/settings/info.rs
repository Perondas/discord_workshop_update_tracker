use crate::{db, Context, Error};

/// Get info about your bot
#[poise::command(track_edits, slash_command)]
pub async fn get_info(ctx: Context<'_>) -> Result<(), Error> {
    let guild = match ctx.guild() {
        Some(guild) => guild,
        None => {
            ctx.say("This command can only be used in a guild.").await?;
            return Ok(());
        }
    };

    let subscriptions =
        match db::subscriptions::get_all_subscriptions_of_guild(&ctx.data().pool, guild.id.0) {
            Ok(subscriptions) => subscriptions,
            Err(_) => {
                ctx.say("An error occurred while fetching the subscriptions.")
                    .await?;
                return Ok(());
            }
        };
    let count = subscriptions.len();

    let is_running = ctx.data().scheduler.is_running(guild.id.0);

    let last_update = match db::servers::get_last_update(&ctx.data().pool, guild.id.0) {
        Ok(last_update) => last_update,
        Err(_) => {
            ctx.say("An error occurred while fetching the last update.")
                .await?;
            return Ok(());
        }
    };
    let mut msg = String::new();
    msg.push_str(&format!("Your server is subscribed to {count} mods\n"));
    let status = if is_running { "running" } else { "not running" };
    let time = match last_update {
        Some(last_update) => format!("<t:{last_update}:R>"),
        None => "never".to_string(),
    };
    msg.push_str(&format!("The tracking job is {status}\n"));
    msg.push_str(&format!("The last update was: {time}\n"));

    ctx.say(msg).await?;

    Ok(())
}
