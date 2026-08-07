#!/usr/bin/env python3
"""rhwp HWPX 테스트 데이터 생성기 — 검증된 build-from-ingest 경로 사용.

파이프라인 (각 단계가 실패하면 즉시 비 0 종료):

    템플릿(config_templates.json)
      → ingest JSON 생성 (ingest_schema_v1 범위 내 결정적 생성)
      → `rhwp build-from-ingest <ingest.json> -o <out.hwpx> --json`
      → `rhwp info <out.hwpx> --json` 으로 산출물마다 자동 재검증

#4044 리뷰 2번 반영: 이전 구현은 zip 을 손으로 조립해 `Contents/header.xml`
이 없는 비정합 HWPX 를 만들었다. 지금은 생성 자체를 rhwp 의 표준
`build-from-ingest` 경로에 위임하므로 OWPML 구조 정합은 rhwp 본체가 보장하고,
이 도구는 템플릿 → ingest 변환과 산출물 검증만 책임진다.

rhwp 바이너리 해석 순서: `--rhwp-bin` 인자 > `RHWP_BIN` 환경변수 > PATH.

종료 코드: 0 = 전부 생성·검증 성공 / 1 = 생성 또는 검증 실패 /
2 = 사용법·설정·환경 오류.

표준 라이브러리만 사용한다.
"""

from __future__ import annotations

import argparse
import copy
import json
import os
import shutil
import subprocess
import sys
import zlib
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, List, Optional

EXIT_OK = 0
EXIT_FAIL = 1
EXIT_USAGE = 2

CHOICE_LABELS = ["①", "②", "③", "④", "⑤"]

#: 템플릿이 지원하는 키 — 전부 ingest_schema_v1 이 표현 가능한 범위다.
#: (표·실제 이미지 픽셀·글자 스타일 등 스키마 밖 항목은 지원하지 않는다.)
TEMPLATE_KEYS = {
    "description",
    "questions",
    "choices_per_question",
    "stem_paragraphs",
    "boxed_every",
    "media_every",
    "passages",
    "questions_per_passage",
    "header_text",
    "footer_text",
    "form_label",
    "default_font",
    "page_size",
}

STEM_SENTENCES = [
    "다음 글을 읽고 물음에 가장 적절한 답을 고르시오.",
    "제시된 자료를 바탕으로 추론할 수 있는 내용을 고르시오.",
    "밑줄 친 부분의 의미로 가장 알맞은 것을 고르시오.",
    "글의 흐름으로 보아 빈칸에 들어갈 말로 적절한 것을 고르시오.",
    "다음 설명에 해당하는 개념으로 옳은 것을 고르시오.",
]

BODY_SENTENCES = [
    "환경 오염은 현대 사회가 함께 풀어야 할 과제이다.",
    "기술의 발전은 문서 작업 방식에도 큰 변화를 가져왔다.",
    "표준 형식을 따르는 문서는 도구 사이의 호환성을 보장한다.",
    "검증 가능한 산출물은 회귀 테스트의 기준점이 된다.",
    "페이지 흐름을 확인하려면 본문이 충분히 길어야 한다.",
    "공유 지문은 여러 문항이 같은 본문을 참조할 때 사용한다.",
]

CHOICE_PHRASES = [
    "환경 보호의 중요성",
    "표준 문서 형식의 가치",
    "검증 파이프라인의 역할",
    "페이지 배치의 원리",
    "공유 지문의 활용",
]


class ConfigError(ValueError):
    """템플릿 설정·환경 문제 — 종료 코드 2."""


class GenerationError(RuntimeError):
    """rhwp 생성 또는 검증 실패 — 종료 코드 1."""


# ── 템플릿 로드·검증 ─────────────────────────────────────────────────────────


def default_config_path() -> Path:
    return Path(__file__).resolve().parent / "config_templates.json"


def load_templates(config_path: Path) -> Dict[str, Dict[str, Any]]:
    try:
        raw = json.loads(config_path.read_text(encoding="utf-8"))
    except OSError as e:
        raise ConfigError(f"설정 파일을 읽을 수 없습니다: {config_path}: {e}") from e
    except json.JSONDecodeError as e:
        raise ConfigError(f"설정 파일이 올바른 JSON 이 아닙니다: {config_path}: {e}") from e

    templates = raw.get("templates")
    if not isinstance(templates, dict) or not templates:
        raise ConfigError(f"설정에 비어 있지 않은 'templates' 객체가 필요합니다: {config_path}")

    for name, spec in templates.items():
        validate_template_name(name)
        validate_template(name, spec)
    return templates


