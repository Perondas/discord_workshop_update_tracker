use std::{sync::Arc, time::Duration};

use mysql::Pool;
use poise::{
    builtins,
    serenity_prelude::{self as serenity, Command},
    Event,
};
use tokio::time::sleep;
use tracing::{debug, error};

use crate::commands::{
    actions::{add::mod_add, list::list_mods, remove::mod_remove, restart::restart},
    settings::{register_channel::*, set_schedule::*},
};

mod commands;
mod cron;
mod db;
mod steam;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, AppState, Error>;

#[derive(Clone)]
pub struct AppState {
    pool: Arc<Pool>,
    scheduler: cron::Scheduler,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt::init();

    let url = std::env::var("MYSQL_URL").expect("MYSQL_URL must be set");
    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");

    // Add DB connection pool to app state
    // We loop until we connect
    let pool = loop {
        match db::get_pool(&url) {
            Ok(p) => break p,
            Err(e) => {
                debug!("Failed to connect to DB. Reason {:?}", e);
                debug!("Trying again in 5 seconds");
                sleep(Duration::from_secs(5)).await;
            }
        }
    };

    let options = poise::FrameworkOptions {
        commands: vec![
            help(),
            mod_add(),
            mod_remove(),
            register_channel(),
            set_schedule(),
            list_mods(),
            restart(),
        ],
        on_error: |error| Box::pin(on_error(error)),
        pre_command: |ctx| {
            Box::pin(async move {
                debug!("Executing command {}...", ctx.command().qualified_name);
            })
        },

        post_command: |ctx| {
            Box::pin(async move {
                debug!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        /// This code is run after a command if it was successful (returned Ok)
        event_handler: |_context, event, _framework, state| {
            Box::pin(async move {
                match event {
                    Event::GuildCreate { guild, is_new } => {
                        if *is_new {
                            debug!("New guild found: {}", guild.name);
                            db::servers::add_server(state.pool.clone(), guild)?;
                        }
                        Ok(())
                    }
                    Event::GuildDelete { incomplete, .. } => {
                        debug!("Guild deleted: {}", incomplete.id.0);
                        state.scheduler.remove(incomplete.id.0);
                        db::servers::remove_server(state.pool.clone(), incomplete.id.0);

                        Ok(())
                    }
                    _ => Ok(()),
                }
            })
        },

        ..Default::default()
    };

    let pool = Arc::new(pool);

    let state = AppState {
        pool: pool.clone(),
        scheduler: cron::Scheduler::new(pool.clone()),
    };

    let s = state.clone();

    let framework = poise::Framework::builder()
        .token(token)
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                let commands = builtins::create_application_commands(&framework.options().commands);
                Command::set_global_application_commands(ctx, |c| {
                    *c = commands;
                    c
                })
                .await
                .unwrap();
                Ok(s)
            })
        })
        .options(options)
        .intents(serenity::GatewayIntents::non_privileged())
        .build()
        .await
        .unwrap();

    let framework_client = framework.client().cache_and_http.clone();

    // Start the cron job
    tokio::spawn(async move {
        sleep(Duration::from_secs(5)).await;
        debug!("Cron job started");
        match state.scheduler.start_cron(framework_client).await {
            Ok(_) => {}
            Err(e) => {
                error!("Cron job failed: {:?}", e);
            }
        }
    });

    framework.start().await.unwrap();
}

/// Show this help menu
#[poise::command(track_edits, slash_command)]
async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "\
You can limit availability of commands to specific users or roles trough the server settings.",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, AppState, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx } => {
            error!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error while handling error: {}", e)
            }
        }
    }
}
