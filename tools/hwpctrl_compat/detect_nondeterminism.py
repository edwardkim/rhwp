"""같은 탐침을 여러 번 돌려 **저장본이 갈리는 필드**를 기계적으로 수확한다.

## 왜 있나

저장본 차분 관측(계획서 §4.19)의 적은 오라클의 비결정이다 — 문서를 연 뒤 첫 저장에만
붙었다 안 붙었다 하는 비트(`hwp5_gen_shape_attr_bit28`), 이력에 따라 남기도 지워지기도
하는 표시 비트(§4.22), 편집 표시(`list_header_width_ref` bit 8) 따위. 여태 이것들을
게이트가 붉어진 뒤에 **손으로** 찾아냈다. 이 도구는 같은 탐침을 여러 번 돌려 저장본끼리
전수 비교(`ir-sweep`)하고, 갈린 필드를 그대로 보고한다 — `ignorePaths` 후보와 그 근거가
한 번에 나온다.

같은 실행 안의 반복이 아니라 **프로세스를 갈라** 여러 번 연다 — 비결정의 근원이 "연 뒤 첫
저장" 같은 세션 상태이기 때문이다(같은 세션에서 두 번 저장하면 안 갈린다). 갈릴 확률이
반반이면 N 회가 모두 같은 쪽일 확률이 2^-(N-1) 이라 기본 4회를 돈다 — **"갈린 필드 없음"은
그 횟수 안에서의 이야기지 증명이 아니다.**

## 쓰임

    python tools/hwpctrl_compat/detect_nondeterminism.py tools/hwpctrl_compat/probes/pZ3-bit28.json

동시에 다른 오라클 판정을 돌리지 말 것 — 서로의 Hwp.exe 를 죽인다.
"""

from __future__ import annotations

import argparse
import io
import json
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
DEFAULT_EXE = REPO / "target" / "release" / "rhwp.exe"


def run_once(scenario: Path, out_dir: Path, timeout: int) -> bool:
    out_dir.mkdir(parents=True, exist_ok=True)
    proc = subprocess.run(
        [sys.executable, str(HERE / "runner_ocx.py"), str(scenario), "--out", str(out_dir)],
        timeout=timeout,
        check=False,
    )
    return proc.returncode == 0


def sweep(exe: Path, a: Path, b: Path) -> list[dict] | None:
    proc = subprocess.run(
        [str(exe), "ir-sweep", str(a), str(b), "--json"], capture_output=True, check=False
    )
    if proc.returncode not in (0, 3):
        return None
    try:
        return json.loads(proc.stdout.decode("utf-8", "replace"))["divergences"]
    except (json.JSONDecodeError, KeyError):
        return None


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("scenario", help="SaveAs 를 담은 탐침/시나리오 JSON")
    ap.add_argument("--out", default=str(REPO / "output" / "poc" / "hwpctrl" / "nondet"))
    ap.add_argument("--exe", default=str(DEFAULT_EXE))
    ap.add_argument("--timeout", type=int, default=300)
    ap.add_argument("--runs", type=int, default=4, help="여는 횟수 (기본 4)")
    args = ap.parse_args()

    scenario = Path(args.scenario)
    spec = json.load(io.open(scenario, encoding="utf-8"))
    names = list(spec.get("paths", {}))
    if not names:
        print("이 시나리오는 중간 저장(`paths`)이 없다 — 잴 것이 없다.")
        return 2

    base = Path(args.out) / spec["id"]
    runs = [base / f"run{i + 1}" for i in range(max(2, args.runs))]
    for run_dir in runs:
        if not run_once(scenario, run_dir, args.timeout):
            print(f"오라클 실행 실패: {run_dir}")
            return 1

    exe = Path(args.exe)
    verdicts: dict[str, list[dict]] = {}
    for name in names:
        # 러너는 {out} 아래 시나리오가 적은 이름 그대로 저장한다. 1회차를 기준 삼아
        # 나머지 회차와 각각 견주고 합집합을 낸다.
        rel = Path(str(spec["paths"][name].get("win", name)).replace("{out}/", "")).name
        rows_union: list[dict] = []
        ok = True
        for other in runs[1:]:
            a, b = runs[0] / rel, other / rel
            if not a.exists() or not b.exists():
                print(f"  {name}: 저장본 없음 — 건너뜀")
                ok = False
                break
            rows = sweep(exe, a, b)
            if rows is None:
                print(f"  {name}: ir-sweep 실패")
                ok = False
                break
            rows_union.extend(rows)
        if ok:
            verdicts[name] = rows_union

    fields: dict[str, list[str]] = {}
    for name, rows in verdicts.items():
        for r in rows:
            fields.setdefault(r["path"], []).append(name)

    print(f"저장본 {len(verdicts)}벌 × {len(runs)}회 실행 비교")
    if not fields:
        print(f"**갈린 필드 없음** — {len(runs)}회 안에서는 결정적이다(증명이 아니다).")
        return 0
    print(f"**비결정 필드 {len(fields)}개** — `ignorePaths` 후보와 근거다:")
    for path, where in sorted(fields.items()):
        print(f"  {path}")
        print(f"     갈린 저장본: {sorted(set(where))}")
    report = base / "report.json"
    with io.open(report, "w", encoding="utf-8", newline="\n") as fh:
        json.dump(
            {"schemaVersion": "1.0", "scenario": spec["id"],
             "fields": {k: sorted(set(v)) for k, v in fields.items()}},
            fh, ensure_ascii=False, indent=2)
        fh.write("\n")
    print(f"→ {report}")
    return 3


if __name__ == "__main__":
    raise SystemExit(main())
