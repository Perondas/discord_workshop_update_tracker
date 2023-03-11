use crate::{
    commands::common::{get_guild, ok_or_respond},
    db, Context, Error,
};

/// Set the channel where the bot will send updates to
#[poise::command(track_edits, slash_command, rename = "register_channel")]
pub async fn register_channel(
    ctx: Context<'_>,
    #[description = "Channel Id for update broadcast"] channel_id: Option<u64>,
) -> Result<(), Error> {
    let guild = get_guild!(ctx);

    if !ok_or_respond!(
        ctx,
        db::servers::check_still_in_guild(&ctx.data().pool, guild.id.0),
        "An error occurred while checking if the bot is still in the guild."
    ) {
        ok_or_respond!(
            ctx,
            db::servers::add_server(&ctx.data().pool, &guild),
            "An error occurred while adding the server to the database."
        );
    }

    let channel_id = match channel_id {
        Some(id) => id,

        None => ctx.channel_id().0,
    };

    if !guild.channels.iter().any(|c| c.0 .0 == channel_id) {
        ctx.say("Please provide a valid channel id.").await?;
        return Ok(());
    }

    ok_or_respond!(
        ctx,
        db::servers::set_update_channel(&ctx.data().pool, guild.id.0, channel_id),
        "An error occurred while updating the channel."
    );

    ctx.say("Update channel set.").await?;

    Ok(())
}
