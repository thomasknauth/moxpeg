#!/usr/bin/bash

# Fetch a bunch of mpeg files for testing.

BASE='https://filesamples.com/samples/video/mpeg'
FNS="sample_960x400_ocean_with_audio.mpeg \
sample_640x360.mpeg sample_960x540.mpeg \
sample_1280x720.mpeg \
sample_1920x1080.mpeg \
sample_2560x1440.mpeg \
sample_3840x2160.mpeg"

for fn in $FNS; do
    wget $BASE/$fn
done

wget https://phoboslab.org/files/bjork-all-is-full-of-love.mpg
