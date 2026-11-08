"""독립 한컴 Print의 실제 채우기가 기준 래스터에서 사라지지 않게 보호한다."""

import hashlib
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

from PIL import Image

from scripts import visual_sweep as SWEEP


class HancomPrintRasterTests(unittest.TestCase):
    def test_hancom_print_gradient_is_preserved_in_reference_raster(self):
        if shutil.which(SWEEP.PDF_RASTERIZER) is None:
            self.skipTest("실제 Print 래스터 검증에는 현재 Python 실행 환경이 필요함")
        root = Path(__file__).resolve().parents[2]
        pdf = root / "pdf/issue7642-failed-regression-print-20261015/distribution_doc-2020.pdf"
        self.assertEqual(
            hashlib.sha256(pdf.read_bytes()).hexdigest(),
            "0299869bf040b01e07a19dde600852e44aec406257916b781eabdfbce6ed5aba",
            "같은 원본의 독립 한컴 Print를 대조",
        )
        with tempfile.TemporaryDirectory() as directory:
            prefix = Path(directory) / "print"
            command = SWEEP.pdf_raster_commands(pdf, 96, prefix, [3])[0]
            result = subprocess.run(command, capture_output=True, check=False)
            self.assertEqual(result.returncode, 0, result.stderr.decode(errors="replace"))
            pngs = list(prefix.parent.glob("print-*.png"))
            self.assertEqual(len(pngs), 1, "선택한 독립 Print 3쪽만 출력")
            with Image.open(pngs[0]) as image:
                rgb = image.convert("RGB")
                # 특정 좌표나 픽셀 수를 고정하지 않고 실제 파랑 채우기의 보존을 검사한다.
                pixels = rgb.tobytes()
                blue = any(
                    b - r > 5 and b - g > 2
                    for r, g, b in zip(pixels[0::3], pixels[1::3], pixels[2::3])
                )
            self.assertTrue(blue, "한컴 Print의 파랑 그라데이션을 기준 래스터에서 지우면 안 됨")


if __name__ == "__main__":
    unittest.main()
