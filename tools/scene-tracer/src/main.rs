//! scene-tracer: pre-renders argumentation scenes to JSON for the website.
//!
//! Usage:
//!   cargo run -p scene-tracer -- east-wall 0.5 website/static/traces/east-wall-b05.json
//!   cargo run -p scene-tracer -- siege-cold 0.5 website/static/traces/siege-cold-b05.json
//!   cargo run -p scene-tracer -- siege-warm 0.5 website/static/traces/siege-warm-b05.json
//!   cargo run -p scene-tracer -- hal-carla 0.5 website/static/traces/hal-carla-b05.json

mod scenes;
mod trace;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("usage: scene-tracer <scene> <beta> <out-path>");
        std::process::exit(2);
    }
    let beta: f64 = args[2].parse().expect("beta must be a float");
    let trace = match args[1].as_str() {
        "east-wall" => scenes::east_wall::trace(beta),
        // siege-cold / siege-warm added in Task 2; hal-carla added in Task 5.
        other => {
            eprintln!("unknown scene: {}", other);
            std::process::exit(2);
        }
    };
    let json = serde_json::to_string_pretty(&trace).unwrap();
    fs::write(&args[3], json).expect("write failed");
    println!("wrote {}", args[3]);
}
