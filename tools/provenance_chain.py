#!/usr/bin/env python3
"""문서 이력 체인(provenance chain) 매니페스트 생성·검증 — P1 실물.

문서 하나가 "어떤 입력에서, 어떤 조작 열을 거쳐, 어떤 출력이 되었는가"를
sha256 해시 체인으로 남기고, 나중에 실물 파일들과 대조해 변조를 검출한다.

    create : 입력·출력 파일과 조작 서술(또는 run 저널)을 받아 매니페스트 JSON 생성
    verify : 매니페스트의 해시를 실물과 재대조 — 전부 일치 exit 0, 불일치 exit 1
    chain  : 매니페스트 열의 연결(prev 해시·산출→입력 연속성) 검증

1차 범위는 **무결성 체인(변조 검출)까지**다. 서명이 없으므로 매니페스트와
실물을 함께 바꾸는 위조는 막지 못한다 — 부인 방지·법적 효력을 주장하지 않는다.
설계 논거: mydocs/tech/agent_architecture/provenance_chain.md
표준 라이브러리만 사용한다.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

MANIFEST_VERSION = "1.0"
EXIT_OK = 0
EXIT_MISMATCH = 1
EXIT_USAGE = 2


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 16), b""):
            h.update(block)
    return h.hexdigest()


def tool_version(rhwp_bin: str | None) -> dict:
    """RHWP_BIN(또는 --rhwp-bin) 실행 결과로 도구 버전을 기록한다.

    바이너리가 없으면 지어내지 않고 versionString:null + unavailable 로 남긴다.
    """
    binary = rhwp_bin or os.environ.get("RHWP_BIN")
    if not binary:
        return {"versionString": None, "versionSource": "unavailable: RHWP_BIN 미지정"}
    try:
        out = subprocess.run(
            [binary, "--version"], capture_output=True, text=True, timeout=30
        )
        line = (out.stdout or out.stderr).strip().splitlines()
        if line:
            return {"versionString": line[0], "versionSource": f"{Path(binary).name} --version"}
        return {"versionString": None, "versionSource": "unavailable: --version 출력 없음"}
    except OSError as e:
        return {"versionString": None, "versionSource": f"unavailable: {e}"}


def file_entry(path_str: str, base: Path) -> dict:
    p = (base / path_str) if not Path(path_str).is_absolute() else Path(path_str)
    if not p.is_file():
        raise SystemExit(f"오류: 파일이 없습니다 - {path_str}")
    return {
        "path": path_str.replace("\\", "/"),
        "bytes": p.stat().st_size,
        "sha256": sha256_file(p),
    }


def resolve(path_str: str, base: Path) -> Path:
    p = Path(path_str)
    return p if p.is_absolute() else base / p


def cmd_create(args: argparse.Namespace) -> int:
    base = Path(args.base_dir)
    if not args.journal and not args.op:
        print("오류: --journal 또는 --op 가 최소 하나 필요합니다", file=sys.stderr)
        return EXIT_USAGE

    operations: dict = {}
    if args.journal:
        jpath = resolve(args.journal, base)
        if not jpath.is_file():
            print(f"오류: 저널 파일이 없습니다 - {args.journal}", file=sys.stderr)
            return EXIT_USAGE
        try:
            journal = json.loads(jpath.read_text(encoding="utf-8"))
        except (json.JSONDecodeError, UnicodeDecodeError) as e:
            print(f"오류: 저널 JSON 파싱 실패 - {e}", file=sys.stderr)
            return EXIT_USAGE
        operations = {
            "source": "run-journal",
            "steps": journal.get("steps", []),
            "journal": {
                "path": args.journal.replace("\\", "/"),
                "sha256": sha256_file(jpath),
            },
        }
    else:
        operations = {
            "source": "description",
            "steps": [{"description": s} for s in args.op],
            "journal": None,
        }

    prev = None
    if args.prev:
        ppath = resolve(args.prev, base)
        if not ppath.is_file():
            print(f"오류: 이전 매니페스트가 없습니다 - {args.prev}", file=sys.stderr)
            return EXIT_USAGE
        prev = {"path": args.prev.replace("\\", "/"), "sha256": sha256_file(ppath)}

    manifest = {
        "manifestVersion": MANIFEST_VERSION,
        "createdAt": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "generator": "tools/provenance_chain.py",
        "tool": tool_version(args.rhwp_bin),
        "input": file_entry(args.input, base),
        "output": file_entry(args.output, base),
        "operations": operations,
        "prev": prev,
    }
    out = resolve(args.manifest, base)
    out.write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(f"매니페스트 생성: {args.manifest} (sha256 {sha256_file(out)[:16]}…)")
    return EXIT_OK


def check_entry(label: str, entry: dict | None, base: Path, failures: list[str]) -> None:
    """기록된 {path, sha256} 를 실물과 대조하고 불일치를 모은다."""
    if entry is None:
        return
    p = resolve(entry["path"], base)
    if not p.is_file():
        failures.append(f"{label}: 파일 없음 - {entry['path']}")
        return
    actual = sha256_file(p)
    if actual != entry["sha256"]:
        failures.append(
            f"{label}: sha256 불일치 - {entry['path']}\n"
            f"  기록 {entry['sha256']}\n  실물 {actual}"
        )
    else:
        print(f"  일치: {label} {entry['path']} ({entry['sha256'][:16]}…)")


def load_manifest(path_str: str, base: Path) -> dict | None:
    p = resolve(path_str, base)
    if not p.is_file():
        return None
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, UnicodeDecodeError):
        return None


def verify_one(path_str: str, base: Path) -> list[str]:
    manifest = load_manifest(path_str, base)
    if manifest is None:
        return [f"매니페스트를 읽을 수 없습니다 - {path_str}"]
    failures: list[str] = []
    check_entry("input", manifest.get("input"), base, failures)
    check_entry("output", manifest.get("output"), base, failures)
    journal = (manifest.get("operations") or {}).get("journal")
    check_entry("journal", journal, base, failures)
    check_entry("prev-manifest", manifest.get("prev"), base, failures)
    return failures


def cmd_verify(args: argparse.Namespace) -> int:
    base = Path(args.base_dir)
    print(f"검증: {args.manifest}")
    failures = verify_one(args.manifest, base)
    if failures:
        print(f"실패 {len(failures)}건:")
        for f in failures:
            print(f"  {f}")
        return EXIT_MISMATCH
    print("검증 통과: 기록된 해시 전부 실물과 일치")
    return EXIT_OK


def cmd_chain(args: argparse.Namespace) -> int:
    base = Path(args.base_dir)
    failures: list[str] = []
    manifests = []
    for m in args.manifests:
        loaded = load_manifest(m, base)
        if loaded is None:
            print(f"오류: 매니페스트를 읽을 수 없습니다 - {m}", file=sys.stderr)
            return EXIT_USAGE
        manifests.append((m, loaded))

    for i, (path_str, manifest) in enumerate(manifests):
        print(f"[{i}] {path_str}")
        if args.files:
            failures.extend(verify_one(path_str, base))
        if i == 0:
            continue
        prev_path, _prev_manifest = manifests[i - 1]
        prev_record = manifest.get("prev")
        if not prev_record:
            failures.append(f"[{i}] prev 기록 없음 - 체인이 끊겨 있습니다")
            continue
        actual_prev = sha256_file(resolve(prev_path, base))
        if prev_record["sha256"] != actual_prev:
            failures.append(
                f"[{i}] prev 해시 불일치 - 이전 매니페스트({prev_path})가 변조되었거나 "
                f"다른 파일입니다\n  기록 {prev_record['sha256']}\n  실물 {actual_prev}"
            )
        else:
            print(f"  연결: prev 해시 일치 ({actual_prev[:16]}…)")
        prev_out = (manifests[i - 1][1].get("output") or {}).get("sha256")
        this_in = (manifest.get("input") or {}).get("sha256")
        if prev_out and this_in and prev_out != this_in:
            failures.append(
                f"[{i}] 산출→입력 연속성 불일치 - 이전 출력({prev_out[:16]}…)과 "
                f"이번 입력({this_in[:16]}…)이 다른 파일입니다"
            )
        elif prev_out and this_in:
            print(f"  연속: 이전 출력 = 이번 입력 ({this_in[:16]}…)")

    if failures:
        print(f"체인 검증 실패 {len(failures)}건:")
        for f in failures:
            print(f"  {f}")
        return EXIT_MISMATCH
    print(f"체인 검증 통과: 매니페스트 {len(manifests)}개 연결·해시 전부 일치")
    return EXIT_OK


def main() -> int:
    parser = argparse.ArgumentParser(
        prog="provenance_chain.py", description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--base-dir", default=".", help="상대 경로의 기준 디렉터리 (기본 .)")
    sub = parser.add_subparsers(dest="command", required=True)

    c = sub.add_parser("create", help="매니페스트 생성")
    c.add_argument("--input", required=True, help="편집 입력 파일")
    c.add_argument("--output", required=True, help="편집 산출 파일")
    c.add_argument("--manifest", required=True, help="생성할 매니페스트 경로")
    c.add_argument("--journal", help="rhwp run 저널 JSON (조작 열의 1차 출처)")
    c.add_argument("--op", action="append", default=[],
                   help="조작 서술 문자열 (저널이 없을 때, 반복 가능)")
    c.add_argument("--prev", help="이전 매니페스트 (체인 연결)")
    c.add_argument("--rhwp-bin", help="도구 버전 기록용 rhwp 바이너리 (기본 $RHWP_BIN)")
    c.set_defaults(func=cmd_create)

    v = sub.add_parser("verify", help="매니페스트 1개를 실물과 대조")
    v.add_argument("manifest")
    v.set_defaults(func=cmd_verify)

    ch = sub.add_parser("chain", help="매니페스트 열의 연결을 검증")
    ch.add_argument("manifests", nargs="+", help="시간순 매니페스트 경로들")
    ch.add_argument("--files", action="store_true", help="연결에 더해 각 매니페스트의 실물 해시도 대조")
    ch.set_defaults(func=cmd_chain)

    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
