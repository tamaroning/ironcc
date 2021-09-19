mod version;
mod lexer;
mod node;

use std::env;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        version::show_version();
        version::show_usage();
    } else {
        let filepath = args[1].clone();
        // test
        lexer::run(filepath);

    }
}
