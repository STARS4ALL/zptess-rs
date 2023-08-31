use std::path::PathBuf;
use tracing::{Level};


// Include this module as part of the binary, not the library
// as this contains the actual implementation of the logging facility
mod logging;

fn main() {
    //logging::init(tracing::Level::DEBUG);
    let mut _guards = logging::init(Level::INFO, true, Some(PathBuf::from("zptess.log")));
    zptess::database::init("zptess.db").unwrap();
}
