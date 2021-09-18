mod version;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        version::show_version();
    } else {
        println!("Hello, Rust Kaleidoscope!");
    }
}
