use crate::{commands::actions::get_guild_channel, db, steam::get_mod, Context, Error};

/// Add multiple mods to the tracked mods
#[poise::command(track_edits, slash_command)]
pub async fn mod_batch_add(
    ctx: Context<'_>,
    #[description = "The id of the mod to be tracked. Separated by a comma(,)"] mod_ids: String,
) -> Result<(), Error> {
    let mut errors = vec![];
    let mod_ids: Vec<_> = mod_ids
        .split(',')
        .into_iter()
        .map(|s| {
            let s = s.trim();
            s.parse::<u64>()
        })
        .filter_map(|r| r.map_err(|e| errors.push(e)).ok())
        .collect();

    if !errors.is_empty() {
        ctx.say("An error occurred while parsing the mod ids.")
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

    ctx.say("Success").await?;

    let g = match get_guild_channel(&guild, mod_channel) {
        Some(g) => g,
        None => {
            ctx.say("The update channel is no longer available").await?;
            return Ok(());
        }
    };

    for mod_id in mod_ids {
        let mod_info = match get_mod(&ctx.data().pool, mod_id).await {
            Ok(mod_info) => mod_info,
            Err(_) => {
                g.send_message(ctx, |d| {
                    d.content(format!(
                        "An error occurred while fetching the mod {}.",
                        mod_id
                    ));
                    d
                })
                .await?;
                continue;
            }
        };

        match db::subscriptions::add_subscription(&ctx.data().pool, guild.id.0, mod_info.id) {
            Ok(_) => (),
            Err(_) => {
                g.send_message(ctx, |d| {
                    d.content(format!(
                        "An error occurred while subscribing to the mod {}.",
                        mod_info.name
                    ));
                    d
                })
                .await?;
                continue;
            }
        };

        g.send_message(ctx, |d| {
            d.content(format!("Added mod {} to the tracked mods:", mod_info.name));

            d.embed(|e| {
                e.title(mod_info.name);
                e.url(format!(
                    "https://steamcommunity.com/sharedfiles/filedetails/?id={}",
                    mod_id
                ));
                if let Some(url) = mod_info.preview_url {
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
