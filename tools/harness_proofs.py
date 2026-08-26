#!/usr/bin/env python3
"""[#4389] 하네스 성질 실검증 러너 — 주장마다 실행, 판정은 PASS/FAIL.

이 저장소의 에이전트 하네스가 문서로 주장하는 성질을 **제3자가 명령 하나로
직접 검증**한다. 폐쇄 런타임의 "믿어 달라"와 반대편에 서는 실물이다.

사용법::

    python tools/harness_proofs.py                 # 표 출력, 하나라도 FAIL 이면 exit 1
    python tools/harness_proofs.py --json          # 기계용 JSON
    RHWP_BIN=path/to/rhwp python tools/harness_proofs.py   # 바이너리 지정

바이너리 탐색: RHWP_BIN → target/release/rhwp → target/debug/rhwp → PATH.
검증 6종은 전부 devel 머지본만으로 돈다(미머지 성질은 스코어카드 문서가 계약
테스트·PR 링크로 안내한다 — 이 러너는 거짓 PASS 를 만들지 않는다).
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SAMPLE = ROOT / "samples" / "basic" / "issue2007_nested_cell_pagination_42065.hwp"
# [#4868] 도달성·주소 축(P7·P8)의 표본. 본문이 넉넉하고 `search` 가 본문 문단을 그대로
# 찾아 주는 판이라 "도구가 준 좌표를 다음 호출에 쓴다"를 한 문서 안에서 닫을 수 있다.
TEXT_SAMPLE = ROOT / "samples" / "hwp3-sample.hwp"
# [#4870] 정확 일치가 아니라 **하한**이다. P2 가 지키려는 것은 (a) 모든 명령이 자기
# 계약을 싣는다 (b) 표면이 조용히 줄어들지 않는다 — 둘 다 하한으로 충분하다. 정확
# 일치로 두면 명령이 하나 늘 때마다 러너 전체가 red 가 되고, 상시 red 인 게이트는
# 아무도 돌리지 않아 진짜 회귀까지 같이 묻힌다(실제로 68→85 로 자란 뒤 그렇게 됐다).
EXPECTED_COMMAND_FLOOR = 68
REQUIRED_EXIT_CODES = ("0", "1", "2")
REQUIRED_JSON_CONTRACT_FIELDS = ("stdout", "schemaPolicy")
# [#4868] 도달성 판정에 쓸 최소 줄 길이. 짧은 줄("1.", "가.")은 바이너리 어디에나 우연히
# 나타나 "원시 경로로도 읽힌다"는 반대 결론을 만든다 — 우연 일치를 걸러야 판정이 정직하다.
MIN_LINE_CHARS = 8
# 표본 하나의 수치를 성질로 굳히지 않는다. 실측은 98%대지만 임계는 과반으로 느슨하게 둔다 —
# 표본이 바뀌어도 살아남는 것만 성질이다.
UNREACHABLE_THRESHOLD_PERCENT = 50.0


def find_binary() -> str:
    exe = ".exe" if os.name == "nt" else ""
    env = os.environ.get("RHWP_BIN", "").strip()
    candidates = [env] if env else []
    candidates += [
        str(ROOT / "target" / "release" / f"rhwp{exe}"),
        str(ROOT / "target" / "debug" / f"rhwp{exe}"),
    ]
    for c in candidates:
        if c and Path(c).is_file():
            return c
    which = shutil.which("rhwp")
    if which:
        return which
    sys.exit("rhwp 바이너리를 찾지 못했습니다 — RHWP_BIN 지정 또는 cargo build")


def run(bin_path: str, args: list, timeout: int = 120):
    return subprocess.run(
        [bin_path, *args], capture_output=True, timeout=timeout, cwd=ROOT
    )


def command_surface_contract(caps: object) -> tuple[bool, str]:
    if not isinstance(caps, dict):
        return False, f"capabilities JSON 형식 오류: object가 아님 ({type(caps).__name__})"

    commands = caps.get("commands")
    if not isinstance(commands, list):
        return False, f"commands 형식 오류: array가 아님 ({type(commands).__name__})"
    if len(commands) < EXPECTED_COMMAND_FLOOR:
        return (
            False,
            f"commands={len(commands)} (floor={EXPECTED_COMMAND_FLOOR} — 표면이 줄었다)",
        )

    names = []
    for index, command in enumerate(commands):
        if not isinstance(command, dict):
            return False, f"commands[{index}] 형식 오류: object가 아님"
        name = command.get("name")
        if not isinstance(name, str) or not name.strip():
            return False, f"commands[{index}].name 형식 오류: 비어 있거나 문자열이 아님"
        names.append(name)
    if len(set(names)) != len(names):
        duplicate = next(name for name in names if names.count(name) > 1)
        return False, f"commands[].name 중복: {duplicate!r}"

    exit_codes = caps.get("exitCodes")
    if not isinstance(exit_codes, dict):
        return False, f"exitCodes 형식 오류: object가 아님 ({type(exit_codes).__name__})"
    for code in REQUIRED_EXIT_CODES:
        meaning = exit_codes.get(code)
        if not isinstance(meaning, str) or not meaning.strip():
            return False, f"exitCodes[{code!r}] 의미가 비어 있거나 문자열이 아님"

    json_contract = caps.get("jsonContract")
    if not isinstance(json_contract, dict):
        return False, f"jsonContract 형식 오류: object가 아님 ({type(json_contract).__name__})"
    for field in REQUIRED_JSON_CONTRACT_FIELDS:
        meaning = json_contract.get(field)
        if not isinstance(meaning, str) or not meaning.strip():
            return False, f"jsonContract[{field!r}] 의미가 비어 있거나 문자열이 아님"

    return (
        True,
        f"commands={len(commands)} (floor={EXPECTED_COMMAND_FLOOR}, unique names), "
        f"exitCodes={list(REQUIRED_EXIT_CODES)}, "
        f"jsonContract={list(REQUIRED_JSON_CONTRACT_FIELDS)}",
    )


def provenance_marker_contract(envelope: object) -> tuple[bool, str]:
    if not isinstance(envelope, dict):
        return False, f"info JSON 형식 오류: object가 아님 ({type(envelope).__name__})"

    untrusted_content = envelope.get("untrustedContent")
    untrusted_fields = envelope.get("untrustedFields")
    fields_ok = (
        isinstance(untrusted_fields, list)
        and len(untrusted_fields) > 0
        and all(isinstance(field, str) and field.strip() for field in untrusted_fields)
    )
    ok = untrusted_content is True and fields_ok
    detail = (
        f"untrustedContent={untrusted_content!r}, "
        f"untrustedFields={untrusted_fields!r}"
    )
    return ok, detail


def body_lines(bin_path: str, sample: Path) -> tuple[list, str]:
    """`export-text --json` 이 낸 본문을 (쪽, 줄) 목록으로 편다. 실패는 빈 목록 + 사유."""
    out = run(bin_path, ["export-text", str(sample), "--json"])
    if out.returncode != 0:
        return [], f"export-text exit={out.returncode}"
    try:
        env = json.loads(out.stdout)
    except Exception as e:  # noqa: BLE001 - 판정용 러너
        return [], f"export-text JSON 파싱 실패: {e}"
    lines = []
    for page in env.get("pages", []):
        for line in str(page.get("text", "")).splitlines():
            line = line.strip()
            if len(line) >= MIN_LINE_CHARS:
                lines.append((page.get("page"), line))
    return lines, f"본문 줄 {len(lines)}개"


def raw_decodings(sample: Path) -> list:
    """파일 바이트를 텍스트로 볼 수 있는 두 갈래. 범용 파일·셸 경로가 보는 전부다.

    UTF-16LE 를 함께 보는 이유는 공정성이다 — 한글 문서의 여러 바이너리 포맷이
    UTF-16LE 로 문자열을 담으므로, UTF-8 만 대조하면 허수아비가 된다.
    """
    data = sample.read_bytes()
    even = data[: len(data) // 2 * 2]
    return [
        data.decode("utf-8", errors="replace"),
        even.decode("utf-16-le", errors="replace"),
    ]


def proofs(bin_path: str) -> list:
    results = []

    def record(pid: str, claim: str, command: str, ok: bool, detail: str) -> None:
        results.append(
            {"id": pid, "claim": claim, "command": command, "pass": bool(ok), "detail": detail}
        )

    # P1 결정론 — 같은 호출은 바이트까지 같다 (자기서술에 모델·시각이 끼지 않는다).
    a = run(bin_path, ["capabilities"])
    b = run(bin_path, ["capabilities"])
    record(
        "P1",
        "자기서술 결정론 — capabilities 2회 호출의 stdout 이 바이트 동일",
        "rhwp capabilities (×2 비교)",
        a.returncode == 0 and a.stdout == b.stdout and len(a.stdout) > 1000,
        f"exit={a.returncode}, bytes={len(a.stdout)}, identical={a.stdout == b.stdout}",
    )

    # P2 자기서술 규모 — 명령 표면이 기계 계약으로 전수 서술된다.
    try:
        caps = json.loads(a.stdout)
        ok, detail = command_surface_contract(caps)
    except Exception as e:  # noqa: BLE001 - 판정용 러너
        ok, detail = False, f"JSON 파싱 실패: {e}"
    record(
        "P2",
        f"명령 표면 전수 자기서술 — 모든 명령이 계약을 싣고 표면이 {EXPECTED_COMMAND_FLOOR}개 밑으로 줄지 않았다",
        "rhwp capabilities | jq '.commands|length'",
        ok,
        detail,
    )

    # P3 종료코드 사전 — 미지 옵션은 exit 2, stdout 은 0바이트(반쪽 JSON 금지).
    c = run(bin_path, ["info", str(SAMPLE), "--nope", "--json"])
    record(
        "P3",
        "사용법 오류 사전 — 미지 옵션은 exit 2 + stdout 0바이트",
        "rhwp info <sample> --nope --json",
        c.returncode == 2 and c.stdout == b"",
        f"exit={c.returncode}, stdout_bytes={len(c.stdout)}",
    )

    # P4 실패 stdout 순수성 — 런타임 실패도 stdout 을 오염시키지 않는다.
    d = run(bin_path, ["info", "no_such_file_hopefully.hwp", "--json"])
    record(
        "P4",
        "실패 경로 stdout 순수성 — 없는 파일 info 는 exit 1 + stdout 0바이트",
        "rhwp info no_such_file.hwp --json",
        d.returncode == 1 and d.stdout == b"",
        f"exit={d.returncode}, stdout_bytes={len(d.stdout)}",
    )

    # P5 출처 표지 — 문서 파생 값을 싣는 봉투는 신뢰 경계를 스스로 밝힌다.
    e = run(bin_path, ["info", str(SAMPLE), "--json"])
    try:
        env = json.loads(e.stdout)
        ok, detail = provenance_marker_contract(env)
    except Exception as ex:  # noqa: BLE001
        ok, detail = False, f"JSON 파싱 실패: {ex}"
    record(
        "P5",
        "출처 표지 S1 — 봉투가 untrustedContent/untrustedFields 를 스스로 싣는다",
        "rhwp info <sample> --json",
        e.returncode == 0 and ok,
        detail,
    )

    # P6 설명 결정론 — explain 도 2회 동일(생성 문장 아님이 드리프트 가드의 전제).
    f1 = run(bin_path, ["explain", str(SAMPLE), "--json"])
    f2 = run(bin_path, ["explain", str(SAMPLE), "--json"])
    record(
        "P6",
        "explain 결정론 — 같은 문서, 같은 서술(바이트 동일)",
        "rhwp explain <sample> --json (×2 비교)",
        f1.returncode == 0 and f1.stdout == f2.stdout and len(f1.stdout) > 100,
        f"exit={f1.returncode}, identical={f1.stdout == f2.stdout}",
    )

    # [#4868] P7 본문 도달성 — "이 도구가 없으면 못 하는 일"을 처음으로 판정하는 행.
    # P1~P6 은 전부 도구가 예의 바른가(결정론·종료 코드·stdout 순수성)를 물었다.
    lines, detail7 = body_lines(bin_path, TEXT_SAMPLE)
    target = None
    if lines:
        decodings = raw_decodings(TEXT_SAMPLE)
        unreachable = [
            (page, line)
            for page, line in lines
            if all(line not in decoded for decoded in decodings)
        ]
        percent = len(unreachable) * 100.0 / len(lines)
        ok7 = percent >= UNREACHABLE_THRESHOLD_PERCENT
        detail7 = (
            f"본문 줄 {len(lines)}개 중 원시 디코딩(UTF-8·UTF-16LE) 어디에도 없는 줄 "
            f"{len(unreachable)}개 = {percent:.1f}% (임계 {UNREACHABLE_THRESHOLD_PERCENT:.0f}%)"
        )
        if unreachable:
            # 가장 긴 줄이 우연 일치에 가장 강하다 — P8 의 표적으로 그대로 넘긴다.
            target = max(unreachable, key=lambda pair: len(pair[1]))
    else:
        ok7 = False
    record(
        "P7",
        "본문 도달성 — 원시 바이트를 어떻게 디코딩해도 안 나오는 본문을 도구는 준다",
        "rhwp export-text <sample> --json (+ 원시 바이트 대조)",
        ok7,
        detail7,
    )

    # [#4868] P8 주소 왕복 — 도구가 준 좌표를 다음 호출에 그대로 쓸 수 있다.
    # 원시 바이트 경로는 그 줄을 못 찾으므로 돌려줄 좌표가 없고, 찾더라도 바이트
    # 오프셋은 쪽·문단 어느 좌표계로도 번역되지 않는다.
    if target is None:
        ok8, detail8 = False, "전제 불충족 — P7 이 표적 줄을 찾지 못했습니다"
    else:
        expect_page, line = target
        s = run(bin_path, ["search", str(TEXT_SAMPLE), line, "--json"])
        if s.returncode != 0:
            ok8, detail8 = False, f"search exit={s.returncode}"
        else:
            try:
                sv = json.loads(s.stdout)
                matches = sv.get("matches") or []
                got_page = matches[0].get("page") if matches else None
                ok8 = bool(matches) and got_page == expect_page
                detail8 = (
                    f"표적 줄 {len(line)}자 · matchCount={sv.get('matchCount')} · "
                    f"search page={got_page} vs export-text page={expect_page}"
                )
            except Exception as e:  # noqa: BLE001
                ok8, detail8 = False, f"search JSON 파싱 실패: {e}"
    record(
        "P8",
        "주소 왕복 — search 가 준 쪽 주소가 export-text 가 그 줄을 실은 쪽과 같다",
        "rhwp search <sample> \"<본문 줄>\" --json",
        ok8,
        detail8,
    )

    return results


def main() -> None:
    # Windows cp949 콘솔에서도 스스로 깨지지 않는다 (#4106 선례).
    for stream in (sys.stdout, sys.stderr):
        if hasattr(stream, "reconfigure"):
            stream.reconfigure(encoding="utf-8", errors="replace")
    as_json = "--json" in sys.argv
    bin_path = find_binary()
    results = proofs(bin_path)
    passed = sum(1 for r in results if r["pass"])
    if as_json:
        print(
            json.dumps(
                {"binary": bin_path, "passed": passed, "total": len(results), "proofs": results},
                ensure_ascii=False,
                indent=2,
            )
        )
    else:
        print(f"하네스 성질 실검증 — {bin_path}")
        for r in results:
            mark = "PASS" if r["pass"] else "FAIL"
            print(f"  [{mark}] {r['id']} {r['claim']}")
            print(f"         $ {r['command']}")
            print(f"         {r['detail']}")
        print(f"판정: {passed}/{len(results)}")
    sys.exit(0 if passed == len(results) else 1)


if __name__ == "__main__":
    main()
