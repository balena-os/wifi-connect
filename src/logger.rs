use env_logger::LogBuilder;
use log::{LogLevelFilter, LogRecord};
use std::env;

pub fn init() {
    let mut builder = LogBuilder::new();

    if env::var("RUST_LOG").is_ok() {
        builder.parse(&env::var("RUST_LOG").unwrap());
    } else {
        let format = |record: &LogRecord| {
            format!(
                "[{}:{}:{}:{}] {}",
                record.location().file(),
                record.location().line(),
                record.location().module_path(),
                record.level(),
                record.args()
            )
        };

        builder.format(format).filter(None, LogLevelFilter::Info);
        builder.parse("wifi-connect=info,iron::iron=info");
    }

    builder.init().unwrap();
}