def validate_template_name(name: str) -> None:
    """템플릿 이름이 산출물 폴더 밖으로 나가지 않는 단일 파일명인지 확인한다."""
    if (
        not name
        or name in {".", ".."}
        or "\x00" in name
        or "/" in name
        or "\\" in name
        or Path(name).is_absolute()
    ):
        raise ConfigError(
            f"템플릿 이름은 경로 구분자 없는 단일 파일명이어야 합니다: {name!r}"
        )


def validate_template(name: str, spec: Any) -> None:
    if not isinstance(spec, dict):
        raise ConfigError(f"템플릿 '{name}' 은 객체여야 합니다")

    if "raw_ingest" in spec:
        # 고급: ingest JSON 원본 통짜 지정 (경계 사례 fixture 제작용).
        extra = set(spec) - {"description", "raw_ingest"}
        if extra:
            raise ConfigError(
                f"템플릿 '{name}': raw_ingest 템플릿에는 description 외 다른 키를 둘 수 없습니다: {sorted(extra)}"
            )
        if not isinstance(spec["raw_ingest"], dict):
            raise ConfigError(f"템플릿 '{name}': raw_ingest 는 객체여야 합니다")
        return

    unknown = set(spec) - TEMPLATE_KEYS
    if unknown:
        raise ConfigError(
            f"템플릿 '{name}': 지원하지 않는 키 {sorted(unknown)} — "
            f"지원 키: {sorted(TEMPLATE_KEYS)}"
        )

    def _int(key: str, default: int, lo: int, hi: Optional[int] = None) -> int:
        v = spec.get(key, default)
        if not isinstance(v, int) or isinstance(v, bool) or v < lo or (hi is not None and v > hi):
            rng = f"{lo} 이상" + (f" {hi} 이하" if hi is not None else "")
            raise ConfigError(f"템플릿 '{name}': '{key}' 는 {rng} 정수여야 합니다 (현재: {v!r})")
        return v

    n_questions = _int("questions", 1, 1)
    _int("choices_per_question", 5, 1, len(CHOICE_LABELS))
    _int("stem_paragraphs", 1, 1)
    _int("boxed_every", 0, 0)
    _int("media_every", 0, 0)
    n_passages = _int("passages", 0, 0)
    per = _int("questions_per_passage", 2, 1)
    if n_passages * per > n_questions:
        raise ConfigError(
            f"템플릿 '{name}': passages({n_passages}) × questions_per_passage({per}) 가 "
            f"questions({n_questions}) 를 초과합니다"
        )


# ── ingest JSON 생성 (결정적) ────────────────────────────────────────────────


def _pick(pool: List[str], index: int, salt: int) -> str:
    return pool[(index + salt) % len(pool)]


def build_ingest(spec: Dict[str, Any], seed: int) -> Dict[str, Any]:
    """템플릿 하나를 ingest_schema_v1 문서로 변환한다. 같은 (spec, seed) 는 항상 같은 결과."""
    if "raw_ingest" in spec:
        return copy.deepcopy(spec["raw_ingest"])

    n_questions = spec.get("questions", 1)
    n_choices = spec.get("choices_per_question", 5)
    stem_paragraphs = spec.get("stem_paragraphs", 1)
    boxed_every = spec.get("boxed_every", 0)
    media_every = spec.get("media_every", 0)
    n_passages = spec.get("passages", 0)
    per_passage = spec.get("questions_per_passage", 2)
    salt = seed & 0xFFFF

    doc: Dict[str, Any] = {"version": "1"}
    if "page_size" in spec:
        doc["page_size"] = copy.deepcopy(spec["page_size"])
    if "default_font" in spec:
        doc["default_font"] = spec["default_font"]
    for key in ("header_text", "footer_text", "form_label"):
        if spec.get(key) is not None and key in spec:
            doc[key] = spec[key]

    # 공유 지문: 앞쪽 문항부터 per_passage 개씩 묶는다.
    passages = []
    passage_of_question: Dict[int, str] = {}
    for p in range(n_passages):
        first = p * per_passage + 1
        last = first + per_passage - 1
        pid = f"p{first}-{last}"
        passages.append(
            {
                "id": pid,
                "blocks": [
                    {"type": "text", "text": f"[{first}~{last}] 다음 글을 읽고 물음에 답하시오."},
                    {"type": "text", "text": _pick(BODY_SENTENCES, p, salt)},
                ],
            }
        )
        for q in range(first, last + 1):
            passage_of_question[q] = pid
    if passages:
        doc["passages"] = passages

    questions = []
    for n in range(1, n_questions + 1):
        stem_first = _pick(STEM_SENTENCES, n, salt)
        stem_blocks: List[Dict[str, Any]] = [{"type": "text", "text": stem_first}]
        for extra in range(1, stem_paragraphs):
            stem_blocks.append(
                {"type": "text", "text": _pick(BODY_SENTENCES, n * 3 + extra, salt)}
            )
        if boxed_every and n % boxed_every == 0:
            stem_blocks.append(
                {
                    "type": "boxed",
                    "title": "<보기>",
                    "blocks": [
                        {"type": "text", "text": _pick(BODY_SENTENCES, n * 5, salt)}
                    ],
                }
            )

        question: Dict[str, Any] = {
            "number": n,
            "stem": stem_first,
            "stem_blocks": stem_blocks,
            "choices": [
                {
                    "label": CHOICE_LABELS[c],
                    "text": f"{_pick(CHOICE_PHRASES, n + c, salt)}에 대한 진술 {c + 1}",
                }
                for c in range(n_choices)
            ],
        }

        if media_every and n % media_every == 0:
            media_id = f"img/q{n}.png"
            stem_blocks.append({"type": "image", "ref": media_id, "placement": "between"})
            # 파일 실물 없이도 build-from-ingest 가 placeholder 로 처리한다.
            question["media"] = [
                {"id": media_id, "natural_w": 640, "natural_h": 480, "target_w_mm": 80.0}
            ]

        if n in passage_of_question:
            question["passage_ref"] = passage_of_question[n]

        questions.append(question)

    doc["questions"] = questions
    return doc


