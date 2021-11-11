use clap::Parser;

const DEFAULT_SSID: &str = "WiFiConnect";

#[derive(Parser)]
pub struct Opts {
    #[clap(short, long, default_value = DEFAULT_SSID)]
    pub ssid: String,
}
