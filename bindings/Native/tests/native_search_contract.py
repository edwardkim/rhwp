#!/usr/bin/env python3
"""Actual Native worker search and explicit range replacement."""

import argparse
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "examples"))
from native_worker import NativeWorker


FORM = (
    Path(__file__).resolve().parents[3]
    / "samples/issue5599_oracle/3191107_leave_request_form.hwpx"
)


def accepted(worker, command):
    result = worker.wait(worker.submit(command))
    assert result["ok"], result
    return result


def verify_save_round_trips(library, worker, target, search_expectations):
    initial_info = accepted(worker, {"op": "info"})["info"]
    for file_format, signature in (("hwp", bytes.fromhex("d0cf11e0a1b11ae1")),
                                   ("hwpx", b"PK")):
        def export(host, current):
            prepared = accepted(host, {"op": "prepareSave", "target": current,
                                       "format": file_format})
            data = host.copy_buffer(prepared)
            assert data.startswith(signature) and prepared["report"]["count"] == 0, prepared
            assert prepared["revision"] == current["revision"], prepared
            # The round trip uses bytes in memory and leaves the original
            # editor's saved checkpoint and live selection as they were.
            accepted(host, {"op": "cancelSave", "documentId": current["documentId"],
                            "token": prepared["token"]})
            return data

        data = export(worker, target)
        with NativeWorker(library) as verifier:
            for pass_number in range(2):
                reopened = verifier.wait(verifier.open_bytes(data))
                assert reopened["ok"] and not reopened["info"]["isDirty"], reopened
                current = {key: reopened["info"][key] for key in target}
                for request, expected in search_expectations:
                    result = accepted(verifier, {"op": "searchText", "target": current,
                                                 "request": request})["result"]
                    assert not result["truncated"] and result["matches"] == [expected], result
                if pass_number == 0:
                    data = export(verifier, current)
                    accepted(verifier, {"op": "closeDocument", "target": current})
    assert accepted(worker, {"op": "info"})["info"] == initial_info


def native_search_contract(library):
    request = {"query": "생년월일", "caseSensitive": True, "maxMatches": 2}
    with NativeWorker(library) as worker:
        opened = worker.wait(worker.open_bytes(FORM.read_bytes()))
        assert opened["ok"], opened
        initial = opened["info"]
        target = {"documentId": initial["documentId"], "revision": initial["revision"]}
        command = {"op": "searchText", "target": target, "request": request}

        result = accepted(worker, command)
        assert result["kind"] == "searchText" and result["target"] == target
        matches = result["result"]["matches"]
        assert len(matches) == 2 and not result["result"]["truncated"]
        try_request = {**command, "request": {**request, "maxMatches": 1}}
        limited = accepted(worker, try_request)["result"]
        assert limited["truncated"] and limited["matches"] == matches[:1]

        first, second = (match["selection"] for match in matches)
        assert all(match["editable"] for match in matches)
        assert len(first["anchor"]["cellPath"]) == 2
        assert len(second["anchor"]["cellPath"]) == 2
        assert first["anchor"]["cellPath"] != second["anchor"]["cellPath"]
        assert first["anchor"]["cellPath"][0]["paragraph"] == 1  # applicant table
        assert second["anchor"]["cellPath"][0]["paragraph"] == 4  # child table
        assert accepted(worker, {"op": "info"})["info"] == initial

        # Leave the live selection on the second hit. The first hit is the
        # target of the atomic request even with an unchanged live revision.
        accepted(worker, {"op": "beginEdit", "target": target, "selection": second})
        replacement = {
            "op": "replaceRange",
            "target": target,
            "selection": first,
            "text": "신청일자",
        }
        changed = accepted(worker, replacement)["outcome"]
        assert changed["documentChanged"], changed
        stale = worker.wait(worker.submit(replacement))
        assert not stale["ok"] and stale["code"] == "staleRevision", stale

        current = {**target, "revision": changed["revision"]}
        remaining = accepted(worker, {**command, "target": current})["result"]["matches"]
        assert len(remaining) == 1 and remaining[0]["selection"] == second
        new_request = {**request, "query": replacement["text"]}
        verify_save_round_trips(library, worker, current,
                                ((request, matches[1]), (new_request, matches[0])))
        undone = accepted(worker, {"op": "undo", "target": current})["outcome"]
        assert undone["selection"] == second, undone
        after_undo = {**target, "revision": undone["revision"]}
        again = accepted(worker, {**command, "target": after_undo})["result"]["matches"]
        assert again == matches


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("library", type=Path)
    native_search_contract(parser.parse_args().library)
