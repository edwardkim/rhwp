"""독립 검사기가 손상된 전송을 승인하지 않는지 확인한다."""

import base64
import importlib.util
import io
import unittest
from pathlib import Path

from PIL import Image

SPEC = importlib.util.spec_from_file_location("verify_kitty_output", Path(__file__).parents[1] / "verify-kitty-output.py")
VERIFY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VERIFY)


class KittyDecoderTest(unittest.TestCase):
    def setUp(self):
        png = io.BytesIO()
        Image.new("RGBA", (2, 1), (3, 17, 129, 255)).save(png, format="PNG")
        self.png = png.getvalue()
        self.payload = base64.b64encode(self.png)
        self.stream = b"\x1b_Ga=T,f=100,t=d,q=2,m=0;" + self.payload + b"\x1b\\\r\n"

    def test_valid_png_and_short_nonfinal_chunk(self):
        split = b"\x1b_Ga=T,f=100,t=d,q=2,m=1;" + self.payload[:4] + b"\x1b\\"
        split += b"\x1b_Gm=0,q=2;" + self.payload[4:] + b"\x1b\\\r\n"
        for stream, count in [(self.stream, 1), (split, 2)]:
            png, image, chunks = VERIFY.decode_stream(stream)
            self.assertEqual(png, self.png)
            self.assertEqual(image.getpixel((1, 0)), (3, 17, 129, 255))
            self.assertEqual(chunks, count)

    def test_rejects_corrupt_framing_controls_and_payload(self):
        bad = [b"", self.stream[:-2], b"log\n" + self.stream,
               self.stream[:-2] + b"extra\r\n",
               self.stream.replace(b"m=0", b"m=1"),
               self.stream.replace(b"q=2", b"q=2,q=2"),
               self.stream.replace(b"q=2", b"q=1"),
               self.stream.replace(b"f=100", b"f=32"),
               self.stream.replace(self.payload, b"????"),
               self.stream.replace(self.payload, base64.b64encode(b"not PNG")),
               self.stream.replace(self.payload, b"a" * 4100)]
        for stream in bad:
            with self.subTest(stream=stream[:50]):
                with self.assertRaises(ValueError):
                    VERIFY.decode_stream(stream)


if __name__ == "__main__":
    unittest.main()
