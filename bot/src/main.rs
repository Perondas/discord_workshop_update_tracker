use std::{sync::Arc, time::Duration};

use commands::actions::add_multiple::mod_batch_add;
use mysql::Pool;
use poise::{
    builtins,
    serenity_prelude::{self as serenity, Command},
    Event,
};
use tokio::time::sleep;
use tracing::{debug, error, info};

use crate::commands::{
    actions::{add::mod_add, list::list_mods, remove::mod_remove, restart::restart},
    settings::{info::get_info, register_channel::*, set_schedule::*},
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
                info!("Failed to connect to DB. Reason {:?}", e);
                info!("Trying again in 5 seconds");
                sleep(Duration::from_secs(5)).await;
            }
        }
    };

    info!("Connected to DB");

    let options = poise::FrameworkOptions {
        commands: vec![
            help(),
            mod_add(),
            mod_batch_add(),
            mod_remove(),
            register_channel(),
            set_schedule(),
            list_mods(),
            restart(),
            get_info(),
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
                            db::servers::add_server(&state.pool, guild)?;
                        }
                        Ok(())
                    }
                    Event::GuildDelete { incomplete, .. } => {
                        debug!("Guild left: {}", incomplete.id.0);
                        state.scheduler.remove(incomplete.id.0);
                        match db::servers::remove_server(&state.pool, incomplete.id.0) {
                            Ok(_) => {}
                            Err(e) => {
                                error!("Failed to remove server from DB: {:?}", e);
                            }
                        }

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
        scheduler: cron::Scheduler::new(pool),
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
                .expect("Failed to set global application commands");
                Ok(s)
            })
        })
        .options(options)
        .intents(serenity::GatewayIntents::non_privileged())
        .build()
        .await
        .expect("Failed to create framework");

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

    info!("Starting bot");

    framework.start().await.expect("Failed to start framework");
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

async fn on_error(e: poise::FrameworkError<'_, AppState, Error>) {
    match e {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {error}"),
        poise::FrameworkError::Command { error, ctx } => {
            error!("Error in command `{}`: {error:?}", ctx.command().name,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error while handling error: {}", e);
            }
        }
    }
}
