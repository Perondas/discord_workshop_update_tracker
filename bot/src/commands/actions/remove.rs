use crate::{commands::actions::get_guild_channel, db, steam::get_item, Context, Error};

/// Remove a item from the tracked items
#[poise::command(track_edits, slash_command)]
pub async fn item_remove(
    ctx: Context<'_>,
    #[description = "The id of the item to be removed"] item_id: u64,
) -> Result<(), Error> {
    let guild = match ctx.guild() {
        Some(guild) => guild,
        None => {
            ctx.say("This command can only be used in a guild.").await?;
            return Ok(());
        }
    };

    let item_channel = match db::servers::get_update_channel(&ctx.data().pool, guild.id.0) {
        Ok(c) => c,
        Err(_) => {
            ctx.say("An error occurred while fetching the update channel.")
                .await?;
            return Ok(());
        }
    };

    let item_channel = match item_channel {
        Some(c) => c,
        None => {
            ctx.say("Please set an update channel first.").await?;
            return Ok(());
        }
    };

    let item_info = get_item(&ctx.data().pool, item_id).await?;

    match db::subscriptions::remove_subscription(&ctx.data().pool, guild.id.0, item_id) {
        Ok(_) => (),
        Err(_) => {
            ctx.say("An error occurred while removing the item.")
                .await?;
            return Ok(());
        }
    };

    let g = match get_guild_channel(&guild, item_channel) {
        Some(g) => g,
        None => {
            ctx.say("The update channel is no longer available").await?;
            return Ok(());
        }
    };

    g.send_message(ctx, |d| {
        d.content(format!(
            "Removed item {} from the tracked items:",
            item_info.name
        ));

        d.embed(|e| {
            e.title(item_info.name);
            e.url(format!(
                "https://steamcommunity.com/sharedfiles/filedetails/?id={}",
                item_id
            ));
            if let Some(preview_url) = item_info.preview_url {
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
