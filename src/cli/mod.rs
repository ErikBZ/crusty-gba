use clap::Parser;

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
}

#[derive(Debug, clap::ValueEnum, Clone, Default)]
pub enum Renderer {
    #[default]
    Terminal,
    Gui,
}
