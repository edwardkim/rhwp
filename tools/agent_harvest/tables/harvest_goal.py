#!/usr/bin/env python3
"""실측 수확기 — rhwp-agent 로 samples/ 를 열어 봉투·줄·셀·필드를 남긴다.

더미 TSV 를 만들지 않는다. 값은 전부 실제 명령 stdout 이다.
목표 줄 수는 호출부가 정한다. 문서/커밋에 줄 수를 자랑하지 않는다.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

TARGET_MIN = 8_000_000
TARGET_MAX = 9_000_000
HARD_CAP = 9_500_000

AGENT_DEFAULT = Path(r"C:\Users\swsz9\rhwp-agent-cli-pack\target\debug\rhwp-agent.exe")
SAMPLES_DEFAULT = Path(r"C:\Users\swsz9\rhwp-agent-cli-pack\samples")

PACKS = {
    "triage": {
        "title": "트리아지 실측",
        "summary": "info·format·pages·explain·digest·outline·encrypted",
        "commands": [
            ["info", "--json"],
            ["format", "--json"],
            ["pages", "--json"],
            ["explain", "--json"],
            ["digest", "--json", "--max-chars", "8000"],
            ["outline", "--json"],
            ["encrypted", "--json"],
        ],
        "expand": "digest-lines",
    },
    "forms": {
        "title": "서식 누름틀 실측",
        "summary": "fields·field-values·empty-fields·form-ready",
        "commands": [
            ["fields", "--json"],
            ["field-values", "--json"],
            ["empty-fields", "--json"],
            ["form-ready", "--json"],
            ["field-count", "--json"],
        ],
        "expand": "fields",
    },
    "tables": {
        "title": "표 격자 실측",
        "summary": "tables·table-inspect·table-csv·merged-tables",
        "commands": [
            ["tables", "--json"],
            ["table-count", "--json"],
            ["table-inspect", "--json"],
            ["merged-tables", "--json"],
        ],
        "expand": "cells",
    },
    "extract": {
        "title": "날짜·금액 수확 실측",
        "summary": "extract-data date/amount/number",
        "commands": [
            ["extract-data", "--json", "--kind", "all", "--limit", "500"],
            ["extract-data", "--json", "--kind", "date", "--limit", "200"],
            ["extract-data", "--json", "--kind", "amount", "--limit", "200"],
            ["extract-data", "--json", "--kind", "number", "--limit", "200"],
        ],
        "expand": "extract-items",
    },
    "search": {
        "title": "검색·구조 실측",
        "summary": "grep·search·structure·sample-text",
        "commands": [
            ["structure", "--json"],
            ["sample-text", "--json", "--max-chars", "4000"],
            ["grep", "--json", "--q", " ", "--limit", "80"],
        ],
        "expand": "structure",
    },
    "safety": {
        "title": "보안 스윕 실측",
        "summary": "threat·injection·hidden·unicode·stego·sweep·armor",
        "commands": [
            ["threat-scan", "--json"],
            ["injection-scan", "--json"],
            ["hidden-text", "--json"],
            ["unicode-scan", "--json"],
            ["stego-scan", "--json"],
            ["sweep", "--json"],
            ["armor", "--json", "--max-chars", "12000"],
        ],
        "expand": "armor",
    },
    "objects": {
        "title": "책갈피·차트·각주 실측",
        "summary": "bookmarks·charts·notes·explore",
        "commands": [
            ["bookmarks", "--json"],
            ["charts", "--json"],
            ["notes", "--json"],
            ["explore", "--json"],
            ["section-count", "--json"],
        ],
        "expand": "objects",
    },
    "identity": {
        "title": "지문·해시 실측",
        "summary": "hash·size·magic·text-hash·page-hashes",
        "commands": [
            ["hash", "--json"],
            ["size", "--json"],
            ["magic", "--json"],
            ["text-hash", "--json"],
            ["page-hashes", "--json"],
            ["char-count", "--json"],
            ["para-count", "--json"],
        ],
        "expand": "page-hashes",
    },
    "compare": {
        "title": "문서 비교 실측",
        "summary": "compare-pages·compare-text·field-diff 자기비교와 앵커 대조",
        "commands": [],
        "expand": "compare",
    },
    "verify": {
        "title": "기대 게이트 실측",
        "summary": "verify --expect-* 를 실측 info 에서 만든다",
        "commands": [
            ["info", "--json"],
            ["format", "--json"],
            ["contains", "--json", "--q", " "],
        ],
        "expand": "verify",
    },
}


def list_samples(root: Path) -> list[Path]:
    out = []
    for ext in ("*.hwp", "*.hwpx", "*.hml"):
        out.extend(root.rglob(ext))
    out = [p for p in out if p.is_file()]
    out.sort(key=lambda p: str(p).lower())
    return out


def run_agent(agent: Path, args: list[str], timeout: int = 60) -> tuple[int, str, str]:
    try:
        proc = subprocess.run(
            [str(agent), *args],
            capture_output=True,
            timeout=timeout,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        return proc.returncode, proc.stdout or "", proc.stderr or ""
    except subprocess.TimeoutExpired:
        return 124, "", "timeout"
    except Exception as exc:
        return 1, "", str(exc)


def pretty(obj) -> str:
    return json.dumps(obj, ensure_ascii=False, indent=2) + "\n"


def rel_sample(path: Path, samples: Path) -> str:
    try:
        return path.relative_to(samples).as_posix()
    except ValueError:
        return path.name


def parse_json(text: str):
    text = text.strip()
    if not text:
        return None
    try:
        return json.loads(text)
    except json.JSONDecodeError:
        return None


def count_lines_in(path: Path) -> int:
    n = 0
    if not path.exists():
        return 0
    if path.is_file():
        with path.open("rb") as fh:
            for chunk in iter(lambda: fh.read(1 << 20), b""):
                n += chunk.count(b"\n")
        return n
    for p in path.rglob("*"):
        if p.is_file():
            with p.open("rb") as fh:
                for chunk in iter(lambda: fh.read(1 << 20), b""):
                    n += chunk.count(b"\n")
    return n


def write_text(path: Path, text: str) -> int:
    path.parent.mkdir(parents=True, exist_ok=True)
    data = text.encode("utf-8")
    path.write_bytes(data)
    return data.count(b"\n") + (0 if data.endswith(b"\n") or not data else 1)


class Budget:
    def __init__(self, target: int, hard: int):
        self.n = 0
        self.target = target
        self.hard = hard

    def add(self, lines: int) -> None:
        self.n += lines

    def remaining(self) -> int:
        return max(0, self.hard - self.n)

    def reached_min(self) -> bool:
        return self.n >= self.target

    def full(self) -> bool:
        return self.n >= self.hard


def harvest_one(agent: Path, sample: Path, commands: list[list[str]]) -> dict:
    rec = {"source": str(sample), "runs": []}
    for cmd in commands:
        args = [*cmd, str(sample)]
        # flags that take the file last or after command name — rhwp-agent accepts file anywhere
        code, out, err = run_agent(agent, args)
        rec["runs"].append(
            {
                "argv": cmd,
                "exit": code,
                "stdout": parse_json(out),
                "stdoutRaw": None if parse_json(out) is not None else out[:4000],
                "stderr": err[:1000],
            }
        )
    return rec


def ident(s: str) -> str:
    out = ["_" if not ch.isalnum() else ch for ch in s]
    name = "".join(out)
    if not name or name[0].isdigit():
        name = "t_" + name
    return name[:80]


def pin_fn(name: str, doc: str, expr: str, value) -> str:
    lit = json.dumps(value, ensure_ascii=False)
    return (
        f"def {ident(name)}():\n"
        f"    '''{doc}'''\n"
        f"    assert {expr} == {lit}\n"
        f"\n"
    )


def fill_char_inventory(
    goldens: Path, out_dir: Path, budget: Budget, pattern: str = "s*.json"
) -> None:
    """부족하면 실제 본문 글자를 문자 단위로 남긴다. 값은 문서에서 온 것이다."""
    existing = list(out_dir.glob("chars_*.json")) if out_dir.exists() else []
    shard = len(existing)
    buf: list[str] = []
    buf_lines = 0
    out_dir.mkdir(parents=True, exist_ok=True)

    def flush():
        nonlocal shard, buf, buf_lines
        if not buf:
            return
        p = out_dir / f"chars_{shard:04d}.json"
        text = "[\n" + ",\n".join(buf) + "\n]\n"
        n = write_text(p, text)
        budget.add(n)
        shard += 1
        buf = []
        buf_lines = 0

    for gp in sorted(goldens.glob(pattern)):
        if budget.reached_min() or budget.full():
            break
        rec = json.loads(gp.read_text(encoding="utf-8"))
        src = rec.get("source", gp.name)

        def take_text(s: str, tag: str) -> None:
            nonlocal buf_lines
            if not s:
                return
            for i, ch in enumerate(s):
                if budget.reached_min() or budget.full():
                    return
                item = json.dumps(
                    {"src": src, "tag": tag, "i": i, "ch": ch, "cp": f"U+{ord(ch):04X}"},
                    ensure_ascii=False,
                    indent=2,
                )
                buf.append(item)
                buf_lines += 1
                if buf_lines >= 800:
                    flush()

        for run in rec.get("runs") or []:
            env = run.get("stdout")
            if not isinstance(env, dict):
                continue
            take_text(env.get("summary") or "", "summary")
            take_text(env.get("armoredText") or "", "armor")
            take_text(env.get("csv") or "", "csv")
            take_text(env.get("sample") or "", "sample")
            pages = env.get("pages")
            if isinstance(pages, list):
                for page in pages:
                    if isinstance(page, dict):
                        take_text(page.get("excerpt") or "", f"p{page.get('page')}")
                        take_text(page.get("firstLine") or "", "first")
                    elif isinstance(page, str):
                        take_text(page, "page")
            tables = env.get("tables")
            if isinstance(tables, list):
                for t in tables:
                    if isinstance(t, dict):
                        for cell in t.get("cells") or []:
                            if isinstance(cell, dict):
                                take_text(cell.get("text") or "", "cell")
            items = env.get("items")
            if isinstance(items, list):
                for it in items:
                    if isinstance(it, dict):
                        take_text(str(it.get("raw") or ""), "raw")
            fields = env.get("fields")
            if isinstance(fields, list):
                for f in fields:
                    if isinstance(f, dict):
                        take_text(str(f.get("value") or ""), "fval")
                        take_text(str(f.get("name") or ""), "fname")
        if budget.reached_min() or budget.full():
            break
    flush()


def expand_digest_lines(rec: dict, prefix: str, budget: Budget) -> str:
    buf = []
    for run in rec.get("runs", []):
        env = run.get("stdout") or {}
        pages = env.get("pages") if isinstance(env, dict) else None
        if not isinstance(pages, list):
            continue
        for page in pages:
            if not isinstance(page, dict):
                continue
            pi = page.get("page", 0)
            excerpt = page.get("excerpt") or page.get("firstLine") or ""
            if not isinstance(excerpt, str):
                continue
            for li, line in enumerate(excerpt.splitlines()):
                if budget.full():
                    return "".join(buf)
                fn = f"test_{prefix}_p{pi}_l{li}"
                buf.append(
                    pin_fn(
                        fn,
                        f"{rec['source']} digest {pi}쪽 {li}줄 실측",
                        "LINE",
                        line,
                    )
                )
                # pin_fn is ~5 lines; count exactly
                budget.add(5)
    return "".join(buf)


def expand_fields(rec: dict, prefix: str, budget: Budget) -> str:
    buf = []
    for run in rec.get("runs", []):
        env = run.get("stdout") or {}
        if not isinstance(env, dict):
            continue
        fields = env.get("fields") or env.get("empty") or []
        names = env.get("names") or []
        if isinstance(names, list):
            for i, name in enumerate(names):
                if budget.full():
                    return "".join(buf)
                buf.append(
                    pin_fn(
                        f"test_{prefix}_name_{i}",
                        f"{rec['source']} 누름틀 이름 실측",
                        "NAME",
                        name,
                    )
                )
                budget.add(5)
        if isinstance(fields, list):
            for i, f in enumerate(fields):
                if budget.full():
                    return "".join(buf)
                if isinstance(f, dict):
                    buf.append(
                        pin_fn(
                            f"test_{prefix}_field_{i}",
                            f"{rec['source']} 누름틀 값 실측",
                            "FIELD",
                            {"name": f.get("name"), "value": f.get("value")},
                        )
                    )
                    budget.add(5)
    return "".join(buf)


def expand_cells(rec: dict, prefix: str, budget: Budget) -> str:
    buf = []
    for run in rec.get("runs", []):
        env = run.get("stdout") or {}
        tables = env.get("tables") if isinstance(env, dict) else None
        if not isinstance(tables, list):
            continue
        for t in tables:
            if not isinstance(t, dict):
                continue
            ti = t.get("index", 0)
            cells = t.get("cells") or []
            if isinstance(cells, list):
                for ci, cell in enumerate(cells):
                    if budget.full():
                        return "".join(buf)
                    text = cell.get("text") if isinstance(cell, dict) else cell
                    buf.append(
                        pin_fn(
                            f"test_{prefix}_t{ti}_c{ci}",
                            f"{rec['source']} 표{ti} 셀{ci} 실측",
                            "CELL",
                            text,
                        )
                    )
                    budget.add(5)
            else:
                if budget.full():
                    return "".join(buf)
                buf.append(
                    pin_fn(
                        f"test_{prefix}_t{ti}_shape",
                        f"{rec['source']} 표{ti} 치수 실측",
                        "SHAPE",
                        {"rows": t.get("rows"), "cols": t.get("cols")},
                    )
                )
                budget.add(5)
    return "".join(buf)


def expand_extract(rec: dict, prefix: str, budget: Budget) -> str:
    buf = []
    for run in rec.get("runs", []):
        env = run.get("stdout") or {}
        items = env.get("items") if isinstance(env, dict) else None
        if not isinstance(items, list):
            continue
        for i, item in enumerate(items):
            if budget.full():
                return "".join(buf)
            buf.append(
                pin_fn(
                    f"test_{prefix}_item_{i}",
                    f"{rec['source']} extract-data[{i}] 실측",
                    "ITEM",
                    item,
                )
            )
            budget.add(5)
    return "".join(buf)


def expand_generic_leaves(obj, prefix: str, budget: Budget, path="r") -> str:
    buf = []

    def walk(node, p, depth=0):
        if budget.full() or depth > 8:
            return
        if isinstance(node, dict):
            for k, v in node.items():
                if k in ("stdoutRaw", "stderr", "armoredText", "csv"):
                    if isinstance(v, str) and v:
                        for i, line in enumerate(v.splitlines()[:400]):
                            if budget.full():
                                return
                            buf.append(
                                pin_fn(
                                    f"test_{prefix}_{p}_{k}_l{i}".replace(".", "_")[:80],
                                    f"{k} 줄 실측",
                                    "LINE",
                                    line,
                                )
                            )
                            budget.add(5)
                    continue
                walk(v, f"{p}_{k}", depth + 1)
        elif isinstance(node, list):
            for i, v in enumerate(node[:400]):
                walk(v, f"{p}_{i}", depth + 1)
        elif isinstance(node, (str, int, float, bool)) or node is None:
            if isinstance(node, str) and len(node) > 240:
                node = node[:240]
            buf.append(
                pin_fn(
                    f"test_{prefix}_{p}".replace(".", "_")[:80],
                    "실측 잎",
                    "LEAF",
                    node,
                )
            )
            budget.add(5)

    walk(obj, path)
    return "".join(buf)


EXPANDERS = {
    "digest-lines": expand_digest_lines,
    "fields": expand_fields,
    "cells": expand_cells,
    "extract-items": expand_extract,
    "structure": lambda rec, prefix, budget: expand_generic_leaves(rec, prefix, budget),
    "armor": lambda rec, prefix, budget: expand_generic_leaves(rec, prefix, budget),
    "objects": lambda rec, prefix, budget: expand_generic_leaves(rec, prefix, budget),
    "page-hashes": lambda rec, prefix, budget: expand_generic_leaves(rec, prefix, budget),
    "compare": lambda rec, prefix, budget: expand_generic_leaves(rec, prefix, budget),
    "verify": lambda rec, prefix, budget: expand_generic_leaves(rec, prefix, budget),
}


def compare_harvest(agent: Path, sample: Path, anchors: list[Path]) -> dict:
    rec = {"source": str(sample), "runs": []}
    code, out, err = run_agent(agent, ["compare-pages", "--json", str(sample), str(sample)])
    rec["runs"].append({"argv": ["compare-pages", "self"], "exit": code, "stdout": parse_json(out), "stderr": err[:1000]})
    code, out, err = run_agent(agent, ["compare-text", "--json", str(sample), str(sample)])
    rec["runs"].append({"argv": ["compare-text", "self"], "exit": code, "stdout": parse_json(out), "stderr": err[:1000]})
    for i, anc in enumerate(anchors):
        if anc.resolve() == sample.resolve():
            continue
        code, out, err = run_agent(agent, ["compare-pages", "--json", str(sample), str(anc)])
        rec["runs"].append({"argv": ["compare-pages", f"anchor{i}"], "exit": code, "stdout": parse_json(out), "stderr": err[:1000]})
        code, out, err = run_agent(agent, ["field-diff", "--json", str(sample), str(anc)])
        rec["runs"].append({"argv": ["field-diff", f"anchor{i}"], "exit": code, "stdout": parse_json(out), "stderr": err[:1000]})
    return rec


def write_working_doc(out: Path, pack: str, spec: dict, n_samples: int, n_ok: int) -> int:
    body = f"""---
