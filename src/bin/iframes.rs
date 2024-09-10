use mpeg_ox::{MpegDecoder, PersistFrames};

use gflags;
use std::env;
use std::io;

extern crate env_logger;

gflags::define! {
    -f, --file: &std::path::Path
}

gflags::define! {
    --stats = false
}

// Extract key frames from a video source.

fn main() -> io::Result<()> {
    env_logger::init();
    let _args = gflags::parse();

    if FILE.is_present() {
        let path = FILE.flag;
        let mut decoder = MpegDecoder::new();

        decoder.stats = STATS.is_present();
        decoder.parse_mpeg(path.to_str().unwrap(), &mut PersistFrames::new())?;
    } else {
        gflags::print_help_and_exit(0);
    }
    Ok(())
}
