use crate::{db, steam::get_mod, Context, Error};

/// Add a mod to the tracked mods
#[poise::command(track_edits, slash_command)]
pub async fn mod_add(
    ctx: Context<'_>,
    #[description = "The id of the mod to be tracked"] mod_id: u64,
) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();

    let mod_channel = db::servers::get_update_channel(ctx.data().pool.clone(), guild.id.0)?;

    if mod_channel.is_none() {
        ctx.say("Please set an update channel first.").await?;
        return Ok(());
    }

    let mod_info = match get_mod(ctx.data().pool.clone(), mod_id).await {
        Ok(mod_info) => mod_info,
        Err(_) => {
            ctx.say("An error occurred while fetching the mod.").await?;
            return Ok(());
        }
    };

    match db::subscriptions::add_subscription(ctx.data().pool.clone(), guild.id.0, mod_info.id) {
        Ok(_) => (),
        Err(_) => {
            ctx.say("An error occurred while adding the mod.").await?;
            return Ok(());
        }
    };

    let (_, c) = guild
        .channels
        .iter()
        .find(|c| c.0 .0 == mod_channel.unwrap())
        .unwrap();

    c.clone()
        .guild()
        .unwrap()
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

    ctx.say("Success").await?;
    Ok(())
}
