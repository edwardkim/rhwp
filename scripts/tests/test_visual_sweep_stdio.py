"""Rust/Node의 UTF-8 출력은 호스트 기본 인코딩과 무관하게 보존한다."""
import sys
import tempfile
import unittest
from pathlib import Path

from scripts import visual_sweep as sweep


class Utf8ProcessOutputTests(unittest.TestCase):
    def test_utf8_child_output_and_log_preserve_non_ascii(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            log = root / 'capture.log'
            # CLI JSON·Node 출력의 실제 wire 계약을 독립 자식으로 재현한다.
            program = (
                "import sys; "
                "sys.stdout.buffer.write('한글 경로 Ω 🚀\\n'.encode('utf-8')); "
                "sys.stderr.buffer.write('글꼴 진단 Ω 🚀\\n'.encode('utf-8'))"
            )
            result = sweep.run([sys.executable, '-c', program], cwd=root,
                               log_path=log, verbose=False)
            self.assertEqual(result.stdout, '한글 경로 Ω 🚀\n')
            self.assertEqual(result.stderr, '글꼴 진단 Ω 🚀\n')
            self.assertEqual(log.read_text(encoding='utf-8'), result.stdout + result.stderr)
