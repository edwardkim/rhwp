"""Maintainer-only, scoped Gym simulation; not a registered pack or Hancom oracle.

Run from the repository root:
python3 mydocs/tech/investigations/issue-3587/probes/gym-labnote.py \
  target/pr-review/release-test/rhwp output/3587/gym-stage24
Uses existing product APIs and Gym score_task/checks without modifying either.
An existing output directory is refused. Do not run with python -O (assertions).
"""
import hashlib
import json
import pathlib
import platform
import shutil
import subprocess
import sys
import time
import xml.etree.ElementTree as ET
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[5]
sys.path.insert(0, str(ROOT / "gym"))
from core.runner import score_task  # noqa: E402


def sha(file):
    return hashlib.sha256(pathlib.Path(file).read_bytes()).hexdigest()


def write(file, value):
    with pathlib.Path(file).open("x", encoding="utf-8") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def binding(key, paragraph, control, cell, end=0):
    return {"key": key, "target": {"kind": "textRange", "path": [
        {"kind": "paragraph", "index": paragraph},
        {"kind": "control", "index": control},
        {"kind": "cell", "index": cell},
        {"kind": "paragraph", "index": 0}], "start": 0, "end": end}}


def xml_state(file):
    # Ignore only regenerated identity and saved line-layout cache in structural comparison.
    def local(node):
        return node.tag.rsplit("}", 1)[-1]

    def tree(node):
        return [node.tag, {k: v for k, v in node.attrib.items() if k not in ("id", "instid")},
                node.text or "", [tree(c) for c in node if local(c) != "linesegarray"]]

    with zipfile.ZipFile(file) as archive:
        root = ET.fromstring(archive.read("Contents/section0.xml"))
    paragraphs = [tree(p) for p in root if local(p) == "p"]
    ids = [e.get("id") for e in root.iter() if local(e) == "tbl"]
    return {"paragraphs": paragraphs, "tableIds": ids,
            "pagePr": [tree(e) for e in root.iter() if local(e) == "pagePr"]}


