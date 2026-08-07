#!/usr/bin/env python3
"""에이전트 표면 기여자용 선검사 — 재작업을 막는다.

## 왜 있는가

에이전트 표면(#3630, #3719)에 명령을 추가할 때 반복되는 실패 패턴이 있다.

    코드 작성 → cargo build (6분) → 드리프트 가드 1건 실패 → 고침
             → cargo build (6분) → 다음 가드 1건 실패 → 고침
             → cargo build (6분) → 또 다음 …

가드는 저마다 독립이라 **한 번에 하나씩만** 드러난다. 실제로 이 순서로 세 번 연속
걸린 적이 있고(누락된 MCP 도구 → 누락된 `required` 배열 → 배선 안 된 속성),
빌드만 18분을 썼다.

이 스크립트는 **모든 검사를 한 번에 돌리고 실패를 전부 모아서 보고**한다.
검사 대부분은 빌드가 필요 없고, 빌드가 필요한 것도 이미 만들어진 바이너리를 쓴다.

## 쓰는 법

    # 코드를 쓰는 중에 (빌드 불필요, 1초)
    py tools/agent_preflight.py --static-only

    # 커밋·푸시 직전에 (빌드된 바이너리로 가드 전부)
    py tools/agent_preflight.py --bin target/release/rhwp

    # 커밋 범위를 선언하면 오염까지 검사한다
    py tools/agent_preflight.py --scope src/main.rs --scope tests/my_contract.rs

큐 규율 검사(잔량 캡·동일 이슈 중복 PR·미할당 착수)는 gh 가 있고 인증돼 있으면
자동으로 함께 돈다 — 결과는 **경고 전용**이라 종료 코드에 영향이 없고, 네트워크가
없으면 조용히 건너뛴다. `--no-network` 로 명시적으로 끌 수 있다.
(병렬 세션 규약 §8-6, #3914 수용 기준 2)

종료 코드: 0 통과 / 1 실패 있음 / 2 사용법 오류 — 경고는 코드에 반영되지 않는다
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

# 자기 출력 인코딩 고정 — Windows 콘솔(cp949)에서 보고 메시지의 em-dash 가
# UnicodeEncodeError 를 내며 검사기 자신을 죽인다(건너뜀·경고 출력 경로에서 실측).
# 자식 프로세스는 run() 이 PYTHONIOENCODING 으로 고정하지만 자기 stdout 은 별개다.
for _s in (sys.stdout, sys.stderr):
    if hasattr(_s, "reconfigure"):
        _s.reconfigure(encoding="utf-8", errors="replace")

# ── 검사 결과 ────────────────────────────────────────────────────────────────


@dataclass
class Finding:
    check: str
    detail: str
    fix: str


@dataclass
class Report:
    findings: list[Finding] = field(default_factory=list)
    skipped: list[tuple[str, str]] = field(default_factory=list)
    passed: list[str] = field(default_factory=list)
    warnings: list[tuple[str, str]] = field(default_factory=list)

    def fail(self, check: str, detail: str, fix: str) -> None:
        self.findings.append(Finding(check, detail, fix))

    def skip(self, check: str, why: str) -> None:
        self.skipped.append((check, why))

    def ok(self, check: str) -> None:
        self.passed.append(check)

    def warn(self, check: str, detail: str) -> None:
        """경고 전용 채널 — 종료 코드에 영향을 주지 않는다.

        큐 규율(§11 절) 같은 검사의 실패는 "계약 위반(고쳐야 함)"이 아니라
        "규율 이탈(판단 필요)"이다. fail 로 내면 캡("10건 내외")이 하드 리밋으로
        둔갑하고, 곧 --static-only 로 우회당한다.
        """
        self.warnings.append((check, detail))


def run(cmd: list[str], cwd: Path | None = None, stdin_data: str | None = None):
    """UTF-8 로 고정해서 실행한다. Windows 콘솔 코드페이지에 휘둘리지 않기 위해서다."""
    env = dict(os.environ, PYTHONIOENCODING="utf-8")
    return subprocess.run(
        cmd,
        cwd=cwd,
        input=stdin_data,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        env=env,
        timeout=180,
    )


# ── 1. 오염 검사 (빌드 불필요) ───────────────────────────────────────────────

# 커밋에 절대 들어가면 안 되는 경로. `git add -A` 한 번에 워크트리 전체나
# 빌드 산출물이 딸려 들어가 PR 이 통째로 깨진 적이 있다.
NEVER_COMMIT = (
    ".agentwt/",
    "target/",
    "node_modules/",
    "output/",
    ".git/",
)

# 소스가 아닌데 큰 파일. 샘플 문서가 실수로 섞이면 리뷰가 불가능해진다.
BIG_FILE_BYTES = 100 * 1024
SOURCE_SUFFIXES = {".rs", ".py", ".ts", ".js", ".md", ".toml", ".json", ".yml", ".yaml"}


def check_contamination(repo: Path, scope: list[str], rep: Report) -> None:
    check = "오염 — 커밋 범위 밖 파일"
    r = run(["git", "diff", "--cached", "--name-only"], cwd=repo)
    if r.returncode != 0:
        rep.skip(check, "git diff --cached 실패")
        return
    staged = [p for p in r.stdout.splitlines() if p.strip()]
    if not staged:
        rep.skip(check, "staged 파일 없음 — `git add` 뒤에 다시 돌려라")
        return

    for path in staged:
        for bad in NEVER_COMMIT:
            if path.startswith(bad) or f"/{bad}" in path:
                rep.fail(
                    check,
                    f"{path} — 절대 커밋 금지 경로({bad})",
                    "`git restore --staged` 로 빼라. `git add -A` 대신 파일을 명시해서 add 하라.",
                )

        full = repo / path
        if full.exists() and full.suffix.lower() not in SOURCE_SUFFIXES:
            size = full.stat().st_size
            if size > BIG_FILE_BYTES:
                rep.fail(
                    check,
                    f"{path} — 비소스 대용량 {size:,} bytes",
                    "의도한 픽스처가 맞는지 확인하라. 아니면 staged 에서 빼라.",
                )

    if scope:
        allowed = tuple(s.replace("\\", "/") for s in scope)
        for path in staged:
            if not path.startswith(allowed):
                rep.fail(
                    check,
                    f"{path} — 선언한 --scope 밖",
                    "다른 작업의 파일이 섞였다. staged 에서 빼거나 --scope 를 정확히 선언하라.",
                )

    if not any(f.check == check for f in rep.findings):
        rep.ok(f"{check} ({len(staged)}개 staged)")


# ── 2. doc 주석 오배치 (빌드 불필요) ─────────────────────────────────────────

DOC_LINE = re.compile(r"^\s*///")
ITEM_LINE = re.compile(r"^\s*(pub\s+)?(async\s+)?(fn|struct|enum|trait|const|static|mod|type)\s+(\w+)")
# 주석 안에서 언급된 함수형 식별자. `cmd_foo` 나 백틱 안의 snake_case 를 본다.
MENTION = re.compile(r"`(\w+)`|\b(cmd_\w+)\b")


def check_doc_placement(repo: Path, files: list[Path], rep: Report) -> None:
    """doc 주석과 그 아래 아이템의 이름이 어긋나는지 본다.

    실제로 겪은 사고: 새 함수를 **기존 함수의 doc 주석과 그 함수 사이에** 끼워 넣어,
    주석이 엉뚱한 함수에 붙었다. rustdoc 은 연속된 doc 블록을 이어 붙이므로
    컴파일러도 clippy 도 아무 말을 하지 않는다.
    """
    check = "doc 주석 오배치"
    hits = 0
    for path in files:
        if path.suffix != ".rs":
            continue
        try:
            lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
        except OSError:
            continue

        block: list[str] = []
        block_start = 0
        for i, line in enumerate(lines):
            if DOC_LINE.match(line):
                if not block:
                    block_start = i + 1
                block.append(line)
                continue
            if not block:
                continue
            m = ITEM_LINE.match(line)
            if m:
                item = m.group(4)
                text = "\n".join(block)
                named = {a or b for a, b in MENTION.findall(text) if (a or b)}
                # 주석이 어떤 함수형 이름을 명시했는데 그게 붙은 아이템과 다르면 의심.
                fnish = {n for n in named if n.startswith("cmd_") or n == item}
                if fnish and item not in fnish:
                    rep.fail(
                        check,
                        f"{path.relative_to(repo)}:{block_start} — 주석은 {sorted(fnish)} 를 "
                        f"말하는데 붙은 아이템은 `{item}`",
                        "함수를 남의 doc 주석과 그 함수 사이에 끼워 넣지 않았는지 확인하라. "
                        "rustdoc 은 연속 doc 블록을 이어 붙여서 어떤 린트도 잡지 못한다.",
                    )
                    hits += 1
            block = []
    if hits == 0:
        rep.ok(check)


# ── 3. ReDoS 정규식 (빌드 불필요) ────────────────────────────────────────────

# ① 지수형: `(…+)+` — 그룹 안팎에 수량자가 겹친다.
EXPONENTIAL = re.compile(r"\((?:\?:)?[^()]*[+*]\)[+*]")
# ② 다항형: `([A-Z]+)([A-Z][a-z])` — 수량자가 붙은 문자class 그룹 바로 뒤에
#    **겹치는 문자class** 로 시작하는 그룹이 온다. 첫 그룹이 어디까지 먹을지
#    정해지지 않아 백트래킹이 입력 길이의 제곱으로 커진다.
#    CodeQL js/polynomial-redos 가 잡는 형태이고, 이 저장소에서 실제로 한 번 걸렸다.
POLYNOMIAL = re.compile(r"\((?:\?:)?\[([^\]]+)\][+*]\)\s*\((?:\?:)?\[([^\]]+)\]")


def _classes_overlap(a: str, b: str) -> bool:
    """두 문자class 본문이 겹치는지 대략 본다.

    범위(`A-Z`)는 범위째로, 낱문자는 낱문자로 비교한다. 정확한 집합 연산까지 갈
    필요는 없다 — 겹칠 가능성이 있으면 사람이 보게 하는 게 목적이다.
    """
    def atoms(s: str) -> set[str]:
        out, i = set(), 0
        while i < len(s):
            if i + 2 < len(s) and s[i + 1] == "-":
                out.add(s[i : i + 3])
                i += 3
            else:
                out.add(s[i])
                i += 1
        return out

    return bool(atoms(a) & atoms(b))


def check_redos(repo: Path, files: list[Path], rep: Report) -> None:
    check = "ReDoS — 백트래킹 폭발 정규식"
    hits = 0
    for path in files:
        if path.suffix not in {".rs", ".py", ".ts", ".js"}:
            continue
        # 자기 자신은 스캔하지 않는다 — 검출 패턴(EXPONENTIAL·POLYNOMIAL·예시 문자열)이
        # 문자 그대로 들어 있어, 이 파일이 변경 집합에 들면 자기 검출부를 잡는
        # 자기스캔 오탐이 난다(2026-08-06 실측: 검출기 리터럴 3건).
        if path.name == "agent_preflight.py":
            continue
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for i, line in enumerate(text.splitlines(), 1):
            rel = path.relative_to(repo)
            m = EXPONENTIAL.search(line)
            if m:
                rep.fail(
                    check,
                    f"{rel}:{i} — 지수형 {m.group(0)!r}",
                    "그룹 안팎의 수량자를 하나로 합쳐라. 입력 길이에 선형이어야 한다.",
                )
                hits += 1
                continue
            m = POLYNOMIAL.search(line)
            if m and _classes_overlap(m.group(1), m.group(2)):
                rep.fail(
                    check,
                    f"{rel}:{i} — 다항형 {m.group(0)!r} "
                    f"(class `{m.group(1)}` 와 `{m.group(2)}` 가 겹친다)",
                    "앞 그룹의 수량자를 없애라. 예: `([A-Z]+)([A-Z][a-z])` → `([A-Z])([A-Z][a-z])`. "
                    "치환 결과가 같은지 대표 입력으로 확인하고 근거를 남겨라.",
                )
                hits += 1
    if hits == 0:
        rep.ok(check)


# ── 4. rustfmt (빌드 불필요) ─────────────────────────────────────────────────


def check_fmt(repo: Path, files: list[Path], rep: Report) -> None:
    """`cargo fmt --all` 은 이 저장소 규모에서 Windows 인자 길이 한계(32K)에 걸려
    `os error 206` 으로 **통째로 실패**한다. 그러면 "Diff in" 줄이 0개라 통과처럼 보인다.
    실패를 통과로 오독한 사고가 두 번 있었다. 그래서 파일을 나눠 직접 rustfmt 를 부른다.
    """
    check = "rustfmt"
    rs = [p for p in files if p.suffix == ".rs" and p.exists()]
    if not rs:
        rep.skip(check, "변경된 .rs 없음")
        return
    cfg = repo / "rustfmt.toml"
    base = ["rustfmt", "--edition", "2021", "--check", "--config", "newline_style=Auto"]
    if cfg.exists():
        base += ["--config-path", str(cfg)]

    dirty: list[str] = []
    hard_error = None
    for i in range(0, len(rs), 10):
        chunk = [str(p) for p in rs[i : i + 10]]
        try:
            r = run(base + chunk, cwd=repo)
        except (OSError, subprocess.SubprocessError) as exc:
            hard_error = str(exc)
            break
        if r.returncode not in (0, 1):
            hard_error = (r.stderr or r.stdout).strip()[:200]
            break
        for line in r.stdout.splitlines():
            if line.startswith("Diff in "):
                dirty.append(line[len("Diff in ") :].split(" at ")[0])

    if hard_error:
        rep.fail(
            check,
            f"rustfmt 자체가 실패했다: {hard_error}",
            "통과가 아니라 **검사 불능**이다. 0건을 통과로 읽지 마라.",
        )
        return
    if dirty:
        uniq = sorted(set(dirty))
        rep.fail(
            check,
            f"{len(uniq)}개 파일 미정렬: " + ", ".join(os.path.basename(d) for d in uniq[:6]),
            "`rustfmt --edition 2021 --config-path rustfmt.toml <파일>` 로 정렬하라. "
            "`cargo fmt --all` 은 이 저장소에서 os error 206 으로 실패한다.",
        )
    else:
        rep.ok(f"{check} ({len(rs)}개 파일)")


# ── 5~9. 드리프트 가드 (바이너리 필요, 전부 한 번에) ─────────────────────────


def load_surface(binary: Path, rep: Report):
    """capabilities 두 축과 help 를 한 번에 읽는다."""
    caps = run([str(binary), "capabilities"])
    mcp = run([str(binary), "capabilities", "--mcp"])
    helptext = run([str(binary), "--help"])
    try:
        caps_j = json.loads(caps.stdout)
        mcp_j = json.loads(mcp.stdout)
    except json.JSONDecodeError as exc:
        rep.fail(
            "표면 읽기",
            f"capabilities 출력이 JSON 이 아니다: {exc}",
            "바이너리가 최신인지 확인하라. `cargo build --release --bin rhwp`",
        )
        return None
    return caps_j, mcp_j, (helptext.stdout + helptext.stderr)


def check_mcp_input_schema(mcp_j, rep: Report) -> None:
    check = "MCP inputSchema 모양"
    bad = 0
    for t in mcp_j.get("tools", []):
        s = t.get("inputSchema")
        name = t.get("name", "?")
        if not isinstance(s, dict):
            rep.fail(check, f"{name} — inputSchema 없음", "inputSchema 를 선언하라.")
            bad += 1
            continue
        if s.get("type") != "object":
            rep.fail(check, f'{name} — type 이 "object" 가 아니다', 'type 을 "object" 로.')
            bad += 1
        if not isinstance(s.get("properties"), dict):
            rep.fail(check, f"{name} — properties 없음", "properties 객체를 선언하라.")
            bad += 1
        if not isinstance(s.get("required"), list):
            rep.fail(
                check,
                f"{name} — required 배열 없음",
                "필수 인자가 없어도 `required: []` 를 **반드시** 넣어라. "
                "이것만으로 가드 1회·빌드 6분을 날린 적이 있다.",
            )
            bad += 1
    if bad == 0:
        rep.ok(f"{check} ({len(mcp_j.get('tools', []))}개 도구)")


PLACEHOLDER = re.compile(r"\{(\w+)\}")

# argv 로 배선되지 않는 속성(stdin 으로 전달된다). 이 목록의 **권위는 계약 테스트**이므로
# 여기에 베끼지 않고 그 파일에서 읽는다 — 베끼면 언젠가 어긋나고, 어긋나면 이 선검사가
# 실제 가드와 다른 말을 하게 된다. 그게 정확히 이 스크립트가 없애려는 재작업이다.
MCP_CONTRACT = Path("tests") / "mcp_server_contract.rs"
CLI_CONTRACT = Path("tests") / "cli_json_contract.rs"


def _read(repo: Path, rel: Path) -> str:
    try:
        return (repo / rel).read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


def load_allowlist(repo: Path, rel: Path, const: str, fallback: set[str], rep: Report) -> set[str]:
    """계약 테스트의 `const NAME: &[(&str, &str)] = &[…]` 허용목록을 그 파일에서 읽는다.

    베끼지 않는 이유: 베낀 목록은 언젠가 원본과 어긋나고, 어긋나면 이 선검사가 실제
    가드와 다른 말을 한다. 헛울리는 검사기는 곧 무시당하고, 무시당하는 검사기는
    없느니만 못하다 — 그게 정확히 이 스크립트가 없애려는 재작업이다.
    """
    text = _read(repo, rel)
    if not text:
        rep.skip(f"{const} 동기화", f"{rel} 를 못 읽어 기본값 사용")
        return set(fallback)
    m = re.search(rf"const\s+{const}[^=]*=\s*&\[(.*?)\];", text, re.S)
    if not m:
        rep.skip(f"{const} 동기화", "상수를 못 찾아 기본값 사용 — 계약 테스트가 바뀌었나?")
        return set(fallback)
    keys = {mm.group(1) for mm in re.finditer(r'\(\s*"([^"]+)"\s*,', m.group(1))}
    return keys or set(fallback)


def load_mcp_exclusions(repo: Path, rep: Report) -> set[str]:
    """`capabilities_mcp_covers_every_json_command` 안의 인라인 제외를 읽는다.

    상수가 아니라 `.filter(|n| *n != "capabilities")` 형태로 사유 주석과 함께 박혀 있다.
    (`capabilities` 는 도구가 아니라 도구 목록의 원천이고, `dump-pages` 는 진단 계약만
    노출하기로 한 결정이다.)
    """
    fallback = {"capabilities", "dump-pages"}
    text = _read(repo, CLI_CONTRACT)
    if not text:
        rep.skip("MCP 제외 목록 동기화", f"{CLI_CONTRACT} 를 못 읽어 기본값 사용")
        return fallback
    m = re.search(
        r"fn\s+capabilities_mcp_covers_every_json_command\s*\(\s*\)\s*\{(.*?)\n\}", text, re.S
    )
    if not m:
        rep.skip("MCP 제외 목록 동기화", "가드 함수를 못 찾아 기본값 사용")
        return fallback
    names = set(re.findall(r'\*n\s*!=\s*"([^"]+)"', m.group(1)))
    return names or fallback


def check_property_wiring(mcp_j, non_argv: set[str], rep: Report) -> None:
    """선언한 입력 속성이 전부 CLI 인자에 배선됐는지.

    배선 경로는 넷뿐이다:
      1) `cli.args` 의 `{key}` 자리표시자
      2) `cli.optionalArgs[].when == key`
      3) `cli.passwordStdin.argument == key`
      4) NON_ARGV_PROPERTIES — argv 가 아니라 stdin 으로 흘려 넣는 속성
    """
    check = "선언 속성 ↔ CLI 배선"
    bad = 0
    for t in mcp_j.get("tools", []):
        name = t.get("name", "?")
        props = set((t.get("inputSchema") or {}).get("properties", {}))
        cli = t.get("cli") or {}
        wired: set[str] = set()
        for a in cli.get("args", []):
            wired |= set(PLACEHOLDER.findall(str(a)))
        for opt in cli.get("optionalArgs", []) or []:
            if opt.get("when"):
                wired.add(opt["when"])
            for a in opt.get("args", []):
                wired |= set(PLACEHOLDER.findall(str(a)))
        ps = cli.get("passwordStdin") or {}
        if ps.get("argument"):
            wired.add(ps["argument"])

        orphan = props - wired - non_argv
        if orphan:
            rep.fail(
                check,
                f"{name} — 선언만 하고 배선 안 됨: {sorted(orphan)}",
                "`optionalArgs` 에 `{\"when\": \"<속성>\", \"args\": [\"--플래그\"]}` 를 추가하거나 "
                "`args` 에 `{<속성>}` 자리표시자를 넣어라. 선언과 배선은 항상 같이 간다.",
            )
            bad += 1
    if bad == 0:
        rep.ok(check)


def command_map(caps_j) -> dict[str, dict]:
    """`capabilities.commands` 는 **객체 배열**이다(각 원소에 `name` 키).
    dict 로 가정했다가 전 검사가 조용히 건너뛴 적이 있어 여기서 한 번만 정규화한다."""
    cmds = caps_j.get("commands")
    if isinstance(cmds, dict):  # 형태가 바뀌어도 견디게
        return {k: v for k, v in cmds.items() if isinstance(v, dict)}
    if isinstance(cmds, list):
        return {c["name"]: c for c in cmds if isinstance(c, dict) and c.get("name")}
    return {}


def check_help_coverage(caps_j, helptext: str, hidden: set[str], rep: Report) -> None:
    check = "capabilities ↔ --help 상호 커버"
    cmds = command_map(caps_j)
    if not cmds:
        rep.skip(check, "capabilities.commands 를 못 읽음")
        return
    missing = [
        c
        for c in cmds
        if c not in hidden and not re.search(rf"(^|\s){re.escape(c)}(\s|$)", helptext, re.M)
    ]
    if missing:
        rep.fail(
            check,
            f"capabilities 에 있는데 --help 에 없음: {sorted(missing)}",
            "help 텍스트에 명령 줄을 추가하라. 두 축은 항상 같이 늘어야 한다.",
        )
    else:
        rep.ok(f"{check} ({len(cmds)}개 명령)")


def check_json_has_mcp_tool(caps_j, mcp_j, excluded: set[str], rep: Report) -> None:
    check = "--json 명령 ↔ MCP 도구 커버"
    cmds = command_map(caps_j)
    if not cmds:
        rep.skip(check, "capabilities.commands 를 못 읽음")
        return
    tool_cmds = {(t.get("cli") or {}).get("command") for t in mcp_j.get("tools", [])}
    missing = [
        name
        for name, spec in cmds.items()
        if spec.get("json") is True and name not in excluded and name not in tool_cmds
    ]
    if missing:
        rep.fail(
            check,
            f"--json 을 지원하는데 MCP 도구가 없음: {sorted(missing)}",
            "MCP 도구를 등록하라. `--json` 명령은 예외 없이 도구가 있어야 한다. "
            "정말 제외해야 하면 계약 테스트에 **사유와 함께** 제외를 명시하라.",
        )
    else:
        rep.ok(check)


def check_declared_flags_real(binary: Path, caps_j, rep: Report) -> None:
    """선언한 플래그가 실제로 수용되는지. 존재하지 않는 플래그를 문서에 적으면
    에이전트가 그대로 호출했다가 usage error 를 맞는다."""
    check = "선언 flags 실재"
    cmds = command_map(caps_j)
    if not cmds:
        rep.skip(check, "capabilities.commands 를 못 읽음")
        return
    bad = 0
    checked = 0
    for name, spec in cmds.items():
        for flag in spec.get("flags", []) or []:
            if not str(flag).startswith("--"):
                continue
            checked += 1
            # 인자 없이 플래그만 줘서 CLI 가 그 플래그를 아는지 본다.
            #
            # 판정은 **stderr 만** 본다. stdout 을 섞으면
            # `export-capabilities-schema --bare` 처럼 **종료 코드 사전을 출력하는 명령**이
            # 자기 출력 안의 "사용법 오류 (인자 없음, 알 수 없는 옵션/명령)" 에 걸려
            # 스스로를 미구현으로 신고한다(실측 오탐 2건).
            #
            # 종료 코드도 먼저 본다 — 플래그가 정상 동작하면 0 이다. 0 으로 끝났으면
            # CLI 가 받아들인 것이므로 진단 문자열이 어디에 있든 실패로 보지 않는다.
            r = run([str(binary), name, str(flag)])
            if r.returncode == 0:
                continue
            blob = r.stderr
            if "알 수 없는 옵션" in blob or "unknown option" in blob.lower():
                rep.fail(
                    check,
                    f"{name} {flag} — 선언됐지만 CLI 가 거부한다",
                    "capabilities 선언을 지우거나 CLI 에 플래그를 구현하라.",
                )
                bad += 1
    if bad == 0:
        rep.ok(f"{check} ({checked}개 플래그)")


# 실패 경로에서 stdout 봉투를 내는 것이 **의도된 설계**인 명령.
#
# `run` 은 계획 파싱이 성공한 뒤의 실패를 저널 봉투로 보고한다 —
# MCP `hwp_run_plan` 과 같은 저널을 공유하기 때문이다. 이 예외는
# `capabilities.jsonContract.failure` 에도 명시돼 있다. 이 목록은 전체 명령
# 실패-stdout sweep 이 구조화 저널을 stdout 오염으로 오판하지 않도록 한다.
FAILURE_STDOUT_ALLOWED = {"run"}


def check_failure_stdout_silent(binary: Path, caps_j, rep: Report) -> None:
    """실패 경로에서 stdout 이 0바이트여야 한다.

    반쪽 JSON 이 나가면 소비자가 파싱하다 죽거나, 더 나쁘게는 **잘린 값을 참으로
    읽는다.**

    예전에는 `info`·`export-text` 두 명령만 하드코딩해서 봤다. 그 사이 `--json`
    명령이 31개로 늘었고, 검사 밖에 있던 `bench` 가 exit 1 에 stdout 518바이트를
    흘리고 있었다(#3884 G1). **표면이 자라면 하드코딩한 표본은 자동으로 낡는다** —
    그래서 `capabilities` 에서 `--json` 명령을 전수로 받아 훑는다.
    """
    check = "실패 경로 stdout 0바이트"
    cmds = command_map(caps_j)
    if not cmds:
        rep.skip(check, "capabilities.commands 를 못 읽음")
        return

    missing = "존재하지_않는_파일_preflight.hwp"
    bad = 0
    checked = 0
    skipped_allowed = []

    for name, spec in sorted(cmds.items()):
        # `--json` 선언 명령이 1차 대상이지만, **선언하지 않은 명령도 실패하면
        # stdout 이 비어야 한다** — 규약은 봉투 유무가 아니라 실패 경로에 걸린다.
        # `bench` 가 정확히 그 사각이었다: json 미선언이라 --json 필터를 빠져나가면서
        # exit 1 에 stdout 518바이트를 흘리고 있었다(#3884 G1).
        if name in FAILURE_STDOUT_ALLOWED:
            skipped_allowed.append(name)
            continue
        # 없는 파일을 준다 — 문서를 받는 명령이면 런타임 실패, 아니면 사용법 오류다.
        # 어느 쪽이든 **실패이므로 stdout 은 비어야 한다.**
        for args in ([name, "--json", missing], [name, "--json"]):
            r = run([str(binary)] + args)
            checked += 1
            if r.returncode != 0 and r.stdout.strip():
                rep.fail(
                    check,
                    f"`rhwp {' '.join(args)}` → exit {r.returncode} 인데 "
                    f"stdout {len(r.stdout)}바이트",
                    "실패 경로에서는 stdout 에 아무것도 쓰지 마라. 진단은 stderr 로. "
                    "의도된 예외라면 FAILURE_STDOUT_ALLOWED 에 **사유와 함께** 넣어라.",
                )
                bad += 1

    if bad == 0:
        note = f" (제외 {','.join(skipped_allowed)})" if skipped_allowed else ""
        rep.ok(f"{check} ({checked}개 경로{note})")


def check_undeclared_commands_are_not_invisible(binary: Path, caps_j, helptext: str, rep: Report) -> None:
    """`capabilities` 에 `flags` 를 선언하지 않은 명령은 **모든 플래그 검사에서 사라진다.**

    `check_declared_flags_real` 은 선언한 플래그만 본다(선언 → 실물). 역방향이 없어서
    **아무것도 선언하지 않으면 검사를 통째로 피한다** — 선언을 안 하는 것이 가드를
    피하는 가장 쉬운 길이 되는 구조다.

    실제로 그 구멍으로 `dump` 가 미지 플래그를 exit 0 으로 무시하고 있었다(#3884 G2).

    여기서는 **그런 명령이 있다는 사실 자체**를 드러낸다. 자동으로 고칠 수는 없다 —
    진단 전용 명령을 자기서술에 넣을지, "자기서술 밖 명령"이라는 부류를 정의할지는
    판단이 필요하기 때문이다. 다만 **판단하지 않은 채 조용히 넘어가지는 않게** 한다.
    """
    check = "자기서술 밖 명령"
    cmds = command_map(caps_j)
    if not cmds:
        rep.skip(check, "capabilities.commands 를 못 읽음")
        return

    silent = sorted(
        name
        for name, spec in cmds.items()
        if not (spec.get("flags") or []) and spec.get("json") is not True
    )
    if not silent:
        rep.ok(check)
        return

    # 미지 플래그를 실제로 무시하는지 확인한다 — 선언이 없다고 다 문제인 것은 아니다.
    offenders = []
    for name in silent:
        r = run([str(binary), name, "--rhwp-preflight-bogus-flag"])
        if r.returncode == 0:
            offenders.append(name)

    if offenders:
        rep.fail(
            check,
            f"미지 플래그를 exit 0 으로 무시: {offenders}",
            "선언이 없으면 플래그 검사가 통째로 비켜 간다. capabilities 에 등재하거나, "
            "자기서술 밖 명령의 규약을 문서로 정의하라(#3884 G2).",
        )
    else:
        rep.ok(f"{check} ({len(silent)}개 미선언 명령, 미지 플래그는 거부함)")


# ── 11. 큐 규율 (네트워크, 경고 전용) ────────────────────────────────────────

# 병렬 세션 규약 §8-6 (mydocs/tech/autonomous_maintenance/parallel_session_protocol.md,
# 확정 이슈 #3914 수용 기준 2)의 구현이다. 기존 검사와 성질이 셋 다르다:
#   ① gh + 네트워크 + 인증 의존 — 없으면 조용히 건너뛴다. 네트워크 실패로 선검사가
#      통째로 빨개지면 다음 사람은 우회하고, 로컬 검사까지 함께 꺼진다.
#   ② 실패의 뜻이 "계약 위반"이 아니라 "규율 이탈 — 판단 필요"다. warn 전용이고
#      종료 코드에 영향이 없다(캡은 "10건 내외"지 하드 리밋이 아니다).
#   ③ 근거는 선언만 쓴다 — PR 이 스스로 적은 대상 이슈(제목·closes/refs 행)와
#      브랜치 이름의 이슈 번호. 파일 겹침 추정은 쓰지 않는다(오탐이 정상 작업을 막는다).

QUEUE_REPO = "edwardkim/rhwp"
QUEUE_CAP = 10
# 본문 전체의 #N 을 긁지 않는다 — 조망 이슈(#3907 등) 참조가 어디에나 있어 헛울린다.
# 제목의 #N 과 본문의 closes/fixes/resolves/refs 행만 "선언된 대상"으로 센다.
TARGET_IN_BODY = re.compile(r"(?im)^\s*(?:closes|fixes|resolves|refs)\s*:?\s+#(\d{3,5})\b")
TARGET_IN_TITLE = re.compile(r"#(\d{3,5})\b")
BRANCH_ISSUE = re.compile(r"^(?:task|wip/fix|fix|feat)[/-](\d{3,5})-")
# 잠금은 protocol §5-1의 명시 형식만 인정한다. 단순히 "착수"가 들어간 회고·인용·
# "아직 착수하지 않음"은 잠금이 아니므로, 부분 문자열 검사는 새 세션을 잘못 통과시킨다.
CLAIM_COMMENT = re.compile(r"(?m)^\s*착수합니다\s*[—-]\s*\S")


def check_queue_discipline(repo: Path, rep: Report) -> None:
    check = "큐 규율"
    try:
        probe = run(["gh", "auth", "status"], cwd=repo)
    except (FileNotFoundError, subprocess.TimeoutExpired, OSError):
        rep.skip(check, "gh 없음/응답 없음 — 네트워크 검사 건너뜀")
        return
    if probe.returncode != 0:
        rep.skip(check, "gh 미인증 — 네트워크 검사 건너뜀")
        return

    try:
        r = run(
            ["gh", "pr", "list", "--repo", QUEUE_REPO, "--state", "open",
             "--limit", "100", "--json", "number,title,body"],
            cwd=repo,
        )
        if r.returncode != 0:
            rep.skip(check, f"gh pr list 실패 — {(r.stderr or '').strip()[:80]}")
            return
        prs = json.loads(r.stdout or "[]")
    except Exception as e:  # 네트워크 검사는 어떤 이유로든 선검사를 막지 않는다
        rep.skip(check, f"조회 실패({type(e).__name__}) — 네트워크 검사 건너뜀")
        return

    warned = False

    # ① 잔량 — 열린 PR 캡(10건 내외, protocol §8-1)
    if len(prs) > QUEUE_CAP:
        warned = True
        rep.warn(
            check,
            f"열린 PR {len(prs)}건 — 캡 {QUEUE_CAP}건 내외 초과. "
            "신규 제출을 멈추고 머지·리베이스 우선 (#3719 §7, 슬롯이 빌 때까지 이슈로만 예약)",
        )

    # ② 동일 이슈 중복 — 선언된 대상 이슈가 겹치는 열린 PR (protocol §8-5 고신뢰 신호)
    by_issue: dict[str, list[int]] = {}
    for pr in prs:
        targets = set(TARGET_IN_TITLE.findall(pr.get("title") or ""))
        targets |= set(TARGET_IN_BODY.findall(pr.get("body") or ""))
        for n in targets:
            by_issue.setdefault(n, []).append(pr["number"])
    for n, nums in sorted(by_issue.items()):
        if len(nums) >= 2:
            warned = True
            pretty = " ".join(f"#{p}" for p in sorted(nums))
            rep.warn(
                check,
                f"이슈 #{n} 를 대상으로 선언한 열린 PR 이 {len(nums)}건 — {pretty}. "
                "중복이면 protocol §6 판정(포함 관계→검증 깊이→접합 비용), §7 인계 절차",
            )

    # ③ 미할당 착수 — 현재 브랜치가 가리키는 이슈에 잠금이 있는가 (protocol §5-1)
    br = run(["git", "symbolic-ref", "--short", "-q", "HEAD"], cwd=repo)
    m = BRANCH_ISSUE.match(br.stdout.strip()) if br.returncode == 0 else None
    if m is None:
        if not warned:
            rep.ok(f"{check} (열린 PR {len(prs)}건; 브랜치에 이슈 번호 없음 — 잠금 검사 생략)")
        return
    issue = m.group(1)
    try:
        r = run(
            ["gh", "issue", "view", issue, "--repo", QUEUE_REPO,
             "--json", "assignees,comments"],
            cwd=repo,
        )
        if r.returncode != 0:
            rep.skip(f"{check}·잠금", f"이슈 #{issue} 조회 실패 — {(r.stderr or '').strip()[:80]}")
            return
        data = json.loads(r.stdout or "{}")
    except Exception as e:
        rep.skip(f"{check}·잠금", f"조회 실패({type(e).__name__})")
        return
    assignees = [a.get("login", "") for a in data.get("assignees", []) if a.get("login")]
    claimed = any(
        CLAIM_COMMENT.search(c.get("body") or "") for c in data.get("comments", [])
    )
    if not assignees and not claimed:
        rep.warn(
            check,
            f"브랜치가 이슈 #{issue} 를 가리키는데 assignee 도 착수 코멘트도 없다 — "
            f'선점하라: gh issue comment {issue} --repo {QUEUE_REPO} '
            f'--body "착수합니다 — <범위>" (외부 기여자는 assignee 편집이 거부된다, protocol §5-1 실측)',
        )
    elif not warned:
        lock = ",".join(assignees) if assignees else "착수 코멘트"
        rep.ok(f"{check} (열린 PR {len(prs)}건; #{issue} 잠금={lock})")


# ── 엔트리 ───────────────────────────────────────────────────────────────────


def changed_files(repo: Path) -> list[Path]:
    out: set[str] = set()
    for args in (["diff", "--name-only", "HEAD"], ["diff", "--cached", "--name-only"]):
        r = run(["git"] + args, cwd=repo)
        if r.returncode == 0:
            out |= {p for p in r.stdout.splitlines() if p.strip()}
    return [repo / p for p in sorted(out)]


def main() -> int:
    ap = argparse.ArgumentParser(
        description="에이전트 표면 기여자용 선검사 — 모든 실패를 한 번에 보고한다",
    )
    ap.add_argument("--bin", help="빌드된 rhwp 바이너리 경로. 없으면 정적 검사만")
    ap.add_argument("--static-only", action="store_true", help="빌드 필요한 검사를 건너뛴다")
    ap.add_argument(
        "--scope",
        action="append",
        default=[],
        help="이 커밋이 건드려야 할 경로 접두사. 선언하면 그 밖의 staged 파일을 오염으로 본다",
    )
    ap.add_argument("--repo", default=".", help="저장소 루트 (기본: 현재 디렉터리)")
    ap.add_argument(
        "--no-network",
        action="store_true",
        help="큐 규율 검사(gh 필요)를 건너뛴다. 네트워크가 없으면 자동으로 건너뛰므로 보통 불필요",
    )
    args = ap.parse_args()

    repo = Path(args.repo).resolve()
    if not (repo / ".git").exists():
        print(f"오류: {repo} 는 git 저장소가 아니다", file=sys.stderr)
        return 2

    rep = Report()
    files = changed_files(repo)

    check_contamination(repo, args.scope, rep)
    check_doc_placement(repo, files, rep)
    check_redos(repo, files, rep)
    check_fmt(repo, files, rep)
    if not args.no_network:
        check_queue_discipline(repo, rep)

    if not args.static_only:
        binary = Path(args.bin) if args.bin else repo / "target" / "release" / "rhwp"
        if not binary.exists():
            for cand in (binary.with_suffix(".exe"), repo / "target" / "release" / "rhwp.exe"):
                if cand.exists():
                    binary = cand
                    break
        if not binary.exists():
            rep.skip("드리프트 가드 전체", f"바이너리 없음 ({binary}) — `cargo build --release --bin rhwp`")
        else:
            surface = load_surface(binary, rep)
            if surface:
                caps_j, mcp_j, helptext = surface
                # 허용목록은 전부 계약 테스트에서 읽는다 — 여기 베끼면 언젠가 어긋난다.
                non_argv = load_allowlist(
                    repo, MCP_CONTRACT, "NON_ARGV_PROPERTIES", {"paths", "password"}, rep
                )
                help_hidden = load_allowlist(
                    repo, CLI_CONTRACT, "HELP_HIDDEN", set(), rep
                )
                mcp_excluded = load_mcp_exclusions(repo, rep)
                check_mcp_input_schema(mcp_j, rep)
                check_property_wiring(mcp_j, non_argv, rep)
                check_help_coverage(caps_j, helptext, help_hidden, rep)
                check_json_has_mcp_tool(caps_j, mcp_j, mcp_excluded, rep)
                check_declared_flags_real(binary, caps_j, rep)
                check_failure_stdout_silent(binary, caps_j, rep)
                check_undeclared_commands_are_not_invisible(binary, caps_j, helptext, rep)

    # ── 보고 ──
    print()
    for name in rep.passed:
        print(f"  통과   {name}")
    for name, why in rep.skipped:
        print(f"  건너뜀 {name} — {why}")

    if rep.warnings:
        print()
        print(f"경고 {len(rep.warnings)}건 — 차단 아님, 판단 필요 (종료 코드 무관)")
        for check, detail in rep.warnings:
            print(f"  ! [{check}] {detail}")

    if rep.findings:
        print()
        print(f"실패 {len(rep.findings)}건 — 전부 고친 뒤 한 번만 빌드하면 된다")
        print()
        by_check: dict[str, list[Finding]] = {}
        for f in rep.findings:
            by_check.setdefault(f.check, []).append(f)
        for check, items in by_check.items():
            print(f"[{check}]")
            for f in items:
                print(f"  · {f.detail}")
            print(f"  → {items[0].fix}")
            print()
        return 1

    print()
    print("전부 통과.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