# ── rhwp 실행 ────────────────────────────────────────────────────────────────


def resolve_rhwp_bin(explicit: Optional[str]) -> str:
    """`--rhwp-bin` > `RHWP_BIN` > PATH 순서로 rhwp 실행 파일을 찾는다."""
    candidate = explicit or os.environ.get("RHWP_BIN")
    if candidate:
        path = Path(candidate)
        if not path.is_file() or not os.access(path, os.X_OK):
            raise ConfigError(f"rhwp 바이너리가 없거나 실행할 수 없습니다: {candidate}")
        return candidate
    found = shutil.which("rhwp")
    if not found:
        raise ConfigError(
            "rhwp 바이너리를 찾을 수 없습니다 — --rhwp-bin 인자, RHWP_BIN 환경변수, "
            "PATH 순서로 탐색했습니다"
        )
    return found


def run_rhwp(rhwp_bin: str, args: List[str]) -> subprocess.CompletedProcess:
    return subprocess.run(
        [rhwp_bin, *args],
        capture_output=True,
        encoding="utf-8",
        errors="replace",
    )


@dataclass
class GeneratedDoc:
    name: str
    output: str
    bytes: int
    question_count: int
    paragraph_count: int
    page_count: int
    verified: bool  # rhwp info --json 통과 여부 (파이프라인 내장 검증)


def generate_one(
    rhwp_bin: str,
    name: str,
    spec: Dict[str, Any],
    out_dir: Path,
    seed: int,
    keep_ingest: bool,
) -> GeneratedDoc:
    """템플릿 하나 → ingest JSON → HWPX → info 재검증까지 수행한다."""
    # 템플릿 이름을 시드에 섞어, 선택 순서와 무관하게 템플릿별 결정적 출력을 만든다.
    doc_seed = seed ^ zlib.crc32(name.encode("utf-8"))
    ingest = build_ingest(spec, doc_seed)

    ingest_path = out_dir / f"{name}.ingest.json"
    hwpx_path = out_dir / f"{name}.hwpx"
    ingest_path.write_text(
        json.dumps(ingest, ensure_ascii=False, indent=2), encoding="utf-8"
    )

    build = run_rhwp(
        rhwp_bin,
        ["build-from-ingest", str(ingest_path), "-o", str(hwpx_path), "--json"],
    )
    if build.returncode != 0:
        raise GenerationError(
            f"[{name}] build-from-ingest 실패 (exit {build.returncode}): "
            f"{build.stderr.strip() or build.stdout.strip()}"
        )
    try:
        envelope = json.loads(build.stdout)
    except json.JSONDecodeError as e:
        raise GenerationError(
            f"[{name}] build-from-ingest --json 출력 파싱 실패: {e}: {build.stdout!r}"
        ) from e

    # 산출물 검증 — 모든 산출물은 rhwp info --json 을 통과해야 한다.
    info = run_rhwp(rhwp_bin, ["info", str(hwpx_path), "--json"])
    if info.returncode != 0:
        raise GenerationError(
            f"[{name}] 산출물 검증 실패 — rhwp info exit {info.returncode}: "
            f"{info.stderr.strip() or info.stdout.strip()}"
        )
    try:
        info_json = json.loads(info.stdout)
    except json.JSONDecodeError as e:
        raise GenerationError(f"[{name}] rhwp info --json 출력 파싱 실패: {e}") from e
    if info_json.get("format") != "hwpx":
        raise GenerationError(
            f"[{name}] 산출물 형식이 hwpx 가 아닙니다: {info_json.get('format')!r}"
        )

    if not keep_ingest:
        ingest_path.unlink()

    return GeneratedDoc(
        name=name,
        output=str(hwpx_path),
        bytes=int(envelope.get("bytes", hwpx_path.stat().st_size)),
        question_count=int(envelope.get("questionCount", len(ingest.get("questions", [])))),
        paragraph_count=int(envelope.get("paragraphCount", 0)),
        page_count=int(info_json.get("pageCount", 0)),
        verified=True,
    )


