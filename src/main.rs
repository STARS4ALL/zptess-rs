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
use zptess;

#[tokio::main]
//#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = parse();

    let mut _guards = logging::init(Level::INFO, cli.console, Some(cli.log_file));
    let database_url = zptess::get_database_url();
    zptess::database::init(&database_url);

    let pool = zptess::database::get_connection_pool(&database_url);

    let pool1 = pool.clone();
    tokio::spawn(async move {
        photometer::task(pool1, false).await; // pool1 is moved to the task and gets out of scope
    });

    let pool1 = pool.clone();
    tokio::spawn(async move {
        photometer::task(pool1, true).await; // again: pool1 is moved to the task and gets out of scope
    });

    // Nothing to do on the main task,
    // simply waits here
    signal::ctrl_c().await.expect("Shutdown signal");
}
