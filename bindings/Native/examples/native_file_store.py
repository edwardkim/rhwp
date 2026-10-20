"""Host-owned file persistence and recovery for the native session protocol."""

from dataclasses import asdict, dataclass
import hashlib
import json
import math
import os
from pathlib import Path
import stat
import struct
import tempfile
import time
import uuid


MAX_BYTES = 128 * 1024 * 1024
MAX_RECOVERY_HEADER = 64 * 1024
MAGIC = b"RHWP-R1\n"


class ExternalFileChanged(RuntimeError):
    pass


class PublicationError(OSError):
    """A complete file was published, but finalization/acknowledgement failed.

    Inspect the preserved destination before retrying. In particular, a failed
    directory flush does not acknowledge the save or adopt a new source stamp.
    """

    def __init__(self, path):
        self.published = True
        self.path = Path(path).absolute()
        super().__init__(
            f"complete file published at {self.path}; save finalization failed; "
            "destination preserved, inspect it before retrying"
        )


class ContentLossNeedsAcceptance(RuntimeError):
    def __init__(self, report):
        self.report = report
        super().__init__("accept the serializer content-loss report before saving")


@dataclass(frozen=True)
class FileStamp:
    digest: str
    size: int
    modified_ns: int
    device: int
    inode: int


def read_document(path):
    """Read one bounded regular file; reject symlinks and detectable mid-read edits."""
    path = Path(path)
    before = path.lstat()
    if not stat.S_ISREG(before.st_mode):
        raise ValueError("document must be a regular file, not a symlink")
    if before.st_size > MAX_BYTES:
        raise ValueError("document exceeds the byte limit")
    flags = os.O_RDONLY | getattr(os, "O_BINARY", 0) | getattr(os, "O_NOFOLLOW", 0)
    descriptor = os.open(path, flags)
    with os.fdopen(descriptor, "rb") as source:
        opened = os.fstat(source.fileno())
        if not stat.S_ISREG(opened.st_mode):
            raise ValueError("document must remain a regular file")
        data = source.read(MAX_BYTES + 1)
        after = os.fstat(source.fileno())
    signature = lambda value: (value.st_dev, value.st_ino, value.st_size, value.st_mtime_ns)
    if signature(before) != signature(opened) or signature(opened) != signature(after):
        raise ExternalFileChanged("document changed while it was read")
    if len(data) > MAX_BYTES or len(data) != after.st_size:
        raise ValueError("document exceeds the byte limit or changed during reading")
    return data, FileStamp(hashlib.sha256(data).hexdigest(), len(data), after.st_mtime_ns,
                           after.st_dev, after.st_ino)


def _check_expected(path, expected):
    try:
        _, actual = read_document(path)
    except FileNotFoundError:
        if expected is None:
            return
        raise ExternalFileChanged("the opened document was removed") from None
    if expected is None:
        raise FileExistsError(path)
    if actual != expected:
        raise ExternalFileChanged("the destination changed since it was opened")


