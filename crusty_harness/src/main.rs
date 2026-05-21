mod models;
mod report;
mod cli;

use clap::Parser;
use std::{io::Read, fs::OpenOptions};
use crate::{models::{Test, run_test}, report::SuiteReport};


#[tokio::main]
async fn main() {
    let args = cli::Cli::parse();
    if args.file.is_some() && args.path.is_some() {
        panic!("Cannot pass --file and --path in the same command")
    }

    let path = "test.json";
    let mut file = OpenOptions::new().read(true).open(path).expect("Could not read file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap(); 

    let tests: Vec<Test> = serde_json::from_str(&contents).unwrap();
    println!("{:?}", tests.len());
    let num_of_threads = args.number_of_threads;
    let report = run_tests(tests, num_of_threads).await;

    println!("{:?}", report);
}

async fn run_tests(tests: Vec<Test>, threads: usize) -> SuiteReport {
    let mut curr_threds = vec![];
    let mut report = SuiteReport::new();

    for (i, t) in tests.into_iter().enumerate() {
        curr_threds.push(
            tokio::spawn(
                run_test(t, i)
            )
        );
        if curr_threds.len() >= threads {
            for task in curr_threds {
                let res = task.await;
                match res {
                    Ok(_) => report.add_success(),
                    Err(_) => report.add_failed(),
                }
            }
            curr_threds = vec![];
        }
    }

    // if anything is left over run it here
    for task in curr_threds {
        let res = task.await;
        match res {
            Ok(_) => report.add_success(),
            Err(_) => report.add_failed(),
        }
    }

    report
}
