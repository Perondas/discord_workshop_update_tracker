use crate::{
    commands::common::{get_channel, get_guild, get_guild_channel, ok_or_respond},
    db,
    steam::get_item,
    Context, Error,
};

/// Remove a item from the tracked items
#[poise::command(track_edits, slash_command, rename = "remove")]
pub async fn item_remove(
    ctx: Context<'_>,
    #[description = "The id of the item to be removed"] item_id: u64,
) -> Result<(), Error> {
    let guild = get_guild!(ctx);

    let item_channel = get_channel!(ctx, guild.id.0);

    let item_info = ok_or_respond!(
        ctx,
        get_item(&ctx.data().pool, item_id).await,
        "An error occurred while fetching the item."
    );

    ok_or_respond!(
        ctx,
        db::subscriptions::remove_subscription(&ctx.data().pool, guild.id.0, item_id),
        "An error occurred while removing the item."
    );

    let g = get_guild_channel!(ctx, guild, item_channel);

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
