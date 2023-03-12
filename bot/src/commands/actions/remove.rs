use crate::commands::autocomplete::autocomplete_name;
use poise::serenity_prelude::ButtonStyle;

use crate::{
    commands::common::{get_by_name, get_channel, get_guild, get_guild_channel, ok_or_respond},
    db::{self, items::get_item_by_name},
    steam::get_item,
    Context, Error,
};

/// Remove an item from the tracked items
#[poise::command(slash_command, rename = "remove")]
pub async fn item_remove(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_name"]
    #[description = "The id of name of item to be removed"]
    item: String,
) -> Result<(), Error> {
    let guild = get_guild!(ctx);

    let item_channel = get_channel!(ctx, guild.id.0);

    let item_info = get_by_name!(ctx, item);

    ok_or_respond!(
        ctx,
        db::subscriptions::remove_subscription(&ctx.data().pool, guild.id.0, item_info.id),
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
                item_info.id
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

/// Remove all items from the tracked items
#[poise::command(slash_command, rename = "remove_all")]
pub async fn remove_all(ctx: Context<'_>) -> Result<(), Error> {
    let guild = get_guild!(ctx);

    let item_channel = get_channel!(ctx, guild.id.0);

    let reply = ctx
        .send(|b| {
            b.content("Are you sure you want to remove all items?");
            b.components(|c| {
                c.create_action_row(|r| {
                    r.create_button(|b| {
                        b.style(ButtonStyle::Primary);
                        b.label("Yes");
                        b.custom_id("yes");

                        b
                    });
                    r.create_button(|b| {
                        b.style(ButtonStyle::Danger);
                        b.label("No");
                        b.custom_id("no");

                        b
                    });

                    r
                });

                c
            });
            b
        })
        .await?;

    let interaction = reply
        .message()
        .await?
        .await_component_interaction(ctx)
        .author_id(ctx.author().id)
        .await;

    reply
        .edit(ctx, |b| {
            b.components(|b| b).content("Processing... Please wait.")
        })
        .await?;
    // remove buttons after button press and edit message
    let pressed_button_id = match &interaction {
        Some(m) => &m.data.custom_id,
        None => {
            ctx.say(":warning: You didn't interact in time - please run the command again.")
                .await?;
            return Ok(());
        }
    };

    if pressed_button_id == "no" {
        reply
            .edit(ctx, |b| b.components(|b| b).content("Cancelled."))
            .await?;
        return Ok(());
    }

    ok_or_respond!(
        ctx,
        db::subscriptions::remove_all_subscriptions(&ctx.data().pool, guild.id.0),
        "An error occurred while removing all items."
    );

    reply
        .edit(ctx, |b| b.components(|b| b).content("Done!"))
        .await?;

    let g = get_guild_channel!(ctx, guild, item_channel);

    g.send_message(ctx, |d| {
        d.content("Removed all items.");

        d
    })
    .await?;

    Ok(())
}
