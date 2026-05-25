mod models;
mod report;
mod cli;

use glob::glob;
use clap::Parser;
use tracing::{info, error, warn, debug};
use tracing_subscriber::filter::{Targets, LevelFilter};
use tracing_subscriber::{fmt, prelude::*};
use std::fs::File;
use std::io::prelude::Write;
use std::process;
use std::{io::Read, fs::OpenOptions};
use crate::{models::{Test, run_test}, report::SuiteReport};

#[tokio::main]
async fn main() {
    let args = cli::Cli::parse();
    if args.file.is_some() && args.path.is_some() {
        panic!("Cannot pass --file and --path in the same command")
    }

    let log_level: LevelFilter = args.log_level.map(|l| l.into()).unwrap_or(LevelFilter::INFO);

    let filter = Targets::default()
        .with_target("crusty_harness", log_level);

    let filter = if args.index.is_some() {
        filter.with_target("crusty", LevelFilter::TRACE)
    } else {
        filter
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .init();

    let mut output = match File::create(args.out_file) {
        Ok(o) => o,
        Err(e) => {
            error!("{}", e);
            process::exit(1)
        }
    };

    if let Some(p) = args.path {
        let mut results = vec![];
        let mut passed: usize = 0;
        let mut failed: usize = 0;
        let mut skipped: usize = 0;

        let entries = match glob(&format!("{}/*.json", p)) {
            Ok(g) => g,
            Err(e) => {
                error!("{}", e);
                process::exit(1)
            }
        };

        for entry in entries {
            let path = match entry {
                Ok(f) => f,
                Err(e) => {
                    error!("{}", e);
                    continue
                }
            };

            let file = match path.to_str() {
                Some(f) => f,
                None => {
                    error!("Cannot get string from {:?}", path);
                    continue
                }
            };

            info!("Grabbing test suite from tile: {}", file);
            let tests = match deserialize_test(file) {
                Ok(tests) => tests,
                Err(e) => {
                    error!("Error for file: {}, {}", file, e);
                    continue
                }
            };
            info!("Running test on path: {}", file);
            let res = run_tests(tests, args.number_of_threads, file.to_owned()).await;
            info!("Test Results: Passed: {}, Failed: {}, Total: {}, Skipped: {}", res.passed, res.failed, res.total, res.skipped);
            passed += res.passed;
            failed += res.failed;
            skipped += res.skipped;
            results.push(res);
        }

        let j = serde_json::to_string_pretty(&results).expect("Couldn't write output json");
        output.write_all(j.as_bytes()).expect("Cannot write to file");

        info!("Total Tests Run {}: Passed: {}, Failed {} Skipped: {}", passed + failed, passed, failed, skipped);
    } else if let Some(f) = args.file {
        info!("Grabbing test suite from tile: {}", f);
        let tests = match deserialize_test(&f) {
            Ok(v) => v,
            Err(e) => {
                error!("{}", e);
                process::exit(1);
            }
        };
        info!("Running test suite");
        if let Some(i) = args.index {
            let test = match tests.into_iter().nth(i) {
                Some(t) => t,
                None => {
                    error!("Test number: {} does not exist in test suite: {}", i, f);
                    process::exit(1);
                }
            };
            let is_thumb = f.contains("thumb");
            let res = run_test(test, i, is_thumb).await;
            info!("Test Result: {:?}", res);

            let j = serde_json::to_string_pretty(&res).expect("Couldn't write output json");
            output.write_all(j.as_bytes()).expect("Cannot write to file");
        } else {
            let res = run_tests(tests, args.number_of_threads, f).await;
            info!("Test Results: Passed: {}, Failed: {}, Total: {} Skipped: {}", res.passed, res.failed, res.total, res.skipped);

            let j = serde_json::to_string_pretty(&res).expect("Couldn't write output json");
            output.write_all(j.as_bytes()).expect("Cannot write to file");
        }
    }
}

fn deserialize_test(path: &str) -> Result<Vec<Test>, serde_json::Error> {
    let mut file = OpenOptions::new().read(true).open(path).expect("Could not read file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap(); 
    serde_json::from_str(&contents)
}

async fn run_tests(tests: Vec<Test>, threads: usize, path: String) -> SuiteReport {
    let mut curr_threds = vec![];
    let is_thumb = path.contains("thumb");
    let mut report = SuiteReport::new(path);

    for (i, t) in tests.into_iter().enumerate() {
        curr_threds.push(
            tokio::spawn(
                run_test(t, i, is_thumb)
            )
        );
        if curr_threds.len() >= threads {
            for task in curr_threds {
                let res = task.await;
                match res {
                    Ok(Ok(_)) => report.add_success(),
                    Ok(Err((idx, e))) => report.add_failed(idx, e),
                    Err(e) => {
                        report.add_skipped();
                        error!("Issue running test: {}", e)
                    }
                }
            }
            curr_threds = vec![];
        }
    }

    // if anything is left over run it here
    for task in curr_threds {
        let res = task.await;
        match res {
            Ok(Ok(_)) => report.add_success(),
            Ok(Err((idx, e))) => report.add_failed(idx, e),
            Err(e) => {
                report.add_skipped();
                error!("Issue running test: {}", e)
            }
        }
    }

    report
}
