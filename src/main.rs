use anyhow::Result;
use argparse::{Cli, Commands, Operation};
use chrono::prelude::*;
use tokio::signal;
use tokio::sync::mpsc;
use tracing::{info, Level};
use zptess;
use zptess::database::Pool;
use zptess::photometer::discovery::Info;
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
fn map_model(model: argparse::Model) -> zptess::Model {
    match model {
        argparse::Model::TessW => zptess::Model::Tessw,
        argparse::Model::TAS => zptess::Model::Tas,
        argparse::Model::TessP => zptess::Model::Tessp,
    }
}

async fn do_read(model: argparse::Model, role: argparse::Role, pool: &Pool) -> Result<()> {
    let model = map_model(model);
    let (tx1, rx) = mpsc::channel::<(Timestamp, Payload)>(32);
    let tx2 = tx1.clone();
    let mut test_info: Option<Info> = None;
    let mut ref_info: Option<Info> = None;
    match role {
        argparse::Role::Test => {
            let _test_info = photometer::discover_test(model).await?;
            info!("{_test_info:#?}");
            test_info = Some(_test_info);
            let _ftest = tokio::spawn(async move {
                let _ = photometer::reading_task(tx1, false).await; // again: pool1 is moved to the task and gets out of scope
            });
        }
        argparse::Role::Ref => {
            let _ref_info = photometer::discover_ref(&pool).await?;
            info!("{_ref_info:#?}");
            ref_info = Some(_ref_info);
            let _fref = tokio::spawn(async move {
                let _ = photometer::reading_task(tx2, true).await; // again: pool1 is moved to the task and gets out of scope
            });
        }
        argparse::Role::Both => {
            let _test_info = photometer::discover_test(model).await?;
            info!("{_test_info:#?}");
            test_info = Some(_test_info);
            let _ref_info = photometer::discover_ref(&pool).await?;
            info!("{_ref_info:#?}");
            ref_info = Some(_ref_info);
            let _ftest = tokio::spawn(async move {
                let _ = photometer::reading_task(tx1, false).await; // pool1 is moved to the task and gets out of scope
            });
            let _fref = tokio::spawn(async move {
                let _ = photometer::reading_task(tx2, true).await; // again: pool1 is moved to the task and gets out of scope
            });
        }
    }
    let pool1 = pool.clone();
    tokio::spawn(async move {
        let _ = statistics::reading_task(pool1, rx, 9, ref_info, test_info).await;
        // again: pool1 is moved to the task and gets out of scope
    });
    signal::ctrl_c().await?;
    Ok(())
}

async fn do_calibrate(
    model: argparse::Model,
    pool: &Pool,
    _update: bool,
    _test: bool,
) -> Result<()> {
    let _session = Utc::now();
    let test_info = photometer::discover_test(map_model(model)).await?;
    info!("{test_info:#?}");
    let ref_info = photometer::discover_ref(&pool).await?;
    info!("{ref_info:#?}");
    let (tx1, rx) = mpsc::channel::<(Timestamp, Payload)>(32);
    let tx2 = tx1.clone();
    let ftest = tokio::spawn(async move {
        let _ = photometer::reading_task(tx1, false).await;
    });

    let fref = tokio::spawn(async move {
        let _ = photometer::reading_task(tx2, true).await;
    });
    let pool1 = pool.clone();
    let fstats = tokio::spawn(async move {
        let _ = statistics::calibration_task(pool1, rx, 9, 5, 5000, ref_info, test_info).await;
        // again: pool1 is moved to the task and gets out of scope
    });
    futures::future::join_all(vec![ftest, fref, fstats]).await;
    info!("All tasks terminated");
    Ok(())
}

//#[tokio::main]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let cli = argparse::parse();

    let mut g_author: Option<String> = None;
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

    match cli.command {
        Commands::Calibrate {
            model,
            author,
            operation,
            ..
        } => {
            match operation {
                Operation {
                    dry_run,
                    update,
                    test,
                } => {
                    let test_info = photometer::discover_test(map_model(model)).await?;
                    info!("{test_info:#?}");
                    // Display photometer info and bail out
                    if dry_run {
                        return Ok(());
                    }
                    if let Some(a) = author {
                        g_author = Some(a.join(" "));
                    }
                    do_calibrate(model, &pool, update, test).await?
                }
            }
        }

        Commands::Migrate {} => {
            return Ok(());
        }

        Commands::Update { model, zero_point } => {
            photometer::write_zero_point(map_model(model), zero_point).await?;
            return Ok(());
        }

        Commands::Read { model, role } => {
            do_read(model, role, &pool).await?;
            return Ok(());
        }
    }

    // =========================================================================
    // =========================================================================
    // =========================================================================

    return Ok(());
}
