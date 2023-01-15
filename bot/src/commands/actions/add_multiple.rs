use crate::{db, steam::get_mod, Context, Error};

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

    if errors.len() > 0 {
        ctx.say("An error occurred while parsing the mod ids.")
            .await?;
        return Ok(());
    }

    let guild = ctx.guild().unwrap();

    let mod_channel = db::servers::get_update_channel(ctx.data().pool.clone(), guild.id.0)?;

    if mod_channel.is_none() {
        ctx.say("Please set an update channel first.").await?;
        return Ok(());
    }

    let (_, c) = guild
        .channels
        .iter()
        .find(|c| c.0 .0 == mod_channel.unwrap())
        .unwrap();
    let res_c = c.clone().guild().unwrap();

    for mod_id in mod_ids {
        let mod_info = match get_mod(ctx.data().pool.clone(), mod_id).await {
            Ok(mod_info) => mod_info,
            Err(_) => {
                res_c
                    .send_message(ctx, |d| {
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

        match db::subscriptions::add_subscription(ctx.data().pool.clone(), guild.id.0, mod_info.id)
        {
            Ok(_) => (),
            Err(_) => {
                res_c
                    .send_message(ctx, |d| {
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

        res_c
            .send_message(ctx, |d| {
                d.content(format!("Added mod {} to the tracked mods:", mod_info.name));

                if mod_info.preview_url.is_some() {
                    d.embed(|e| {
                        e.title(mod_info.name);
                        e.url(format!(
                            "https://steamcommunity.com/sharedfiles/filedetails/?id={}",
                            mod_id
                        ));
                        e.image(mod_info.preview_url.unwrap());
                        e
                    });
                }

                d
            })
            .await?;
        continue;
    }

    ctx.say("Success").await?;
    Ok(())
}
