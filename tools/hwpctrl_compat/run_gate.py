"""차등 게이트 오케스트레이터 — 시나리오 전체를 양쪽에 돌리고 판정한다 (P0).

한 번의 호출로 아래를 한다.

1. Windows live 모드에서 시나리오마다 **새 프로세스**로 오라클 러너를 돌린다
   (COM 규약: 문서당 프로세스 1개).
2. macOS/Linux 기본 모드에서는 COM을 호출하지 않고 rhwp WASM 시나리오의 호출·저장 결과를 검사한다.
3. `--fixture` 또는 `--oracle-dir`는 어느 OS에서나 고정 Hancom 2022 결과와 차등 비교한다.
4. live/fixture 모드는 `compare.py`로 판정하고, 불일치를 실패로 돌려준다.

## 쓰임

    python tools/hwpctrl_compat/run_gate.py --impl legacy
    python tools/hwpctrl_compat/run_gate.py --only field-read --timeout 300
    python tools/hwpctrl_compat/run_gate.py --impl npm/hwpctrl-ocx/src/index.mjs --fixture

## 왜 직렬인가

COM 판정을 동시에 돌리면 서로의 `Hwp.exe` 를 죽여 "무응답" 오판을 만든다. 병렬화하지 말 것.
"""

from __future__ import annotations

import argparse
import csv
import io
import json
import platform
import subprocess
import sys
import time
from pathlib import Path

from oracle_version import matches_expected_version
from scenario_spec import check_call, contracts

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
SCENARIO_DIR = HERE / "scenarios"
OUT_ROOT = REPO / "output" / "poc" / "hwpctrl"
FIXTURE_ROOT = HERE / "fixtures" / "hancom2022"

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


def cleanup_and_wait_for_hwp_exit(baseline: set[str], settle_seconds: float) -> set[str]:
    """명시적 강제 종료 뒤에도 PID가 사라질 때까지 기다린다.

    ``taskkill``은 요청을 수락한 직후 반환할 수 있다. 곧바로 PID를 다시 읽으면 아직 종료 중인
    ``Hwp.exe``를 LEFTOVER로 오인하고 다음 시나리오를 OCCUPIED로 건너뛴다.
    """
    cleanup_spawned_hwp(baseline)
    return wait_for_hwp_exit(baseline, settle_seconds)


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
    # 읽기 전용으로 열렸다 — 편집 액션이 조용히 무시되므로 정답지로 쓸 수 없다(계획서 §4.24).
    if proc.returncode == 4:
        return "READONLY"
    # 시나리오가 표본 파일을 고쳤다 — 그 정답지는 물론 **다음 실행 전부**를 못 믿는다.
    if proc.returncode == 5:
        return "SAMPLE_DIRTY"
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
    # node 가 **정상 출력을 다 쓴 뒤** 종료 어서션(`UV_HANDLE_CLOSING`)으로 비영 코드를
    # 내는 일이 매 실행 한 건꼴로 있다(매번 다른 시나리오). 산출 JSON 은 멀쩡하므로 한 번
    # 다시 돌려 가른다 — 진짜 실패면 재시도도 같은 코드로 죽고, 종료 잡음이면 초록이 된다.
    last = None
    for attempt in range(2):
        try:
            proc = subprocess.run(cmd, capture_output=True, timeout=timeout, check=False)
        except subprocess.TimeoutExpired:
            return "STALL"
        sys.stdout.write(proc.stdout.decode("utf-8", "replace"))
        if proc.returncode == 0:
            return "OK"
        last = proc.stderr.decode("utf-8", "replace")
        if attempt == 0:
            sys.stdout.write(f"  (재시도 — 종료 코드 {proc.returncode})\n")
    sys.stderr.write((last or "")[-2000:])
    return "ERR"


