#!/usr/bin/env python3
"""배포본 동일성 검증 워크플로 — "이 두 파일, 같은 문서인가?"
(플레이북 시나리오 8)

    python3 distribution_verify.py <원본> <배포본> [-o report.json] [--skip-svg]

시퀀스: ① rhwp render-diff A B --json (기하 게이트 — 변위 px·쪽수·구조)
→ 기하가 같을 때만 ② export-svg 양쪽 → 페이지별 SVG 바이트 대조
(render-diff 는 기하 게이트라 같은 자리·같은 크기의 이미지 내용 교체를
못 본다 — 바이트 대조가 그 구멍을 닫는다).

판정은 rhwp 검증 게이트와 같은 계열의 종료 코드로 낸다:
  0 동일 (기하 PASS + 전 페이지 SVG 바이트 일치)
  1 실행 실패 (파일 손상 등 — 판정 불능)
  2 입력 오류
  3 다름 (기하 회귀 또는 SVG 바이트 불일치 — 무엇이 다른지 보고서에 남김)
"""

import argparse
import json
import shutil
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "lib"))
from toolkit import (  # noqa: E402
    EXIT_DIFFERENT,
    EXIT_OK,
    EXIT_RUNTIME,
    EXIT_USAGE,
    RhwpToolkit,
    ToolkitError,
    add_common_args,
    emit_summary,
    ensure_output_absent,
    ensure_utf8_stdio,
    resolve_rhwp,
)


def svg_byte_compare(tk, file_a, file_b):
    """export-svg 양쪽 → 페이지별 바이트 대조 → (동일여부, 상세)."""
    workdir = Path(tempfile.mkdtemp(prefix="dist_verify_"))
    try:
        manifests = []
        for tag, f in (("A", file_a), ("B", file_b)):
            out = workdir / tag
            out.mkdir()
            envelope, _ = tk.run_json(["export-svg", str(f), "-o", str(out), "--json"])
            manifests.append(envelope)
        pages_a = sorted(manifests[0].get("pages", []), key=lambda p: p["page"])
        pages_b = sorted(manifests[1].get("pages", []), key=lambda p: p["page"])
        if len(pages_a) != len(pages_b):
            return False, {
                "reason": "renderedPageCountMismatch",
                "pagesA": len(pages_a),
                "pagesB": len(pages_b),
            }
        diff_pages = []
        for pa, pb in zip(pages_a, pages_b):
            if Path(pa["path"]).read_bytes() != Path(pb["path"]).read_bytes():
                diff_pages.append(pa["page"])
        if diff_pages:
            return False, {"reason": "svgBytesDiffer", "pages": diff_pages}
        return True, {"comparedPages": len(pages_a)}
    finally:
        shutil.rmtree(workdir, ignore_errors=True)


def main(argv=None) -> int:
    ensure_utf8_stdio()
    parser = argparse.ArgumentParser(
        description="배포본 동일성 검증 — render-diff 기하 게이트 + SVG 바이트 대조"
    )
    parser.add_argument("file_a", help="원본 문서")
    parser.add_argument("file_b", help="대조할 배포본 문서")
    parser.add_argument("-o", "--report", default=None, help="판정 보고서 JSON 저장 경로")
    parser.add_argument(
        "--skip-svg", action="store_true",
        help="SVG 바이트 대조 생략 (기하 게이트만 — 이미지 내용 교체는 못 잡는다)",
    )
    parser.add_argument(
        "--max-disp", type=float, default=None, metavar="PX",
        help="기하 게이트 변위 임계 px (rhwp render-diff --max-disp 로 전달)",
    )
    add_common_args(parser)
    args = parser.parse_args(argv)

    try:
        file_a, file_b = Path(args.file_a), Path(args.file_b)
        for f in (file_a, file_b):
            if not f.is_file():
                raise ToolkitError(f"파일이 없습니다: {f}", EXIT_USAGE)
        report_path = Path(args.report) if args.report else None
        if report_path:
            ensure_output_absent(report_path, "출력 보고서")

        tk = RhwpToolkit(resolve_rhwp(args.rhwp_bin), verbose=args.verbose)

        # ① 기하 게이트 — --json 은 회귀를 exit 3 으로 낸다
        cmd = ["render-diff", str(file_a), str(file_b), "--json"]
        if args.max_disp is not None:
            cmd += ["--max-disp", str(args.max_disp)]
        geom, geom_exit = tk.run_json(cmd, ok_exits=(0, 3))

        geometry = {
            "status": geom.get("status"),
            "regression": geom.get("regression"),
            "maxDisp": geom.get("maxDisp"),
            "pageCountA": geom.get("pageCountA"),
            "pageCountB": geom.get("pageCountB"),
            "pageCountMismatch": geom.get("pageCountMismatch"),
            "hardStructPages": geom.get("hardStructPages"),
        }
        identical = geom_exit == 0 and not geom.get("regression", False)

        svg_result = None
        if identical and not args.skip_svg:
            # ② 바이트 대조 — 기하가 같을 때만 의미가 있다
            same, detail = svg_byte_compare(tk, file_a, file_b)
            svg_result = detail
            identical = same

        verdict = "identical" if identical else "different"
        final_exit = EXIT_OK if identical else EXIT_DIFFERENT
        report = {
            "workflow": "distribution_verify",
            "fileA": str(file_a),
            "fileB": str(file_b),
            "verdict": verdict,
            "geometry": geometry,
            "svgByteCompare": svg_result if not args.skip_svg else "skipped",
            "exit": final_exit,
        }
        if report_path:
            if report_path.parent and not report_path.parent.exists():
                report_path.parent.mkdir(parents=True, exist_ok=True)
            with open(report_path, "w", encoding="utf-8") as fh:
                json.dump(report, fh, ensure_ascii=False, indent=2)

        human = [
            f"판정: {'동일' if identical else '다름'} "
            f"(기하 {geometry['status']}, maxDisp={geometry['maxDisp']})",
        ]
        if svg_result:
            human.append(f"SVG 바이트 대조: {svg_result}")
        if args.report:
            human.append(f"보고서: {args.report}")
        emit_summary(report, args.json, human)
        return final_exit
    except ToolkitError as e:
        print(f"오류: {e}", file=sys.stderr)
        return e.exit_code


if __name__ == "__main__":
    sys.exit(main())
