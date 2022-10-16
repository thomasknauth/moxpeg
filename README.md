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