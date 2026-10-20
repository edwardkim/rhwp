"""배치 캡처의 쪽 소유와 중단 시 checkpoint 보존 계약."""

import io
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

from scripts.visual_sweep_raster import rasterized_sources


class RasterProtocolTests(unittest.TestCase):
    def test_wrong_second_page_stops_worker_and_preserves_first_checkpoint(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            sources = [(page, root / f'{page}.svg', root / 'tree', root / 'pdf') for page in (1, 2)]
            png_dir = root / 'rhwp_png'
            png_dir.mkdir()
            first_png = png_dir / 'rhwp_001.png'
            first_png.write_bytes(b'completed capture')
            first = {'input': str(sources[0][1]), 'output': str(first_png), 'zoom': 1}
            wrong = {'input': str(sources[0][1]), 'output': str(first_png), 'zoom': 1}
            worker = Mock()
            worker.stdout = io.StringIO('\n'.join(map(json.dumps, [first, wrong])) + '\n')
            worker.poll.return_value = None
            worker.wait.return_value = 143
            checkpoint = root / 'first-checkpoint.json'
            with patch('scripts.visual_sweep_raster.subprocess.Popen', return_value=worker):
                captured = rasterized_sources(root, sources, png_dir, 1, 'webfont', Mock(),
                                              Mock(), lambda path, data: path.write_text(json.dumps(data)))
                self.assertEqual(next(captured), sources[0])
                checkpoint.write_text('first page verified')
                with self.assertRaisesRegex(SystemExit, '쪽 순서'):
                    next(captured)
            self.assertEqual(checkpoint.read_text(), 'first page verified')
            worker.stdin.write.assert_called_once_with('stop\n')
            worker.terminate.assert_not_called()
            self.assertTrue(worker.stdout.closed)

    def test_consumer_abort_terminates_only_its_worker(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            sources = [(page, root / f'{page}.svg', root / 'tree', root / 'pdf') for page in (1, 2)]
            png_dir = root / 'rhwp_png'
            png_dir.mkdir()
            png = png_dir / 'rhwp_001.png'
            png.write_bytes(b'completed capture')
            worker = Mock()
            worker.stdout = io.StringIO(json.dumps({'input': str(sources[0][1]),
                                                  'output': str(png), 'zoom': 1}) + '\n')
            worker.poll.return_value = None
            worker.wait.return_value = 143
            with patch('scripts.visual_sweep_raster.subprocess.Popen', return_value=worker):
                captured = rasterized_sources(root, sources, png_dir, 1, 'webfont', Mock(),
                                              Mock(), lambda path, data: path.write_text(json.dumps(data)))
                self.assertEqual(next(captured), sources[0])
                captured.close()
            worker.stdin.write.assert_called_once_with('stop\n')
            worker.terminate.assert_not_called()
            self.assertEqual(png.read_bytes(), b'completed capture')
            self.assertTrue(worker.stdout.closed)
