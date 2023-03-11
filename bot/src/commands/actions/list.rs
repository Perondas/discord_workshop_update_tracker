use itertools::Itertools;

use crate::{
    commands::common::{get_guild, ok_or_respond},
    db, Context, Error,
};

/// List all the currently subscribed items
#[poise::command(slash_command, rename = "list")]
pub async fn list_items(ctx: Context<'_>) -> Result<(), Error> {
    let guild = get_guild!(ctx);

    let subscriptions = ok_or_respond!(
        ctx,
        db::subscriptions::get_all_subscriptions_of_guild(&ctx.data().pool, guild.id.0),
        "An error occurred while fetching the subscriptions."
    );

    if subscriptions.is_empty() {
        ctx.say("There are no tracked items.").await?;
        return Ok(());
    }

    if subscriptions.len() > 10 {
        let items: Vec<String> = subscriptions
            .iter()
            .chunks(10)
            .into_iter()
            .map(|chunk| {
                chunk
                    .map(|(_, info, _)| {
                        format!(
                            "\n{}: <https://steamcommunity.com/sharedfiles/filedetails/?id={}>",
                            info.name, info.id
                        )
                    })
                    .collect::<Vec<String>>()
                    .join(", ")
            })
            .collect();
        let parts = items.len();

        for (i, part) in items.iter().enumerate() {
            ctx.say(format!(
                "Currently tracked items (part {} of {}): {}",
                i + 1,
                parts,
                part
            ))
            .await?;
        }
    } else {
        let items = subscriptions
            .iter()
            .map(|(_, info, _)| {
                format!(
                    "\n{}: <https://steamcommunity.com/sharedfiles/filedetails/?id={}>",
                    info.name, info.id
                )
            })
            .collect::<Vec<String>>()
            .join(", ");

        ctx.say(format!("Currently tracked items: {}", items))
            .await?;
    }

    Ok(())
}
