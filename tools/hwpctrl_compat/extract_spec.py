"""웹한글컨트롤 공식 규격서 → 기계 판독 스펙 (P0).

정답지는 추측이 아니라 **한컴 공식 문서** 두 건이다.

- `samples/hwpctl_API_v2.4.hwp` — 웹한글컨트롤 API v2.4 (HwpCtrl/Action/CtrlCode/
  ParameterSet/ParameterArray 의 Property·Method·Event 전수)
- `samples/hwpctl_ParameterSetID_Item_v1.2.hwp` — ParameterSet 50종의 Item ID·Type 전수
- `samples/hwpctl_Action_Table__v1.1.hwp` — Action 312개와 각자의 ParameterSet ID

이 스크립트는 세 문서를 rhwp CLI 로 읽어 아래 파일을 만든다. **손으로 고치지 말 것** —
고쳐야 한다면 이 스크립트를 고친다.

    npm/hwpctrl-ocx/spec/webhwpctrl_api.json
    npm/hwpctrl-ocx/spec/parameter_sets.json
    npm/hwpctrl-ocx/spec/actions.json

## 쓰임

    python tools/hwpctrl_compat/extract_spec.py --exe target/release/rhwp.exe

## 주의

- 문서의 절 번호(`8.3.34.`)가 파싱 기준이다. 문서 버전이 바뀌면 이 스크립트가 먼저 깨져야
  한다 — 조용히 빈 결과를 내지 않도록 §검증에서 개수를 단언한다.
"""

from __future__ import annotations

import argparse
import io
import json
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
API_DOC = REPO / "samples" / "hwpctl_API_v2.4.hwp"
PSET_DOC = REPO / "samples" / "hwpctl_ParameterSetID_Item_v1.2.hwp"
ACTION_DOC = REPO / "samples" / "hwpctl_Action_Table__v1.1.hwp"
OUT_DIR = REPO / "npm" / "hwpctrl-ocx" / "spec"

# 절 번호 → 어떤 객체의 무엇인가. 문서 구조(§4~§8)를 그대로 옮긴 것이다.
SECTION_MAP = {
    "4.2": ("Action", "property"),
    "4.3": ("Action", "method"),
    "5.2": ("CtrlCode", "property"),
    "5.3": ("CtrlCode", "method"),
    "6.2": ("ParameterSet", "property"),
    "6.3": ("ParameterSet", "method"),
    "7.2": ("ParameterArray", "property"),
    "7.3": ("ParameterArray", "method"),
    "8.2": ("HwpCtrl", "property"),
    "8.3": ("HwpCtrl", "method"),
    "8.4": ("HwpCtrl", "event"),
}

# 문서가 약속하는 개수. 하나라도 어긋나면 파서가 깨진 것이다 (§검증).
EXPECTED = {
    ("HwpCtrl", "property"): 18,
    ("HwpCtrl", "method"): 67,
    ("HwpCtrl", "event"): 3,
    ("Action", "property"): 2,
    ("Action", "method"): 5,
    ("CtrlCode", "property"): 6,
    ("CtrlCode", "method"): 1,
    ("ParameterSet", "property"): 3,
    ("ParameterSet", "method"): 11,
    ("ParameterArray", "property"): 2,
    ("ParameterArray", "method"): 4,
}
EXPECTED_PSET_COUNT = 50
EXPECTED_PSET_ITEM_COUNT = 521
EXPECTED_ACTION_COUNT = 312

HEADER_RE = re.compile(r"^(\d+(?:\.\d+)+)\.\s+(\S[^\n]*?)\s*$")
SIGNATURE_RE = re.compile(r"^\s*(?:\w+\.)?(\w+)\s*\((.*?)\)\s*$")
PARAMS_COUNT_RE = re.compile(r"^Parameters\s*\t?\s*(\d+)\s*$")
# 제목은 `23) FootnoteShape / EndnoteShape : …` 처럼 **한 표를 두 Set 이 공유**하기도 한다.
# 이름 자리를 좁게 잡으면 그 줄에서 파싱이 조용히 끊긴다(실제로 23번에서 끊겼다).
PSET_HEADING_RE = re.compile(r"^(\d+)\)\s*([A-Za-z][A-Za-z0-9_ /]*?)\s*:\s*(.*)$")


