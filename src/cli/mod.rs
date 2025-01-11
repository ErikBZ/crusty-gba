use clap::Parser;
use tracing::Level;
use tracing_subscriber::filter::LevelFilter;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub render: Renderer,
    // Path to Game Boy Advance Bios
    #[arg(short, long)]
    pub bios: String,
    // Path to Game Boy Advance Rom
    #[arg(short, long)]
    pub game: String,
    // TODO: Add Logging Level
    #[arg(short, long)]
    pub log_level: Option<Level>
}

#[derive(Debug, clap::ValueEnum, Clone, Default)]
pub enum Renderer {
    #[default]
    Terminal,
    Gui,
    Ratatui,
}

#[derive(Debug, clap::ValueEnum, Clone, Default)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
    #[default]
    Off,
}

impl Into<LevelFilter> for LogLevel {
    fn into(self) -> LevelFilter {
        match self {
            LogLevel::Trace => LevelFilter::TRACE,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Warning => LevelFilter::WARN,
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Off => LevelFilter::OFF,
        }
    }
}
