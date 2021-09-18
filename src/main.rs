mod version;

use std::env;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        version::show_version();
        version::show_usage();
    } else {
        let input_file_name = &args[1];
        let mut input_file = File::open(input_file_name).expect("File not found");

        let mut string = String::new();
        input_file.read_to_string(&mut string).expect("Couldn't open the file");

        println!("content: {}", string);

    }
}
