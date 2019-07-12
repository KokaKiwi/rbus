use custom_error::custom_error;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
#[structopt(raw(global_setting = "structopt::clap::AppSettings::ColoredHelp"))]
struct Options {
    #[structopt(name = "verbose", short, long, parse(from_occurrences))]
    /// Enable verbose output (can be set multiple times)
    verbosity: u8,
}

custom_error! {
    pub RBusCliError
        Io { source: std::io::Error } = "I/O Error: {source}",
}

#[paw::main]
fn main(_options: Options) -> Result<(), RBusCliError> {
    Ok(())
}