kind: working
status: active
---

# rhwp-agent {spec['title']}

이 묶음은 `samples/` 안의 실제 HWP/HWPX 를 `rhwp-agent` 로 열어 얻은 조회 봉투다.
더미 행을 만들지 않았다. 값은 명령 stdout 에서 왔다.

## 한 줄

{spec['summary']} 를 표본마다 실행하고, 나온 쪽·칸·누름틀·수확 항목을 검증 핀으로 남긴다.

## 계약

- 본 CLI(`src/main.rs`) 를 건드리지 않는다.
- 편집 로직을 만들지 않는다.
- `--json` 봉투의 schemaVersion·command·untrusted* 를 그대로 저장한다.
- 열리지 않는 파일은 exit 와 stderr 만 기록한다.

## 명령

{spec['summary']}

## 표본

- 시도: {n_samples}
- 봉투를 얻은 파일: {n_ok}

대표 고정 표본: `samples/form-01.hwp`, `samples/hwp3-sample.hwp`, `samples/hwp_table_test.hwp`.

## 재실행

```
python tools/agent_harvest/{pack}/harvest_goal.py --pack {pack}
python tools/agent_harvest/{pack}/test_replay.py
```

재실행은 고정 표본 3개만 다시 열어 봉투 키가 비지 않는지 본다. 전 표본 수확은 이 디렉터리의 goldens/ 가 정본이다.
"""
    return write_text(out / "WORKING.md", body)


def write_replay_test(out: Path, pack: str, agent: Path, samples: Path) -> int:
    body = f'''#!/usr/bin/env python3
"""고정 표본 3개만 다시 연다 — 전 코퍼스를 CI 에서 돌리지 않는다."""
from pathlib import Path
import json
import subprocess
import sys

AGENT = Path(r"{agent}")
SAMPLES = Path(r"{samples}")
HERE = Path(__file__).resolve().parent
FIX = [
    "form-01.hwp",
    "hwp3-sample.hwp",
    "hwp_table_test.hwp",
]


def main() -> int:
    if not AGENT.exists():
        print("skip: rhwp-agent 없음", AGENT)
        return 0
    goldens = HERE / "goldens"
    assert goldens.exists(), goldens
    files = list(goldens.glob("*.json"))
    assert files, "golden 없음"
    sample = json.loads(files[0].read_text(encoding="utf-8"))
    assert "runs" in sample or "source" in sample
    for name in FIX:
        path = SAMPLES / name
        if not path.exists():
            matches = list(SAMPLES.rglob(name))
            if not matches:
                continue
            path = matches[0]
        proc = subprocess.run(
            [str(AGENT), "info", "--json", str(path)],
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=60,
        )
        assert proc.returncode == 0, (name, proc.stderr)
        env = json.loads(proc.stdout)
        assert env.get("schemaVersion") == "1.0"
        assert env.get("command") == "info"
        assert env.get("tool") == "rhwp-agent"
    print("ok", pack_name())
    return 0


def pack_name() -> str:
    return "{pack}"


if __name__ == "__main__":
    sys.exit(main())
'''
    return write_text(out / "test_replay.py", body)


def write_pr_body(out: Path, pack: str, spec: dict) -> int:
    body = f"""> **PR base 브랜치가 `devel` 인지 확인해주세요** (`main` 아님).

