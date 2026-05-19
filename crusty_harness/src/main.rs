mod models;

use std::{io::Read, fs::OpenOptions};
use crate::models::Test;


fn main() {
    println!("Hello, world!");
    let path = "test.json";
    let mut file = OpenOptions::new().read(true).open(path).expect("Could not read file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap(); 

    let tests: Vec<Test> = serde_json::from_str(&contents).unwrap();

    println!("{}", tests.len());
    println!("{:?}", tests);
}