def run_cli(exe: Path, args: list[str]) -> dict:
    """rhwp CLI 를 --json 으로 돌려 파싱한다."""
    proc = subprocess.run(
        [str(exe), *args],
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        sys.stderr.write(proc.stderr.decode("utf-8", "replace"))
        raise SystemExit(f"rhwp CLI 실패 (rc={proc.returncode}): {' '.join(args)}")
    return json.loads(proc.stdout.decode("utf-8", "replace"))


def doc_text(exe: Path, path: Path) -> str:
    data = run_cli(exe, ["export-text", str(path), "--json"])
    return "\n".join(p["text"] for p in data["pages"])


def doc_tables(exe: Path, path: Path) -> list[dict]:
    data = run_cli(exe, ["export-tables", str(path), "--json"])
    return data["tables"] if isinstance(data, dict) and "tables" in data else data


def split_sections(text: str) -> list[tuple[str, str, list[str]]]:
    """(절번호, 제목, 본문줄들) 목록. 문서 순서를 보존한다."""
    sections: list[tuple[str, str, list[str]]] = []
    current: tuple[str, str, list[str]] | None = None
    for line in text.split("\n"):
        m = HEADER_RE.match(line)
        if m and m.group(1).count(".") >= 2:
            if current:
                sections.append(current)
            current = (m.group(1), m.group(2), [])
        elif current:
            current[2].append(line)
    if current:
        sections.append(current)
    return sections


def parse_entry(number: str, title: str, body: list[str]) -> dict | None:
    prefix = ".".join(number.split(".")[:2])
    mapped = SECTION_MAP.get(prefix)
    if not mapped:
        return None
    obj, kind = mapped
    name = title.strip()
    if not re.fullmatch(r"\w+", name):
        return None

    syntax = ""
    args: list[str] = []
    declared_arity = None
    returns: list[str] = []
    description: list[str] = []

    block = None
    for line in body:
        stripped = line.strip()
        if stripped == "Syntax":
            block = "syntax"
            continue
        if stripped.startswith("Description"):
            block = "description"
            continue
        m = PARAMS_COUNT_RE.match(stripped)
        if m:
            declared_arity = int(m.group(1))
            block = "params"
            continue
        if stripped.startswith("Parameters"):
            block = "params"
            continue
        if stripped.startswith("Return Values"):
            block = "returns"
            continue
        if stripped.startswith(("Remarks", "Example", "See Also")):
            block = None
            continue
        if not stripped:
            continue
        if block == "syntax":
            # 서명이 쪽/줄 경계에서 잘리는 경우가 있다(예: InsertPicture 는 인자 8개가
            # 두 줄에 걸친다). 괄호가 닫힐 때까지 이어 붙이지 않으면 인자를 통째로 놓친다.
            if not syntax:
                syntax = stripped
            elif syntax.count("(") > syntax.count(")"):
                syntax = syntax.rstrip() + (" " if syntax.rstrip().endswith(",") else "") + stripped
            else:
                continue
            m = SIGNATURE_RE.match(syntax)
            if m:
                raw = m.group(2).strip()
                args = [a.strip() for a in raw.split(",") if a.strip()] if raw else []
        elif block == "description":
            description.append(stripped)
        elif block == "returns":
            returns.append(stripped)

    return {
        "object": obj,
        "kind": kind,
        "name": name,
        "section": number,
        "syntax": syntax,
        "args": args,
        # 서명에서 센 인자 수. `Parameters N` 이 있으면 그것과 대조해 불일치를 표시한다.
        "arity": len(args),
        "declaredArity": declared_arity,
        "arityMismatch": declared_arity is not None and declared_arity != len(args),
        "returns": " ".join(returns).strip(),
        "description": " ".join(description).strip(),
    }


def extract_api(exe: Path) -> list[dict]:
    text = doc_text(exe, API_DOC)
    entries = []
    seen = set()
    for number, title, body in split_sections(text):
        entry = parse_entry(number, title, body)
        if not entry:
            continue
        key = (entry["object"], entry["kind"], entry["name"])
        if key in seen:
            continue
        seen.add(key)
        entries.append(entry)
    return entries


def extract_parameter_sets(exe: Path) -> list[dict]:
    """ParameterSet 50종과 각 Item 을 뽑는다.

    문서는 `N) SetID : 설명` 제목 뒤에 표 하나가 오는 구조다. 텍스트에서 제목 순서를,
    표 추출에서 Item 행을 얻어 **순서로 짝짓는다**. 표 개수가 제목보다 많은 것은 뒤쪽의
    부록 표(열거형 등) 때문이므로 앞에서부터 50개만 쓴다.
    """
    text = doc_text(exe, PSET_DOC)
    headings = []
    for line in text.split("\n"):
        m = PSET_HEADING_RE.match(line.strip())
        if m and int(m.group(1)) == len(headings) + 1:
            headings.append((m.group(2), m.group(3).strip()))
        if len(headings) == EXPECTED_PSET_COUNT:
            break

    tables = doc_tables(exe, PSET_DOC)
    item_tables = []
    for table in tables:
        cells = table["cells"]
        header = {c["col"]: c["text"].strip() for c in cells if c["row"] == 0}
        if header.get(0) != "Item ID":
            continue
        rows: dict[int, dict[int, str]] = {}
        for c in cells:
            if c["row"] == 0:
                continue
            rows.setdefault(c["row"], {})[c["col"]] = c["text"].strip()
        items = []
        for _, row in sorted(rows.items()):
            item_id = row.get(0, "")
            if not item_id or not re.fullmatch(r"[A-Za-z][\w.]*", item_id):
                continue
            items.append(
                {
                    "item": item_id,
                    "type": row.get(1, ""),
                    "subType": row.get(2, ""),
                    "description": row.get(3, ""),
                }
            )
        item_tables.append(items)

    sets = []
    for idx, (raw_name, desc) in enumerate(headings):
        names = [n.strip() for n in raw_name.split("/") if n.strip()]
        items = item_tables[idx] if idx < len(item_tables) else []
        sets.append(
            {
                "setId": names[0],
                # 같은 Item 표를 공유하는 형제 Set (예: FootnoteShape / EndnoteShape).
                "aliases": names[1:],
                "description": desc,
                "itemCount": len(items),
                "items": items,
            }
        )
    return sets


def extract_actions(exe: Path) -> list[dict]:
    """Action 312개. 표의 `ParameterSet ID` 열은 기호를 달고 온다.

    - `-`    : ParameterSet 없음 → `Run()` 직접 호출 가능
    - `Set*` : 외부에서 Set 을 만들어 줘야 정상 동작 (`Run()` 불가)
    """
    tables = doc_tables(exe, ACTION_DOC)
    actions = []
    for table in tables:
        cells = table["cells"]
        header = {c["col"]: c["text"].strip() for c in cells if c["row"] == 0}
        if header.get(0) != "Action ID":
            continue
        rows: dict[int, dict[int, str]] = {}
        for c in cells:
            if c["row"] == 0:
                continue
            rows.setdefault(c["row"], {})[c["col"]] = c["text"].strip()
        for _, row in sorted(rows.items()):
            action_id = row.get(0, "")
            if not action_id or not re.fullmatch(r"[A-Za-z][\w]*", action_id):
                continue
            raw_set = row.get(1, "").strip()
            needs_set = raw_set.endswith("*")
            set_id = raw_set.rstrip("*").strip()
            actions.append(
                {
                    "actionId": action_id,
                    "parameterSetId": None if set_id in ("", "-") else set_id,
                    # `Run()` 직접 호출이 불가능하고 Set 을 반드시 만들어 줘야 하는 Action.
                    "requiresExternalSet": needs_set,
                    "description": row.get(2, ""),
                }
            )
    return actions


def verify(api: list[dict], sets: list[dict], actions: list[dict]) -> list[str]:
    """개수가 문서 약속과 다르면 실패 사유를 모은다. 조용히 통과시키지 않는다."""
    problems = []
    counts: dict[tuple[str, str], int] = {}
    for e in api:
        counts[(e["object"], e["kind"])] = counts.get((e["object"], e["kind"]), 0) + 1
    for key, want in EXPECTED.items():
        got = counts.get(key, 0)
        if got != want:
            problems.append(f"{key[0]}.{key[1]}: 기대 {want} 실제 {got}")
    if len(sets) != EXPECTED_PSET_COUNT:
        problems.append(f"ParameterSet: 기대 {EXPECTED_PSET_COUNT} 실제 {len(sets)}")
    item_count = sum(len(s["items"]) for s in sets)
    if item_count != EXPECTED_PSET_ITEM_COUNT:
        problems.append(f"ParameterSet Item: 기대 {EXPECTED_PSET_ITEM_COUNT} 실제 {item_count}")
    empty = [s["setId"] for s in sets if not s["items"]]
    if empty:
        problems.append(f"Item 이 비어 있는 Set: {', '.join(empty)}")
    if len(actions) != EXPECTED_ACTION_COUNT:
        problems.append(f"Action: 기대 {EXPECTED_ACTION_COUNT} 실제 {len(actions)}")
    dupes = {a["actionId"] for a in actions if [x["actionId"] for x in actions].count(a["actionId"]) > 1}
    if dupes:
        problems.append(f"중복 Action ID: {', '.join(sorted(dupes))}")
    return problems


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--exe", default=str(REPO / "target" / "release" / "rhwp.exe"))
    ap.add_argument("--out", default=str(OUT_DIR))
    args = ap.parse_args()

    exe = Path(args.exe)
    if not exe.exists():
        raise SystemExit(f"rhwp 실행 파일 없음: {exe}")

    api = extract_api(exe)
    sets = extract_parameter_sets(exe)
    actions = extract_actions(exe)

    problems = verify(api, sets, actions)
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)

    api_doc = {
        "schemaVersion": "1.0",
        "source": "samples/hwpctl_API_v2.4.hwp",
        "spec": "웹한글컨트롤 API v2.4",
        "entryCount": len(api),
        "entries": api,
    }
    pset_doc = {
        "schemaVersion": "1.0",
        "source": "samples/hwpctl_ParameterSetID_Item_v1.2.hwp",
        "spec": "ParameterSet Table v1.2",
        "setCount": len(sets),
        "itemCount": sum(s["itemCount"] for s in sets),
        "sets": sets,
    }
    with io.open(out_dir / "webhwpctrl_api.json", "w", encoding="utf-8", newline="\n") as fh:
        json.dump(api_doc, fh, ensure_ascii=False, indent=2)
        fh.write("\n")
    action_doc = {
        "schemaVersion": "1.0",
        "source": "samples/hwpctl_Action_Table__v1.1.hwp",
        "spec": "Action Table v1.1",
        "actionCount": len(actions),
        "actions": actions,
    }
    with io.open(out_dir / "parameter_sets.json", "w", encoding="utf-8", newline="\n") as fh:
        json.dump(pset_doc, fh, ensure_ascii=False, indent=2)
        fh.write("\n")
    with io.open(out_dir / "actions.json", "w", encoding="utf-8", newline="\n") as fh:
        json.dump(action_doc, fh, ensure_ascii=False, indent=2)
        fh.write("\n")

    print(f"API 항목 {len(api)}개 → {out_dir / 'webhwpctrl_api.json'}")
    print(f"ParameterSet {len(sets)}종 / Item {pset_doc['itemCount']}개 → {out_dir / 'parameter_sets.json'}")
    print(f"Action {len(actions)}개 → {out_dir / 'actions.json'}")
    mismatched = [e["name"] for e in api if e["arityMismatch"]]
    if mismatched:
        print(f"서명↔Parameters 개수 불일치 {len(mismatched)}건: {', '.join(mismatched)}")
    if problems:
        print("\n검증 실패 — 파서가 문서 구조를 놓쳤다:")
        for p in problems:
            print(f"  - {p}")
        return 1
    print("검증 통과 — 문서가 약속한 개수와 일치")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
