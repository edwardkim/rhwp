#!/usr/bin/env python3
"""서식 자동 작성 워크플로 — 누름틀 채우기 + 재독 검증 (플레이북 시나리오 1)

    python3 form_filling.py <서식.hwp|hwpx> <값.json> -o <출력.hwp|hwpx>

값 파일은 {"필드이름": "값"} JSON 객체다. 같은 이름 누름틀이 여러 개면
"이름[N]" (0 기준 등장 순번) 으로 지목한다.

시퀀스: fields(재독 기준선) → edit fill-fields → fields 재독 대조.

성공(exit 0) 조건 — 전부 만족해야 한다:
  * fill-fields 봉투의 notFound / ambiguous / confusable 이 모두 비었다
  * filledCount == 요청한 값 개수
  * 출력 파일이 실제로 존재한다
  * 재독한 fields[].value 가 요청 값과 일치한다
어느 하나라도 어긋나면 만들어진 출력 파일을 지우고 비 0 으로 끝낸다
(rhwp 는 notFound 가 있어도 exit 0 으로 파일을 만든다 — 그대로 두면
"성공처럼 보이는 미완성 산출물"이 남는다).

종료 코드: 0 성공 / 1 실행·검증 실패 / 2 입력 오류.
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
    resolve_rhwp,
    verify_filled,
)


def load_values(path: Path) -> dict:
    if not path.is_file():
        raise ToolkitError(f"값 파일이 없습니다: {path}", EXIT_USAGE)
    try:
        with open(path, encoding="utf-8-sig") as fh:
            data = json.load(fh)
    except (json.JSONDecodeError, UnicodeDecodeError) as e:
        raise ToolkitError(f"값 파일 JSON 파싱 실패: {path}: {e}", EXIT_USAGE)
    if not isinstance(data, dict) or not data:
        raise ToolkitError(
            f'값 파일은 비어 있지 않은 {{"필드이름":"값"}} 객체여야 합니다: {path}',
            EXIT_USAGE,
        )
    bad = [k for k, v in data.items() if not isinstance(k, str) or not isinstance(v, str)]
    if bad:
        raise ToolkitError(
            f"값 파일의 키·값은 모두 문자열이어야 합니다: {bad[:3]}", EXIT_USAGE
        )
    return data


def main(argv=None) -> int:
    ensure_utf8_stdio()
    parser = argparse.ArgumentParser(
        description="누름틀 서식 자동 작성 — fill-fields + 재독 검증"
    )
    parser.add_argument("template", help="누름틀이 있는 서식 문서 (.hwp/.hwpx)")
    parser.add_argument("data", help='{"필드이름":"값"} JSON 파일')
    parser.add_argument("-o", "--output", required=True, help="출력 문서 경로")
    add_common_args(parser)
    args = parser.parse_args(argv)

    output = None
    output_owned = False
    try:
        template = Path(args.template)
        if not template.is_file():
            raise ToolkitError(f"서식 파일이 없습니다: {template}", EXIT_USAGE)
        values = load_values(Path(args.data))
        output = Path(args.output)
        if output.resolve() == template.resolve():
            raise ToolkitError("출력 경로가 서식과 같습니다 — 덮어쓰기 금지", EXIT_USAGE)
        ensure_output_absent(output, "출력 파일")
        output_owned = True
        if output.parent and not output.parent.exists():
            output.parent.mkdir(parents=True, exist_ok=True)

        tk = RhwpToolkit(resolve_rhwp(args.rhwp_bin), verbose=args.verbose)

        # ① 서식이 요구하는 이름 목록 (입력 검증 겸 재독 기준선)
        before, _ = tk.run_json(["fields", str(template), "--json"])
        if before.get("fieldCount", 0) == 0:
            raise ToolkitError(
                f"서식에 누름틀이 없습니다 (fieldCount=0): {template} — "
                "표 칸 양식이면 edit set-cell 축을 쓰세요",
                EXIT_RUNTIME,
            )

        # ② 값 채우기
        fill, _ = tk.run_json(
            [
                "edit", "fill-fields", str(template),
                "--data", "@" + str(Path(args.data)),
                "-o", str(output),
                "--json",
            ]
        )

        problems = []
        for key in ("notFound", "ambiguous", "confusable"):
            if fill.get(key):
                problems.append(f"{key}={json.dumps(fill[key], ensure_ascii=False)}")
        if fill.get("filledCount", 0) != len(values):
            problems.append(
                f"filledCount={fill.get('filledCount')} != 요청 {len(values)}건"
            )

        mismatches = []
        if not problems:
            if not output.is_file():
                raise ToolkitError(
                    f"fill-fields 가 성공을 보고했지만 출력이 없습니다: {output}",
                    EXIT_RUNTIME,
                )
            # ③ 재독 — 쓴 값을 다시 읽어 기계 대조
            reread, _ = tk.run_json(["fields", str(output), "--json"])
            mismatches = verify_filled(fill.get("filled", []), reread)
            if mismatches:
                problems.append(
                    "재독 불일치=" + json.dumps(mismatches, ensure_ascii=False)
                )

        if problems:
            raise ToolkitError(
                "서식 채우기 검증 실패: " + "; ".join(problems), EXIT_RUNTIME
            )

        summary = {
            "workflow": "form_filling",
            "source": str(template),
            "output": str(output),
            "requestedCount": len(values),
            "filledCount": fill.get("filledCount"),
            "changedPages": fill.get("changedPages", []),
            "rereadVerified": True,
            "exit": EXIT_OK,
        }
        emit_summary(
            summary,
            args.json,
            [
                f"채움 {fill.get('filledCount')}건 → {output}",
                f"재독 검증 통과 (불일치 0건, changedPages={fill.get('changedPages', [])})",
            ],
        )
        return EXIT_OK
    except ToolkitError as e:
        # 충돌 검사를 통과한 이번 호출의 산출물만 정리한다.
        if output_owned and output is not None and output.is_file():
            try:
                output.unlink()
            except OSError:
                pass
        print(f"오류: {e}", file=sys.stderr)
        return e.exit_code


if __name__ == "__main__":
    sys.exit(main())
