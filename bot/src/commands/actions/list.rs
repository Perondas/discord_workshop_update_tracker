use crate::{db, Context, Error};

/// List all the currently subscribed mods
#[poise::command(track_edits, slash_command)]
pub async fn list_mods(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx.guild().unwrap();

    let subscriptions =
        db::subscriptions::get_all_subscriptions_of_guild(ctx.data().pool.clone(), guild.id.0)?;

    if subscriptions.is_empty() {
        ctx.say("There are no tracked mods.").await?;
        return Ok(());
    }

    let mods = subscriptions
        .iter()
        .map(|(_, info)| {
            format!(
                "\n{}: <https://steamcommunity.com/sharedfiles/filedetails/?id={}>",
                info.name, info.id
            )
        })
        .collect::<Vec<String>>()
        .join(", ");

    ctx.say(format!("Currently tracked mods: {}", mods)).await?;

    Ok(())
}
