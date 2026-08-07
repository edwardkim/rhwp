"""차등 게이트 오케스트레이터 — 시나리오 전체를 양쪽에 돌리고 판정한다 (P0).

한 번의 호출로 아래를 한다.

1. 시나리오마다 **새 프로세스**로 오라클 러너를 돌린다(COM 규약: 문서당 프로세스 1개).
2. 시간 제한을 걸고, 한글 프로세스가 남으면 기본적으로 중단한다. 정리는 명시적 opt-in이다.
3. rhwp 러너를 돌린다(여기는 프로세스 격리가 필요 없다).
4. `compare.py` 로 판정하고 요약을 찍는다.

## 쓰임

    python tools/hwpctrl_compat/run_gate.py --impl legacy
    python tools/hwpctrl_compat/run_gate.py --only field-read --timeout 300

## 왜 직렬인가

COM 판정을 동시에 돌리면 서로의 `Hwp.exe` 를 죽여 "무응답" 오판을 만든다. 병렬화하지 말 것.
"""

from __future__ import annotations

import argparse
import csv
import io
import json
import subprocess
import sys
import time
from pathlib import Path

from oracle_version import matches_expected_version

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
SCENARIO_DIR = HERE / "scenarios"
OUT_ROOT = REPO / "output" / "poc" / "hwpctrl"

# 오라클은 **한글2022(12.x)** 로 고정한다(계획서 §9-4). 이 머신에는 2024(13.x)도 깔려 있어서
# 고정하지 않으면 두 버전의 정답지가 한 표에 섞인다. 전환 방법은 계획서 §4.5.1.
ORACLE_VERSION = "12"
HWP_IMAGES = {"hwp.exe", "hwpframe.exe"}


def hwp_pids(tasklist_output: str | None = None) -> set[str]:
    """실행 중인 한글 PID를 읽는다. 어떤 프로세스도 이 단계에서 종료하지 않는다."""
    if tasklist_output is None:
        proc = subprocess.run(["tasklist", "/FO", "CSV", "/NH"], capture_output=True, check=False)
        tasklist_output = proc.stdout.decode("utf-8", "replace")
    pids = set()
    for row in csv.reader(io.StringIO(tasklist_output)):
        if len(row) >= 2 and row[0].strip().lower() in HWP_IMAGES:
            pids.add(row[1].strip())
    return pids


def new_hwp_pids(baseline: set[str]) -> set[str]:
    """clean start 뒤 생긴 한글 PID를 보고한다. 기본 경로에서는 종료하지 않는다."""
    return hwp_pids() - baseline


def cleanup_spawned_hwp(baseline: set[str]) -> None:
    """명시적 opt-in에서만 새 PID를 종료한다. 전용 Windows 계정에서만 사용한다."""
    for pid in sorted(new_hwp_pids(baseline)):
        subprocess.run(["taskkill", "/F", "/PID", pid], capture_output=True, check=False)


def wait_for_hwp_exit(baseline: set[str], settle_seconds: float, poll_seconds: float = 0.25) -> set[str]:
    """`com.Quit()` 뒤 비동기 종료를 기다리고, 상한 뒤에도 남은 PID만 돌려준다."""
    deadline = time.monotonic() + settle_seconds
    leftovers = new_hwp_pids(baseline)
    while leftovers and time.monotonic() < deadline:
        time.sleep(min(poll_seconds, max(0.0, deadline - time.monotonic())))
        leftovers = new_hwp_pids(baseline)
    return leftovers


def stored_oracle_status(path: Path, expect_version: str | None) -> str:
    """`--skip-ocx`가 읽을 기존 정답지가 현재 오라클인지 판정한다."""
    if not path.exists():
        return "NO_ORACLE"
    try:
        with io.open(path, encoding="utf-8") as fh:
            version = (json.load(fh).get("oracle") or {}).get("version")
    except (OSError, json.JSONDecodeError, AttributeError):
        return "INVALID_ORACLE"
    return "SKIPPED" if matches_expected_version(version, expect_version) else "STALE_ORACLE"


def run_ocx(scenario: Path, out_dir: Path, timeout: int, expect_version: str | None) -> str:
    cmd = [sys.executable, str(HERE / "runner_ocx.py"), str(scenario), "--out", str(out_dir)]
    if expect_version:
        cmd += ["--expect-version", expect_version]
    try:
        proc = subprocess.run(cmd, capture_output=True, timeout=timeout, check=False)
    except subprocess.TimeoutExpired:
        return "STALL"
    sys.stdout.write(proc.stdout.decode("utf-8", "replace"))
    if proc.returncode == 3:
        return "VERSION"
    if proc.returncode != 0:
        sys.stderr.write(proc.stderr.decode("utf-8", "replace")[-2000:])
        return "ERR"
    return "OK"


