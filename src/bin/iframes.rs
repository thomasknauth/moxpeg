use mpeg_ox::parse_mpeg;

use std::io;
use std::env;
use std::fs::OpenOptions;
use std::io::BufReader;

extern crate env_logger;

// Extract key frames from a video source.

fn main() -> io::Result<()> {

    env_logger::init();

    let args: Vec<String> = env::args().collect();

    if args.len() >= 2 {
        parse_mpeg(&args[1], &mut PersistFrames::new())?;
    } else {
        println!("Usage: ./binary <mpeg video file name>");
    }
    Ok(())
}
