#!/usr/bin/env python3
"""Actual Native worker search and explicit range replacement."""

import argparse
from io import BytesIO
from pathlib import Path
import sys
import xml.etree.ElementTree as ElementTree
from zipfile import ZipFile

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
        all_edit = accepted(worker, {**command, "op": "replaceAllMatches", "target": after_undo,
                                     "text": "신청일자"})["result"]
        assert all_edit["replacedCount"] == 2 and all_edit["skippedReadOnly"] == 0, all_edit
        assert all_edit["outcome"]["revision"] == after_undo["revision"] + 1, all_edit
        current = {**target, "revision": all_edit["outcome"]["revision"]}
        updated = accepted(worker, {**command, "target": current, "request": new_request})["result"]["matches"]
        assert updated == matches, updated
        undone = accepted(worker, {"op": "undo", "target": current})["outcome"]
        restored = {**current, "revision": undone["revision"]}
        assert accepted(worker, {**command, "target": restored})["result"]["matches"] == matches


def native_replace_all_contract(library):
    """Changing two Unicode-mapped matches is one revision and one Undo."""
    with NativeWorker(library) as worker:
        assert worker.host.call("capabilities")["replaceAllMatches"]
        initial = accepted(worker, {"op": "new"})["info"]
        target = {key: initial[key] for key in ("documentId", "revision")}
        start = {"section": 0, "paragraph": 0, "scalarOffset": 0, "cellPath": []}
        accepted(worker, {"op": "beginEdit", "target": target,
                          "selection": {"anchor": start, "focus": start}})
        setup = accepted(worker, {"op": "replace", "target": target, "text": "İ İ 😀"})["outcome"]
        target["revision"] = setup["revision"]
        command = {
            "op": "replaceAllMatches",
            "target": target,
            "request": {"query": "i", "caseSensitive": False, "maxMatches": 2},
            "text": "한글",
        }
        original = accepted(worker, {"op": "info"})["info"]
        precise = {**command, "request": {**command["request"], "caseSensitive": True}}
        no_hit = accepted(worker, precise)["result"]
        assert no_hit["replacedCount"] == 0 and not no_hit["outcome"]["historyChanged"], no_hit
        truncated = {**command, "request": {**command["request"], "maxMatches": 1}}
        failed = worker.wait(worker.submit(truncated))
        assert not failed["ok"] and failed["code"] == "limitExceeded", failed
        oversized = worker.wait(worker.submit({**command, "text": "x" * (128 * 1024 + 1)}))
        assert not oversized["ok"] and oversized["code"] == "limitExceeded", oversized
        bad_text = worker.wait(worker.submit({**command, "text": "한글\n문단"}))
        assert not bad_text["ok"] and bad_text["code"] == "unsupported", bad_text
        assert accepted(worker, {"op": "info"})["info"] == original

        receipt = accepted(worker, command)
        assert receipt["kind"] == "replaceAllMatches" and receipt["target"] == target, receipt
        changed = receipt["result"]
        outcome = changed["outcome"]
        assert changed["replacedCount"] == 2 and changed["skippedReadOnly"] == 0, changed
        assert outcome["historyChanged"] and outcome["revision"] == target["revision"] + 1, changed
        stale = worker.wait(worker.submit(command))
        assert not stale["ok"] and stale["code"] == "staleRevision", stale
        current = {**target, "revision": outcome["revision"]}
        assert accepted(worker, {"op": "imeContext", "target": current})["context"]["text"] == "한글 한글 😀"
        noop = accepted(worker, {**command, "target": current,
                                 "request": {**command["request"], "query": "한글"}})["result"]
        assert noop["replacedCount"] == 0 and noop["outcome"]["revision"] == current["revision"], noop
        assert not noop["outcome"]["historyChanged"], noop
        assert noop["outcome"]["selection"] == outcome["selection"], noop
        undone = accepted(worker, {"op": "undo", "target": current})["outcome"]
        assert undone["selection"] == setup["selection"], undone
        restored = {**target, "revision": undone["revision"]}
        assert accepted(worker, {"op": "imeContext", "target": restored})["context"]["text"] == "İ İ 😀"

        handle = worker.host.open_bytes()
        try:
            worker.host.request(handle, op="beginEdit", expectedRevision=0,
                                selection={"anchor": start, "focus": start})
            seed = worker.host.request(handle, op="replaceSelection", expectedRevision=0, text="İ İ")["outcome"]
            direct = worker.host.request(handle, op="replaceAllMatches",
                                         expectedRevision=seed["revision"],
                                         request=command["request"], text=command["text"])
            assert direct["kind"] == "replaceAllMatches" and direct["result"]["replacedCount"] == 2, direct
        finally:
            worker.host.call("close", handle)