def validate_rhwp_output(scenario: Path, out_dir: Path) -> str:
    """WASM 단독 실행이 실제 시나리오를 끝까지 수행했는지 확인한다.

    macOS/Linux에는 Hancom COM이 없으므로 새 Oracle 결과를 만들 수 없다. 그렇다고
    반환 JSON만 생성되고 API 오류가 기록된 실행을 성공으로 취급하면 안 된다. 이 검사는
    호출 개수, 호출 순서, 각 호출 오류, 시나리오가 선언한 기대 반환값, SaveAs 산출물을
    확인하는 플랫폼 공통 하한선이다.

    **오류 거부 규칙은 무르게 하지 않는다.** 일부러 죽는 호출은 시나리오가 `expectError` 로
    미리 선언해야 하고, 선언한 문구와 다르게 죽으면 그것도 실패다. 선언해 놓고 안 죽는 것
    역시 실패다 — 계약이 깨진 것은 어느 쪽이든 같다(#4274 리뷰).
    """
    try:
        definition = json.loads(scenario.read_text(encoding="utf-8"))
        result_path = out_dir / f"{scenario.stem}.returns.json"
        result = json.loads(result_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError, AttributeError):
        return "INVALID_OUTPUT"

    if result.get("fatal"):
        return "FATAL"
    expected_calls = (["Open"] if definition.get("open") else []) + [call[0] for call in definition.get("calls", [])]
    calls = result.get("calls")
    if not isinstance(calls, list) or [call.get("call") for call in calls] != expected_calls:
        return "CALL_SEQUENCE"
    try:
        declared = contracts(definition)
    except ValueError as exc:
        print(f"  시나리오 계약 오류: {exc}")
        return "INVALID_CONTRACT"
    breaches = [
        (call, reason)
        for contract, call in zip(declared, calls)
        if (reason := check_call(contract, call, "rhwp")) is not None
    ]
    if breaches:
        for _, reason in breaches:
            print(f"  계약 위반: {reason}")
        return "CALL_ERROR" if any(call.get("error") for call, _ in breaches) else "EXPECT_DIFF"

    if definition.get("saveAs"):
        saved = result.get("saved") or {}
        saved_path = saved.get("path")
        if not saved.get("ok") or not saved_path or not Path(saved_path).is_file():
            return "SAVE_ERROR"
    return "OK"


