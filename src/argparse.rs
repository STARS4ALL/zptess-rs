use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

pub fn parse() -> Cli {
    Cli::parse()
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Turn console debugging information on
    #[arg(short, long)]
    pub console: bool,

    /// Log to a file
    #[arg(short, long, value_name = "FILE", default_value = "zptess.log")]
    pub log_file: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Database migration options
    Database {},

    /// Calibration options
    Calibrate {
        /// Photometer model
        #[arg(short, long, value_enum)]
        model: Model,

        /// Installed filter
        #[arg(long, default_value = "UV/IR-740")]
        filter: String,

        /// Power supply plug
        #[arg(long, default_value = "USB-A")]
        plug: String,

        /// Box model
        #[arg(long, default_value = "FSH714")]
        box_model: String,

        /// Author
        #[arg(short, long, action = clap::ArgAction::Append)]
        author: Option<String>,

        #[command(flatten)]
        operation: Operation,
    },
}

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct Operation {
    /// display photometer info and exit
    #[arg(short, long)]
    dry_run: bool,

    /// Calibrate and update zero point
    #[arg(short, long)]
    update: bool,

    /// Overwrites zero point
    #[arg(short, long, value_name = "ZP")]
    write_zero_point: Option<i32>,

    /// calibrate but don't update database
    #[arg(short, long)]
    test: bool,

    /// Read photometer
    #[arg(short, long, value_name = "ROLE", value_enum)]
    read: Option<Role>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum Model {
    /// TESS WiFi model
    TessW,

    /// TESS Portable model
    TessP,

    /// TESS Auto Scan model
    TAS,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[clap(rename_all = "lower")]
pub enum Role {
    /// Test photometer
    Test,

    /// Reference photometer
    Ref,

    /// Both photometers
    Both,
}