# ── CLI ──────────────────────────────────────────────────────────────────────


def main(argv: Optional[List[str]] = None) -> int:
    parser = argparse.ArgumentParser(
        description=(
            "템플릿 기반 ingest JSON 을 rhwp build-from-ingest 로 HWPX 로 만들고 "
            "산출물마다 rhwp info --json 으로 검증한다"
        )
    )
    parser.add_argument("--output-dir", help="산출물 폴더 (필수, --list 제외)")
    parser.add_argument(
        "--template",
        action="append",
        help="생성할 템플릿 이름 (반복 지정 가능, 기본: 전체)",
    )
    parser.add_argument(
        "--config",
        default=None,
        help="템플릿 설정 파일 (기본: 스크립트 옆 config_templates.json)",
    )
    parser.add_argument(
        "--rhwp-bin",
        default=None,
        help="rhwp 실행 파일 경로 (기본: RHWP_BIN 환경변수, 그다음 PATH)",
    )
    parser.add_argument("--seed", type=int, default=42, help="결정적 생성 시드 (기본 42)")
    parser.add_argument(
        "--keep-ingest",
        action="store_true",
        help="중간 ingest JSON(*.ingest.json)을 산출물 옆에 남긴다",
    )
    parser.add_argument("--list", action="store_true", help="템플릿 목록만 출력")
    parser.add_argument("--json", action="store_true", help="요약을 JSON 으로 출력")
    args = parser.parse_args(argv)

    for stream in (sys.stdout, sys.stderr):
        if hasattr(stream, "reconfigure"):
            try:
                stream.reconfigure(encoding="utf-8")
            except (OSError, ValueError):
                pass

    config_path = Path(args.config) if args.config else default_config_path()
    try:
        templates = load_templates(config_path)

        if args.list:
            for name, spec in templates.items():
                print(f"{name}: {spec.get('description', '')}")
            return EXIT_OK

        if not args.output_dir:
            raise ConfigError("--output-dir 가 필요합니다 (--list 제외)")

        selected = args.template or list(templates)
        unknown = [t for t in selected if t not in templates]
        if unknown:
            raise ConfigError(
                f"알 수 없는 템플릿: {unknown} — 사용 가능: {sorted(templates)}"
            )

        rhwp_bin = resolve_rhwp_bin(args.rhwp_bin)
    except ConfigError as e:
        print(f"오류: {e}", file=sys.stderr)
        return EXIT_USAGE

    out_dir = Path(args.output_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    results: List[GeneratedDoc] = []
    for name in selected:
        try:
            doc = generate_one(
                rhwp_bin, name, templates[name], out_dir, args.seed, args.keep_ingest
            )
        except GenerationError as e:
            print(f"오류: {e}", file=sys.stderr)
            return EXIT_FAIL
        results.append(doc)
        if not args.json:
            print(
                f"생성·검증 완료: {doc.output} "
                f"({doc.bytes}바이트, 문제 {doc.question_count}개, "
                f"문단 {doc.paragraph_count}개, {doc.page_count}쪽)"
            )

    if args.json:
        print(
            json.dumps(
                {
                    "schemaVersion": "1.0",
                    "generator": "hwp_test_data_generator",
                    "seed": args.seed,
                    "count": len(results),
                    "documents": [
                        {
                            "name": d.name,
                            "output": d.output,
                            "bytes": d.bytes,
                            "questionCount": d.question_count,
                            "paragraphCount": d.paragraph_count,
                            "pageCount": d.page_count,
                            "verified": d.verified,
                        }
                        for d in results
                    ],
                },
                ensure_ascii=False,
                indent=2,
            )
        )
    else:
        print(f"합계: {len(results)}개 문서 전부 rhwp info --json 검증 통과")

    return EXIT_OK


if __name__ == "__main__":
    sys.exit(main())
