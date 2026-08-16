#!/usr/bin/env python3
"""HWPX 크래시 최소화기 — 실패 재현 문서를 최소 픽스처로 자동 축소한다.

## 왜 있는가 (fuzz_corpus 와의 분업)

`gym/tools/fuzz_corpus.py`(#4884 계열)는 **발견** 엔진이다 — 전 코퍼스를 두들겨
"어느 문서가 어느 위치에서 패닉하는가"를 클러스터링한다. 그런데 발견된 재현체는
대개 수백 KB~수 MB 실문서라서, 그대로는 이슈에 못 싣고(개인정보·저작권·용량)
회귀 테스트 픽스처로도 못 쓴다. 지금까지는 사람이 `tools/make_issue*_fixture.py`
같은 **일회용 스크립트를 이슈마다 새로 짜서** 손으로 줄여 왔다 (#3738·#3751·
#3765·#3798 각각 별도 파일이 그 증거다).

이 도구는 그 수작업을 일반화한다: 실패 시그니처(패닉 위치·abort·timeout)를
**보존하는 한도 안에서** 문서를 델타 디버깅(ddmin)으로 깎아, "같은 자리에서
같은 방식으로 죽는 가장 작은 문서"를 만든다. 발견(fuzz_corpus) → **축소(이 도구)**
→ 이슈/픽스처가 한 파이프라인이 된다.

## 범위와 한계

- **HWPX 전용.** HWPX 는 ZIP + XML 이라 표준 라이브러리(zipfile·ElementTree)로
  안전하게 재작성할 수 있다. HWP5 는 CFB 컨테이너라 표준 라이브러리만으로는
  재작성이 위험해 범위 밖이다 (필요해지면 rhwp 자신의 convert 를 오라클로 삼는
  별도 설계가 맞다).
- 기본 시그니처는 **패닉·abort·timeout 만** 버그로 친다. 깨끗한 오류 종료(exit 1
  + 진단 메시지)는 정상 동작이므로 기본값에선 재현 실패로 처리한다
  (`--accept-exit N` 으로 명시적으로 켤 수 있다).
- 축소는 같은 시그니처를 유지할 때만 채택한다 — 다른 버그로 미끄러진 축소본은
  버린다. "panicked at src/a.rs:10" 이 "src/b.rs:99" 로 바뀌면 그건 다른 이슈다.

## 축소 전략 (순서대로, 각각 고정점까지)

1. **ZIP 멤버 가지치기** — 미리보기(Preview/*)·바이너리(BinData/*)·스크립트 등
   없어도 재현되는 멤버를 통째로 제거한다.
2. **문단 ddmin** — 각 `Contents/section*.xml` 의 최상위 문단(`<hp:p>`)을
   델타 디버깅으로 깎는다. 청크 제거 → 실패 유지 확인 → 채택의 반복.
3. **런 ddmin** — 남은 문단 안의 `<hp:run>` 들을 같은 방식으로 깎는다.

## 사용

    # 기본: info --json 으로 재현 확인하며 축소
    python3 tools/crash_minimizer.py crash.hwpx --bin target/debug/rhwp \
        --cmd "info {doc} --json" -o minimal.hwpx

    # 이슈 초안까지 (시그니처·전후 크기·재현 명령 포함)
    python3 tools/crash_minimizer.py crash.hwpx --bin target/debug/rhwp \
        --cmd "export-text {doc}" -o minimal.hwpx --emit-issue issue_draft.md

    # 오라클을 임의 명령으로 (rhwp 아닌 스크립트도 가능 — 자가 테스트가 이 경로를 쓴다)
    python3 tools/crash_minimizer.py crash.hwpx --oracle "python3 my_check.py {doc}"

종료 코드: 0 = 축소 성공(산출물 실존), 1 = 실행 실패, 2 = 입력 오류 또는
원본이 애초에 재현되지 않음(축소할 실패가 없음). exit 0 이면 산출물이 원본과
같은 시그니처로 실패함을 마지막에 한 번 더 재검증한 상태다 — agent-toolkit 의
"성공처럼 보이는 미완성 산출물을 남기지 않는다" 계약과 같다.
"""

from __future__ import annotations

import argparse
import base64
import io
import json
import re
import shlex
import subprocess
import sys
import tempfile
import time
import xml.etree.ElementTree as ET
import zipfile
from pathlib import Path

