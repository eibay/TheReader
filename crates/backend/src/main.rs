#![warn(
    clippy::expect_used,
    // clippy::unwrap_used,
)]

#![allow(clippy::manual_map)]

// TODO: Ping/Pong if currently viewing book. View time. How long been on page. Etc.


use clap::Parser;
use actix_web::web;

pub mod cli;
pub mod database;
pub mod error;
pub mod http;
pub mod metadata;
pub mod scanner;
pub mod task;
pub mod model;
pub mod util;

pub use cli::CliArgs;
pub use task::{queue_task, Task};
pub use util::*;
pub use error::{Result, WebResult, WebError, Error, InternalError};


#[actix_web::main]
async fn main() -> Result<()> {
    let cli_args = CliArgs::parse();

    // Load Config - Otherwise it'll be lazily loaded whenever this fn is first called.
    let _ = config::get_config();

    let db = database::init().await?;

    let db_data = web::Data::new(db);

    task::start_task_manager(db_data.clone());

    println!("Starting HTTP Server on port {}", cli_args.port);

    http::register_http_service(&cli_args, db_data).await?;

    Ok(())
}