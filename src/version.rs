const VERSION_STR: &'static str = env!("CARGO_PKG_VERSION");

pub fn show_version() {
    println!("ironcc version {}", VERSION_STR);
}

pub fn show_usage() {
    println!("Usage: ironcc [options] <filepath>");
}