# Rust 2021 패닉 형식: "thread 'main' panicked at src/parser/x.rs:123:45:"
PANIC_RE = re.compile(r"panicked at\s+([^\r\n:]+\.rs:\d+)")

# 이 멤버들이 없으면 HWPX 가 문서로 열리지 않는다 — 가지치기 대상에서 제외.
ESSENTIAL_MEMBERS = ("mimetype", "version.xml", "META-INF/", "Contents/content.hpf")

DEFAULT_TIMEOUT = 30.0


def log(msg: str) -> None:
    print(msg, file=sys.stderr)


class Oracle:
    """문서 하나를 명령에 먹여 실패 시그니처를 판정한다."""

    def __init__(self, cmd_template: list[str], timeout: float, accept_exits: set[int]):
        self.cmd_template = cmd_template
        self.timeout = timeout
        self.accept_exits = accept_exits
        self.runs = 0

    def signature(self, doc_path: Path) -> tuple | None:
        """실패면 정규화된 시그니처 튜플, 실패가 아니면 None."""
        cmd = [a.replace("{doc}", str(doc_path)) for a in self.cmd_template]
        self.runs += 1
        try:
            proc = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                encoding="utf-8",
                errors="replace",
                timeout=self.timeout,
            )
        except subprocess.TimeoutExpired:
            return ("timeout",)

        panic = PANIC_RE.search(proc.stderr or "")
        if panic:
            # 열 번호·경로 구분자 차이를 무시하고 file.rs:line 만 본다.
            return ("panic", panic.group(1).replace("\\", "/"))
        rc = proc.returncode
        if rc is None:
            return ("abort", "unknown")
        # Windows 액세스 위반(0xC0000005 등)·시그널 종료는 큰 값/음수로 온다.
        if rc < 0 or rc >= 0xC0000000:
            return ("abort", rc)
        if rc != 0 and rc in self.accept_exits:
            return ("exit", rc)
        return None


class HwpxDoc:
    """ZIP 멤버 딕셔너리로 든 HWPX 한 부. 순서·압축 방식을 보존해 재작성한다."""

    def __init__(self, names: list[str], data: dict[str, bytes], compress: dict[str, int]):
        self.names = names
        self.data = data
        self.compress = compress

    @classmethod
    def load(cls, path: Path) -> "HwpxDoc":
        with zipfile.ZipFile(path) as z:
            infos = z.infolist()
            return cls(
                [i.filename for i in infos],
                {i.filename: z.read(i.filename) for i in infos},
                {i.filename: i.compress_type for i in infos},
            )

    def without_members(self, drop: set[str]) -> "HwpxDoc":
        names = [n for n in self.names if n not in drop]
        return HwpxDoc(
            names,
            {n: self.data[n] for n in names},
            {n: self.compress[n] for n in names},
        )

    def with_member(self, name: str, payload: bytes) -> "HwpxDoc":
        doc = HwpxDoc(list(self.names), dict(self.data), dict(self.compress))
        doc.data[name] = payload
        return doc

    def write(self, path: Path) -> None:
        with zipfile.ZipFile(path, "w") as z:
            for n in self.names:
                info = zipfile.ZipInfo(n)
                info.compress_type = self.compress[n]
                z.writestr(info, self.data[n])

    def total_bytes(self) -> int:
        return sum(len(v) for v in self.data.values())