def run_rhwp(scenario: Path, out_dir: Path, impl: str, timeout: int) -> str:
    cmd = [
        "node",
        str(HERE / "runner_rhwp.mjs"),
        str(scenario),
        "--out",
        str(out_dir),
        "--impl",
        impl,
    ]
    try:
        proc = subprocess.run(cmd, capture_output=True, timeout=timeout, check=False)
    except subprocess.TimeoutExpired:
        return "STALL"
    sys.stdout.write(proc.stdout.decode("utf-8", "replace"))
    if proc.returncode != 0:
        sys.stderr.write(proc.stderr.decode("utf-8", "replace")[-2000:])
        return "ERR"
    return "OK"


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--impl", default="legacy", help="rhwp 측 구현 (legacy | 패키지 엔트리 경로)")
    ap.add_argument("--only", help="시나리오 id 하나만")
    ap.add_argument("--timeout", type=int, default=300, help="시나리오당 초 (COM 무응답 대비)")
    ap.add_argument(
        "--quit-settle-seconds",
        type=float,
        default=10.0,
        help="COM Quit 뒤 한글 프로세스 자연 종료 대기 시간 (기본 10초)",
    )
    ap.add_argument(
        "--expect-version",
        default=ORACLE_VERSION,
        help=(
            f"오라클 major 버전 고정 (기본 '{ORACLE_VERSION}' = 한글2022). "
            "빈 문자열을 주면 검사하지 않는다 — 두 버전의 결과가 섞이므로 권하지 않는다."
        ),
    )
    ap.add_argument("--skip-ocx", action="store_true", help="오라클 재실행 없이 기존 정답지 사용")
    ap.add_argument(
        "--cleanup-spawned",
        action="store_true",
        help="시간 초과 뒤 새 한글 PID를 종료 (전용 Windows 계정에서만 명시적으로 사용)",
    )
    args = ap.parse_args()

    scenarios = sorted(SCENARIO_DIR.glob("*.json"))
    if args.only:
        scenarios = [p for p in scenarios if p.stem == args.only]
    if not scenarios:
        print("시나리오 없음")
        return 2

    ocx_dir = OUT_ROOT / "ocx"
    rhwp_dir = OUT_ROOT / "rhwp"
    verdict_dir = OUT_ROOT / "verdict"
    for d in (ocx_dir, rhwp_dir, verdict_dir):
        d.mkdir(parents=True, exist_ok=True)

    status = {}
    comparable = []
    for path in scenarios:
        name = path.stem
        if args.skip_ocx:
            status[name] = stored_oracle_status(ocx_dir / f"{name}.returns.json", args.expect_version)
        else:
            baseline = hwp_pids()
            if baseline:
                status[name] = "OCCUPIED"
                print(f"  오라클 {name}: OCCUPIED (기존 한글 PID: {', '.join(sorted(baseline))})")
                continue
            started = time.monotonic()
            try:
                status[name] = run_ocx(path, ocx_dir, args.timeout, args.expect_version)
            finally:
                leftovers = wait_for_hwp_exit(baseline, args.quit_settle_seconds)
                if leftovers and args.cleanup_spawned:
                    cleanup_spawned_hwp(baseline)
                    leftovers = new_hwp_pids(baseline)
                if leftovers:
                    status[name] = f"{status.get(name, 'ERR')}/LEFTOVER"
                    print(f"  오라클 {name}: 남은 한글 PID {', '.join(sorted(leftovers))} — 자동 종료하지 않음")
            print(f"  오라클 {name}: {status[name]} ({time.monotonic() - started:.1f}s)")
        if status[name] not in ("OK", "SKIPPED"):
            continue
        rhwp_status = run_rhwp(path, rhwp_dir, args.impl, args.timeout)
        print(f"  rhwp {name}: {rhwp_status}")
        if rhwp_status != "OK":
            status[name] = f"{status[name]}/RHWP_{rhwp_status}"
        else:
            comparable.append(name)

    compare_cmd = [
        sys.executable,
        str(HERE / "compare.py"),
        "--ocx",
        str(ocx_dir),
        "--rhwp",
        str(rhwp_dir),
        "--out",
        str(verdict_dir),
    ]
    if comparable:
        for name in comparable:
            compare_cmd += ["--scenario", name]
    else:
        compare_cmd.append("--empty")
    subprocess.run(compare_cmd, check=False)

    with io.open(verdict_dir / "run_status.json", "w", encoding="utf-8", newline="\n") as fh:
        json.dump(
            {"impl": args.impl, "status": status, "comparableScenarios": comparable},
            fh,
            ensure_ascii=False,
            indent=2,
        )
        fh.write("\n")
    bad = {k: v for k, v in status.items() if v not in ("OK", "SKIPPED")}
    if bad:
        print(f"실행 문제: {bad}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
