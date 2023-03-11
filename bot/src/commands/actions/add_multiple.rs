use crate::{commands::actions::get_guild_channel, db, steam::get_item, Context, Error};

/// Add multiple items to the tracked items
#[poise::command(track_edits, slash_command)]
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

    let guild = match ctx.guild() {
        Some(g) => g,
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

    ctx.say("Success").await?;

    let g = match get_guild_channel(&guild, item_channel) {
        Some(g) => g,
        None => {
            ctx.say("The update channel is no longer available").await?;
            return Ok(());
        }
    };

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