class Minimizer:
    def __init__(self, oracle: Oracle, target_sig: tuple, workdir: Path):
        self.oracle = oracle
        self.target_sig = target_sig
        self.workdir = workdir
        self._probe_counter = 0

    def still_fails(self, doc: HwpxDoc) -> bool:
        self._probe_counter += 1
        probe = self.workdir / f"probe_{self._probe_counter}.hwpx"
        doc.write(probe)
        sig = self.oracle.signature(probe)
        probe.unlink(missing_ok=True)
        return sig == self.target_sig

    # --- 전략 1: ZIP 멤버 가지치기 -------------------------------------

    def prune_members(self, doc: HwpxDoc) -> HwpxDoc:
        candidates = [
            n
            for n in doc.names
            if not any(n == e or n.startswith(e) for e in ESSENTIAL_MEMBERS)
            and not re.match(r"Contents/section\d+\.xml$", n)
            and n != "Contents/header.xml"
        ]
        for name in candidates:
            trial = doc.without_members({name})
            if self.still_fails(trial):
                log(f"  멤버 제거: {name}")
                doc = trial
        return doc

    # --- 전략 2·3: XML 자식 ddmin ---------------------------------------

    def ddmin_children(self, doc: HwpxDoc, member: str, child_localname: str) -> HwpxDoc:
        """member XML 의 (임의 깊이) 부모 아래 child_localname 자식들을 ddmin 으로 깎는다."""
        try:
            root, ns_map = parse_with_ns(doc.data[member])
        except ET.ParseError:
            return doc

        parents = [
            p for p in root.iter() if any(localname(c.tag) == child_localname for c in p)
        ]
        for parent in parents:
            children = [c for c in parent if localname(c.tag) == child_localname]
            if len(children) <= 1:
                continue
            keep = self._ddmin(
                children,
                lambda kept, _p=parent, _all=children: self._probe_with_children(
                    doc, member, root, ns_map, _p, _all, kept
                ),
            )
            if len(keep) < len(children):
                log(f"  {member}: <{child_localname}> {len(children)} → {len(keep)}")
                _replace_children(parent, children, keep)
                doc = doc.with_member(member, serialize_with_ns(root, ns_map))
        return doc

    def _probe_with_children(self, doc, member, root, ns_map, parent, all_children, kept):
        removed = [c for c in all_children if c not in kept]
        _replace_children(parent, all_children, kept)
        trial = doc.with_member(member, serialize_with_ns(root, ns_map))
        ok = self.still_fails(trial)
        _replace_children(parent, kept, all_children)  # 원상복구
        _ = removed
        return ok

    @staticmethod
    def _ddmin(items: list, test_keep) -> list:
        """고전 ddmin: kept 부분집합이 실패를 유지하면 채택, 청크를 절반씩 좁힌다."""
        cur = list(items)
        n = 2
        while len(cur) >= 2:
            chunk = max(1, len(cur) // n)
            reduced = False
            for start in range(0, len(cur), chunk):
                candidate = cur[:start] + cur[start + chunk :]
                if candidate and test_keep(candidate):
                    cur = candidate
                    n = max(n - 1, 2)
                    reduced = True
                    break
            if not reduced:
                if n >= len(cur):
                    break
                n = min(len(cur), n * 2)
        return cur


def localname(tag: str) -> str:
    return tag.rsplit("}", 1)[-1]


def parse_with_ns(payload: bytes):
    ns_map = {}
    for event, item in ET.iterparse(io.BytesIO(payload), events=("start-ns",)):
        prefix, uri = item
        ns_map[prefix] = uri
    root = ET.fromstring(payload)
    return root, ns_map


def serialize_with_ns(root, ns_map) -> bytes:
    for prefix, uri in ns_map.items():
        ET.register_namespace(prefix, uri)
    return ET.tostring(root, encoding="UTF-8", xml_declaration=True)


def _replace_children(parent, current: list, new: list) -> None:
    """parent 에서 current 집합을 걷어내고, 그 첫 위치에 new 를 순서대로 되꽂는다."""
    all_children = list(parent)
    insert_at = all_children.index(current[0])
    for c in current:
        parent.remove(c)
    for offset, c in enumerate(new):
        parent.insert(insert_at + offset, c)


def emit_issue_draft(path: Path, args, sig: tuple, before: int, after: int, minimal: Path):
    repro_cmd = " ".join(args.oracle_template).replace("{doc}", minimal.name)
    kind = sig[0]
    where = sig[1] if len(sig) > 1 else ""
    payload = minimal.read_bytes()
    attach = (
        f"```\nbase64 -d <<'EOF' > {minimal.name}\n"
        + base64.b64encode(payload).decode("ascii")
        + "\nEOF\n```"
        if len(payload) <= 48 * 1024
        else f"(픽스처 {len(payload)}바이트 — 파일 첨부: `{minimal.name}`)"
    )
    path.write_text(
        f"""# [자동 최소화] {kind} {where}

## 증상

`{repro_cmd}` 실행 시 {kind}{f' — `{where}`' if where else ''}.

## 재현

원본 {before:,}바이트 → 최소화 {after:,}바이트 (tools/crash_minimizer.py, 오라클 {args.timeout:.0f}s).
같은 시그니처를 유지하는 한도에서 ZIP 멤버·문단·런을 델타 디버깅으로 제거한 결과다.

{attach}

## 판정 기준

- 시그니처: `{sig}`
- 최소화 산출물이 위 명령에서 같은 시그니처로 실패함을 저장 직전 재검증했다.
""",
        encoding="utf-8",
    )


def main(argv=None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")

    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("input", help="크래시가 재현되는 .hwpx")
    ap.add_argument("--bin", help="rhwp 바이너리 경로 (--cmd 와 함께)")
    ap.add_argument("--cmd", help='rhwp 하위 명령 템플릿, 예: "info {doc} --json"')
    ap.add_argument("--oracle", help='임의 오라클 전체 명령 템플릿, 예: "python3 chk.py {doc}"')
    ap.add_argument("-o", "--output", default=None, help="최소화 산출물 경로 (기본: <입력>.min.hwpx)")
    ap.add_argument("--timeout", type=float, default=DEFAULT_TIMEOUT)
    ap.add_argument("--accept-exit", type=int, action="append", default=[],
                    help="이 종료 코드도 실패 시그니처로 인정 (기본: 패닉/abort/timeout 만)")
    ap.add_argument("--emit-issue", help="이슈 초안 markdown 경로")
    ap.add_argument("--max-passes", type=int, default=4, help="전략 반복 고정점 상한")
    args = ap.parse_args(argv)

    input_path = Path(args.input)
    if not input_path.is_file():
        log(f"입력이 없다: {input_path}")
        return 2
    if not zipfile.is_zipfile(input_path):
        log("입력이 ZIP(HWPX)이 아니다 — 이 도구는 HWPX 전용이다 (docstring 참조).")
        return 2

    if args.oracle:
        template = shlex.split(args.oracle)
    elif args.bin and args.cmd:
        template = [args.bin] + shlex.split(args.cmd)
    else:
        log("--oracle 또는 (--bin 과 --cmd) 가 필요하다.")
        return 2
    args.oracle_template = template

    import shutil
    exe = template[0].strip('"')
    if not (Path(exe).is_file() or shutil.which(exe)):
        log(f"오라클 실행 파일을 찾을 수 없다: {exe}"
            " (Git Bash 의 /c/... 경로는 Windows 에서 안 통한다 — C:\\... 로)")
        return 2

    oracle = Oracle(template, args.timeout, set(args.accept_exit))
    target_sig = oracle.signature(input_path)
    if target_sig is None:
        log("원본이 실패하지 않는다 — 최소화할 것이 없다. (깨끗한 오류 종료를 실패로 치려면 --accept-exit)")
        return 2
    log(f"목표 시그니처: {target_sig}")

    output = Path(args.output) if args.output else input_path.with_suffix(".min.hwpx")
    before_bytes = input_path.stat().st_size

    started = time.monotonic()
    with tempfile.TemporaryDirectory(prefix="crashmin_") as td:
        mini = Minimizer(oracle, target_sig, Path(td))
        doc = HwpxDoc.load(input_path)

        for pass_no in range(1, args.max_passes + 1):
            log(f"[pass {pass_no}]")
            size_before_pass = doc.total_bytes()
            doc = mini.prune_members(doc)
            for member in [n for n in doc.names if re.match(r"Contents/section\d+\.xml$", n)]:
                doc = mini.ddmin_children(doc, member, "p")
                doc = mini.ddmin_children(doc, member, "run")
            if doc.total_bytes() >= size_before_pass:
                break

        doc.write(output)

    final_sig = oracle.signature(output)
    if final_sig != target_sig:
        output.unlink(missing_ok=True)
        log(f"재검증 실패(시그니처 {final_sig}) — 산출물을 남기지 않는다.")
        return 1

    after_bytes = output.stat().st_size
    elapsed = time.monotonic() - started
    print(json.dumps({
        "input": str(input_path),
        "output": str(output),
        "signature": list(target_sig),
        "bytesBefore": before_bytes,
        "bytesAfter": after_bytes,
        "oracleRuns": oracle.runs,
        "seconds": round(elapsed, 1),
    }, ensure_ascii=False))

    if args.emit_issue:
        emit_issue_draft(Path(args.emit_issue), args, target_sig, before_bytes, after_bytes, output)
        log(f"이슈 초안: {args.emit_issue}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
