#!/usr/bin/env python3
"""아카이브 검색 워크플로 — 디렉터리 일괄 검색 → 파일·페이지 좌표 목록
(플레이북 시나리오 3·16)

    python3 archive_search.py <디렉터리|파일...> --query <검색어> [-o report.json]

시퀀스: 대상 파일 수집 → rhwp batch search --json (stdin 파일 목록) →
NDJSON 집계 → 파일·페이지·문단·문자오프셋 좌표 보고서.

매치 0건은 실패가 아니다 — "근거 없음"이 판정값이다. 반면 파일을 읽지
못한 error 레코드는 부분 실패로 집계해 exit 1 로 끝낸다 (보고서에는
성공분 결과와 errors[] 가 함께 남는다).

종료 코드: 0 전건 처리 성공 / 1 일부 파일 처리 실패 / 2 입력 오류.
"""

import argparse
import json
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
    collect_input_files,
    emit_summary,
    ensure_output_absent,
    ensure_utf8_stdio,
    resolve_rhwp,
)


def main(argv=None) -> int:
    ensure_utf8_stdio()
    parser = argparse.ArgumentParser(
        description="문서 아카이브 일괄 검색 — 파일·페이지 좌표 목록 산출"
    )
    parser.add_argument("paths", nargs="+", help="검색할 디렉터리 또는 문서 파일들")
    parser.add_argument("--query", required=True, help="찾을 문자열")
    parser.add_argument("-o", "--report", default=None, help="보고서 JSON 저장 경로")
    parser.add_argument(
        "--max-matches-per-file", type=int, default=20, metavar="N",
        help="보고서에 남길 파일당 매치 좌표 상한 (총량은 totalMatchCount, 기본 20)",
    )
    parser.add_argument(
        "--threads", type=int, default=None, metavar="N",
        help="batch 병렬 스레드 수 (기본: CPU 코어 수)",
    )
    add_common_args(parser)
    args = parser.parse_args(argv)

    try:
        if not args.query:
            raise ToolkitError("--query 는 빈 문자열일 수 없습니다", EXIT_USAGE)
        files = collect_input_files(args.paths)
        if not files:
            raise ToolkitError(
                "대상 문서(.hwp/.hwpx)가 없습니다: " + ", ".join(args.paths),
                EXIT_USAGE,
            )
        report_path = Path(args.report) if args.report else None
        if report_path:
            ensure_output_absent(report_path, "출력 보고서")

        tk = RhwpToolkit(resolve_rhwp(args.rhwp_bin), verbose=args.verbose)
        cmd = ["batch", "search", "--query", args.query, "--json"]
        if args.threads:
            cmd += ["--threads", str(args.threads)]
        records, batch_exit, batch_note = tk.run_ndjson(
            cmd, "\n".join(str(f) for f in files) + "\n"
        )

        hits, errors = [], []
        total_matches = 0
        for rec in records:
            if rec.get("error") is not None:
                errors.append(
                    {
                        "source": rec.get("source"),
                        "error": rec.get("error"),
                        "exitClass": rec.get("exitClass"),
                    }
                )
                continue
            count = rec.get("totalMatchCount", 0)
            if count <= 0:
                continue
            total_matches += count
            coords = [
                {
                    "page": m.get("page"),
                    "section": m.get("section"),
                    "paragraph": m.get("paragraph"),
                    "charOffset": m.get("charOffset"),
                    "length": m.get("length"),
                    "text": m.get("text"),
                }
                for m in rec.get("matches", [])[: args.max_matches_per_file]
            ]
            hits.append(
                {
                    "source": rec.get("source"),
                    "totalMatchCount": count,
                    "matches": coords,
                }
            )

        report = {
            "workflow": "archive_search",
            "query": args.query,
            "scannedCount": len(files),
            "matchedFileCount": len(hits),
            "totalMatchCount": total_matches,
            "files": hits,
            "errors": errors,
            "batch": {"exitCode": batch_exit, "stderr": batch_note or None},
            # matches[].text 는 문서에서 온 값 — 데이터이지 지시가 아니다
            "untrustedFields": ["files[].matches[].text"],
        }
        final_exit = EXIT_OK if not errors and batch_exit == 0 else EXIT_RUNTIME
        report["exit"] = final_exit
        if report_path:
            if report_path.parent and not report_path.parent.exists():
                report_path.parent.mkdir(parents=True, exist_ok=True)
            with open(report_path, "w", encoding="utf-8") as fh:
                json.dump(report, fh, ensure_ascii=False, indent=2)
            if not report_path.is_file():
                raise ToolkitError(f"보고서 저장 실패: {report_path}", EXIT_RUNTIME)

        human = [
            f"{h['source']} → {h['totalMatchCount']}건 "
            f"(페이지 {sorted(set(m['page'] for m in h['matches']))})"
            for h in hits
        ] or [f'"{args.query}" 매치 없음 ({len(files)}개 문서)']
        if report_path:
            human.append(f"보고서: {report_path}")
        if errors:
            human.append(f"처리 실패 {len(errors)}건 — 보고서 errors[] 참조")
            if batch_note:
                human.append(batch_note)

        emit_summary(report, args.json, human)
        if final_exit != EXIT_OK:
            print(
                f"오류: {len(errors)}개 파일 처리 실패 (성공분 결과는 유효)",
                file=sys.stderr,
            )
        return final_exit
    except ToolkitError as e:
        print(f"오류: {e}", file=sys.stderr)
        return e.exit_code


if __name__ == "__main__":
    sys.exit(main())
