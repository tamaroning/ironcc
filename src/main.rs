mod version;
mod lexer;

use std::env;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        version::show_version();
        version::show_usage();
    } else {
        let input_file_path = args[1].clone();
        let mut input_file = File::open(input_file_path.clone()).expect("File not found");

        let mut content = String::new();
        input_file.read_to_string(&mut content).expect("Couldn't open the file");

        let mut lexer = lexer::Lexer::new(input_file_path.clone(), content.as_str());

        // test
        println!("{:?}", lexer.read_token());
        println!("{:?}", lexer.read_token());
        println!("{:?}", lexer.read_token());
        println!("{:?}", lexer.read_token());
        println!("{:?}", lexer.read_token());
        println!("{:?}", lexer.read_token());
        println!("{:?}", lexer.read_token());
        println!("{:?}", lexer.read_token());
        println!("{:?}", lexer.read_token());
        println!("{:?}", lexer.read_token());
        println!("{:?}", lexer.read_token());

    }
}
