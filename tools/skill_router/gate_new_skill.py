#!/usr/bin/env python3
"""Gate for adding or editing a skill under .agents/skills/.

Mirrors tests/skills_contract.rs (frontmatter + executable `rhwp <cmd>`),
re-scans every skill three times, checks catalog.json paths, and probes
tools/skill_router/route.py three times per catalog skill.

When `--rhwp-bin PATH` is supplied, every extracted `rhwp <cmd>` token (and
group subcommand) must appear in `rhwp capabilities` ∪ `rhwp --help` ∪
{help}. Without an explicitly selected candidate binary, that optional live
command layer is skipped so a stale local Cargo artifact cannot change the
gate result.

    python tools/skill_router/gate_new_skill.py
    python tools/skill_router/gate_new_skill.py --json
    python tools/skill_router/gate_new_skill.py --rhwp-bin target/review/release/rhwp

Exit 0 pass, 1 fail, 2 usage. stdlib only.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
SKILLS_DIR = REPO / ".agents" / "skills"
CATALOG_JSON = HERE / "catalog.json"
ROUTE_PY = HERE / "route.py"

SCHEMA_VERSION = "1.0"
SCAN_TIMES = 3
ROUTE_PROBES = 3
ROUTE_TIMEOUT_SEC = 20
RHWP_TIMEOUT_SEC = 20
MIN_DESCRIPTION_CHARS = 20
MIN_KNOWN_COMMANDS = 20
MIN_EDIT_SUBCOMMANDS = 4
USAGE = "usage: python tools/skill_router/gate_new_skill.py [--json] [--rhwp-bin PATH]"

ENVELOPE_KEYS = (
    "schemaVersion",
    "request",
    "intent",
    "requiredCapabilities",
    "skillSelection",
    "executionGraph",
    "untrustedContent",
    "untrustedFields",
)

# Distinct Korean/English probes per catalog skill. Every catalog id must have
# a dedicated 3-tuple here; unknown catalog skills fail closed (no generator).
# Mapping to the same skill is best-effort; a valid route.py JSON envelope is
# enough.
PROBES: dict[str, tuple[str, str, str]] = {
    "rhwp-agent-surface": (
        "새 MCP 도구 추가해줘",
        "드리프트 가드 확인해",
        "capabilities 가 SSOT 인지 확인해",
    ),
    "rhwp-bug-hunter": (
        "버그 찾아줘 실사용 기준으로",
        "정답지와 비교해",
        "playbook 여정 실행해",
    ),
    "rhwp-bulk-pipeline": (
        "폴더 전체를 텍스트로 변환해",
        "rhwp batch the whole corpus",
        "여러 hwp 대량 처리해줘",
    ),
    "rhwp-chief": (
        "요청 큐 돌려줘",
        "서비스 루프 감시해",
        "needs-agent 수거해",
    ),
    "rhwp-cli": (
        "페이지네이션 조판부호 덤프",
        "dump pagination and the render tree",
        "레이아웃 겹침 버그 디버깅해",
    ),
    "rhwp-codex": (
        "rhwp 사용법 전체 명령 보여줘",
        "navigate the rhwp command codex",
        "뭘 쓸지 모르겠어",
    ),
    "rhwp-contributor": (
        "PR 올려",
        "open a pull request",
        "기여 절차 알려줘",
    ),
    "rhwp-doc-triage": (
        "이 hwp 뭔 문서야?",
        "summarize this document without reading it all",
        "목차 뽑아줘",
    ),
    "rhwp-exam-ingest": (
        "한글 시험지로 변환해줘",
        "exam ingest this paper to hwpx",
        "시험문제 변환해",
    ),
    "rhwp-explore": (
        "이 문서로 뭘 할 수 있어?",
        "어포던스 메뉴 보여줘",
        "문서 탐색해봐",
    ),
    "rhwp-fde": (
        "고객이 이 문서가 안 열린대",
        "현장 증상 트리아지해줘",
        "필드가 안 채워진대 대응해줘",
    ),
    "rhwp-fidelity-compare": (
        "한컴 PDF와 비교해줘",
        "run fidelity_compare against the official PDF",
        "한컴이 뽑은 PDF랑 rhwp가 같은지",
    ),
    "rhwp-form-fill": (
        "이 서식 채워줘",
        "fill this form",
        "누름틀에 값 넣어줘",
    ),
    "rhwp-handoff": (
        "세션 핸드오프 해줘",
        "컨텍스트 바닥이라 핸드오프해",
        "작업 인수인계 result.json 읽어",
    ),
    "rhwp-knowledge-map": (
        "지식 지도 어디 문서부터",
        "이 필드가 뭐야",
        "llms.txt 다음이 뭐야",
    ),
    "rhwp-mcp-session": (
        "rhwp 를 MCP로 붙여줘",
        "start mcp-serve and list session tools",
        "hwp_open 으로 문서 열어",
    ),
    "rhwp-onboarding": (
        "rhwp 처음인데 온보딩해줘",
        "rhwp_doctor로 온보딩해",
        ".mcp.json 만들어줘",
    ),
    "rhwp-provenance": (
        "이 값이 문서에서 온 건가",
        "mark untrustedFields provenance",
        "출처 모르는 문서 처리해",
    ),
    "rhwp-recipes": (
        "어떤 레시피로 가?",
        "실무 플레이북 골라줘",
        "결번 레시피 07 08 없지",
    ),
    "rhwp-safe-edit": (
        "안전하게 편집해줘",
        "dry-run the replace-text plan first",
        "여러 편집을 한 번에 원자적으로",
    ),
    "rhwp-security-sweep": (
        "이 문서 보내도 돼?",
        "inspect hidden text and redact PII",
        "받은 첨부 안전한지 확인",
    ),
    "rhwp-skill-author": (
        "새 스킬 만들어",
        "create a new SKILL.md with the 3-pass gate",
        "SKILL.md 작성해줘",
    ),
    "rhwp-skill-router": (
        "어떤 스킬을 쓰지",
        "route this request through the execution graph",
        "라우터에 통과시켜줘",
    ),
    "rhwp-strategist": (
        "이 문서들로 전략 보고서 만들어",
        "근거 대장에 주장마다 좌표",
        "정부과제 수주 근거 모아줘",
    ),
    "rhwp-table-exchange": (
        "표를 CSV로 뽑아줘",
        "csv-to-table 로 되돌려",
        "표 셀 하나만 고쳐줘",
    ),
    "rhwp-visual-regression": (
        "편집 전후 화면 비교해",
        "run render-diff for visual regression",
        "레이아웃 회귀 깨졌는지 확인",
    ),
    "rhwp-work-receipt": (
        "이 작업 영수증 남겨",
        "replay the work capsule and audit lineage",
        "재현율 검증해줘",
    ),
}


class GateFail(Exception):
    """A contract check failed. exit 1."""


class UsageError(Exception):
    """Bad argv. exit 2."""


def _configure_stdio() -> None:
    for stream in (sys.stdout, sys.stderr):
        reconfigure = getattr(stream, "reconfigure", None)
        if reconfigure is None:
            continue
        try:
            reconfigure(encoding="utf-8", errors="replace")
        except (OSError, ValueError):
            pass


def _human_stream(json_mode: bool):
    return sys.stderr if json_mode else sys.stdout


def _say(json_mode: bool, line: str) -> None:
    print(line, file=_human_stream(json_mode), flush=True)


def _is_token_char(ch: str) -> bool:
    return ch.isascii() and (ch.islower() or ch.isdigit() or ch == "-")


def _is_token(s: str) -> bool:
    return bool(s) and not s.startswith("-") and all(_is_token_char(ch) for ch in s)


def referenced_commands(body: str) -> list[tuple[str, str | None]]:
    """Extract `rhwp <token> [subtoken]` the same way skills_contract.rs does.

    After each `rhwp `, take `[a-z0-9-]+` that starts with a-z. Placeholders
    such as `rhwp <명령>` and Korean particles (`rhwp 를`) do not count.
    """
    pat = "rhwp "
    refs: list[tuple[str, str | None]] = []
    idx = 0
    while True:
        pos = body.find(pat, idx)
        if pos < 0:
            break
        start = pos + len(pat)
        tok_chars: list[str] = []
        for ch in body[start:]:
            if not _is_token_char(ch):
                break
            tok_chars.append(ch)
        tok = "".join(tok_chars)
        if tok and tok[0].islower():
            after = start + len(tok)
            sub: str | None = None
            if body[after : after + 1] == " ":
                sub_chars: list[str] = []
                for ch in body[after + 1 :]:
                    if not _is_token_char(ch):
                        break
                    sub_chars.append(ch)
                candidate = "".join(sub_chars)
                if _is_token(candidate):
                    sub = candidate
            refs.append((tok, sub))
        idx = start
    return refs


def parse_frontmatter(body: str) -> tuple[str | None, str | None]:
    """Read `name:` / `description:` from the opening YAML --- block."""
    lines = body.splitlines()
    if not lines or lines[0] != "---":
        raise GateFail("frontmatter start (---) missing")
    fm_name: str | None = None
    fm_desc: str | None = None
    for line in lines[1:]:
        if line == "---":
            break
        if line.startswith("name:"):
            fm_name = line[len("name:") :].strip()
        if line.startswith("description:"):
            fm_desc = line[len("description:") :].strip()
    return fm_name, fm_desc


def check_skill(folder_name: str, body: str) -> list[str]:
    """Return referenced command tokens after validating frontmatter."""
    fm_name, fm_desc = parse_frontmatter(body)
    if fm_name != folder_name:
        raise GateFail(
            f"frontmatter name {fm_name!r} does not match folder {folder_name!r}"
        )
    desc_len = len(fm_desc or "")
    if desc_len < MIN_DESCRIPTION_CHARS:
        raise GateFail(f"description too short ({desc_len} chars, need {MIN_DESCRIPTION_CHARS})")
    refs = referenced_commands(body)
    if not refs:
        raise GateFail(
            "no executable `rhwp <ascii-lowercase-command>` "
            "(placeholder `rhwp <명령>` does not count)"
        )
    return [tok for tok, _sub in refs]


def list_skill_dirs() -> list[Path]:
    if not SKILLS_DIR.is_dir():
        raise GateFail(f"missing skills directory: {SKILLS_DIR}")
    dirs = sorted(
        path for path in SKILLS_DIR.iterdir() if path.is_dir() and not path.name.startswith(".")
    )
    if len(dirs) < 1:
        raise GateFail("no skill folders under .agents/skills/")
    return dirs


def read_skill(dir_path: Path) -> str:
    md = dir_path / "SKILL.md"
    if not md.is_file():
        raise GateFail(f"{dir_path.name}/SKILL.md missing")
    try:
        return md.read_text(encoding="utf-8")
    except OSError as exc:
        raise GateFail(f"{dir_path.name}/SKILL.md read failed: {exc}") from exc
    except UnicodeDecodeError as exc:
        raise GateFail(f"{dir_path.name}/SKILL.md is not utf-8: {exc}") from exc


def scan_skills(json_mode: bool) -> list[dict[str, Any]]:
    """Run the full SKILL.md contract scan SCAN_TIMES times. Fail on first mismatch."""
    snapshots: list[list[tuple[str, tuple[str, ...]]]] = []
    last_records: list[dict[str, Any]] = []
    for scan_i in range(1, SCAN_TIMES + 1):
        rows: list[tuple[str, tuple[str, ...]]] = []
        records: list[dict[str, Any]] = []
        for dir_path in list_skill_dirs():
            name = dir_path.name
            try:
                body = read_skill(dir_path)
                commands = check_skill(name, body)
            except GateFail as exc:
                _say(json_mode, f"[{scan_i}/{SCAN_TIMES}] {name}: FAIL: {exc}")
                raise
            rows.append((name, tuple(commands)))
            records.append(
                {
                    "id": name,
                    "ok": True,
                    "commandCount": len(commands),
                    "commands": commands[:8],
                }
            )
            _say(json_mode, f"[{scan_i}/{SCAN_TIMES}] {name}: pass")
        if snapshots and rows != snapshots[0]:
            _say(json_mode, f"[{scan_i}/{SCAN_TIMES}] SCAN: FAIL: mismatch vs scan 1")
            raise GateFail(f"scan {scan_i} disagrees with scan 1")
        snapshots.append(rows)
        last_records = records
    return last_records


def find_rhwp_binary(candidate: str | None) -> Path | None:
    """Return only an explicitly selected candidate binary.

    Auto-discovering target/release or PATH makes this Python-only gate depend
    on unrelated, often stale local build output. Callers that need the live
    command contract must opt in with the exact candidate they built.
    """
    if candidate is None:
        return None
    binary = Path(candidate)
    if not binary.is_absolute():
        binary = REPO / binary
    if not binary.is_file():
        raise GateFail(f"selected rhwp binary does not exist: {binary}")
    return binary


def _run_rhwp(binary: Path, args: list[str]) -> str:
    argv = [str(binary), *args]
    try:
        proc = subprocess.run(
            argv,
            cwd=str(REPO),
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=RHWP_TIMEOUT_SEC,
            env=_cli_env(),
            check=False,
        )
    except subprocess.TimeoutExpired as exc:
        raise GateFail(f"rhwp {' '.join(args)} hung after {RHWP_TIMEOUT_SEC}s") from exc
    if proc.returncode != 0:
        err = (proc.stderr or "").strip()[:400]
        raise GateFail(f"rhwp {' '.join(args)} exit {proc.returncode} stderr={err!r}")
    return proc.stdout or ""


def _help_head_token(tok: str) -> bool:
    return bool(tok) and all(_is_token_char(ch) for ch in tok)


def parse_known_commands(caps_stdout: str, help_stdout: str) -> set[str]:
    """capabilities names ∪ --help two-space heads ∪ {help}. Same as skills_contract.rs."""
    known: set[str] = set()
    try:
        caps = json.loads(caps_stdout)
    except json.JSONDecodeError as exc:
        preview = caps_stdout.strip()[:240].replace("\n", "\\n")
        raise GateFail(f"rhwp capabilities is not JSON: {exc}: {preview!r}") from exc
    commands = caps.get("commands") if isinstance(caps, dict) else None
    if not isinstance(commands, list):
        raise GateFail("rhwp capabilities missing commands array")
    for item in commands:
        if not isinstance(item, dict):
            continue
        name = item.get("name")
        if not isinstance(name, str) or not name:
            continue
        known.add(name)
        head = name.split()[0] if name.split() else ""
        if head:
            known.add(head)
    for line in help_stdout.splitlines():
        if not line.startswith("  "):
            continue
        rest = line[2:]
        parts = rest.split()
        if not parts:
            continue
        tok = parts[0]
        if _help_head_token(tok):
            known.add(tok)
    known.add("help")
    if len(known) < MIN_KNOWN_COMMANDS:
        raise GateFail(
            f"real command set too small ({len(known)}) — self-description parse regression"
        )
    return known


def parse_group_subcommands(help_stdout: str) -> dict[str, set[str]]:
    """Harvest `rhwp <head> <sub>` names from --help, including `<a|b>` lists."""
    groups: dict[str, set[str]] = {}
    for line in help_stdout.splitlines():
        if not line.startswith("  "):
            continue
        parts = line[2:].split()
        if len(parts) < 2:
            continue
        head, sub = parts[0], parts[1]
        if not _is_token(head):
            continue
        if _is_token(sub):
            groups.setdefault(head, set()).add(sub)
            continue
        if sub.startswith("<") and "|" in sub:
            for alt in sub.strip("<>").split("|"):
                if _is_token(alt):
                    groups.setdefault(head, set()).add(alt)
    edit_subs = groups.get("edit")
    if edit_subs is None or len(edit_subs) < MIN_EDIT_SUBCOMMANDS:
        raise GateFail(f"edit subcommand harvest regression: {groups!r}")
    return groups


def check_real_commands(json_mode: bool, rhwp_bin: str | None) -> dict[str, Any]:
    """Check every extracted token against an explicitly selected binary."""
    binary = find_rhwp_binary(rhwp_bin)
    if binary is None:
        _say(json_mode, "rhwp binary: skip (absent)")
        return {
            "present": False,
            "ok": True,
            "skipped": "no rhwp binary",
            "checked": 0,
            "dead": [],
        }
    _say(json_mode, f"rhwp binary: {binary}")
    try:
        caps_stdout = _run_rhwp(binary, ["capabilities"])
        help_stdout = _run_rhwp(binary, ["--help"])
    except FileNotFoundError:
        _say(json_mode, "rhwp binary: skip (absent)")
        return {
            "present": False,
            "ok": True,
            "skipped": "no rhwp binary",
            "checked": 0,
            "dead": [],
        }
    known = parse_known_commands(caps_stdout, help_stdout)
    groups = parse_group_subcommands(help_stdout)
    dead: list[str] = []
    seen: set[str] = set()
    checked = 0
    for dir_path in list_skill_dirs():
        name = dir_path.name
        refs = referenced_commands(read_skill(dir_path))
        for tok, sub in refs:
            checked += 1
            if tok not in known:
                msg = f"{name}: `rhwp {tok}`"
                if msg not in seen:
                    seen.add(msg)
                    dead.append(msg)
                    _say(json_mode, f"rhwp {name}: FAIL: unknown `{tok}`")
                continue
            if sub is None:
                continue
            subs = groups.get(tok)
            if subs is None or sub in subs:
                continue
            listed = ",".join(sorted(subs))
            msg = f"{name}: `rhwp {tok} {sub}` (real subs: {listed})"
            if msg not in seen:
                seen.add(msg)
                dead.append(msg)
                _say(json_mode, f"rhwp {name}: FAIL: unknown `{tok} {sub}`")
    if dead:
        raise GateFail("skills reference unknown rhwp commands:\n  " + "\n  ".join(dead))
    _say(json_mode, f"rhwp commands: pass ({len(known)} known, {checked} refs)")
    return {
        "present": True,
        "ok": True,
        "binary": str(binary),
        "knownCount": len(known),
        "checked": checked,
        "dead": [],
    }


def _as_catalog_entry(skill_id: str, payload: Any) -> dict[str, Any] | None:
    if isinstance(payload, str):
        return {"id": skill_id, "path": payload}
    if not isinstance(payload, dict):
        return None
    path = payload.get("path") or payload.get("skillPath") or payload.get("skill")
    if not path:
        return None
    return {
        "id": str(payload.get("id") or skill_id),
        "path": str(path),
        "triggers": payload.get("triggers") or [],
    }


def load_catalog_entries() -> list[dict[str, Any]] | None:
    if not CATALOG_JSON.is_file():
        return None
    try:
        raw = json.loads(CATALOG_JSON.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise GateFail(f"catalog.json unreadable: {exc}") from exc
    entries: list[dict[str, Any]] = []
    payload: Any = raw
    if isinstance(raw, dict) and "skills" in raw:
        payload = raw["skills"]
    if isinstance(payload, dict):
        for key, value in payload.items():
            if key in ("schemaVersion", "catalogVersion", "version"):
                continue
            entry = _as_catalog_entry(str(key), value)
            if entry is None:
                raise GateFail(f"catalog skill {key!r} has no path")
            entries.append(entry)
        return entries
    if isinstance(payload, list):
        for item in payload:
            if not isinstance(item, dict):
                raise GateFail("catalog skills list contains a non-object")
            skill_id = item.get("id") or item.get("skill") or item.get("name")
            if not skill_id:
                raise GateFail("catalog skill entry missing id")
            entry = _as_catalog_entry(str(skill_id), item)
            if entry is None:
                raise GateFail(f"catalog skill {skill_id!r} has no path")
            entries.append(entry)
        return entries
    raise GateFail("catalog.json has no skills map or list")


def check_catalog_paths(json_mode: bool) -> dict[str, Any]:
    entries = load_catalog_entries()
    if entries is None:
        _say(json_mode, "catalog.json: skip (absent)")
        return {"present": False, "ok": True, "checked": 0, "missing": []}
    missing: list[str] = []
    for entry in entries:
        rel = entry["path"].replace("\\", "/")
        path = Path(rel)
        if not path.is_absolute():
            path = REPO / path
        if path.is_file():
            _say(json_mode, f"catalog {entry['id']}: pass")
        else:
            missing.append(rel)
            _say(json_mode, f"catalog {entry['id']}: FAIL: missing path {rel}")
            raise GateFail(f"catalog skill {entry['id']} path does not exist: {rel}")
    return {"present": True, "ok": True, "checked": len(entries), "missing": missing}


def _normalize_probe(text: object) -> str | None:
    if not isinstance(text, str):
        return None
    cleaned = " ".join(text.split())
    return cleaned or None


def _dedicated_probes(skill_id: str) -> list[str]:
    """Return the dedicated 3-probe tuple. Catalog skills never fall back."""
    if skill_id not in PROBES:
        raise GateFail(
            f"new skill {skill_id} has no PROBES; add 3 unique route requests"
        )
    raw = PROBES[skill_id]
    if not isinstance(raw, (tuple, list)):
        raise GateFail(
            f"new skill {skill_id} has no PROBES; add 3 unique route requests"
        )
    seen: list[str] = []
    for item in raw:
        cleaned = _normalize_probe(item)
        if cleaned is None:
            continue
        if cleaned not in seen:
            seen.append(cleaned)
    if len(seen) != ROUTE_PROBES:
        raise GateFail(
            f"new skill {skill_id} has no PROBES; add 3 unique route requests"
        )
    return seen


def require_catalog_probes(json_mode: bool) -> dict[str, Any]:
    """Every catalog skill id must have a dedicated 3-probe tuple in PROBES."""
    entries = load_catalog_entries()
    if entries is None:
        _say(json_mode, "catalog probes: skip (catalog absent)")
        return {"present": False, "ok": True, "checked": 0, "missing": []}
    missing: list[str] = []
    seen_ids: set[str] = set()
    for entry in entries:
        skill_id = entry["id"]
        if skill_id in seen_ids:
            continue
        seen_ids.add(skill_id)
        try:
            _dedicated_probes(skill_id)
        except GateFail:
            missing.append(skill_id)
            _say(
                json_mode,
                f"catalog probes {skill_id}: FAIL: no dedicated 3-probe tuple",
            )
    if missing:
        msgs = [
            f"new skill {sid} has no PROBES; add 3 unique route requests"
            for sid in missing
        ]
        raise GateFail(msgs[0] if len(msgs) == 1 else " ".join(msgs))
    _say(json_mode, f"catalog probes: pass ({len(seen_ids)} skills)")
    return {"present": True, "ok": True, "checked": len(seen_ids), "missing": []}


def _probe_requests(skill_id: str, entry: dict[str, Any] | None = None) -> list[str]:
    # Catalog skills must use the dedicated PROBES tuple. Triggers and slug
    # generators are not a substitute for unique route tests.
    del entry
    return _dedicated_probes(skill_id)


def _cli_env() -> dict[str, str]:
    env = dict(os.environ)
    env["PYTHONUTF8"] = "1"
    env["PYTHONIOENCODING"] = "utf-8"
    return env


def _parse_one_json(stdout: str) -> Any:
    text = stdout.strip()
    if not text:
        raise GateFail("route.py stdout empty; expected one JSON object")
    decoder = json.JSONDecoder()
    try:
        obj, idx = decoder.raw_decode(text)
    except json.JSONDecodeError as exc:
        preview = text[:240].replace("\n", "\\n")
        raise GateFail(f"route.py stdout is not JSON: {exc}: {preview!r}") from exc
    leftover = text[idx:].strip()
    if leftover:
        preview = leftover[:120].replace("\n", "\\n")
        raise GateFail(f"route.py stdout had trailing non-JSON: {preview!r}")
    return obj


def _valid_envelope(obj: Any) -> None:
    if not isinstance(obj, dict):
        raise GateFail(f"route envelope is {type(obj).__name__}, expected object")
    missing = [key for key in ENVELOPE_KEYS if key not in obj]
    if missing:
        raise GateFail(f"route envelope missing keys: {missing}")


def probe_router(json_mode: bool) -> dict[str, Any]:
    if not ROUTE_PY.is_file():
        _say(json_mode, "route.py: skip (absent)")
        return {"present": False, "ok": True, "probes": []}
    entries = load_catalog_entries()
    if not entries:
        _say(json_mode, "route.py: skip (no catalog skill ids)")
        return {"present": True, "ok": True, "probes": [], "skipped": "no catalog"}
    probes: list[dict[str, Any]] = []
    # Preserve first-seen id order; skip duplicate ids.
    seen_ids: set[str] = set()
    ordered: list[dict[str, Any]] = []
    for entry in entries:
        if entry["id"] in seen_ids:
            continue
        seen_ids.add(entry["id"])
        ordered.append(entry)
    ordered.sort(key=lambda item: item["id"])
    for entry in ordered:
        skill_id = entry["id"]
        requests = _probe_requests(skill_id, entry)
        for probe_i, request in enumerate(requests, start=1):
            label = f"route {skill_id} [{probe_i}/{ROUTE_PROBES}] {request!r}"
            try:
                proc = subprocess.run(
                    [sys.executable, str(ROUTE_PY), request, "--json"],
                    cwd=str(REPO),
                    capture_output=True,
                    text=True,
                    encoding="utf-8",
                    errors="replace",
                    timeout=ROUTE_TIMEOUT_SEC,
                    env=_cli_env(),
                    check=False,
                )
            except subprocess.TimeoutExpired as exc:
                _say(json_mode, f"{label}: FAIL: hung after {ROUTE_TIMEOUT_SEC}s")
                raise GateFail(
                    f"{skill_id} route probe hung after {ROUTE_TIMEOUT_SEC}s"
                ) from exc
            if proc.returncode != 0:
                err = (proc.stderr or "").strip()[:400]
                _say(json_mode, f"{label}: FAIL: exit {proc.returncode}")
                raise GateFail(
                    f"{skill_id} route.py exit {proc.returncode} stderr={err!r}"
                )
            try:
                obj = _parse_one_json(proc.stdout)
                _valid_envelope(obj)
            except GateFail as exc:
                _say(json_mode, f"{label}: FAIL: {exc}")
                raise
            selected = ""
            selection = obj.get("skillSelection")
            if isinstance(selection, list) and selection and isinstance(selection[0], dict):
                selected = str(selection[0].get("id") or "")
            elif isinstance(selection, list) and selection and isinstance(selection[0], str):
                selected = selection[0]
            if selected != skill_id:
                _say(
                    json_mode,
                    f"{label}: FAIL: selected {selected!r}, expected {skill_id!r}",
                )
                raise GateFail(
                    f"{skill_id} probe {request!r} selected {selected!r}; "
                    "use a unique request that this skill wins"
                )
            probes.append(
                {
                    "skill": skill_id,
                    "request": request,
                    "ok": True,
                    "selected": selected,
                }
            )
            _say(json_mode, f"{label}: pass")
    return {"present": True, "ok": True, "probes": probes}


def run_gate(json_mode: bool, rhwp_bin: str | None = None) -> dict[str, Any]:
    envelope: dict[str, Any] = {
        "schemaVersion": SCHEMA_VERSION,
        "kind": "skillGate",
        "ok": False,
        "exit": 1,
        "scans": SCAN_TIMES,
        "skills": [],
        "catalog": {"present": CATALOG_JSON.is_file(), "ok": False},
        "route": {"present": ROUTE_PY.is_file(), "ok": False},
        "rhwp": {"present": find_rhwp_binary(rhwp_bin) is not None, "ok": False},
    }
    envelope["skills"] = scan_skills(json_mode)
    envelope["rhwp"] = check_real_commands(json_mode, rhwp_bin)
    envelope["catalog"] = check_catalog_paths(json_mode)
    require_catalog_probes(json_mode)
    envelope["route"] = probe_router(json_mode)
    envelope["ok"] = True
    envelope["exit"] = 0
    n_skills = len(envelope["skills"])
    n_catalog = envelope["catalog"].get("checked", 0)
    n_probes = len(envelope["route"].get("probes") or [])
    rhwp_info = envelope["rhwp"]
    if rhwp_info.get("present"):
        rhwp_summary = str(rhwp_info.get("knownCount", 0))
    else:
        rhwp_summary = "skip"
    _say(
        json_mode,
        f"OK: {n_skills} skills x {SCAN_TIMES} scans, "
        f"catalog={n_catalog}, route_probes={n_probes}, rhwp={rhwp_summary}",
    )
    return envelope


def _parse_argv(argv: list[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="gate_new_skill.py",
        description="Gate for new or edited .agents/skills/*/SKILL.md files.",
        add_help=False,
    )
    parser.add_argument("--json", action="store_true", help="write a summary envelope to stdout")
    parser.add_argument(
        "--rhwp-bin",
        metavar="PATH",
        help="check live commands against this explicitly built rhwp binary",
    )
    parser.add_argument("-h", "--help", action="store_true", help="show usage")
    args, unknown = parser.parse_known_args(argv)
    if args.help or unknown:
        raise UsageError(USAGE)
    return args


def main(argv: list[str] | None = None) -> int:
    _configure_stdio()
    try:
        args = _parse_argv(argv)
    except UsageError as exc:
        print(str(exc), file=sys.stderr)
        return 2
    json_mode = bool(args.json)
    envelope: dict[str, Any] | None = None
    exit_code = 1
    try:
        envelope = run_gate(json_mode, args.rhwp_bin)
        exit_code = 0
    except GateFail as exc:
        _say(json_mode, f"FAIL: {exc}")
        envelope = {
            "schemaVersion": SCHEMA_VERSION,
            "kind": "skillGate",
            "ok": False,
            "exit": 1,
            "error": str(exc),
            "scans": SCAN_TIMES,
        }
        exit_code = 1
    if json_mode and envelope is not None:
        sys.stdout.write(json.dumps(envelope, ensure_ascii=False, indent=2))
        sys.stdout.write("\n")
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
