"""Staged file persistence for a document owned by NativeWorker.

The event loop sends ``prepare_command``, passes its completion to
``receive_prepared``, and obtains content-loss acceptance when requested. Only
``publish`` runs on an I/O executor. Its completed future is handled back on the
event loop, which sends ``acknowledge_command`` and calls ``acknowledged``.

On rejection or failure the event loop sends ``cancel_command`` when present,
then calls ``cancelled`` after receiving cleanup's completion (even if the old
document no longer exists). NativeWorker and buffer release stay on their owner
thread. Closing/replacing a document must wait for publication or preserve this
operation until its original document/token cleanup has completed.
"""

from copy import deepcopy
from pathlib import Path
from threading import Lock

from native_file_store import (
    ContentLossNeedsAcceptance,
    FileStamp,
    MAX_BYTES,
    PublicationError,
    _save_format,
    publish_document,
)


def _integer(value, name, minimum=0):
    if isinstance(value, bool) or not isinstance(value, int) or value < minimum:
        raise ValueError(f"{name} must be an integer >= {minimum}")
    return value


class NativeWorkerFileStore:
    """One document's observed source and at most one pending save.

    Supply ``source_stamp`` from the same :func:`native_file_store.read_document`
    call that produced the bytes sent to ``NativeWorker.open_bytes``. Never stat
    the path later to attach a newer stamp to an older in-memory document.
    """

    def __init__(self, document_id, source_path=None, source_stamp=None):
        self._document_id = _integer(document_id, "documentId", 1)
        if (source_path is None) != (source_stamp is None):
            raise ValueError("source path and observed stamp must be supplied together")
        if source_stamp is not None and not isinstance(source_stamp, FileStamp):
            raise ValueError("source stamp must come from read_document")
        self.source_path = Path(source_path).absolute() if source_path is not None else None
        self.source_stamp = source_stamp
        self._active = None

    @property
    def document_id(self):
        return self._document_id

    @property
    def busy(self):
        return self._active is not None

    def begin_save(self, target, destination=None, *, destination_stamp=None):
        """Bind one save to an explicit document revision and destination.

        Saving the source checks its original observed stamp. Other destinations
        are exclusive new files unless the host explicitly supplies their own
        observed ``destination_stamp`` after obtaining overwrite acceptance.
        """
        if self.busy:
            raise RuntimeError("finish the pending save and token cleanup first")
        if _integer(target.get("documentId"), "documentId", 1) != self.document_id:
            raise ValueError("save target belongs to a different document")
        revision = _integer(target.get("revision"), "revision")
        path = Path(destination).absolute() if destination is not None else self.source_path
        if path is None:
            raise ValueError("a new document needs a save destination")
        file_format = _save_format(path)
        if destination_stamp is not None and not isinstance(destination_stamp, FileStamp):
            raise ValueError("destination stamp must come from read_document")
        if path == self.source_path:
            if destination_stamp is not None and destination_stamp != self.source_stamp:
                raise ValueError("the opened source must retain its original observed stamp")
            expected = self.source_stamp
        else:
            expected = destination_stamp
        operation = WorkerSave(self, revision, path, file_format, expected)
        self._active = operation
        return operation


