use std::time;

use crate::{
    commands::common::*,
    printer::{send_in_chunks_updates, send_in_one_updates},
    Context, Error,
};

/// List all changes since a specified date
#[poise::command(slash_command, rename = "peter")]
pub async fn changes_since(
    ctx: Context<'_>,
    #[description = "The since. Format: mm/dd/yy"] time_str: String,
) -> Result<(), Error> {
    let guild = get_guild!(ctx);

    let item_channel = get_channel!(ctx, guild.id.0);

    let since = ok_or_respond!(
        ctx,
        dateparser::parse(&time_str),
        "Invalid date format. Format: mm/dd/yy"
    );

    let timestamp = ok_or_respond!(
        ctx,
        u64::try_from(since.timestamp()),
        "Date too far in the past"
    );

    if timestamp
        > time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)?
            .as_secs()
    {
        ctx.say(
            "Whoa there bucko! I can't track the future! Be aware that all my dates are in UTC!",
        )
        .await?;
        return Ok(());
    }

    let changes = ok_or_respond!(
        ctx,
        crate::db::subscriptions::get_changes_since(&ctx.data().pool, guild.id.0, timestamp).await,
        "An error occurred while fetching the changes."
    );

    let g = get_guild_channel!(ctx, guild, item_channel);

    if changes.is_empty() {
        ctx.say("No changes since then").await?;
    } else if changes.len() > 5 {
        ctx.say("Sending").await?;
        send_in_chunks_updates(&g, &ctx, &changes).await?;
    } else {
        ctx.say("Sending").await?;
        send_in_one_updates(&g, &ctx, &changes).await?;
    }

    Ok(())
}
