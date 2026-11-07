"""Native owner/TLS/queue contracts, independent of a Flutter SDK."""

import argparse
import ctypes
import json
from pathlib import Path
import threading
import time
import unittest


class Reply(ctypes.Structure):
    _fields_ = [('request_id', ctypes.c_uint64), ('operation', ctypes.c_int32), ('error', ctypes.c_int32),
                ('json', ctypes.c_void_p), ('release_json', ctypes.c_void_p), ('bytes', ctypes.c_void_p),
                ('byte_length', ctypes.c_size_t), ('charged_bytes', ctypes.c_size_t), ('next', ctypes.c_void_p)]


class OwnerContracts(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.api = ctypes.CDLL(str(ARGS.dispatcher.resolve()))
        cls.mock = ctypes.CDLL(str(ARGS.mock.resolve()))
        signatures = {
            'create': ([ctypes.c_char_p], ctypes.c_void_p),
            'submit': ([ctypes.c_void_p, ctypes.c_uint64, ctypes.c_int,
                        ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t,
                        ctypes.c_uint64, ctypes.c_size_t], ctypes.c_int),
            'poll': ([ctypes.c_void_p], ctypes.POINTER(Reply)),
            'close': ([ctypes.c_void_p], None),
            'finished': ([ctypes.c_void_p], ctypes.c_int),
            'destroy': ([ctypes.c_void_p], ctypes.c_int),
            'retained_bytes': ([ctypes.c_void_p], ctypes.c_size_t),
        }
        for name, (arguments, returns) in signatures.items():
            function = getattr(cls.api, 'rhwp_flutter_owner_' + name)
            function.argtypes, function.restype = arguments, returns
        cls.api.rhwp_flutter_reply_free.argtypes = [ctypes.c_void_p, ctypes.POINTER(Reply)]
        cls.api.rhwp_flutter_reply_free.restype = None
        cls.mock.mock_allow_finish.argtypes = [ctypes.c_int]
        cls.mock.mock_snapshot_poll.argtypes = [ctypes.c_int]
        cls.api.rhwp_flutter_reply_claim_snapshot.argtypes = [ctypes.c_void_p, ctypes.POINTER(Reply)]
        cls.api.rhwp_flutter_reply_claim_snapshot.restype = ctypes.c_uint64
        cls.mock.rhwp_presentation_request.argtypes = [ctypes.c_char_p, ctypes.c_size_t]
        cls.mock.rhwp_presentation_request.restype = ctypes.c_void_p
        cls.mock.rhwp_string_free.argtypes = [ctypes.c_void_p]

    def setUp(self):
        self.owner = self.api.rhwp_flutter_owner_create(str(ARGS.mock.resolve()).encode())
        self.assertTrue(self.owner)
        self.serial = 0
        startup = self.take()
        self.assertEqual(startup['id'], 0)
        self.assertTrue(startup['json']['worker'])
        self.mock.mock_allow_finish(1)
        self.mock.mock_snapshot_poll(0)

    def tearDown(self):
        self.mock.mock_allow_finish(1)
        if self.owner:
            self.api.rhwp_flutter_owner_close(self.owner)
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                self.drain()
                if self.api.rhwp_flutter_owner_finished(self.owner):
                    result = self.api.rhwp_flutter_owner_destroy(self.owner)
                    if result >= 0:
                        self.assertEqual(result, 0)
                        self.owner = None
                        break
                time.sleep(0.001)
            self.assertIsNone(self.owner, 'native cleanup did not finish')
        self.assertEqual(self.mock.mock_wrong_thread_calls(), 0)

    def snapshot_reply(self):
        self.assertEqual(self.submit(1, {'op': 'workerCreate'}), 0)
        self.take()
        self.mock.mock_snapshot_poll(1)
        self.assertEqual(self.submit(3, handle=11), 0)
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            pointer = self.api.rhwp_flutter_owner_poll(self.owner)
            if pointer:
                return pointer
            time.sleep(0.001)
        self.fail('snapshot reply missing')

    def test_unclaimed_snapshot_reply_is_released_even_during_close(self):
        baseline = self.mock.mock_snapshot_releases()
        pointer = self.snapshot_reply()
        self.assertEqual(self.mock.mock_snapshot_live(), 1)
        self.api.rhwp_flutter_owner_close(self.owner)
        self.api.rhwp_flutter_reply_free(self.owner, pointer)
        self.assertEqual(self.mock.mock_snapshot_live(), 0)
        self.assertEqual(self.mock.mock_snapshot_releases(), baseline + 1)

    def test_claimed_snapshot_moves_to_consumer_without_double_release(self):
        baseline = self.mock.mock_snapshot_releases()
        pointer = self.snapshot_reply()
        self.assertEqual(self.api.rhwp_flutter_reply_claim_snapshot(self.owner, pointer), 77)
        self.assertEqual(self.api.rhwp_flutter_reply_claim_snapshot(self.owner, pointer), 0)
        self.api.rhwp_flutter_reply_free(self.owner, pointer)
        self.assertEqual(self.mock.mock_snapshot_releases(), baseline)
        payload = b'{"op":"snapshotRelease","snapshot":77}'
        reply = self.mock.rhwp_presentation_request(payload, len(payload))
        self.assertTrue(json.loads(ctypes.string_at(reply))['ok'])
        self.mock.rhwp_string_free(reply)
        self.assertEqual(self.mock.mock_snapshot_live(), 0)
        self.assertEqual(self.mock.mock_snapshot_releases(), baseline + 1)

    def submit(self, operation, request=None, handle=0, length=0):
        self.serial += 1
        encoded = json.dumps(request, separators=(',', ':')).encode() if request is not None else b''
        storage = ctypes.create_string_buffer(encoded) if encoded else None
        return self.api.rhwp_flutter_owner_submit(self.owner, self.serial, operation, storage,
                                                  len(encoded), None, 0, handle, length)

    def read(self, pointer):
        try:
            reply = pointer.contents
            return {'id': reply.request_id, 'operation': reply.operation, 'error': reply.error,
                    'json': json.loads(ctypes.string_at(reply.json)) if reply.json else None,
                    'release': json.loads(ctypes.string_at(reply.release_json)) if reply.release_json else None,
                    'bytes': ctypes.string_at(reply.bytes, reply.byte_length) if reply.bytes else None,
                    'length': reply.byte_length}
        finally:
            self.api.rhwp_flutter_reply_free(self.owner, pointer)

    def take(self):
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            pointer = self.api.rhwp_flutter_owner_poll(self.owner)
            if pointer:
                return self.read(pointer)
            time.sleep(0.001)
        self.fail('no owner reply')

    def drain(self):
        while True:
            pointer = self.api.rhwp_flutter_owner_poll(self.owner)
            if not pointer:
                return
            self.read(pointer)

    def create_frame(self):
        self.assertEqual(self.submit(2, {'op': 'workerCreate', 'config': {}}), 0)
        self.assertEqual(self.take()['json']['worker'], 11)
        self.assertEqual(self.submit(3, handle=11), 0)
        return self.take()['json']['completion']

    def test_calls_and_buffer_release_stay_on_native_owner_thread(self):
        frame = self.create_frame()
        before = self.mock.mock_buffer_releases()
        self.assertEqual(self.submit(5, handle=frame['buffer'], length=frame['byteLength']), 0)
        copied = self.take()
        self.assertTrue(copied['json']['ok'])
        self.assertTrue(copied['release']['ok'])
        self.assertEqual(copied['bytes'], bytes(range(64)))
        self.assertEqual(self.mock.mock_buffer_releases(), before + 1)
        self.assertEqual(self.api.rhwp_flutter_owner_retained_bytes(self.owner), 0)

    def test_copy_failure_still_releases_and_exposes_no_uninitialized_bytes(self):
        self.create_frame()
        before = self.mock.mock_buffer_releases()
        self.assertEqual(self.submit(5, handle=42, length=63), 0)
        failed = self.take()
        self.assertFalse(failed['json']['ok'])
        self.assertTrue(failed['release']['ok'])
        self.assertIsNone(failed['bytes'])
        self.assertEqual(failed['length'], 0)
        self.assertEqual(self.mock.mock_buffer_releases(), before + 1)

    def test_completed_unread_rpcs_keep_count_credit(self):
        for _ in range(64):
            self.assertEqual(self.submit(1, {'op': 'workerStatus', 'worker': 11}), 0)
        self.assertEqual(self.submit(1, {'op': 'workerStatus', 'worker': 11}), -3)
        for _ in range(64):
            self.take()
        self.assertEqual(self.api.rhwp_flutter_owner_retained_bytes(self.owner), 0)

    def test_worker_limit_rejects_before_an_untracked_worker_is_created(self):
        for _ in range(4):
            self.assertEqual(self.submit(2, {'op': 'workerCreate', 'config': {}}), 0)
            self.assertEqual(self.take()['error'], 0)
        self.assertEqual(self.submit(2, {'op': 'workerCreate', 'config': {}}), 0)
        reply = self.take()
        self.assertEqual(reply['error'], -3)
        self.assertIsNone(reply['json'])

    def test_buffer_reservations_reject_excess_before_allocation(self):
        for _ in range(4):
            self.assertEqual(self.submit(5, handle=42, length=16 * 1024 * 1024), 0)
        self.assertEqual(self.submit(5, handle=42, length=16 * 1024 * 1024), -4)
        for _ in range(4):
            self.take()
        self.assertEqual(self.api.rhwp_flutter_owner_retained_bytes(self.owner), 0)

    def test_close_is_nonblocking_and_releases_published_buffers_and_worker(self):
        self.create_frame()
        before_buffers, before_workers = self.mock.mock_buffer_releases(), self.mock.mock_worker_releases()
        self.mock.mock_allow_finish(0)
        self.api.rhwp_flutter_owner_close(self.owner)
        self.assertFalse(self.api.rhwp_flutter_owner_finished(self.owner))
        self.assertEqual(self.submit(1, {'op': 'workerStatus', 'worker': 11}), -2)
        self.mock.mock_allow_finish(1)
        deadline = time.monotonic() + 5
        while not self.api.rhwp_flutter_owner_finished(self.owner) and time.monotonic() < deadline:
            time.sleep(0.001)
        self.assertTrue(self.api.rhwp_flutter_owner_finished(self.owner))
        self.assertEqual(self.mock.mock_buffer_releases(), before_buffers + 1)
        self.assertEqual(self.mock.mock_worker_releases(), before_workers + 1)

    def test_concurrent_admission_and_close_leave_no_stranded_reservations(self):
        storage = ctypes.create_string_buffer(b'x' * (16 * 1024 * 1024))
        request = ctypes.create_string_buffer(b'{"op":"workerStatus","worker":11}')
        start = threading.Barrier(9)
        results = []
        def submit(index):
            start.wait()
            results.append(self.api.rhwp_flutter_owner_submit(self.owner, index + 1, 1,
                request, len(request.value), storage, 16 * 1024 * 1024, 0, 0))
        threads = [threading.Thread(target=submit, args=(index,)) for index in range(8)]
        for thread in threads:
            thread.start()
        start.wait()
        self.api.rhwp_flutter_owner_close(self.owner)
        for thread in threads:
            thread.join(timeout=5)
            self.assertFalse(thread.is_alive())
        self.assertTrue(all(result in (0, -2, -4) for result in results), results)
        for _ in range(results.count(0)):
            self.take()
        self.assertEqual(self.api.rhwp_flutter_owner_retained_bytes(self.owner), 0)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--dispatcher', type=Path, required=True)
    parser.add_argument('--mock', type=Path, required=True)
    ARGS = parser.parse_args()
    unittest.main(argv=[__file__], verbosity=2)
