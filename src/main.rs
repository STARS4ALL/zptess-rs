use anyhow::Result;
use argparse::{Cli, Commands, Operation};
use chrono::prelude::*;
use tokio::signal;
use tokio::sync::mpsc;
use tracing::info;
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

async fn do_read(model: argparse::Model, role: argparse::Role, pool: &Pool) -> Result<()> {
    let model = model.map_model();
    let (tx1, rx) = mpsc::channel::<(Timestamp, Payload)>(32);
    let tx2 = tx1.clone();
    let mut test_info: Option<Info> = None;
    let mut ref_info: Option<Info> = None;
    match role {
        argparse::Role::Test => {
            let _test_info = photometer::discover_test(&model).await?;
            info!("{_test_info:#?}");
            test_info = Some(_test_info);
            let _ftest = tokio::spawn(async move {
                let _ = photometer::reading_task(tx1, false).await; // again: pool1 is moved to the task and gets out of scope
            });
        }
        argparse::Role::Ref => {
            let _ref_info = photometer::discover_ref(pool).await?;
            info!("{_ref_info:#?}");
            ref_info = Some(_ref_info);
            let _fref = tokio::spawn(async move {
                let _ = photometer::reading_task(tx2, true).await; // again: pool1 is moved to the task and gets out of scope
            });
        }
        argparse::Role::Both => {
            let _test_info = photometer::discover_test(&model).await?;
            info!("{_test_info:#?}");
            test_info = Some(_test_info);
            let _ref_info = photometer::discover_ref(pool).await?;
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
    _author: Option<String>,
) -> Result<()> {
    let session = Utc::now();
    let model = model.map_model();
    let test_info = photometer::discover_test(&model).await?;
    info!("{test_info:#?}");
    let ref_info = photometer::discover_ref(pool).await?;
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
        let result =
            statistics::calibration_task(pool1, session, rx, 9, 5, 5000, ref_info, test_info).await;
        let zp = result.expect("Calibrated ZP");
        if _update {
            photometer::write_zero_point(&model, zp)
                .await
                .expect("Written ZP OK");
        }
    });
    futures::future::join_all(vec![ftest, fref, fstats]).await;
    info!("All tasks terminated");
    Ok(())
}

//#[tokio::main]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let cli = argparse::parse();
    let Cli {
        console,
        log_file,
        verbose,
        command,
    } = cli;

    let level = Cli::log_level(verbose);
    let _guards = logging::init(level, console, Some(log_file));

    let database_url = zptess::get_database_url();
    zptess::database::init(&database_url);
    let pool = zptess::database::get_connection_pool(&database_url);

    match command {
        Commands::Calibrate {
            model,
            // filter,
            // plug,
            // box_model,
            author,
            operation,
            ..
        } => {
            let Operation {
                dry_run,
                update,
                test,
            } = operation;

            //let mut g_author: Option<String> = None;
            // Display photometer info and bail out
            if dry_run {
                let model = model.map_model();
                let test_info = photometer::discover_test(&model).await?;
                info!("{test_info:#?}");
                return Ok(());
            }
            // Join the vector of strings into a single string
            let author = author.map(|a| a.join(" "));
            do_calibrate(model, &pool, update, test, author).await?
        }

        Commands::Migrate {} => {
            return Ok(());
        }

        Commands::Update { model, zero_point } => {
            let model = model.map_model();
            photometer::write_zero_point(&model, zero_point).await?;
            return Ok(());
        }

        Commands::Read { model, role } => {
            do_read(model, role, &pool).await?;
            return Ok(());
        }
    }

    Ok(())
}
