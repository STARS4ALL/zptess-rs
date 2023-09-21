use clap::ArgAction::{Append, Count};
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

pub fn parse() -> Cli {
    Cli::parse()
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
#[clap(rename_all = "kebab-case")]
pub enum Model {
    /// TESS WiFi model
    TessW,

    /// TESS Portable model
    TessP,

    /// TESS Auto Scan model
    TAS,
}

impl Model {
    pub fn map_model(&self) -> zptess::Model {
        match self {
            Model::TessW => zptess::Model::Tessw,
            Model::TAS => zptess::Model::Tas,
            Model::TessP => zptess::Model::Tessp,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
#[clap(rename_all = "lower")]
pub enum Role {
    /// Test photometer
    Test,

    /// Reference photometer
    Ref,

    /// Both photometers
    Both,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Log to console
    #[arg(short, long)]
    pub console: bool,

    /// Log to a file
    #[arg(short, long, value_name = "FILE", default_value = "zptess.log")]
    pub log_file: PathBuf,

    /// Log level, multiple times
    #[arg(short, long, action = Count)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Database migration options
    Migrate {},

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
        #[arg(short, long, action = Append, value_delimiter = ' ', num_args = 1..)]
        author: Option<Vec<String>>,

        /// Specific operation
        #[command(flatten)]
        operation: Operation,
    },

    // Continuosly read photometer(s)
    Read {
        /// Photometer model
        #[arg(short, long, value_enum, default_value = "tess-w")]
        model: Model,

        /// Read photometer
        #[arg(short, long, value_name = "ROLE", value_enum)]
        role: Role,
    },

    // Updates Zero point directly
    Update {
        /// Photometer model
        #[arg(short, long, value_enum, default_value = "tess-w")]
        model: Model,

        /// Overwrites zero point
        #[arg(short, long, value_name = "ZP")]
        zero_point: f32,
    },
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
pub struct Operation {
    /// display photometer info and exit
    #[arg(short, long)]
    pub dry_run: bool,

    /// Calibrate and update zero point
    #[arg(short, long)]
    pub update: bool,

    /// calibrate but don't update database
    #[arg(short, long)]
    pub test: bool,
}
