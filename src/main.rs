use std::path::PathBuf;
use tracing::Level;
use zptess::argparse::parse;
use zptess::photometer;
// Include this module as part of the binary crate, not the library crate
// as this contains the actual implementation of the logging facility
mod logging;

/*
fn get_default_log_path() -> PathBuf {
    let mut path = env::current_exe().unwrap();
    path.pop();
    path.push("log/debug.log");
    path
}

#[derive(Parser, Debug)]
struct Cli {
    #[arg(default_value=get_default_log_path().into_os_string())]
    log_path: PathBuf,
}

*/

use tokio::signal;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = parse();

    let mut _guards = logging::init(Level::INFO, cli.console, Some(cli.log_file));
    let database_url = zptess::get_database_url();
    let _connection = zptess::database::init(&database_url);

    tracing::info!("Alla que vamos!");

    tokio::spawn(async move {
        photometer::task(false).await;
    });
    // Nothing to do on the main task,
    // simply waits here
    signal::ctrl_c().await.expect("Shutdown signal");
}
