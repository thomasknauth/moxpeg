use mpeg_ox::{MpegDecoder, PersistFrames};

use gflags;
use std::io;
use std::env;

extern crate env_logger;

gflags::define! {
    -f, --file: &std::path::Path
}

// Extract key frames from a video source.

fn main() -> io::Result<()> {

    env_logger::init();
    let _args = gflags::parse();

    if FILE.is_present() {
        let path = FILE.flag;
        let mut decoder = MpegDecoder::new();

        decoder.parse_mpeg(path.to_str().unwrap(), &mut PersistFrames::new())?;
    } else {
        gflags::print_help_and_exit(0);
    }
    Ok(())
}
