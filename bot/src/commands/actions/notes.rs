use poise::Modal;
type ApplicationContext<'a> = poise::ApplicationContext<'a, AppState, Error>;

use crate::{
    commands::{
        autocomplete::autocomplete_name,
        common::{get_by_name, get_guild, ok_or_respond},
    },
    db, AppState, Error,
};

#[derive(Debug, Modal)]
#[name = "Edit Note"]
struct NoteModal {
    #[name = "Note"]
    #[max_length = 500]
    #[paragraph]
    note: Option<String>,
}

impl NoteModal {
    fn new(note: Option<String>) -> Self {
        Self { note }
    }
}

#[poise::command(slash_command, rename = "edit_note")]
pub async fn edit_note(
    ctx: ApplicationContext<'_>,
    #[autocomplete = "autocomplete_name"]
    #[description = "The id or the name of the item"]
    item: String,
) -> Result<(), Error> {
    let guild = get_guild!(ctx);

    let item_info = get_by_name!(ctx, item);

    let note = ok_or_respond!(
        ctx,
        db::subscriptions::get_note(&ctx.data().pool, guild.id.0, item_info.id),
        "An error occurred while fetching the subscription."
    );

    let response = NoteModal::execute_with_defaults(ctx, NoteModal::new(note)).await?;

    if let Some(note) = response {
        ok_or_respond!(
            ctx,
            db::subscriptions::update_subscription_note(
                &ctx.data().pool,
                guild.id.0,
                item_info.id,
                note.note
            ),
            "An error occurred while updating the note."
        );
    }

    ctx.say("Success").await?;

    Ok(())
}
