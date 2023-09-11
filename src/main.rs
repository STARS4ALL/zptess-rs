use crate::argparse::{Cli, Commands, Operation};
use tracing::Level;
use zptess::photometer;
// Include these modules as part of the binary crate, not the library crate
// as this contains the actual implementation of the logging facility
mod argparse;
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
    let cli = argparse::parse();

    let mut g_dry_run = false;
    let mut g_update = false;
    let mut g_test = false;
    let mut g_write_zp = None;
    let mut g_migrate = false;
    let mut g_verbose = 0;
    let mut g_author = "".to_string();

    let (g_console, g_log_file, g_verbose) = match &cli {
        Cli {
            console,
            log_file,
            verbose,
            ..
        } => (console, log_file, verbose),
    };

    match cli.command {
        Commands::Calibrate {
            model,
            author,
            operation,
            ..
        } => match operation {
            Operation {
                dry_run,
                update,
                write_zero_point,
                test,
                read,
            } => {
                g_dry_run = dry_run;
                g_update = update;
                g_write_zp = write_zero_point;
                g_test = test;
                if let Some(a) = author {
                    g_author = a.join(" ");
                }
            }
        },
        Commands::Migrate {} => {
            g_migrate = true;
        }
    }

    if let Some(zp) = g_write_zp {}
    /*
        let level = match g_verbose {
            0 => Level::ERROR,
            1 => Level::INFO,
            _ => Level::DEBUG,
        };
    */
    // =========================================================================
    // =========================================================================
    // =========================================================================

    let mut _guards = logging::init(Level::INFO, cli.console, Some(cli.log_file));
    let database_url = zptess::get_database_url();
    zptess::database::init(&database_url);

    let pool = zptess::database::get_connection_pool(&database_url);

    let pool1 = pool.clone();
    let ftest = tokio::spawn(async move {
        photometer::calibrate(pool1, false, false).await; // pool1 is moved to the task and gets out of scope
    });

    let pool1 = pool.clone();
    let fref = tokio::spawn(async move {
        photometer::calibrate(pool1, true, false).await; // again: pool1 is moved to the task and gets out of scope
    });
    futures::future::join_all(vec![ftest, fref]).await;
    // Nothing to do on the main task,
    // simply waits here
    signal::ctrl_c().await.expect("Shutdown signal");
}
