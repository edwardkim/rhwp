"""Original grayscale geometry/resolution control; MIT OR Apache-2.0.

The upstream external sample_gray.jp2 is absent from this source distribution.
This synthetic control preserves the 816x1056 DeviceGray and nonuniform-sample
premises of the existing decoder contracts. It does not recreate the lost scan
or serve as visual-fidelity evidence. Requires Pillow with OpenJPEG support.
"""
from pathlib import Path
from PIL import Image
width, height = 816, 1056
samples = bytes((3*x + 5*y) % 256 for y in range(height) for x in range(width))
Image.frombytes("L", (width, height), samples).save(
    Path(__file__).with_name("gray-816x1056.jp2"), format="JPEG2000",
    irreversible=False, num_resolutions=6,
)
