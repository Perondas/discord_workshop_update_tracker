use crate::{
    commands::common::{get_channel, get_guild, get_guild_channel},
    db,
    steam::get_item,
    Context, Error,
};

/// Add multiple items to the tracked items
#[poise::command(track_edits, slash_command, rename = "add_multiple")]
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
