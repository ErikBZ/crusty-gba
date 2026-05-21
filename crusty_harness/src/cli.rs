use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, default_value_t = 5)]
    /// Number of threads to use for running
    pub number_of_threads: usize,

    /// Path to directory with tests. Cannot be used with --file or --index
    #[arg(short, long)]
    pub path: Option<String>,

    /// Path to test suite. If --index is missing will run all tests in file
    #[arg(short, long)]
    pub file: Option<String>,

    /// The test to run from the suite. Requires --file
    #[arg(short, long, requires = "file")]
    pub index: Option<usize>,
}
