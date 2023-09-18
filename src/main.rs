use crate::argparse::{Cli, Commands, Operation};
use anyhow::Result;
use chrono::prelude::*;
use tokio::signal;
use tokio::sync::mpsc;
use tracing::{info, Level};
use zptess;
use zptess::photometer::payload::Payload;
use zptess::Timestamp;
use zptess::{photometer, statistics};

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

//#[tokio::main]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let cli = argparse::parse();

    let mut g_dry_run = false;
    let mut g_update = false;
    let mut g_test = false;
    let g_model;
    let g_role;
    let mut g_author = "".to_string();
    let mut _guards;

    // parse CLI to establish logging levels and sinks
    match cli {
        Cli {
            console,
            log_file,
            verbose,
            ..
        } => {
            let level = match verbose {
                0 => Level::ERROR,
                1 => Level::INFO,
                _ => Level::DEBUG,
            };
            _guards = logging::init(level, console, Some(log_file));
        }
    };

    let database_url = zptess::get_database_url();
    zptess::database::init(&database_url);
    let pool = zptess::database::get_connection_pool(&database_url);
    let session = Utc::now();

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
                test,
            } => {
                g_dry_run = dry_run;
                g_update = update;
                if let Some(a) = author {
                    g_author = a.join(" ");
                }
            }
        },
        Commands::Migrate {} => {
            return Ok(());
        }
        Commands::Read { model, role } => {
            g_model = model;
            g_role = role;
        }

        Commands::Update { model, zero_point } => {
            photometer::write_zero_point(model, zero_point).await?;
            return Ok(());
        }
    }

    // =========================================================================
    // =========================================================================
    // =========================================================================

    let test_info = photometer::discover_test().await?;
    info!("{test_info:#?}");
    // Display photometer info and bail out
    if g_dry_run {
        return Ok(());
    }

    let ref_info = photometer::discover_ref(&pool).await?;
    info!("{ref_info:#?}");

    let (tx1, rx) = mpsc::channel::<(Timestamp, Payload)>(32);
    let tx2 = tx1.clone();

    let _session1 = session.clone(); // To move it to the proper thread
    let ftest = tokio::spawn(async move {
        let _ = photometer::calibrate_task(tx1, false).await; // pool1 is moved to the task and gets out of scope
    });

    let fref = tokio::spawn(async move {
        let _ = photometer::calibrate_task(tx2, true).await; // again: pool1 is moved to the task and gets out of scope
    });

    let pool1 = pool.clone();
    let stats = tokio::spawn(async move {
        let _ = statistics::collect_task(pool1, rx, 9, 5, 5000, ref_info, test_info).await;
        // again: pool1 is moved to the task and gets out of scope
    });

    futures::future::join_all(vec![ftest, fref, stats]).await;
    info!("All tasks terminated");
    // Nothing to do on the main task,
    // simply waits here
    signal::ctrl_c().await?;
    return Ok(());
}
