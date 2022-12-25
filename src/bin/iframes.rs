use mpeg_ox::parse_mpeg;

use std::env;
use std::fs::OpenOptions;
use std::io::BufReader;

extern crate env_logger;

fn main() {

    env_logger::init();

    let args: Vec<String> = env::args().collect();

    let mut file = "/Users/thomas/code/mpeg/bjork-v2-short-2.mpg";

    if args.len() >= 2 {
        file = &args[1];
    }

    parse_mpeg(&file);
}