class WorkerSave:
    """A prepared byte/report/token snapshot with explicit event-loop stages."""

    def __init__(self, store, revision, path, file_format, expected):
        self._store = store
        self._document_id = store.document_id
        self._revision = revision
        self._path = path
        self._format = file_format
        self._expected = expected
        self._token = None
        self._bytes = b""
        self._report = None
        self._caret = {}
        self._observed = None
        self._published = False
        self._stage = "preparing"
        self._lock = Lock()

    @property
    def document_id(self):
        return self._document_id

    @property
    def revision(self):
        return self._revision

    @property
    def path(self):
        return self._path

    @property
    def format(self):
        return self._format

    @property
    def prepare_command(self):
        if self._stage != "preparing":
            raise RuntimeError("this operation has already received its prepared bytes")
        return {"op": "prepareSave", "target": {"documentId": self.document_id,
                                                   "revision": self.revision},
                "format": self.format}

    @property
    def report(self):
        return deepcopy(self._report)

    @property
    def requires_loss_acceptance(self):
        return self._stage == "acceptance"

    def receive_prepared(self, response, copy_buffer):
        """Copy/release on NativeWorker's thread; keep this exact payload to save.

        A loss dialog may retain this operation, but accepting it never prepares
        another serialization. The report property is a defensive copy.
        """
        with self._lock:
            if self._stage != "preparing":
                raise RuntimeError("save preparation has already completed")
            self._stage = "receiving"
        try:
            # Keep a matching token available for cleanup even if copying or
            # subsequent validation fails. Never cancel another document's token.
            token = response.get("token")
            if (response.get("documentId") == self.document_id and isinstance(token, int)
                    and not isinstance(token, bool) and token > 0):
                self._token = token
            data = copy_buffer(response) if "buffer" in response else None
            if not response.get("ok") or response.get("kind") != "preparedSave":
                raise RuntimeError(response.get("error", "native save preparation failed"))
            if (_integer(response.get("documentId"), "documentId", 1) != self.document_id
                    or self._token is None):
                raise ValueError("prepared save belongs to a different document")
            if _integer(response.get("revision"), "prepared revision") != self.revision:
                raise ValueError("prepared save belongs to a different revision")
            if data is None or len(data) > MAX_BYTES:
                raise ValueError("prepared save has no bounded document bytes")
            if _integer(response.get("byteLength"), "byteLength") != len(data):
                raise ValueError("prepared save byte length mismatch")
            report = response.get("report")
            if not isinstance(report, dict):
                raise ValueError("prepared save has no content-loss report")
            losses = _integer(report.get("count"), "content-loss count")
            self._bytes = bytes(data)
            self._report = deepcopy(report)
            self._caret = {name: response[name] for name in
                           ("caretSaved", "caretPreservedFromSource", "caretPolicy")
                           if name in response}
            self._stage = "acceptance" if losses else "ready"
        except BaseException:
            self._stage = "failed"
            raise

    def accept_loss(self):
        with self._lock:
            if self._stage != "acceptance":
                raise RuntimeError("this save is not waiting for content-loss acceptance")
            self._stage = "ready"

    def publish(self):
        """Run only this method on an I/O executor; never call NativeWorker here."""
        with self._lock:
            if self._stage == "acceptance":
                raise ContentLossNeedsAcceptance(self.report)
            if self._stage != "ready":
                raise RuntimeError("save bytes are not ready for publication")
            self._stage = "publishing"
        try:
            observed = publish_document(self.path, self._bytes, self._expected)
        except BaseException as error:
            with self._lock:
                self._published = bool(getattr(error, "published", False))
                self._stage = "failed"
            raise
        with self._lock:
            self._observed = observed
            self._published = True
            self._stage = "published"
        return observed

    @property
    def acknowledge_command(self):
        if self._stage != "published":
            raise RuntimeError("publish and verify the file before acknowledging it")
        return {"op": "acknowledgeSave", "documentId": self.document_id, "token": self._token}

    def acknowledged(self, response):
        if self._stage != "published":
            raise RuntimeError("this save has no publication awaiting acknowledgement")
        try:
            if not response.get("ok") or response.get("kind") != "saveStatus":
                raise RuntimeError(response.get("error", "native save acknowledgement failed"))
            if response.get("documentId") != self.document_id:
                raise ValueError("save acknowledgement belongs to a different document")
            status = response.get("status", {})
            saved_revision = _integer(status.get("savedRevision"), "savedRevision")
            current_revision = _integer(status.get("revision"), "current revision")
            dirty = response.get("isDirty")
            if (saved_revision != self.revision or current_revision < saved_revision
                    or not isinstance(dirty, bool) or status.get("dirty") is not dirty
                    or dirty != (current_revision != saved_revision)):
                raise ValueError("save acknowledgement does not match the prepared checkpoint")
        except Exception as error:
            self._stage = "failed"
            raise self.failure(error) from error
        self._store.source_path, self._store.source_stamp = self.path, self._observed
        result = {"path": str(self.path), "documentId": self.document_id,
                  "revision": self.revision, "currentRevision": current_revision,
                  "isDirty": dirty, "report": self.report, **self._caret}
        self._finish("complete")
        return result

    def failure(self, error):
        """Describe a preserved publication when submission/acknowledgement fails.

        This does not release the store or token. The host still performs cleanup
        before calling ``cancelled``, including on a stale-document response.
        """
        if self._published and not isinstance(error, PublicationError):
            publication = PublicationError(self.path)
            publication.__cause__ = error
            return publication
        return error

    @property
    def cancel_command(self):
        if self._stage == "publishing":
            raise RuntimeError("receive the pending publication before cancelling its token")
        if self._token is None or self._stage in ("complete", "cancelled"):
            return None
        return {"op": "cancelSave", "documentId": self.document_id, "token": self._token}

    def cancelled(self):
        """Finish after token cleanup; preserve every already-published file."""
        with self._lock:
            if self._stage == "publishing":
                raise RuntimeError("receive the pending publication before cancelling")
            if self._stage in ("complete", "cancelled"):
                raise RuntimeError("save operation has already completed")
            self._finish("cancelled")

    def _finish(self, stage):
        self._stage = stage
        self._bytes = b""
        if self._store._active is self:
            self._store._active = None
