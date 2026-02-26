use std::env;

mod api;
mod app;
mod git;
mod models;
mod ui;

use anyhow::Result;
use app::App;
use tokio::sync::oneshot;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("usage: browsegit <github_token>");
        return Ok(());
    }

    let token = args[1].clone();

    let (tx, rx) = oneshot::channel();
    tokio::spawn(async move {
        let result = api::get_repos(&token).await;
        let _ = tx.send(result);
    });

    let mut app = App::loading();
    ui::run(&mut app, rx)?;

    Ok(())
}