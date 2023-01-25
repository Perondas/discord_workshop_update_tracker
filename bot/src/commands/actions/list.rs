use itertools::Itertools;

use crate::{db, Context, Error};

/// List all the currently subscribed mods
#[poise::command(track_edits, slash_command)]
pub async fn list_mods(ctx: Context<'_>) -> Result<(), Error> {
    let guild = match ctx.guild() {
        Some(guild) => guild,
        None => {
            ctx.say("This command can only be used in a guild.").await?;
            return Ok(());
        }
    };

    let subscriptions =
        match db::subscriptions::get_all_subscriptions_of_guild(&ctx.data().pool, guild.id.0) {
            Ok(subscriptions) => subscriptions,
            Err(_) => {
                ctx.say("An error occurred while fetching the subscriptions.")
                    .await?;
                return Ok(());
            }
        };

    if subscriptions.is_empty() {
        ctx.say("There are no tracked mods.").await?;
        return Ok(());
    }

    if subscriptions.len() > 10 {
        let mods: Vec<String> = subscriptions
            .iter()
            .chunks(10)
            .into_iter()
            .map(|chunk| {
                chunk
                    .map(|(_, info)| {
                        format!(
                            "\n{}: <https://steamcommunity.com/sharedfiles/filedetails/?id={}>",
                            info.name, info.id
                        )
                    })
                    .collect::<Vec<String>>()
                    .join(", ")
            })
            .collect();
        let parts = mods.len();

        for (i, part) in mods.iter().enumerate() {
            ctx.say(format!(
                "Currently tracked mods (part {} of {}): {}",
                i + 1,
                parts,
                part
            ))
            .await?;
        }
    } else {
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
    }

    Ok(())
}
