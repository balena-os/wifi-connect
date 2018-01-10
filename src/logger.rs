use std::env;
use log::{LogLevel, LogLevelFilter, LogRecord};
use env_logger::LogBuilder;

pub fn init() {
    let mut builder = LogBuilder::new();

    if env::var("RUST_LOG").is_ok() {
        builder.parse(&env::var("RUST_LOG").unwrap());
    } else {
        let format = |record: &LogRecord| {
            if record.level() == LogLevel::Info {
                format!("{}", record.args())
            } else {
                format!(
                    "[{}:{}] {}",
                    record.location().module_path(),
                    record.level(),
                    record.args()
                )
            }
        };

        builder.format(format).filter(None, LogLevelFilter::Info);

        builder.parse("wifi-connect=info,iron::iron=off");
    }

    builder.init().unwrap();
}
