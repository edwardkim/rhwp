# JPEG allocation contract inputs

Synthetic 19 x 17 pixel images, generated with Pillow 10.2.0 / libjpeg 8.0, quality 95.
For pixel index i and channel c: (i * 17 + c * 67) mod 256; Gray uses c=0.
RGB uses subsampling 0/1/2, Gray and CMYK use 0. Each has baseline (0) and
progressive (1) coding. File name: MODE-PROGRESSIVE-SUBSAMPLING.jpg.
These exercise codec allocation and preservation, not Hancom layout goldens.
Expected decoded pixels come from the unchanged legacy consumer, not the patch.

Reproduction (Pillow version above; output can differ with another encoder):

```python
from PIL import Image
for mode in ["RGB", "L", "CMYK"]:
    for progressive in [False, True]:
        for subsampling in ([0, 1, 2] if mode == "RGB" else [0]):
            im = Image.new(mode, (19, 17))
            im.putdata([
                tuple((i * 17 + c * 67) % 256 for c in range(len(mode)))
                if mode != "L" else i * 17 % 256
                for i in range(19 * 17)
            ])
            im.save(f"{mode}-{int(progressive)}-{subsampling}.jpg",
                    quality=95, progressive=progressive, subsampling=subsampling)
```

CMYK allocation controls additionally include `YCCK-large.jpg` and
`CMYK-large-progressive.jpg` (257 x 129). Reproduce with libjpeg 8.0:
`cc make_cmyk_scans.c -ljpeg -o make_cmyk_scans`, then modes 6 and 7 respectively.
The larger width exercises the upstream Native worker selection threshold.
`CMYK-lossless-8.jpg` is a synthetic 19 x 17 SOF3 four-component JPEG: precision 8,
1x1 sampling, one category-0 DC Huffman code, predictor 1 and point transform 0.
All 19*17*4 entropy symbols are zero (pad final byte with one bits); decoded
samples are the independent midpoint 128, before the PDF consumer color transform.
