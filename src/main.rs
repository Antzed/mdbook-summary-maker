mod summary_maker;

use summary_maker::generate_summary;

use std::path::PathBuf;
use mdbook::config::Config;

fn main() {
    let root = PathBuf::from("./");
    let config = Config::default();
    eprintln!("running summary maker");
    generate_summary(&root, &config).unwrap();
}
