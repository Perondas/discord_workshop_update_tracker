use crate::{db, Context, Error};

/// Set the channel where the bot will send updates
#[poise::command(track_edits, slash_command)]
pub async fn register_channel(
    ctx: Context<'_>,
    #[description = "Channel Id for update broadcast"] channel_id: Option<u64>,
) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();

    if !db::servers::check_still_in_guild(ctx.data().pool.clone(), guild.id.0)? {
        db::servers::add_server(ctx.data().pool.clone(), &guild)?;
    }

    let channel_id = match channel_id {
        Some(id) => id,

        None => ctx.channel_id().0,
    };

    if !guild.channels.iter().any(|c| c.0 .0 == channel_id) {
        ctx.say("Please provide a valid channel id.").await?;
        return Ok(());
    }

    match db::servers::set_update_channel(
        ctx.data().pool.clone(),
        ctx.guild().unwrap().id.0,
        channel_id,
    ) {
        Ok(_) => (),
        Err(_) => {
            ctx.say("An error occurred while updating the channel.")
                .await?;
            return Ok(());
        }
    }

    ctx.say("Update channel set.").await?;

    Ok(())
}