def protected_fixture():
    """Read a public HWPX and add a synthetic header and protection in memory."""
    result = BytesIO()
    with ZipFile(FORM) as source, ZipFile(result, "w") as changed:
        for entry in source.infolist():
            data = source.read(entry)
            if entry.filename == "Contents/section0.xml":
                label = data.index("생년월일".encode())
                tag = data.rfind(b"<hp:tc ", 0, label)
                end = data.index(b">", tag)
                attrs = data[tag:end]
                assert b'protect="0"' in attrs
                data = data[:tag] + attrs.replace(b'protect="0"', b'protect="1"', 1) + data[end:]
                end_run = data.index(b"</hp:run>")
                header = ('<hp:ctrl><hp:header id="12345" applyPageType="BOTH">'
                          '<hp:subList><hp:p id="12346" paraPrIDRef="1" styleIDRef="0">'
                          '<hp:run charPrIDRef="1"><hp:t>생년월일</hp:t></hp:run>'
                          '</hp:p></hp:subList></hp:header></hp:ctrl>').encode()
                data = data[:end_run] + header + data[end_run:]
            changed.writestr(entry, data)
    return result.getvalue()


def native_protected_replace_all_contract(library):
    with NativeWorker(library) as worker:
        info = worker.wait(worker.open_bytes(protected_fixture()))["info"]
        target = {key: info[key] for key in ("documentId", "revision")}
        request = {"query": "생년월일", "caseSensitive": True, "maxMatches": 2}
        search = {"op": "searchText", "target": target, "request": request}
        found = accepted(worker, search)["result"]
        assert found["omittedContent"] and not found["truncated"], found
        assert [match["editable"] for match in found["matches"]] == [False, True], found
        protected, editable = (match["selection"] for match in found["matches"])
        assert len(editable["anchor"]["cellPath"]) == 2
        accepted(worker, {"op": "beginEdit", "target": target, "selection": protected})
        result = accepted(worker, {**search, "op": "replaceAllMatches", "text": "신청일자"})["result"]
        assert result["replacedCount"] == 1 and result["skippedReadOnly"] == 1, result
        assert result["omittedContent"] and result["outcome"]["historyChanged"], result
        current = {**target, "revision": result["outcome"]["revision"]}
        remaining = accepted(worker, {**search, "target": current})["result"]["matches"]
        assert remaining == [found["matches"][0]], remaining
        prepared = accepted(worker, {"op": "prepareSave", "target": current, "format": "hwpx"})
        asset = worker.copy_buffer(prepared)
        with ZipFile(BytesIO(asset)) as saved:
            section = ElementTree.fromstring(saved.read("Contents/section0.xml"))
            headers = [node for node in section.iter() if node.tag.endswith("}header")]
            assert any("생년월일" in "".join(node.itertext()) for node in headers)
        accepted(worker, {"op": "cancelSave", "documentId": target["documentId"],
                          "token": prepared["token"]})
        undo = accepted(worker, {"op": "undo", "target": current})["outcome"]
        assert undo["selection"] == protected and undo["revision"] == current["revision"] + 1, undo
        restored = {**target, "revision": undo["revision"]}
        assert accepted(worker, {**search, "target": restored})["result"] == found


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("library", type=Path)
    library = parser.parse_args().library
    native_search_contract(library)
    native_replace_all_contract(library)
    native_protected_replace_all_contract(library)
