#!/usr/bin/env python
"""[R50] 편집 전후 render-diff 회귀 게이트 실측 러너.

대표 편집 3종(set-cell / fill-fields / replace-text)을 실물 fixture 에 적용하고,
편집 전후를 `rhwp render-diff <전> <후> --json`(pair 모드)으로 비교해 maxDisp
분포·status·종료 코드를 기록한다. 게이트 전제인 **결정성**(같은 비교를 반복해도
봉투가 완전히 동일한지)을 먼저 검증한다.

측정만 한다 — 임계 확정은 메인테이너 몫이다. 결과 해석과 임계 제안은
`mydocs/report/edit_render_gate_r1_20260808.md` 참조.

사용:
    python tools/measure_edit_render_gate.py
    python tools/measure_edit_render_gate.py --bin target/release-test/rhwp.exe --runs 3

산출물: --out-dir (기본 output/r50_edit_render_gate/) 아래 편집 결과물과 diff 봉투
JSON 전부. 저장소에는 아무것도 쓰지 않는다(output/ 은 gitignore).
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# 계약 테스트(tests/edit_render_diff_gate.rs)와 같은 실물 fixture.
FORM_HWPX = ROOT / "samples" / "2025년 기부·답례품 실적 지자체 보고서_양식.hwpx"
FIELD_HWP = ROOT / "samples" / "field-01.hwp"


def run_cmd(exe: Path, args: list[str]) -> tuple[int, bytes, bytes]:
    """rhwp 하위 명령 실행 — (종료코드, stdout, stderr). stdout 은 JSON 계약 채널."""
    p = subprocess.run(
        [str(exe), *args], capture_output=True, timeout=600, check=False
    )
    return p.returncode, p.stdout, p.stderr


def run_json(exe: Path, args: list[str]) -> tuple[int, dict]:
    rc, out, err = run_cmd(exe, args)
    try:
        return rc, json.loads(out.decode("utf-8"))
    except (ValueError, UnicodeDecodeError) as e:
        sys.exit(
            f"오류: stdout 이 순수 JSON 이 아님 ({e})\n명령: rhwp {' '.join(args)}\n"
            f"stderr:\n{err.decode('utf-8', 'replace')[:2000]}"
        )


def save(out_dir: Path, name: str, payload: dict) -> None:
    (out_dir / name).write_text(
        json.dumps(payload, ensure_ascii=False, indent=1), encoding="utf-8"
    )


def changed_pages(envelope: dict) -> list[int]:
    """maxDisp>0 또는 구조 불일치가 있는 페이지 번호 목록 (국소성 판정)."""
    return [
        p["page"]
        for p in envelope["pages"]
        if p["maxDisp"] > 0 or p["structureMismatch"]
    ]


def pair_diff(
    exe: Path,
    a: Path,
    b: Path,
    out_dir: Path,
    tag: str,
    runs: int,
    extra: list[str] | None = None,
) -> tuple[int, dict, bool]:
    """전후 pair diff 를 `runs`회 반복 — (종료코드, 봉투, 결정성 여부)."""
    envelopes = []
    rc = 0
    for i in range(runs):
        rc, v = run_json(
            exe, ["render-diff", str(a), str(b), "--json", *(extra or [])]
        )
        envelopes.append(v)
        save(out_dir, f"{tag}_run{i + 1}.json", v)
    deterministic = all(v == envelopes[0] for v in envelopes[1:])
    return rc, envelopes[0], deterministic


def pick_top_level_cell(exe: Path, doc: Path) -> tuple[int, int, int]:
    """export-tables 로 본문 최상위 표 첫 셀 좌표를 고른다 (계약 테스트와 동일 로직)."""
    rc, v = run_json(exe, ["export-tables", str(doc), "--json"])
    if rc != 0:
        sys.exit(f"오류: export-tables 실패 (exit {rc})")
    table = next(t for t in v["tables"] if "containerPath" not in t)
    cell = table["cells"][0]
    return table["index"], cell["row"], cell["col"]


def main() -> int:
    if hasattr(sys.stdout, "reconfigure"):  # Windows cp949 콘솔 대비
        sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--bin",
        default=str(ROOT / "target" / "release-test" / "rhwp.exe"),
        help="rhwp 바이너리 (기본: target/release-test/rhwp.exe)",
    )
    ap.add_argument(
        "--out-dir", default=str(ROOT / "output" / "r50_edit_render_gate")
    )
    ap.add_argument("--runs", type=int, default=2, help="결정성 반복 횟수 (기본 2)")
    args = ap.parse_args()

    exe = Path(args.bin)
    if not exe.exists():
        sys.exit(
            f"오류: 바이너리 없음: {exe}\n"
            "먼저 빌드: cargo build --profile release-test --bin rhwp"
        )
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    rows: list[tuple[str, int, dict, bool]] = []

    # ── 1. 결정성 선확인: 같은 문서 자기 pair diff 반복 → maxDisp 0.0, 봉투 동일.
    for tag, doc in [("self_field01", FIELD_HWP), ("self_form", FORM_HWPX)]:
        rc, env, det = pair_diff(exe, doc, doc, out_dir, tag, args.runs)
        rows.append((f"결정성 {tag}", rc, env, det))

    # ── 2. set-cell: 본문 최상위 표 첫 셀에 실값 기록 (기존 계약 테스트와 같은 편집).
    tbl, row, col = pick_top_level_cell(exe, FORM_HWPX)
    setcell_out = out_dir / "setcell.hwpx"
    rc, _, _ = run_cmd(
        exe,
        [
            "edit", "set-cell", str(FORM_HWPX),
            "--table", str(tbl), "--row", str(row), "--col", str(col),
            "--text", "실증테스트값", "-o", str(setcell_out), "--json",
        ],
    )
    if rc != 0:
        sys.exit(f"오류: set-cell 실패 (exit {rc})")
    rc, env, det = pair_diff(exe, FORM_HWPX, setcell_out, out_dir, "setcell", args.runs)
    rows.append(("set-cell 전후", rc, env, det))
    # 제안 임계 600px 적용 시 PASS 인지 (편집 페이지 상한 제안 근거).
    rc, env, det = pair_diff(
        exe, FORM_HWPX, setcell_out, out_dir, "setcell_t600", 1, ["--max-disp", "600"]
    )
    rows.append(("set-cell 전후 @600px", rc, env, det))

    # ── 3. fill-fields: 누름틀 채우기.
    filled_out = out_dir / "filled.hwp"
    rc, _, _ = run_cmd(
        exe,
        [
            "edit", "fill-fields", str(FIELD_HWP),
            "--data", '{"회사명":"주식회사 검증"}',
            "-o", str(filled_out), "--json",
        ],
    )
    if rc != 0:
        sys.exit(f"오류: fill-fields 실패 (exit {rc})")
    rc, env, det = pair_diff(exe, FIELD_HWP, filled_out, out_dir, "fill", args.runs)
    rows.append(("fill-fields 전후", rc, env, det))
    # 구조 불일치는 임계를 아무리 키워도 침묵하지 않는지.
    rc, env, det = pair_diff(
        exe, FIELD_HWP, filled_out, out_dir, "fill_t1e5", 1, ["--max-disp", "100000"]
    )
    rows.append(("fill-fields 전후 @1e5px", rc, env, det))

    # ── 4. replace-text 동폭 치환(회사→기관): 기하 변화 0 카나리.
    same_out = out_dir / "replaced_same.hwp"
    rc, _, _ = run_cmd(
        exe,
        [
            "edit", "replace-text", str(FIELD_HWP),
            "--find", "회사", "--replace", "기관",
            "-o", str(same_out), "--json",
        ],
    )
    if rc != 0:
        sys.exit(f"오류: replace-text(동폭) 실패 (exit {rc})")
    rc, env, det = pair_diff(exe, FIELD_HWP, same_out, out_dir, "replace_same", args.runs)
    rows.append(("replace-text 동폭 전후", rc, env, det))

    # ── 5. red 주입: 폭이 크게 다른 장문 치환 — 게이트가 red(exit 3)로 가는지 실증.
    long_out = out_dir / "replaced_long.hwp"
    rc, _, _ = run_cmd(
        exe,
        [
            "edit", "replace-text", str(FIELD_HWP),
            "--find", "회사", "--replace", "주식회사법인등기부등본상호명",
            "-o", str(long_out), "--json",
        ],
    )
    if rc != 0:
        sys.exit(f"오류: replace-text(장문) 실패 (exit {rc})")
    rc, env, det = pair_diff(exe, FIELD_HWP, long_out, out_dir, "replace_long", args.runs)
    rows.append(("replace-text 장문(red 주입)", rc, env, det))

    # ── 요약표.
    print(f"\n=== R50 편집 전후 render-diff 실측 (runs={args.runs}) ===")
    print(f"{'측정':<28} {'exit':>4} {'status':<16} {'maxDisp':>9} "
          f"{'쪽수':>7} {'변화쪽':<10} 결정적")
    ok = True
    for name, rc, env, det in rows:
        pages = f"{env['pageCountA']}->{env['pageCountB']}"
        ch = ",".join(map(str, changed_pages(env))) or "-"
        print(
            f"{name:<28} {rc:>4} {env['status']:<16} {env['maxDisp']:>9.1f} "
            f"{pages:>7} {ch:<10} {det}"
        )
        ok = ok and det
    print(f"\n봉투 저장: {out_dir}")
    if not ok:
        print("경고: 비결정 측정 발견 — 게이트를 hard 로 올리기 전에 원인 규명 필요")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