def oracle_mode(
    system_name: str,
    skip_ocx: bool,
    wasm_only: bool,
    oracle_dir: Path | None,
    fixture: bool = False,
) -> str:
    """실행 호스트와 옵션에서 Oracle 처리 방식을 결정한다.

    Windows 이외의 호스트는 COM을 호출할 수 없다. fixture를 명시하면 읽기 전용 대조를
    하고, 그렇지 않으면 WASM 단독 시나리오 검증으로 낮춘다. 어떤 경우에도 비Windows에서
    Oracle을 새로 수집했다고 주장하지 않는다.
    """
    if wasm_only:
        return "wasm-self-check"
    if skip_ocx or oracle_dir is not None or fixture:
        return "fixture"
    if system_name.lower() == "windows":
        return "live"
    return "wasm-self-check"


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser(description=__doc__)
    # 기본값은 **새 호환 층**이다. 예전 기본값(`legacy`)은 구 studio `hwpctl` 층을 재서,
    # `--impl` 을 빼먹으면 전량 실패가 나고도 원인이 안 보였다(로그 접두어만 `[hwpctl]` 로 다름).
    ap.add_argument(
        "--impl",
        default="npm/hwpctrl-ocx/src/index.mjs",
        help="rhwp 측 구현 (기본: 새 호환 층 | legacy | 패키지 엔트리 경로)",
    )
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
        "--oracle-dir",
        type=Path,
        help=(
            "읽기 전용 Hancom 2022 returns.json 디렉터리. --skip-ocx 없이 지정해도 "
            "fixture 대조를 수행한다. --skip-ocx의 기존 output/ 정답지 재사용과는 별개다."
        ),
    )
    ap.add_argument(
        "--fixture",
        action="store_true",
        help="저장소의 tools/hwpctrl_compat/fixtures/hancom2022 정답지를 읽기 전용으로 대조",
    )
    ap.add_argument(
        "--wasm-only",
        action="store_true",
        help="COM/fixture 대조 없이 WASM 시나리오 호출·저장 자체 검증만 수행",
    )
    ap.add_argument(
        "--cleanup-spawned",
        action="store_true",
        help="시간 초과 뒤 새 한글 PID를 종료 (전용 Windows 계정에서만 명시적으로 사용)",
    )
    args = ap.parse_args()
    if args.wasm_only and (args.skip_ocx or args.oracle_dir is not None or args.fixture):
        ap.error("--wasm-only는 --skip-ocx, --oracle-dir 또는 --fixture와 함께 쓸 수 없습니다")
    if args.fixture and args.oracle_dir is not None:
        ap.error("--fixture와 --oracle-dir는 함께 쓸 수 없습니다")
    if args.cleanup_spawned and platform.system().lower() != "windows":
        ap.error("--cleanup-spawned는 Windows COM 실행에서만 사용할 수 있습니다")

    scenarios = sorted(SCENARIO_DIR.glob("*.json"))
    if args.only:
        scenarios = [p for p in scenarios if p.stem == args.only]
    if not scenarios:
        print("시나리오 없음")
        return 2

    mode = oracle_mode(platform.system(), args.skip_ocx, args.wasm_only, args.oracle_dir, args.fixture)
    ocx_dir = FIXTURE_ROOT if args.fixture else (args.oracle_dir or OUT_ROOT / "ocx")
    rhwp_dir = OUT_ROOT / "rhwp"
    verdict_dir = OUT_ROOT / "verdict"
    directories = (rhwp_dir, verdict_dir) if mode == "fixture" else (ocx_dir, rhwp_dir, verdict_dir)
    for d in directories:
        d.mkdir(parents=True, exist_ok=True)

    status = {}
    comparable = []
    for path in scenarios:
        name = path.stem
        if mode == "fixture":
            status[name] = stored_oracle_status(ocx_dir / f"{name}.returns.json", args.expect_version)
        elif mode == "live":
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
                    leftovers = cleanup_and_wait_for_hwp_exit(baseline, args.quit_settle_seconds)
                if leftovers:
                    status[name] = f"{status.get(name, 'ERR')}/LEFTOVER"
                    print(f"  오라클 {name}: 남은 한글 PID {', '.join(sorted(leftovers))} — 자동 종료하지 않음")
            print(f"  오라클 {name}: {status[name]} ({time.monotonic() - started:.1f}s)")
        else:
            status[name] = "WASM"

        if status[name] not in ("OK", "SKIPPED", "WASM"):
            continue
        rhwp_status = run_rhwp(path, rhwp_dir, args.impl, args.timeout)
        if rhwp_status == "OK" and mode == "wasm-self-check":
            rhwp_status = validate_rhwp_output(path, rhwp_dir)
        print(f"  rhwp {name}: {rhwp_status}")
        if rhwp_status != "OK":
            status[name] = f"{status[name]}/RHWP_{rhwp_status}"
        elif mode != "wasm-self-check":
            comparable.append(name)

    comparison_status = "NOT_RUN"
    if mode != "wasm-self-check":
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
        comparison_status = "OK" if subprocess.run(compare_cmd, check=False).returncode == 0 else "DIFF"
    else:
        with io.open(verdict_dir / "wasm_self_check.json", "w", encoding="utf-8", newline="\n") as fh:
            json.dump({"schemaVersion": "1.0", "scenarios": [path.stem for path in scenarios]}, fh, ensure_ascii=False, indent=2)
            fh.write("\n")

    with io.open(verdict_dir / "run_status.json", "w", encoding="utf-8", newline="\n") as fh:
        json.dump(
            {
                "impl": args.impl,
                "platform": platform.system(),
                "oracleMode": mode,
                "oracleDir": str(ocx_dir) if mode == "fixture" else None,
                "comparisonStatus": comparison_status,
                "status": status,
                "comparableScenarios": comparable,
            },
            fh,
            ensure_ascii=False,
            indent=2,
        )
        fh.write("\n")
    bad = {k: v for k, v in status.items() if v not in ("OK", "SKIPPED", "WASM")}
    if bad:
        print(f"실행 문제: {bad}")
        return 1
    if comparison_status == "DIFF":
        print("차등 비교 불일치")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