def main():
    if not __debug__:
        raise RuntimeError("Assertions required; do not use python -O")
    binary = pathlib.Path(sys.argv[1]).resolve()
    output = pathlib.Path(sys.argv[2]).resolve()
    output.mkdir(parents=True, exist_ok=False)
    commands = []

    def cli(args, parse=True):
        started = time.monotonic()
        process = subprocess.run([str(binary), *map(str, args)], cwd=ROOT,
                                 capture_output=True, text=True, timeout=120)
        row = {"args": list(map(str, args)), "exit": process.returncode,
               "seconds": time.monotonic() - started,
               "stdout": process.stdout, "stderr": process.stderr}
        commands.append(row)
        write(output / f"command-{len(commands):03}.json", row)
        if process.returncode:
            raise RuntimeError(f"Command {len(commands)} failed: {args[0]} exit {process.returncode}")
        return json.loads(process.stdout) if parse else process.stdout

    sample = ROOT / "samples/rnote/labnote-001.hwp"
    source_sha = sha(sample)
    write(output / "identity.json", {
        "sourceHead": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "sourceTree": subprocess.check_output(["git", "rev-parse", "HEAD^{tree}"], cwd=ROOT, text=True).strip(),
        "probeSha256": sha(__file__), "binarySha256": sha(binary),
        "gymRunnerSha256": sha(ROOT / "gym/core/runner.py"),
        "gymChecksSha256": sha(ROOT / "gym/core/checks.py"),
        "sampleSha256": source_sha, "platform": platform.platform(), "python": sys.version,
        "version": cli(["--version"], False), "started": time.strftime("%Y-%m-%dT%H:%M:%S%z")})
    derived = output / "source-derived.hwpx"
    cli(["export-hwpx", sample, derived], False)
    records = [{"title": f"실험 {n}: 온도 관찰", "body": f"실험 {n} 기록: 온도 {20+n}도, 관찰 완료."}
               for n in range(1, 4)]
    write(output / "records.json", records)
    results = []
    for ext, source in (("hwp", sample), ("hwpx", derived)):
        case = output / ext
        case.mkdir()
        before_sha = sha(source)
        stage = case / "LN01"
        stage.mkdir()

        def apply(name, input_file, action):
            file = stage / f"{name}.{ext}"
            plan = {"planVersion": "1.0", "input": str(input_file), "output": str(file),
                    "preconditions": {"inputSha256": sha(input_file)}, "steps": [action]}
            write(stage / f"{name}.plan.json", plan)
            preview = cli(["run", "--plan-json", json.dumps({**plan, "dryRun": True}), "--json"])
            assert not file.exists(), "dry-run wrote output"
            applied = cli(["run", "--plan-json", json.dumps(plan), "--json"])
            assert preview["preview"][0]["operationResult"] == applied["steps"][0]["operationResult"]
            write(stage / f"{name}.journal.json", {"preview": preview, "applied": applied})
            return file

        # Fixture addresses, not new rules in the product or the Gym checker.
        seeded = apply("seeded", source, {"action": "fill_template", "request": {
            "scope": {"sectionIndex": 0, "start": 12, "end": 13},
            "bindings": [binding("title", 0, 0, 1), binding("body", 0, 1, 5)], "record": records[0]}})
        copied = apply("copied", seeded, {"action": "repeat_and_fill_paragraph_block", "request": {
            "block": {"sectionIndex": 0, "sourceStart": 12, "sourceEnd": 13, "insertBefore": 13, "count": 2},
            "bindings": [], "records": [{}, {}]}})
        bindings, record = [], {}
        for index, values in enumerate(records[1:]):
            for key, control, cell in (("title", 0, 1), ("body", 1, 5)):
                name = f"{key}{index}"
                bindings.append(binding(name, index, control, cell, len(records[0][key])))
                record[name] = values[key]
        final = apply("labnote-filled", copied, {"action": "fill_template", "request": {
            "scope": {"sectionIndex": 0, "start": 13, "end": 15}, "bindings": bindings, "record": record}})

        # The same fixed task is used for positive and all negative submissions.
        checks = [{"name": "15 tables", "op": "value_eq", "path": "tableCount", "value": 15,
                   "cmd": ["export-tables", f"{{file:labnote-filled.{ext}}}", "--json"]}]
        for index, values in enumerate(records):
            for key, offset, row, col in (("title", 0, 0, 1), ("body", 1, 5, 0)):
                checks.append({"name": f"record {index+1} {key}", "op": "cell_text_eq",
                    "path": "tables",
                    "table": 6 + index * 3 + offset, "row": row, "col": col, "value": values[key],
                    "cmd": ["export-tables", f"{{file:labnote-filled.{ext}}}", "--json"]})
        # Original introductory tables must remain intact, including empty cells and spans.
        before_tables = cli(["export-tables", source, "--json"])
        for index in range(6):
            checks.append({"name": f"unchanged introductory table {index}", "op": "value_eq",
                "path": f"tables[{index}]", "value": before_tables["tables"][index],
                "cmd": ["export-tables", f"{{file:labnote-filled.{ext}}}", "--json"]})
        task = {"id": "LN01", "tier": 3, "title": "Research-note fill, copy, individualized fill",
                "input": str(source), "instructions": "Fill three distinct experiment records; preserve surrounding content.",
                "submit": {"kind": "artifact", "files": [final.name]}, "checks": checks}
        write(case / "task.json", task)
        positive = score_task(task, str(case), str(binary))
        write(case / "positive.json", positive)
        assert positive["pass"], "Positive Gym checks failed; inspect positive.json"
        negatives = []
        for name, file in (("untouched", source), ("without-copy", seeded), ("without-final-fill", copied)):
            folder = case / name / "LN01"
            folder.mkdir(parents=True)
            shutil.copyfile(file, folder / final.name)
            score = score_task(task, str(folder.parent), str(binary))
            write(folder.parent / "score.json", score)
            assert not score["pass"], f"False pass: {name}"
            # Untouched / without-copy have only 9 tables: missing copied addresses
            # are expected semantic omissions, not a broken runner or missing binary.
            errors = [c for c in score["checks"] if c.get("kind")]
            assert all(name in ("untouched", "without-copy") and c["kind"] == "path-eval"
                       and "IndexError" in c.get("error", "") for c in errors), "Unexpected checker error"
            assert any(not c["ok"] and not c.get("kind") for c in score["checks"]), "Only tool errors rejected control"
            negatives.append({"name": name, "rejected": True,
                              "failedChecks": [c["name"] for c in score["checks"] if not c["ok"]]})

        # Saved output reopened through the public exporter; compare same-format roundtrips.
        before_xml, after_xml = case / "before.hwpx", case / "after.hwpx"
        cli(["export-hwpx", source, before_xml], False)
        cli(["export-hwpx", final, after_xml], False)
        before, after = xml_state(before_xml), xml_state(after_xml)
        write(case / "structure.json", {"before": before, "after": after})
        assert before["paragraphs"][:12] == after["paragraphs"][:12], "Leading paragraphs changed"
        assert before["paragraphs"][13:] == after["paragraphs"][15:], "Trailing paragraphs changed"
        assert before["pagePr"] == after["pagePr"], "Paper setup changed"
        ids = after["tableIds"]
        assert len(ids) == len(set(ids)) == 15 and None not in ids, "Table identity collision"
        assert sha(source) == before_sha and sha(sample) == source_sha, "Input modified"
        results.append({"format": ext, "positiveChecks": len(checks), "negativeControls": negatives,
                        "surroundingStructure": "pass", "pageSettings": "pass", "uniqueTableIds": 15,
                        "file": str(final), "sha256": sha(final)})
    write(output / "result.json", {"ok": True, "scope": "Scoped maintainer simulation, not full Gym or Hancom oracle",
                                   "results": results, "commands": len(commands)})
    print(json.dumps({"ok": True, "output": str(output), "formats": len(results)}, ensure_ascii=False))


if __name__ == "__main__":
    main()
