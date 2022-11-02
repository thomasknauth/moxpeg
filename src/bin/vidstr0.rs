use mpeg_ox::iso11172_stream;

use std::env;
use std::fs::OpenOptions;
use std::io::BufReader;

// This concatenates all video stream 0 packets from an mpeg stream.
//
// Likely Not very useful in general, but it makes development of the
// mpeg decoder slighly simpler since we move the logic to parse
// system layer elements here. The decoder can just focus on
// interpreting the video layer.
fn main() {

    let args: Vec<String> = env::args().collect();

    let mut file = "/Users/thomas/code/mpeg/bjork-v2-short-2.mpg";
    let mut outfile = "out";

    if args.len() >= 2 {
        file = &args[1];
    }

    if args.len() >= 3 {
        outfile = &args[2];
    }

    let mut f = OpenOptions::new()
        .read(true)
        .open(file).expect("Unable to open file");
    let mut reader = BufReader::new(f);

    let mut fout = OpenOptions::new()
        .write(true)
        .create(true)
        .open(outfile).expect("Unable to open file");

    iso11172_stream(&mut reader, &mut fout);
}
