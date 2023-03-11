use poise::Modal;
type ApplicationContext<'a> = poise::ApplicationContext<'a, AppState, Error>;

use crate::{
    commands::common::{get_guild, ok_or_respond},
    db,
    steam::get_item,
    AppState, Error,
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
    #[description = "The id or the name of the item"] item_id: u64,
) -> Result<(), Error> {
    let guild = get_guild!(ctx);

    let item_info = ok_or_respond!(
        ctx,
        get_item(&ctx.data().pool, item_id).await,
        "An error occurred while fetching the item."
    );

    let note = ok_or_respond!(
        ctx,
        db::subscriptions::get_note(&ctx.data().pool, guild.id.0, item_id),
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