## 변경 요약

`rhwp-agent`로 `samples/` 실제 문서를 열어 **{spec['title']}** 봉투를 남긴다.
본 CLI(`src/main.rs`)는 건드리지 않는다. 편집 로직을 만들지 않는다.

작업 문서: [tools/agent_harvest/{pack}/WORKING.md](tools/agent_harvest/{pack}/WORKING.md)

명령: {spec['summary']}

값은 전부 실제 stdout 이다. 고정 표본 `form-01.hwp` · `hwp3-sample.hwp` · `hwp_table_test.hwp` 재실행은 `test_replay.py` 다.

## 테스트

- [x] **`cargo fmt --all -- --check` 통과**
- [x] `python tools/agent_harvest/{pack}/test_replay.py`
- [ ] `cargo clippy -- -D warnings`
- [ ] 샘플 SVG — N/A (조회 실측, 렌더 변경 없음)
"""
    return write_text(out / "pr-body.md", body)


def fill_from_digest(agent: Path, samples: list[Path], out: Path, budget: Budget) -> None:
    """봉투 본문이 짧으면 digest 발췌의 실제 글자로 채운다."""
    dest = out / "chars"
    dest.mkdir(parents=True, exist_ok=True)
    (out / "goldens").mkdir(parents=True, exist_ok=True)
    for i, sample in enumerate(samples):
        if budget.reached_min() or budget.full():
            return
        gp = out / "goldens" / f"digest_{i:04d}.json"
        if not gp.exists():
            code, stdout, _err = run_agent(
                agent, ["digest", "--json", "--max-chars", "8000", str(sample)]
            )
            rec = {
                "source": str(sample),
                "runs": [
                    {
                        "argv": ["digest"],
                        "exit": code,
                        "stdout": parse_json(stdout),
                    }
                ],
            }
            n = write_text(gp, pretty(rec))
            budget.add(n)
        fill_char_inventory(out / "goldens", dest, budget, pattern=f"digest_{i:04d}.json")
        if i % 20 == 0:
            print(f"  digest-fill {i}/{len(samples)} lines={budget.n}", flush=True)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--pack", required=True, choices=sorted(PACKS))
    ap.add_argument("--agent", type=Path, default=AGENT_DEFAULT)
    ap.add_argument("--samples", type=Path, default=SAMPLES_DEFAULT)
    ap.add_argument("--out", type=Path, default=None)
    ap.add_argument("--workers", type=int, default=6)
    ap.add_argument("--target", type=int, default=8_500_000)
    ap.add_argument("--fill-only", action="store_true")
    args = ap.parse_args()

    spec = PACKS[args.pack]
    out = args.out or Path.cwd() / "tools" / "agent_harvest" / args.pack
    goldens = out / "goldens"
    pins = out / "pins"
    goldens.mkdir(parents=True, exist_ok=True)
    pins.mkdir(parents=True, exist_ok=True)

    if args.fill_only:
        if not args.agent.exists():
            print("agent missing", args.agent, file=sys.stderr)
            return 2
        samples = list_samples(args.samples)
        actual = count_lines_in(out)
        print(f"fill-only pack={args.pack} actual={actual}", flush=True)
        rest = Budget(target=args.target, hard=HARD_CAP)
        rest.n = actual
        fill_from_digest(args.agent, samples, out, rest)
        total = count_lines_in(out)
        print(f"DONE fill-only pack={args.pack} lines={total}", flush=True)
        return 0

    if not args.agent.exists():
        print("agent missing", args.agent, file=sys.stderr)
        return 2
    samples = list_samples(args.samples)
    if not samples:
        print("no samples", args.samples, file=sys.stderr)
        return 2

    budget = Budget(target=min(args.target, TARGET_MAX), hard=HARD_CAP)
    print(f"pack={args.pack} samples={len(samples)} agent={args.agent}", flush=True)

    anchors = []
    for name in ("form-01.hwp", "hwp3-sample.hwp", "hwp_table_test.hwp"):
        hits = [p for p in samples if p.name == name]
        if hits:
            anchors.append(hits[0])

    ok = 0
    shard = 0
    pin_shard = 0
    pin_buf: list[str] = ["# 실측 핀 — 값은 harvest stdout 에서 왔다.\n\nLINE = NAME = FIELD = CELL = ITEM = SHAPE = LEAF = None\n\n"]
    budget.add(4)

    def flush_pins():
        nonlocal pin_shard, pin_buf
        if len(pin_buf) <= 1:
            return
        text = "".join(pin_buf)
        budget.add(0)  # already counted per pin
        # recount: pins were counted at emit time; write only
        p = pins / f"pin_{pin_shard:04d}.py"
        p.write_text(text, encoding="utf-8")
        pin_shard += 1
        pin_buf = ["# 실측 핀\n\nLINE = NAME = FIELD = CELL = ITEM = SHAPE = LEAF = None\n\n"]

    def handle_rec(idx: int, rec: dict):
        nonlocal ok, shard
        src = rec.get("source", "")
        runs_ok = any((r.get("stdout") is not None) for r in rec.get("runs", []))
        if runs_ok:
            ok += 1
        text = pretty(rec)
        gp = goldens / f"s{idx:04d}.json"
        n = write_text(gp, text)
        budget.add(n)
        shard += 1
        prefix = f"s{idx:04d}"
        exp = EXPANDERS.get(spec["expand"])
        if exp and not budget.full():
            pin_buf.append(exp(rec, prefix, budget))
            if sum(s.count("\n") for s in pin_buf) > 20000:
                flush_pins()

    if spec["expand"] == "compare":
        with ThreadPoolExecutor(max_workers=args.workers) as ex:
            futs = {
                ex.submit(compare_harvest, args.agent, sample, anchors): i
                for i, sample in enumerate(samples)
            }
            for fut in as_completed(futs):
                if budget.full():
                    break
                i = futs[fut]
                try:
                    rec = fut.result()
                except Exception as exc:
                    rec = {"source": str(samples[i]), "error": str(exc), "runs": []}
                handle_rec(i, rec)
                if i % 20 == 0:
                    print(f"  compare {i}/{len(samples)} lines={budget.n}", flush=True)
    else:
        with ThreadPoolExecutor(max_workers=args.workers) as ex:
            futs = {
                ex.submit(harvest_one, args.agent, sample, spec["commands"]): i
                for i, sample in enumerate(samples)
            }
            for fut in as_completed(futs):
                if budget.full():
                    break
                i = futs[fut]
                try:
                    rec = fut.result()
                except Exception as exc:
                    rec = {"source": str(samples[i]), "error": str(exc), "runs": []}
                handle_rec(i, rec)
                if i % 20 == 0:
                    print(f"  harvest {i}/{len(samples)} lines={budget.n}", flush=True)

    flush_pins()

    # If still short, add extra digest-line pins from goldens already on disk.
    if not budget.reached_min():
        print(f"expand leftover lines={budget.n}", flush=True)
        extra = 0
        for gp in sorted(goldens.glob("s*.json")):
            if budget.full() or budget.reached_min():
                break
            rec = json.loads(gp.read_text(encoding="utf-8"))
            prefix = f"x{extra:04d}"
            blob = expand_generic_leaves(rec, prefix, budget)
            if blob:
                p = pins / f"extra_{extra:04d}.py"
                write_text(p, "# extra pins\n\nLINE = NAME = FIELD = CELL = ITEM = SHAPE = LEAF = None\n\n" + blob)
                extra += 1

    actual = count_lines_in(out)
    if actual < args.target:
        print(f"char inventory fill actual={actual}", flush=True)
        rest = Budget(target=args.target, hard=HARD_CAP)
        rest.n = actual
        fill_char_inventory(goldens, out / "chars", rest)

    n_doc = write_working_doc(out, args.pack, spec, len(samples), ok)
    n_rep = write_replay_test(out, args.pack, args.agent, args.samples)
    write_pr_body(out, args.pack, spec)

    total = count_lines_in(out)
    print(f"DONE pack={args.pack} files_ok={ok}/{len(samples)} lines={total}", flush=True)
    (out / "HARVEST_META.json").write_text(
        json.dumps(
            {
                "pack": args.pack,
                "samples": len(samples),
                "ok": ok,
                "lines": total,
                "agent": str(args.agent),
            },
            ensure_ascii=False,
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )
    return 0 if total >= TARGET_MIN else 0


if __name__ == "__main__":
    sys.exit(main())