def _sync_directory(directory):
    # Windows has no portable directory fsync in Python's standard library.
    if os.name == "nt":
        return
    descriptor = os.open(directory, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def _write_staged(path, data, expected):
    """Stage on the destination filesystem, then publish one complete file.

    Existing-file guards detect changes before staging and immediately before
    replace. They are not a filesystem compare-and-swap against arbitrary other
    writers; applications must serialize their own saves. After publication a
    directory-flush failure leaves the complete new file in place and propagates.
    """
    path = Path(path)
    _check_expected(path, expected)
    descriptor, temporary = tempfile.mkstemp(prefix=".rhwp-save-", dir=path.parent)
    temporary = Path(temporary)
    published = False
    try:
        try:
            with os.fdopen(descriptor, "wb") as output:
                output.write(data)
                output.flush()
                os.fsync(output.fileno())
            _check_expected(path, expected)
            if expected is None:
                # Link is exclusive: a new file appearing after the guard is preserved.
                os.link(temporary, path)
                published = True
                temporary.unlink()
            else:
                os.replace(temporary, path)
                published = True
            _sync_directory(path.parent)
        finally:
            temporary.unlink(missing_ok=True)
    except Exception as error:
        if published:
            raise PublicationError(path) from error
        raise


def _save_format(path):
    extension = Path(path).suffix.lower()
    if extension not in (".hwp", ".hwpx"):
        raise ValueError("save destination must end in .hwp or .hwpx")
    return extension[1:]


class NativeFileStore:
    """Same-thread adapter; the application owns its session and destination paths."""

    def __init__(self, host, handle, source_path=None, source_stamp=None):
        if (source_path is None) != (source_stamp is None):
            raise ValueError("source path and observed stamp must be supplied together")
        self.host = host
        self.handle = handle
        self.source_path = Path(source_path).absolute() if source_path is not None else None
        self.source_stamp = source_stamp
        self._autosave_revision = None
        self._autosave_time = None

    @classmethod
    def open(cls, host, path):
        data, stamp = read_document(path)
        return cls(host, host.open_bytes(data), path, stamp)

    def save(self, destination=None, *, accept_loss=False):
        path = Path(destination).absolute() if destination is not None else self.source_path
        if path is None:
            raise ValueError("a new document needs a save destination")
        file_format = _save_format(path)
        expected = self.source_stamp if path == self.source_path else None
        _check_expected(path, expected)
        prepared = self.host.request(self.handle, op="prepareSave", format=file_format)
        committed = False
        try:
            data = self.host.copy_buffer(prepared)
            if len(data) > MAX_BYTES:
                raise ValueError("serialized document exceeds the host file byte limit")
            report = prepared["report"]
            if report.get("count", 0) and not accept_loss:
                raise ContentLossNeedsAcceptance(report)
            _write_staged(path, data, expected)
            committed = True
            _, observed = read_document(path)
            if observed.digest != hashlib.sha256(data).hexdigest():
                raise ExternalFileChanged("destination changed after file publication")
            self.source_path, self.source_stamp = path, observed
            status = self.host.request(self.handle, op="acknowledgeSave", token=prepared["token"])
            return {"path": str(path), "revision": prepared["revision"],
                    "isDirty": status["isDirty"], "report": report,
                    "caretSaved": prepared["caretSaved"],
                    "caretPreservedFromSource": prepared["caretPreservedFromSource"]}
        except BaseException as error:
            try:
                self.host.request(self.handle, op="cancelSave", token=prepared["token"])
            except Exception as cancel_error:
                if hasattr(error, "add_note"):
                    error.add_note(f"cancelSave also failed: {cancel_error}")
            # Never remove/roll back a published destination after acknowledgement
            # or durability failure. The caller can inspect/reopen the actual file.
            if committed and isinstance(error, Exception) and not isinstance(error, PublicationError):
                raise PublicationError(path) from error
            raise

    def autosave(self, directory, *, now=None, interval=30.0):
        now = time.monotonic() if now is None else now
        if not math.isfinite(now) or not math.isfinite(interval) or interval <= 0:
            raise ValueError("autosave clock and interval must be finite; interval must be positive")
        info = self.host.request(self.handle, op="info")
        if not info["isDirty"] or info["revision"] == self._autosave_revision:
            return None
        if self._autosave_time is not None and now - self._autosave_time < interval:
            return None
        directory = Path(directory)
        directory.mkdir(mode=0o700, parents=True, exist_ok=True)
        prepared = self.host.request(self.handle, op="prepareSave", format="hwpx")
        try:
            data = self.host.copy_buffer(prepared)
            if len(data) > MAX_BYTES:
                raise ValueError("recovery document exceeds the byte limit")
            header = {"version": 1, "format": "hwpx", "revision": prepared["revision"],
                      "sourcePath": str(self.source_path) if self.source_path else None,
                      "sourceStamp": asdict(self.source_stamp) if self.source_stamp else None,
                      "digest": hashlib.sha256(data).hexdigest(), "byteLength": len(data),
                      "report": prepared["report"]}
            encoded = json.dumps(header, ensure_ascii=False).encode("utf-8")
            if len(encoded) > MAX_RECOVERY_HEADER:
                raise ValueError("recovery metadata exceeds the byte limit")
            path = directory / (uuid.uuid4().hex + ".rhwp-recovery")
            _write_staged(path, MAGIC + struct.pack("<I", len(encoded)) + encoded + data, None)
            self._autosave_revision, self._autosave_time = prepared["revision"], now
            return path
        finally:
            # Recovery bytes are not the user's acknowledged document save.
            self.host.request(self.handle, op="cancelSave", token=prepared["token"])


def read_recovery(path):
    """Explicitly inspect one recovery file; do not overwrite or reopen its source."""
    path = Path(path)
    if not stat.S_ISREG(path.lstat().st_mode):
        raise ValueError("recovery must be a regular file")
    flags = os.O_RDONLY | getattr(os, "O_BINARY", 0) | getattr(os, "O_NOFOLLOW", 0)
    with os.fdopen(os.open(path, flags), "rb") as source:
        if source.read(len(MAGIC)) != MAGIC:
            raise ValueError("invalid recovery signature")
        size_bytes = source.read(4)
        if len(size_bytes) != 4:
            raise ValueError("truncated recovery header")
        size, = struct.unpack("<I", size_bytes)
        if size > MAX_RECOVERY_HEADER:
            raise ValueError("recovery metadata exceeds the byte limit")
        header = json.loads(source.read(size).decode("utf-8"))
        data = source.read(MAX_BYTES + 1)
    if not isinstance(header, dict) or header.get("version") != 1 or header.get("format") != "hwpx":
        raise ValueError("unsupported recovery metadata")
    if len(data) > MAX_BYTES or len(data) != header.get("byteLength"):
        raise ValueError("recovery document length mismatch")
    if hashlib.sha256(data).hexdigest() != header.get("digest"):
        raise ValueError("recovery document checksum mismatch")
    return header, data
