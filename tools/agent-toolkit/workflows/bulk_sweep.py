#!/usr/bin/env python3
"""대량 문서 스윕 워크플로 — 메타/본문/구조 일괄 수집 (플레이북 시나리오 4)

    python3 bulk_sweep.py <디렉터리|파일...> -o <결과폴더> \\
        [--tasks info,export-text,export-structure] [--min-pages N]

시퀀스: 대상 수집 → rhwp batch info --json (메타, 항상 실행) →
[--min-pages 필터] → 나머지 task 를 batch 로 실행 → task 별 NDJSON 저장
+ summary.json (성공/실패 목록).

부분 실패 계약: batch 는 일부 파일이 실패해도 성공 레코드를 보존한다.
이 워크플로도 같은 원칙 — 실패 파일은 summary.json 의 failed[] 로 격리하고
성공분 NDJSON 은 그대로 남기되, 실패가 하나라도 있으면 exit 1 로 끝낸다
(재시도 대상은 failed[].source 목록이다).

종료 코드: 0 전건 성공 / 1 일부·전부 실패 / 2 입력 오류.
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
    emit_summary,
    ensure_output_absent,
    ensure_utf8_stdio,
    collect_input_files,
    resolve_rhwp,
)

SUPPORTED_TASKS = ("info", "export-text", "export-structure")


def run_batch_task(tk, task, files, threads, out_dir):
    """batch <task> 실행 → 레코드·파일 실패·NDJSON·프로세스 실패를 반환한다."""
    cmd = ["batch", task, "--json"]
    if threads:
        cmd += ["--threads", str(threads)]
    records, batch_exit, batch_note = tk.run_ndjson(
        cmd, "\n".join(str(f) for f in files) + "\n"
    )
    ndjson_path = out_dir / (task.replace("-", "_") + ".ndjson")
    with open(ndjson_path, "w", encoding="utf-8") as fh:
        for rec in records:
            fh.write(json.dumps(rec, ensure_ascii=False) + "\n")
    failed = [
        {
            "source": r.get("source"),
            "error": r.get("error"),
            "exitClass": r.get("exitClass"),
        }
        for r in records
        if r.get("error") is not None
    ]
    batch_failure = None
    # 파일별 error 레코드가 있으면 부분 실패 계약으로 이미 집계된다. 레코드 없이
    # 끝난 비정상 종료만 별도 실패로 남겨 성공(0) 오판을 막는다.
    if batch_exit != 0 and not failed:
        batch_failure = {"task": task, "exitCode": batch_exit}
        if batch_note:
            batch_failure["stderr"] = batch_note
    return records, failed, ndjson_path, batch_failure


def main(argv=None) -> int:
    ensure_utf8_stdio()
    parser = argparse.ArgumentParser(
        description="문서 뭉치의 메타/본문/구조를 batch 로 일괄 수집"
    )
    parser.add_argument("paths", nargs="+", help="스윕할 디렉터리 또는 문서 파일들")
    parser.add_argument("-o", "--out-dir", required=True, help="NDJSON·요약을 모을 폴더")
    parser.add_argument(
        "--tasks", default="info,export-text,export-structure", metavar="목록",
        help="쉼표 구분 batch 축: info|export-text|export-structure "
        "(기본: 셋 다, info 는 항상 포함)",
    )
    parser.add_argument(
        "--min-pages", type=int, default=None, metavar="N",
        help="info 의 pageCount 가 N 이상인 문서만 본문/구조 수집",
    )
    parser.add_argument(
        "--threads", type=int, default=None, metavar="N",
        help="batch 병렬 스레드 수 (기본: CPU 코어 수)",
    )
    add_common_args(parser)
    args = parser.parse_args(argv)

    try:
        tasks = [t.strip() for t in args.tasks.split(",") if t.strip()]
        bad = [t for t in tasks if t not in SUPPORTED_TASKS]
        if bad:
            raise ToolkitError(
                f"지원하지 않는 task: {bad} (가능: {', '.join(SUPPORTED_TASKS)})",
                EXIT_USAGE,
            )
        if "info" not in tasks:
            tasks.insert(0, "info")  # 메타 축은 필터·요약의 기준이라 항상 돈다

        files = collect_input_files(args.paths)
        if not files:
            raise ToolkitError(
                "대상 문서(.hwp/.hwpx)가 없습니다: " + ", ".join(args.paths),
                EXIT_USAGE,
            )
        out_dir = Path(args.out_dir)
        output_paths = {
            out_dir / "summary.json",
            *(out_dir / (task.replace("-", "_") + ".ndjson") for task in tasks),
        }
        for output_path in output_paths:
            ensure_output_absent(output_path, "출력 파일")
        out_dir.mkdir(parents=True, exist_ok=True)

        tk = RhwpToolkit(resolve_rhwp(args.rhwp_bin), verbose=args.verbose)

        per_task = {}
        outputs = {}
        batch_failures = []

        # ① 메타 축 — 이후 필터의 기준
        info_records, info_failed, info_path, info_batch_failure = run_batch_task(
            tk, "info", files, args.threads, out_dir
        )
        per_task["info"] = {
            "okCount": len(info_records) - len(info_failed),
            "failed": info_failed,
        }
        if info_batch_failure:
            per_task["info"]["batchFailure"] = info_batch_failure
            batch_failures.append(info_batch_failure)
        outputs["info"] = str(info_path)

        # ② 필터 — pageCount 기준 대상 좁히기 (실패 파일은 자동 제외)
        targets = [
            r["source"]
            for r in info_records
            if r.get("error") is None
            and (args.min_pages is None or r.get("pageCount", 0) >= args.min_pages)
        ]

        # ③ 나머지 축
        for task in tasks:
            if task == "info":
                continue
            if not targets:
                per_task[task] = {"okCount": 0, "failed": [], "skipped": "대상 0건"}
                continue
            records, failed, path, batch_failure = run_batch_task(
                tk, task, targets, args.threads, out_dir
            )
            per_task[task] = {"okCount": len(records) - len(failed), "failed": failed}
            if batch_failure:
                per_task[task]["batchFailure"] = batch_failure
                batch_failures.append(batch_failure)
            outputs[task] = str(path)

        all_failed = sorted(
            {
                f["source"]
                for t in per_task.values()
                for f in t["failed"]
                if f.get("source")
            }
        )
        final_exit = EXIT_OK if not all_failed and not batch_failures else EXIT_RUNTIME
        summary = {
            "workflow": "bulk_sweep",
            "inputCount": len(files),
            "targetCount": len(targets),
            "minPages": args.min_pages,
            "tasks": {t: per_task[t] for t in per_task},
            "outputs": outputs,
            "failedSources": all_failed,
            "batchFailures": batch_failures,
            "exit": final_exit,
        }
        summary_path = out_dir / "summary.json"
        with open(summary_path, "w", encoding="utf-8") as fh:
            json.dump(summary, fh, ensure_ascii=False, indent=2)

        human = [
            f"{t}: 성공 {v['okCount']}건, 실패 {len(v['failed'])}건"
            + (f" → {outputs[t]}" if t in outputs else "")
            for t, v in per_task.items()
        ] + [f"요약: {summary_path}"]
        if all_failed:
            human.append(f"실패 파일 {len(all_failed)}건 — summary.json failedSources 참조")
        if batch_failures:
            human.append(f"batch 실행 실패 {len(batch_failures)}건 — summary.json batchFailures 참조")
        emit_summary(summary, args.json, human)
        if final_exit != EXIT_OK:
            print(
                f"오류: 파일 실패 {len(all_failed)}건, batch 실행 실패 {len(batch_failures)}건 "
                "(성공분 NDJSON 은 보존됨)",
                file=sys.stderr,
            )
        return final_exit
    except ToolkitError as e:
        print(f"오류: {e}", file=sys.stderr)
        return e.exit_code


if __name__ == "__main__":
    sys.exit(main())
