use crate::{commands::actions::get_guild_channel, db, steam::get_mod, Context, Error};

/// Remove a mod from the tracked mods
#[poise::command(track_edits, slash_command)]
pub async fn mod_remove(
    ctx: Context<'_>,
    #[description = "The id of the mod to be removed"] mod_id: u64,
) -> Result<(), Error> {
    let guild = match ctx.guild() {
        Some(guild) => guild,
        None => {
            ctx.say("This command can only be used in a guild.").await?;
            return Ok(());
        }
    };

    let mod_channel = match db::servers::get_update_channel(&ctx.data().pool, guild.id.0) {
        Ok(c) => c,
        Err(_) => {
            ctx.say("An error occurred while fetching the update channel.")
                .await?;
            return Ok(());
        }
    };

    let mod_channel = match mod_channel {
        Some(c) => c,
        None => {
            ctx.say("Please set an update channel first.").await?;
            return Ok(());
        }
    };

    let mod_info = get_mod(&ctx.data().pool, mod_id).await?;

    match db::subscriptions::remove_subscription(&ctx.data().pool, guild.id.0, mod_id) {
        Ok(_) => (),
        Err(_) => {
            ctx.say("An error occurred while removing the mod.").await?;
            return Ok(());
        }
    };

    let g = match get_guild_channel(&guild, mod_channel) {
        Some(g) => g,
        None => {
            ctx.say("The update channel is no longer available").await?;
            return Ok(());
        }
    };

    g.send_message(ctx, |d| {
        d.content(format!(
            "Removed mod {} from the tracked mods:",
            mod_info.name
        ));

        d.embed(|e| {
            e.title(mod_info.name);
            e.url(format!(
                "https://steamcommunity.com/sharedfiles/filedetails/?id={}",
                mod_id
            ));
            if let Some(preview_url) = mod_info.preview_url {
                e.image(preview_url);
            }

            e
        });

        d
    })
    .await?;

    ctx.say("Success").await?;
    Ok(())
}
