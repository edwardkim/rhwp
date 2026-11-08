#!/usr/bin/env python3
"""Disabled-build ABI contracts. Pass a freshly built non-Metal Native dylib/SO."""

import argparse
from concurrent.futures import ThreadPoolExecutor
import ctypes
import json
from pathlib import Path
import unittest


class DisabledMetalAbi(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.library = ctypes.CDLL(str(ARGS.library.resolve()))
        cls.library.rhwp_string_free.argtypes = [ctypes.c_void_p]
        cls.library.rhwp_string_free.restype = None
        signatures = {
            'capabilities': [],
            'presenter_create': [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_uint64,
                                 ctypes.c_void_p, ctypes.c_size_t],
            'presenter_render': [ctypes.c_uint64, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_size_t],
            'presenter_request': [ctypes.c_uint64, ctypes.c_void_p, ctypes.c_size_t],
            'frame_check': [ctypes.c_void_p, ctypes.c_size_t],
        }
        for name, arguments in signatures.items():
            function = getattr(cls.library, 'rhwp_metal_' + name)
            function.argtypes = arguments
            function.restype = ctypes.c_void_p

    @classmethod
    def response(cls, operation, *arguments):
        pointer = getattr(cls.library, 'rhwp_metal_' + operation)(*arguments)
        if not pointer:
            raise AssertionError('ABI returned a null response')
        try:
            return json.loads(ctypes.string_at(pointer))
        finally:
            cls.library.rhwp_string_free(pointer)

    def test_disabled_capability_is_explicit(self):
        result = self.response('capabilities')
        self.assertIs(result['ok'], True)
        self.assertEqual(result['abiVersion'], 1)
        self.assertIs(result['available'], False, 'use a build without macOS Metal')
        self.assertIs(result['consumerGpuCompletion'], False)
        self.assertEqual(result['wholeFrameReadbacks'], 0)

    def test_disabled_calls_do_not_dereference_typed_pointers_or_byte_spans(self):
        maximum = ctypes.c_size_t(-1).value
        operations = [
            ('presenter_create', (1, 2, 0, 3, maximum)),
            ('presenter_render', (0, 1, 2, maximum)),
            ('presenter_request', (0, 1, maximum)),
            ('frame_check', (1, maximum)),
        ]
        for operation, arguments in operations:
            with self.subTest(operation=operation):
                result = self.response(operation, *arguments)
                self.assertIs(result['ok'], False)
                self.assertEqual(result['code'], 'featureDisabled')

    def test_feature_probe_is_cross_thread_and_its_strings_can_be_freed(self):
        with ThreadPoolExecutor(max_workers=4) as executor:
            results = list(executor.map(lambda _: self.response('capabilities'), range(16)))
        self.assertTrue(all(result['ok'] and not result['available'] for result in results))


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--library', type=Path, required=True)
    ARGS = parser.parse_args()
    unittest.main(argv=[__file__], verbosity=2)
