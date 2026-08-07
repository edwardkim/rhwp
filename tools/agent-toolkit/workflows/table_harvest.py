#!/usr/bin/env python3
"""표 데이터 수확 워크플로 — HWP 표 → CSV + 행/열 재독 대조 (플레이북 시나리오 2)

    python3 table_harvest.py <문서.hwp|hwpx> -o <출력폴더> [--table N] [--bom]

시퀀스: export-tables --json (격자 계약) → 표마다 table-to-csv → 산출 CSV 를
다시 읽어 행·열 수를 export-tables 격자와 대조.

성공(exit 0) 조건:
  * 수확한 표마다 CSV 파일이 실제로 존재한다
  * CSV 를 재독한 행 수 == export-tables 의 rows, 모든 행의 열 수 == cols
  * 표가 0개면 수확할 것이 없으므로 실패다 (exit 1)
대조가 어긋나면 만들어진 CSV 를 지우고 비 0 으로 끝낸다.

종료 코드: 0 성공 / 1 실행·검증 실패(표 없음 포함) / 2 입력 오류.
"""

import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "lib"))
from toolkit import (  # noqa: E402
    EXIT_OK,
    EXIT_RUNTIME,
    EXIT_USAGE,
    RhwpToolkit,
    ToolkitError,
    add_common_args,
    emit_summary,
    ensure_output_absent,
    ensure_utf8_stdio,
    read_csv_grid,
    resolve_rhwp,
)


def main(argv=None) -> int:
    ensure_utf8_stdio()
    parser = argparse.ArgumentParser(
        description="문서의 최상위 표를 CSV 로 수확하고 행/열 수를 재독 대조"
    )
    parser.add_argument("document", help="표가 있는 문서 (.hwp/.hwpx)")
    parser.add_argument("-o", "--out-dir", required=True, help="CSV 를 모을 폴더")
    parser.add_argument(
        "--table", type=int, default=None, metavar="N",
        help="이 표만 수확 (export-tables 의 index 값; 생략 시 전부)",
    )
    parser.add_argument(
        "--bom", action="store_true",
        help="CSV 에 UTF-8 BOM 추가 (엑셀 한글 깨짐 방지)",
    )
    add_common_args(parser)
    args = parser.parse_args(argv)

    created_csv_paths: list[Path] = []
    try:
        document = Path(args.document)
        if not document.is_file():
            raise ToolkitError(f"문서가 없습니다: {document}", EXIT_USAGE)
        out_dir = Path(args.out_dir)

        tk = RhwpToolkit(resolve_rhwp(args.rhwp_bin), verbose=args.verbose)

        # ① 격자 계약 — 어떤 표가 몇 행 몇 열인지
        grid, _ = tk.run_json(["export-tables", str(document), "--json"])
        tables = grid.get("tables", [])
        if args.table is not None:
            tables = [t for t in tables if t.get("index") == args.table]
            if not tables:
                have = [t.get("index") for t in grid.get("tables", [])]
                raise ToolkitError(
                    f"표 {args.table} 이 없습니다 (있는 index: {have})", EXIT_RUNTIME
                )
        if not tables:
            raise ToolkitError(
                f"수확할 표가 없습니다 (tableCount=0): {document}", EXIT_RUNTIME
            )

        csv_paths = [out_dir / f"table{t['index']}.csv" for t in tables]
        for csv_path in csv_paths:
            ensure_output_absent(csv_path, "출력 CSV")
        out_dir.mkdir(parents=True, exist_ok=True)

        harvested = []
        for t in tables:
            idx = t["index"]
            csv_path = out_dir / f"table{idx}.csv"
            cmd = [
                "table-to-csv", str(document),
                "--table", str(idx),
                "-o", str(csv_path),
                "--json",
            ]
            if args.bom:
                cmd.append("--bom")
            # 사전 충돌 검사를 통과한 새 경로라 실행 중 부분 파일도 정리할 수 있다.
            created_csv_paths.append(csv_path)
            tk.run_json(cmd)
            if not csv_path.is_file():
                raise ToolkitError(
                    f"table-to-csv 가 성공을 보고했지만 CSV 가 없습니다: {csv_path}",
                    EXIT_RUNTIME,
                )

            # ② 재독 대조 — 산출 CSV 의 실제 행·열 수 vs 격자 계약
            rows = read_csv_grid(csv_path)
            want_rows, want_cols = t.get("rows"), t.get("cols")
            bad_cols = [i for i, r in enumerate(rows) if len(r) != want_cols]
            if len(rows) != want_rows or bad_cols:
                raise ToolkitError(
                    f"표 {idx} 재독 불일치: CSV {len(rows)}행"
                    f"(기대 {want_rows}) / 열 수 어긋난 행 {bad_cols[:5]}"
                    f"(기대 {want_cols}열)",
                    EXIT_RUNTIME,
                )
            harvested.append(
                {
                    "index": idx,
                    "rows": want_rows,
                    "cols": want_cols,
                    "cellCount": t.get("cellCount"),
                    "csv": str(csv_path),
                }
            )

        summary = {
            "workflow": "table_harvest",
            "source": str(document),
            "outDir": str(out_dir),
            "tableCount": len(harvested),
            "tables": harvested,
            "rereadVerified": True,
            "exit": EXIT_OK,
        }
        emit_summary(
            summary,
            args.json,
            [f"표 {h['index']}: {h['rows']}x{h['cols']} → {h['csv']}" for h in harvested]
            + [f"재독 검증 통과 ({len(harvested)}개 표, 행/열 수 일치)"],
        )
        return EXIT_OK
    except ToolkitError as e:
        for p in created_csv_paths:  # 이번 호출이 만든 미검증 CSV만 정리한다
            try:
                Path(p).unlink()
            except OSError:
                pass
        print(f"오류: {e}", file=sys.stderr)
        return e.exit_code


if __name__ == "__main__":
    sys.exit(main())
