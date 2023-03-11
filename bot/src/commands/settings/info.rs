use crate::{
    commands::common::{get_guild, ok_or_respond},
    db, Context, Error,
};

/// Get info about your bot
#[poise::command(slash_command, rename = "info")]
pub async fn get_info(ctx: Context<'_>) -> Result<(), Error> {
    let guild = get_guild!(ctx);

    let count = ok_or_respond!(
        ctx,
        db::subscriptions::count_guild_subscriptions(&ctx.data().pool, guild.id.0),
        "An error occurred while fetching the subscriptions."
    );

    let is_running = ctx.data().scheduler.is_running(guild.id.0);

    let last_update = ok_or_respond!(
        ctx,
        db::servers::get_last_update(&ctx.data().pool, guild.id.0),
        "An error occurred while fetching the last update."
    );

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
