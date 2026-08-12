"""3자 차등 대조 오케스트레이터 — 기안기 측정과 3자 판정을 한 번에 (계획서 §6.3.3·§9-6 — PR #4470).

    python tools/hwpctrl_compat/run_3way.py --only doc-basic --only field-read
    python tools/hwpctrl_compat/run_3way.py --skip-web        # 기존 기안기 산출물 재사용

전제: `output/poc/hwpctrl/ocx/`(Windows COM 정답지)와 `output/poc/hwpctrl/rhwp/`(구현 산출물)가
이미 있어야 한다. 이 도구는 기안기 축(`webhwp/`)만 새로 잰 뒤 `compare3.py` 를 부른다.

## 규율 — 어기면 안 되는 것

- **저빈도 수동 전용이다. CI 에 물리지 않는다.** 기본 URL 은 한컴 공개 데모다. 전 시나리오
  실행도 페이지 로드 ~80회라 한 번은 괜찮지만, 반복 실행은 `--only` 로 좁혀서 한다.
- 시나리오당 브라우저 프로세스 하나, 직렬 실행. COM 과 같은 격리 이유다.
- 버전 스탬프는 러너가 자동으로 남기고 `compare3.py` 가 강제한다.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
SCENARIO_DIR = HERE / "scenarios"
OUT_ROOT = REPO / "output" / "poc" / "hwpctrl"


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--only", action="append", dest="only", help="시나리오 id (반복 가능, 권장)")
    ap.add_argument("--url", help="기안기 URL (기본: 러너의 공개 데모. 자가 호스팅이면 지정)")
    ap.add_argument("--timeout-ms", type=int, default=60000)
    ap.add_argument("--skip-web", action="store_true", help="기안기 재측정 없이 기존 산출물로 판정만")
    args = ap.parse_args()

    scenarios = sorted(SCENARIO_DIR.glob("*.json"))
    if args.only:
        allowed = set(args.only)
        scenarios = [p for p in scenarios if p.stem in allowed]
    if not scenarios:
        print("시나리오 없음")
        return 2

    web_dir = OUT_ROOT / "webhwp"
    if not args.skip_web:
        web_dir.mkdir(parents=True, exist_ok=True)
        for path in scenarios:
            cmd = [
                "node", str(HERE / "runner_webhwp.mjs"), str(path),
                "--out", str(web_dir), "--timeout-ms", str(args.timeout_ms),
            ]
            if args.url:
                cmd += ["--url", args.url]
            proc = subprocess.run(cmd, check=False)
            status = "OK" if proc.returncode == 0 else f"ERR({proc.returncode})"
            print(f"  기안기 {path.stem}: {status}")

    compare_cmd = [
        sys.executable, str(HERE / "compare3.py"),
        "--ocx", str(OUT_ROOT / "ocx"),
        "--rhwp", str(OUT_ROOT / "rhwp"),
        "--web", str(web_dir),
        "--out", str(OUT_ROOT / "verdict3"),
    ]
    for path in scenarios:
        compare_cmd += ["--scenario", path.stem]
    return subprocess.run(compare_cmd, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())
