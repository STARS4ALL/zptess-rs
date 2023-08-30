
// Include this module as part of the binary, not the library
// as this contains the actual implementation of the logging facility
mod logging;

fn main() {
    logging::init(tracing::Level::DEBUG);
    zptess::database::init("zptess.db").unwrap();
}
