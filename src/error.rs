use std::process;

pub fn error(line: u32, msg: &str) {
    println!(" Error: line:{} {}", line, msg);
    process::exit(-1);
}