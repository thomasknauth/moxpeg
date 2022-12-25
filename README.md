# 2022-11-14

Able to correctly decode first key frames. One last bug was in the routine to write the ppm file. The calculation for the input offset was wrong, resulting in a corrupt output. I only discovered this after making sure that the decoded bytes used as input to the write_ppm function were identical to the decoded bytes from the reference implementation.

# 2022-11-13

Bug: Forgot to reset dc_predictor for new slice.

# 2022-10-27

https://www.ics.uci.edu/~pattis/common/handouts/macmingweclipse/allexperimental/mac-gdb-install.html

I am giving up on try to be able to use rust-gdb/lldb on MacOS. I'd rather just develop in a Linux environment.

There is a bug in my decoder where it incorrectly moves onto the next slice after processing the first slice it encounters.

Yet another instance where my decoder starts to deviate from the reference. This now happens shortly after the second slice. There is no video layer element boundary like last time though.

Did not rewind the stream when I should have. What tipped me off was the fact that the decoder somehow skipped an entire slice.

Another bug was a typo: what should have been % was a &.

Another bug: next_start_code function was skipping start codes.

loop {
  next_byte() != 0: continue
  next_byte() != 0: continue
  next_byte() != 1: continue
  return next_byte();
}

will skip the following start code ... 0x00 0x00 0x00 0x01 0xbe ...

Another bug: order of conditions is important here. Earlier, I had
conditions on line 2 and 5 switched, which, of course, made no sense.

01    // presentation time stamp (PTS)
02    if (data[idx] & 0b00110000) > 0 {
03        // presentation time stamp (PTS) and decoding time stamp (DTS)
04        idx += 10;
05    } else if (data[idx] & 0b00100000) > 0 {
06        idx += 5;
07    } else {
08        idx += 1;
09    }

# 2022-10-26

All the problems of parsing the stream came down to me completely being oblivious to the system layer aspect of mpeg. My decoder would happily read past the end of an mpeg packet and this would of course throw it off (and also diverge from what the other mpeg decoder was doing).

Some of the mpeg files I was using (e.g., big buck bunny) had indeed no system layer packets, but only video layer components. While others (e.g, bjork) had system layer components. I suppose this is at least in part because big buck bunny is video only, while, for example, bjork is video and audio and those need to be muxed appropriately.

... snip ...

Somehow I thought that stripping all the system layer packs and packets and only keeping their payload would be a grand idea. My decoder still chokes on the, supposedly, video layer only stream. My working theory was that my current decoder tries to interpret system layer data as part of the video layer. By removing the system layer, I was hoping to make my decoder work correctly w/o adding any system layer logic to it. However, after stripping the system layer fluff from the stream, the decoder still fails at exactly the same spot in the stream - the start of macroblock 167. Not sure what this tells me. When I looked at this earlier, I saw it failing at a pack/packet boundary. While pl_mpeg has some cleverness in its get_me_the_next_bits_from_the_stream() function to skip over packs/packets correctly, my decoder did not.

# 2022-10-24

For some reason, the pl_mpeg and my implementation diverge at some
point when parsing a stream.

pl_mpeg: coeff= 8 (0,8), 13 (0,-13), 5 (0,5), 7 (0,-7), 1 (0,1), 513 (2,1), 1 (0,1), 1 (0,1), 1 (0,1), 1 
mpeg_ox: coeff= 8 (0,8), 13 (0,-13), 5 (0,5), 7 (0,7), 0 (0,0), 17 (0,17), 5633 (22,-1), 1 (0,1), 1

not sure why.

# 2022-10-08

idct_23002_2 vs plm_video_idct

Need to figure out why idct_23002_2 and plm_video_idct produce
different results.

Output values of idct_23002_2 have a much larger magnitude than
plm_video_idct.

5pm

Made some progress on the corrupted output. pl_mpeg seems to have
trouble with certain files. The version of bjork-all-is-full-of-love I
have seems to cause trouble with pl_mpeg. I grabbed a another version
of the same video and it decodes without artifacts.