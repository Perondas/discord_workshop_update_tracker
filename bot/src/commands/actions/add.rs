use crate::{commands::actions::get_guild_channel, db, steam::get_item, Context, Error};

/// Add a item to the tracked items
#[poise::command(track_edits, slash_command)]
pub async fn item_add(
    ctx: Context<'_>,
    #[description = "The id of the item to be tracked"] item_id: u64,
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

    let item_info = match get_item(&ctx.data().pool, item_id).await {
        Ok(item_info) => item_info,
        Err(_) => {
            ctx.say("An error occurred while fetching the item.")
                .await?;
            return Ok(());
        }
    };

    match db::subscriptions::add_subscription(&ctx.data().pool, guild.id.0, item_info.id) {
        Ok(_) => (),
        Err(_) => {
            ctx.say("An error occurred while adding the item.").await?;
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
            "Added item {} to the tracked items:",
            item_info.name
        ));

        if item_info.preview_url.is_some() {
            d.embed(|e| {
                e.title(item_info.name);
                e.url(format!(
                    "https://steamcommunity.com/sharedfiles/filedetails/?id={}",
                    item_id
                ));
                if let Some(url) = item_info.preview_url {
                    e.image(url);
                }

                e
            });
        }

        d
    })
    .await?;

    ctx.say("Success").await?;
    Ok(())
}
