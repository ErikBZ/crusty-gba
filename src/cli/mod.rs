use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    // Path to Game Boy Advance Bios
    #[arg(short, long)]
    pub bios: String,
    // Path to Game Boy Advance Rom
    #[arg(short, long)]
    pub game: String,
    // TODO: Add Logging Level
}
