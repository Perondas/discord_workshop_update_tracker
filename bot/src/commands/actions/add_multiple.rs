use poise::serenity_prelude::{Guild, GuildChannel};
use tracing::error;

use crate::{
    commands::common::{get_channel, get_guild, get_guild_channel, ok_or_respond},
    db,
    steam::{self, get_item},
    Context, Error,
};

/// Add multiple items to the tracked items
#[poise::command(slash_command, rename = "add_multiple")]
pub async fn item_batch_add(
    ctx: Context<'_>,
    #[description = "The id of the item to be tracked. Separated by a comma(,)"] item_ids: String,
) -> Result<(), Error> {
    let mut errors = vec![];
    let item_ids: Vec<_> = item_ids
        .split(',')
        .into_iter()
        .map(|s| {
            let s = s.trim();
            s.parse::<u64>()
        })
        .filter_map(|r| r.map_err(|e| errors.push(e)).ok())
        .collect();

    if !errors.is_empty() {
        ctx.say("An error occurred while parsing the item ids.")
            .await?;
        for error in errors {
            ctx.say(format!("Error: {}", error)).await?;
        }
        return Ok(());
    }

    let guild = get_guild!(ctx);

    let item_channel = get_channel!(ctx, guild.id.0);

    let g = get_guild_channel!(ctx, guild, item_channel);

    ctx.say("Success").await?;

    add_by_id(ctx, item_ids, guild, g).await
}

/// Add all members of a collection to the tracked items
#[poise::command(slash_command, rename = "add_collection")]
pub async fn collection_add(
    ctx: Context<'_>,
    #[description = "The id of the collection to be added"] collection_id: u64,
) -> Result<(), Error> {
    let guild = get_guild!(ctx);

    let item_channel = get_channel!(ctx, guild.id.0);

    let g = get_guild_channel!(ctx, guild, item_channel);

    let collection = ok_or_respond!(
        ctx,
        steam::get_collection_ids(&ctx.data().pool, collection_id).await,
        "An error occurred while fetching the collection."
    );

    ctx.say(format!("Got collection. Adding {} items", collection.len()))
        .await?;

    add_by_id(ctx, collection, guild, g).await
}

async fn add_by_id(
    ctx: Context<'_>,
    item_ids: Vec<u64>,
    guild: Guild,
    g: GuildChannel,
) -> Result<(), Error> {
    for item_id in item_ids {
        let item_info = match get_item(&ctx.data().pool, item_id).await {
            Ok(item_info) => item_info,
            Err(_) => {
                g.send_message(ctx, |d| {
                    d.content(format!(
                        "An error occurred while fetching the item {}.",
                        item_id
                    ));
                    d
                })
                .await?;
                continue;
            }
        };

        match db::subscriptions::add_subscription(&ctx.data().pool, guild.id.0, item_info.id) {
            Ok(_) => (),
            Err(_) => {
                error!(
                    "An error occurred while adding the item {} to the database.",
                    item_id
                );
                g.send_message(ctx, |d| {
                    d.content(format!(
                        "An error occurred while subscribing to the item {}.",
                        item_info.name
                    ));
                    d
                })
                .await?;
                continue;
            }
        };

        g.send_message(ctx, |d| {
            d.content(format!(
                "Added item {} to the tracked items:",
                item_info.name
            ));

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

            d
        })
        .await?;
        continue;
    }

    Ok(())
}
