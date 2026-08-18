#!/usr/bin/env python3
"""M-fid fidelity_compare 텍스트층·픽스처 고도화 생성기.

devel 의 ``compare_text_layers`` / ``classify_text_layer_delta`` /
owner-shift / sequence / glyph-risk / visible-excess /
``--text-only`` artifact 계약을 읽어 소실·과잉·치환 표와
픽스처를 디스크에 쓴다. ``scripts/visual_sweep.py`` 는 읽거나
수정하지 않는다. 렌더·serializer·canvaskit·gym 은 건드리지 않는다.

    python tools/fidelity_compare/fatten_text_layer.py
    python tools/fidelity_compare/test_fatten_text_layer.py
"""

from __future__ import annotations

import argparse
import contextlib
import hashlib
import importlib.util
import io
import json
import sys
import unicodedata
from collections import Counter
from collections.abc import Iterable, Sequence
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent
CLAIM_ID = "M-fid"
SCHEMA_VERSION = "1.0"
GENERATOR = "tools/fidelity_compare/fatten_text_layer.py"
KIND = "fidelityTextLayerCatalog"
CASE_SCHEMA = "rhwp.fidelity_compare.text_layer_case.v1"

PUA_CIRCLED = "".join(chr(0xF02B1 + index) for index in range(14))
PUA_BULLET = "\U000F02FB"
FFFD = "\uFFFD"
NFC_GA = unicodedata.normalize("NFC", "가")
NFD_GA = unicodedata.normalize("NFD", "가")


def load_harness() -> Any:
    spec = importlib.util.spec_from_file_location(
        "fidelity_compare", HERE / "fidelity_compare.py"
    )
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


@dataclass(frozen=True)
class SourceDoc:
    key: str
    family: str
    genre: str
    title: str
    text: str
    related_issue: str
    why: str
    registered_key: str


@dataclass
class LayerCase:
    ident: str
    kind: str
    family: str
    genre: str
    registered_key: str
    title: str
    why: str
    reference_text: str
    svg_text: str
    related_issue: str
    review_hint: str
    notes: str
    page: int = 0
    next_reference: str = ""
    next_svg: str = ""
    clip_excluded: int = 0
    extras: dict[str, Any] = field(default_factory=dict)


@dataclass
class PathSpec:
    ident: str
    argv: list[str]
    mode: str
    export_all_svg: bool
    layout_ledger: bool
    expects_error: bool
    error_needle: str
    chrome_required: bool
    pypdf_required: bool
    pypdfium2_required: bool
    title: str
    why: str


@dataclass
class FattenBundle:
    generated_at: str
    out_root: Path
    harness: Any
    cases: list[dict[str, Any]] = field(default_factory=list)
    paths: list[dict[str, Any]] = field(default_factory=list)
    svgs: list[dict[str, Any]] = field(default_factory=list)
    written: list[str] = field(default_factory=list)


def nfc(value: str) -> str:
    return unicodedata.normalize("NFC", value)


def utc_now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not text.endswith("\n"):
        text += "\n"
    path.write_text(text, encoding="utf-8", newline="\n")


def write_json(path: Path, data: Any) -> None:
    write_text(path, json.dumps(data, ensure_ascii=False, indent=2))


def md_cell(value: Any) -> str:
    return nfc(str(value)).replace("|", "\\|").replace("\n", " ")


def short_hash(blob: str) -> str:
    return hashlib.sha256(blob.encode("utf-8")).hexdigest()[:12]


def record(bundle: FattenBundle, path: Path) -> Path:
    rel = path.resolve().relative_to(bundle.out_root.resolve()).as_posix()
    bundle.written.append(rel)
    return path


def drop_phrase(text: str, phrase: str) -> str:
    return text.replace(phrase, "", 1)


def insert_after(text: str, marker: str, insert: str) -> str:
    if marker not in text:
        return text + insert
    return text.replace(marker, marker + insert, 1)


def to_fullwidth_digits(text: str) -> str:
    table = str.maketrans("0123456789", "０１２３４５６７８９")
    return text.translate(table)


def to_nfd_hangul(text: str) -> str:
    return "".join(
        unicodedata.normalize("NFD", char)
        if "HANGUL" in unicodedata.name(char, "")
        else char
        for char in text
    )


def hyphen_to_dash(text: str) -> str:
    return text.replace("-", "–").replace(" - ", " — ")


def circled_to_ascii(text: str) -> str:
    mapping = {
        "①": "(1)",
        "②": "(2)",
        "③": "(3)",
        "④": "(4)",
        "⑤": "(5)",
    }
    out = text
    for src, dst in mapping.items():
        out = out.replace(src, dst)
    return out


SOURCE_DOCS: tuple[SourceDoc, ...] = (
    SourceDoc(
        "plan-cover",
        "plan",
        "업무계획",
        "표지 연도·기관명",
        "2022년도 주요업무계획 행정안전부 디지털정부국",
        "#3389",
        "등록 키 plan 표지. 연도·기관이 빠지면 소실 후보.",
        "plan",
    ),
    SourceDoc(
        "plan-goal",
        "plan",
        "업무계획",
        "추진 목표 문단",
        "추진목표 국민이 체감하는 디지털 행정서비스를 확대하고 데이터 기반 정책을 정착시킨다.",
        "#3389",
        "장문 목표 문단. 한 문장 소실·과잉·치환을 나눈다.",
        "plan",
    ),
    SourceDoc(
        "plan-table-head",
        "plan",
        "업무계획",
        "실적 표 헤더",
        "구분 과제명 주관부서 예산 일정 비고",
        "#3389",
        "표 헤더 한 칸이 빠지면 소실, 중복 paint 되면 과잉.",
        "plan",
    ),
    SourceDoc(
        "plan-budget",
        "plan",
        "업무계획",
        "예산 숫자 행",
        "정보화사업 예산 1,234,567천원 중 본예산 987,000천원 추경 247,567천원",
        "#3389",
        "자릿수·전각 숫자는 치환 후보의 전형.",
        "plan",
    ),
    SourceDoc(
        "plan-footnote",
        "plan",
        "업무계획",
        "각주 표지",
        "1) 본 계획은 2022.1. 국회 보고안을 기준으로 한다. https://www.mois.go.kr/plan/2022",
        "#3389",
        "각주+URL. 쪽 owner 이동과 sequence 후보.",
        "plan",
    ),
    SourceDoc(
        "plan-caption",
        "plan",
        "업무계획",
        "그림 캡션",
        "<그림 3-2> 디지털정부 추진체계 도해 (자료: 행정안전부 2022)",
        "#3389",
        "캡션이 한 쪽 이르게 나오면 owner-shift.",
        "plan",
    ),
    SourceDoc(
        "plan-pua-circled",
        "plan",
        "업무계획",
        "원문자 과제 번호",
        f"과제번호 {PUA_CIRCLED[:4]} 는 CharOverlap 문맥에서 두부가 된다.",
        "#3385",
        "U+F02B1~ 원문자. SVG raw PUA 는 glyph-risk 와 치환의 교집합.",
        "plan",
    ),
    SourceDoc(
        "manual-toc",
        "manual",
        "장문 편람",
        "편람 목차",
        "제1장 총칙 제2장 문서 작성 제3장 결재 제4장 보관 제5장 공개",
        "#3389",
        "장문 편람 목차. 장 제목 누락은 소실.",
        "manual",
    ),
    SourceDoc(
        "manual-article",
        "manual",
        "장문 편람",
        "조문 본문",
        "제12조(문서의 성립) 문서는 해당 문서에 대한 결재가 있음으로써 성립한다.",
        "#3389",
        "조문 번호와 본문. 번호만 전각으로 바뀌면 치환.",
        "manual",
    ),
    SourceDoc(
        "manual-note",
        "manual",
        "장문 편람",
        "해설 각주",
        "각주 14 행정 효율과 협업 촉진에 관한 규정 제6조 및 별표 1 참조.",
        "#3389",
        "법령 인용 각주. 다음 쪽으로 밀리면 sequence.",
        "manual",
    ),
    SourceDoc(
        "manual-long",
        "manual",
        "장문 편람",
        "장문 해설",
        (
            "문서는 용이하게 이해할 수 있도록 간결하게 작성하여야 하며 "
            "일반 국민이 읽었을 때 의미가 분명하도록 표준어와 한글 맞춤법을 따른다. "
            "전문 용어는 필요한 경우에만 쓰고 약어는 처음 한 번 풀어 쓴다."
        ),
        "#3389",
        "장문 한 쪽이 통째로 과잉이면 visible-excess 후보.",
        "manual",
    ),
    SourceDoc(
        "bunjang-header",
        "bunjang",
        "표 중심",
        "거래 명세 헤더",
        "번호 상품명 판매자 구매자 금액 상태 접수일",
        "#3389",
        "표 중심 참고 PDF. 헤더 중복은 과잉.",
        "bunjang",
    ),
    SourceDoc(
        "bunjang-row",
        "bunjang",
        "표 중심",
        "거래 한 행",
        "21868765 빈티지 카메라 판매자A 구매자B 125,000원 거래완료 2024-03-12",
        "#3389",
        "등록 키 bunjang 샘플 번호가 들어 있는 행.",
        "bunjang",
    ),
    SourceDoc(
        "bunjang-footer",
        "bunjang",
        "표 중심",
        "표 하단 합계",
        "합계 12건 1,580,000원 (부가세 별도) ※ 참고 PDF — 버전 미확인",
        "#3389",
        "samples/ 동반 PDF 는 참고 등급. 합계 숫자는 소실에 민감.",
        "bunjang",
    ),
    SourceDoc(
        "korexam-stem",
        "korexam",
        "법학적성 언어이해",
        "지문 도입",
        "다음 글을 읽고 물음에 답하시오. 정의의 관념은 시대와 사회에 따라 다르게 해석되어 왔다.",
        "#3389",
        "A3 2단 지문. 도입문이 빠지면 소실.",
        "korexam",
    ),
    SourceDoc(
        "korexam-choice",
        "korexam",
        "법학적성 언어이해",
        "선지 ①~⑤",
        "① 정의는 절차에 의존한다 ② 정의는 결과에만 의존한다 ③ 정의는 초월적이다 ④ 정의는 측정 불가이다 ⑤ 정의는 관습과 무관하다",
        "#3389",
        "원문자 선지. PUA 치환과 두부 후보.",
        "korexam",
    ),
    SourceDoc(
        "korexam-header",
        "korexam",
        "법학적성 언어이해",
        "시험 머리글",
        "2022학년도 법학적성시험 제1교시 언어이해 홀수형 15쪽 A3",
        "#3389",
        "머리글이 SVG 에만 반복되면 과잉.",
        "korexam",
    ),
    SourceDoc(
        "math-item",
        "math",
        "수학 시험",
        "문항 본문",
        "정적분 1부터 2까지 (2x+1) dx 의 값은? 단, 계산 과정을 쓰시오.",
        "#3389",
        "수식은 텍스트층에 부분만 남는다. 숫자 치환에 주의.",
        "math",
    ),
    SourceDoc(
        "math-choice",
        "math",
        "수학 시험",
        "수식 선지",
        "① 2 ② 3 ③ 4 ④ 5 ⑤ 6",
        "#3389",
        "짧은 선지. 전각 숫자는 치환.",
        "math",
    ),
    SourceDoc(
        "math-caption",
        "math",
        "수학 시험",
        "그래프 캡션",
        "<그림 4> y=2x+1 의 그래프와 정적분 영역 (단위: cm)",
        "#3389",
        "캡션 owner 이동.",
        "math",
    ),
    SourceDoc(
        "eng-passage",
        "eng",
        "영어 시험",
        "영어 지문",
        (
            "The committee published the annual report on digital government "
            "services in 2022 and invited public comments until March 31."
        ),
        "#3389",
        "라틴 혼합. 대소문자·하이픈은 치환 후보.",
        "eng",
    ),
    SourceDoc(
        "eng-question",
        "eng",
        "영어 시험",
        "영어 문항",
        "Which of the following is the best title for the passage? Choose one answer.",
        "#3389",
        "문항 지시문 소실.",
        "eng",
    ),
    SourceDoc(
        "eng-url",
        "eng",
        "영어 시험",
        "출처 URL",
        "Source: https://example.go.kr/exam/eng/2022/item-17-citation",
        "#3389",
        "16자 이상 URL. sequence owner 이동의 정석.",
        "eng",
    ),
    SourceDoc(
        "fn-early",
        "footnote",
        "각주",
        "각주 본문 이르게",
        "각주내용 본 통계는 통계청 2022년 사회조사 원표를 재구성한 것이다.",
        "#3389",
        "각주가 pN SVG 에만 있고 PDF 는 pN+1 — rhwp_earlier.",
        "plan",
    ),
    SourceDoc(
        "fn-late",
        "footnote",
        "각주",
        "각주 본문 늦게",
        "각주내용 별표 2의 서식은 행정안전부령 제1호에 따른다.",
        "#3389",
        "각주가 PDF pN 에만 있고 SVG 는 pN+1 — rhwp_later.",
        "manual",
    ),
    SourceDoc(
        "table-first-row",
        "table",
        "표 조각",
        "표 첫 줄 중복",
        "연번 사업명 소관 예산액 비고 추진율",
        "#3389",
        "p81→p82 형. 같은 헤더가 다음 쪽에 중복.",
        "plan",
    ),
    SourceDoc(
        "table-cell",
        "table",
        "표 조각",
        "셀 본문",
        "스마트워크 센터 확대 행정안전부 정보화통계과 추진중",
        "#3389",
        "셀 텍스트 경계 침범과 별개의 문자 멀티셋.",
        "plan",
    ),
    SourceDoc(
        "citation-law",
        "citation",
        "법령 인용",
        "법령명+조문",
        "공공기록물 관리에 관한 법률 제18조 제1항 및 같은 법 시행령 제22조",
        "#3389",
        "긴 법령명은 sequence 후보.",
        "manual",
    ),
    SourceDoc(
        "citation-url",
        "citation",
        "법령 인용",
        "고시 URL",
        "https://www.law.go.kr/LSW/lsInfoP.do?lsiSeq=246801",
        "#3389",
        "16자 이상 URL 이 쪽을 넘기면 sequence.",
        "manual",
    ),
    SourceDoc(
        "pua-bullet",
        "pua",
        "원문자",
        "한컴 전용 불릿",
        f"{PUA_BULLET} 주요성과 요약 및 향후 계획",
        "#2007",
        "U+F02FB 두부 불릿. glyph-risk 독립 원장.",
        "plan",
    ),
    SourceDoc(
        "pua-range",
        "pua",
        "원문자",
        "원문자 전 구간",
        f"항목 {PUA_CIRCLED} 완료",
        "#3385",
        "U+F02B1~F02C4 14자. CharOverlap tofu 재현 축.",
        "plan",
    ),
    SourceDoc(
        "header-repeat",
        "header",
        "머리글",
        "반복 머리글",
        "비밀 대외비 행정안전부 내부용 — 페이지머리글",
        "#3389",
        "머리글이 본문에 한 번 더 그려지면 과잉.",
        "plan",
    ),
    SourceDoc(
        "page-no",
        "header",
        "머리글",
        "쪽번호",
        "- 12 -",
        "#3389",
        "쪽번호 하이픈은 치환·공백 중립 모두 가능.",
        "plan",
    ),
    SourceDoc(
        "compat-jamo",
        "norm",
        "정규화",
        "호환 자모",
        "한글  rum 과  compatibility jamo 의 혼용 ㄱㄴㄷ 가나다",
        "#3389",
        "NFC 전제. NFD 로만 바뀌면 match.",
        "manual",
    ),
    SourceDoc(
        "nbsp-title",
        "norm",
        "정규화",
        "공백 변형",
        "주요 업무 계획  디지털  전환",
        "#3389",
        "공백·NBSP·표의 공백만 바뀌면 match.",
        "plan",
    ),
    SourceDoc(
        "fullwidth-year",
        "norm",
        "정규화",
        "전각 연도",
        "2024학년도 대비 2022학년도 실적 비교",
        "#3389",
        "전각 숫자는 코드포인트가 달라 치환.",
        "korexam",
    ),
    SourceDoc(
        "hyphen-range",
        "norm",
        "정규화",
        "구간 하이픈",
        "사업기간 2022-01-01 - 2022-12-31 (회계연도)",
        "#3389",
        "하이픈 vs en-dash 는 치환.",
        "plan",
    ),
    SourceDoc(
        "issue-3738-head",
        "direct",
        "direct pair",
        "이슈 3738 머리",
        "215쪽 문서 전수 text-only 첫 후보 수집 — Chrome 없이 SVG 한 번",
        "#3738",
        "direct pair --text-only --export-all-svg 경로의 표본 문장.",
        "plan",
    ),
    SourceDoc(
        "issue-3738-tail",
        "direct",
        "direct pair",
        "이슈 3738 꼬리",
        "마지막 쪽 부록 색인 찾아보기 용 키워드 디지털정부 공공기록 개인정보",
        "#3738",
        "끝쪽 색인이 통째로 소실되면 소실 후보.",
        "plan",
    ),
    SourceDoc(
        "overflow-body",
        "overflow",
        "visible excess",
        "본문+다음쪽 선점",
        (
            "본 쪽 본문은 여기까지이다. 그러나 다음 쪽 서론이 이미 이 쪽에 "
            "함께 그려져 기준 PDF 글자는 모두 있으면서 가시 글자가 48자 이상 과잉이다."
        ),
        "#3389",
        "visible-text-excess 임계(48)를 넘는 장문.",
        "manual",
    ),
)


INSERTS: tuple[tuple[str, str], ...] = (
    ("머리글 ", "반복 머리글이 SVG 에만 붙음"),
    (f"{PUA_BULLET} ", "한컴 불릿 과잉"),
)


DROPS: tuple[tuple[str, str], ...] = (
    ("행정안전부", "기관명 소실"),
    ("추진목표", "제목 소실"),
    ("예산", "열 헤더 소실"),
    ("https://www.mois.go.kr/plan/2022", "URL 소실"),
    ("제12조(문서의 성립)", "조문 번호 소실"),
    ("Which of the following is the best title for the passage?", "영어 문항 소실"),
    ("정적분", "수식 지시 소실"),
    ("①", "선지 원문자 소실"),
)


def source_by_key(key: str) -> SourceDoc:
    for doc in SOURCE_DOCS:
        if doc.key == key:
            return doc
    raise KeyError(key)


def measure_case(
    harness: Any,
    case: LayerCase,
) -> dict[str, Any]:
    row = harness.text_layer_row(case.page, case.reference_text, case.svg_text)
    missing: Counter[str] = row["missing"]
    extra: Counter[str] = row["extra"]
    glyph = harness.svg_glyph_risks(case.svg_text)
    ref_glyph = harness.svg_glyph_risks(case.reference_text)
    owner: list[dict[str, Any]] = []
    sequence: list[dict[str, Any]] = []
    if case.next_reference or case.next_svg:
        diffs = {
            case.page: (missing, extra),
            case.page + 1: harness.compare_text_layers(
                case.next_reference, case.next_svg
            ),
        }
        layers = {
            case.page: (case.reference_text, case.svg_text),
            case.page + 1: (case.next_reference, case.next_svg),
        }
        owner = [
            {
                "page": int(item["page"]),
                "next_page": int(item["next_page"]),
                "direction": item["direction"],
                "shared_count": int(item["shared_count"]),
                "source_coverage": float(item["source_coverage"]),
                "target_coverage": float(item["target_coverage"]),
                "shared": harness.counter_summary(item["shared"])
                if isinstance(item["shared"], Counter)
                else item["shared"],
            }
            for item in harness.adjacent_text_owner_shift_candidates(diffs)
        ]
        sequence = [
            {
                "page": int(item["page"]),
                "next_page": int(item["next_page"]),
                "direction": item["direction"],
                "chars": int(item["chars"]),
                "sequence": item["sequence"],
            }
            for item in harness.adjacent_text_owner_sequence_candidates(layers)
        ]
    visible_missing, visible_extra = harness.compare_text_layers(
        case.reference_text, case.svg_text
    )
    visible = harness.visible_text_excess_candidates(
        {case.page: (visible_missing, visible_extra)},
        {case.page: case.clip_excluded},
    )
    payload = {
        "id": case.ident,
        "schema": CASE_SCHEMA,
        "schemaVersion": SCHEMA_VERSION,
        "claim": CLAIM_ID,
        "generator": GENERATOR,
        "kind": case.kind,
        "classification": row["kind"],
        "family": case.family,
        "genre": case.genre,
        "registeredKey": case.registered_key,
        "title": case.title,
        "why": case.why,
        "page": case.page,
        "page1based": case.page + 1,
        "referenceText": case.reference_text,
        "svgText": case.svg_text,
        "referenceNfc": nfc(case.reference_text),
        "svgNfc": nfc(case.svg_text),
        "referenceSequence": harness.normalized_text_sequence(case.reference_text),
        "svgSequence": harness.normalized_text_sequence(case.svg_text),
        "referenceOnly": int(row["reference_only"]),
        "svgOnly": int(row["svg_only"]),
        "referenceOnlyChars": row["reference_only_chars"],
        "svgOnlyChars": row["svg_only_chars"],
        "glyphRiskCount": int(sum(glyph.values())),
        "glyphRisks": harness.counter_summary(glyph) if glyph else "",
        "referenceGlyphRiskCount": int(sum(ref_glyph.values())),
        "referenceGlyphRisks": harness.counter_summary(ref_glyph) if ref_glyph else "",
        "ownerShift": owner,
        "sequence": sequence,
        "visibleExcess": [
            {
                "page": int(item["page"]),
                "reference_only": int(item["reference_only"]),
                "visible_svg_only": int(item["visible_svg_only"]),
                "clip_excluded_chars": int(item["clip_excluded_chars"]),
            }
            for item in visible
        ],
        "clipExcludedChars": case.clip_excluded,
        "nextReferenceText": case.next_reference,
        "nextSvgText": case.next_svg,
        "textOnly": True,
        "chromeRequired": False,
        "pypdfRequired": True,
        "pypdfium2Required": False,
        "candidateNotVerdict": True,
        "reviewHint": case.review_hint,
        "relatedIssue": case.related_issue,
        "notes": case.notes,
        "digest": short_hash(
            f"{case.ident}\0{case.reference_text}\0{case.svg_text}\0{case.kind}"
        ),
    }
    payload.update(case.extras)
    if payload["classification"] != case.kind and case.kind in {
        harness.TEXT_LAYER_LOSS,
        harness.TEXT_LAYER_EXCESS,
        harness.TEXT_LAYER_SUBSTITUTION,
        harness.TEXT_LAYER_MATCH,
    }:
        payload["kind"] = payload["classification"]
        payload["requestedKind"] = case.kind
    return payload


def build_layer_cases(harness: Any) -> list[LayerCase]:
    cases: list[LayerCase] = []

    def add(case: LayerCase) -> None:
        cases.append(case)

    for doc in SOURCE_DOCS:
        add(
            LayerCase(
                f"match-{doc.key}",
                harness.TEXT_LAYER_MATCH,
                doc.family,
                doc.genre,
                doc.registered_key,
                f"{doc.title} — 동일 텍스트",
                f"{doc.why} 같은 문자열이면 match.",
                doc.text,
                doc.text,
                doc.related_issue,
                "시트를 열 필요 없는 문자 일치. 시각 판정은 별개.",
                "공백·순서 무시 NFC 멀티셋이 같다.",
            )
        )
        if doc.key in {
            "plan-cover",
            "manual-article",
            "eng-passage",
            "pua-range",
            "hyphen-range",
            "nbsp-title",
            "compat-jamo",
        }:
            add(
                LayerCase(
                    f"match-ws-{doc.key}",
                    harness.TEXT_LAYER_MATCH,
                    doc.family,
                    doc.genre,
                    doc.registered_key,
                    f"{doc.title} — 공백만 다름",
                    "개행·연속 공백·NBSP 는 isspace 로 제거되어 match.",
                    doc.text,
                    " \n\t".join(doc.text.split()) + "\u00a0",
                    doc.related_issue,
                    "공백 차이만 있으면 text-report 는 0/0.",
                    "NBSP·TAB·LF 는 문자 멀티셋에서 제외.",
                )
            )
            add(
                LayerCase(
                    f"match-nfd-{doc.key}",
                    harness.TEXT_LAYER_MATCH,
                    doc.family,
                    doc.genre,
                    doc.registered_key,
                    f"{doc.title} — NFD 한글",
                    "한글을 NFD 로 풀어 써도 NFC 뒤 match.",
                    doc.text,
                    to_nfd_hangul(doc.text),
                    doc.related_issue,
                    "NFC 정규화가 깨지면 거짓 치환이 난다.",
                    f"보기: {NFC_GA!r} vs {NFD_GA!r}.",
                )
            )
        if len(doc.text.split()) >= 4:
            add(
                LayerCase(
                    f"match-reorder-{doc.key}",
                    harness.TEXT_LAYER_MATCH,
                    doc.family,
                    doc.genre,
                    doc.registered_key,
                    f"{doc.title} — 어절 순서만 변경",
                    "Counter 는 순서를 무시하므로 어절 재배열은 match.",
                    doc.text,
                    " ".join(reversed(doc.text.split())),
                    doc.related_issue,
                    "순서 보존이 필요하면 sequence 원장을 본다.",
                    "단어 수가 1이면 재배열이 무의미해도 동일 분류.",
                )
            )

        for index, (phrase, label) in enumerate(DROPS):
            if phrase not in doc.text:
                continue
            svg = drop_phrase(doc.text, phrase)
            if svg == doc.text:
                continue
            add(
                LayerCase(
                    f"loss-{doc.key}-{index:02d}",
                    harness.TEXT_LAYER_LOSS,
                    doc.family,
                    doc.genre,
                    doc.registered_key,
                    f"{doc.title} — {label}",
                    f"{doc.why} {label}.",
                    doc.text,
                    svg,
                    doc.related_issue,
                    "reference_only 상위부터 시트 감사.",
                    f"제거 조각: {phrase}",
                )
            )

        for index, (insert, label) in enumerate(INSERTS):
            add(
                LayerCase(
                    f"excess-{doc.key}-{index:02d}",
                    harness.TEXT_LAYER_EXCESS,
                    doc.family,
                    doc.genre,
                    doc.registered_key,
                    f"{doc.title} — {label}",
                    f"{doc.why} {label}.",
                    doc.text,
                    insert + doc.text,
                    doc.related_issue,
                    "svg_only 가 머리글/각주 중복인지 확인.",
                    f"삽입 조각: {insert.strip()}",
                )
            )

        fw = to_fullwidth_digits(doc.text)
        if fw != doc.text:
            add(
                LayerCase(
                    f"sub-fullwidth-{doc.key}",
                    harness.TEXT_LAYER_SUBSTITUTION,
                    doc.family,
                    doc.genre,
                    doc.registered_key,
                    f"{doc.title} — 전각 숫자",
                    "반각 숫자와 전각 숫자는 다른 코드포인트라 치환.",
                    doc.text,
                    fw,
                    doc.related_issue,
                    "추출기 매핑일 수 있다. 시트로 확인.",
                    "0-9 → ０-９",
                )
            )
        dashed = hyphen_to_dash(doc.text)
        if dashed != doc.text:
            add(
                LayerCase(
                    f"sub-dash-{doc.key}",
                    harness.TEXT_LAYER_SUBSTITUTION,
                    doc.family,
                    doc.genre,
                    doc.registered_key,
                    f"{doc.title} — 대시 치환",
                    "HYPHEN-MINUS 와 EN/EM DASH 는 치환.",
                    doc.text,
                    dashed,
                    doc.related_issue,
                    "시각적으로 비슷해도 멀티셋은 갈라진다.",
                    "U+002D vs U+2013/U+2014",
                )
            )
        ascii_circled = circled_to_ascii(doc.text)
        if ascii_circled != doc.text:
            add(
                LayerCase(
                    f"sub-circled-{doc.key}",
                    harness.TEXT_LAYER_SUBSTITUTION,
                    doc.family,
                    doc.genre,
                    doc.registered_key,
                    f"{doc.title} — 원문자→ASCII",
                    "① 과 (1) 은 치환. PUA 원문자와도 다르다.",
                    doc.text,
                    ascii_circled,
                    doc.related_issue,
                    "선지 번호 매핑을 의심.",
                    "①→(1)",
                )
            )
        if any(ord(ch) >= 0xF0000 for ch in doc.text):
            add(
                LayerCase(
                    f"sub-pua-fffd-{doc.key}",
                    harness.TEXT_LAYER_SUBSTITUTION,
                    doc.family,
                    doc.genre,
                    doc.registered_key,
                    f"{doc.title} — PUA→U+FFFD",
                    "디코더가 PUA 를 대치 문자로 바꾸면 치환+glyph-risk.",
                    doc.text,
                    "".join(FFFD if ord(ch) >= 0xF0000 else ch for ch in doc.text),
                    doc.related_issue,
                    "두부 시트는 하네스 오염과 구분(F14).",
                    "SIP PUA → U+FFFD",
                )
            )

    # Adjacent owner-shift / sequence pairs.
    early_pairs = (
        ("fn-early", "각주가 한 쪽 이르게 SVG 에 나타남", "rhwp_earlier_than_reference"),
        ("plan-footnote", "각주+URL 이 pN SVG / pN+1 PDF", "rhwp_earlier_than_reference"),
        ("plan-caption", "그림 캡션 조기 배치", "rhwp_earlier_than_reference"),
        ("math-caption", "수식 그림 캡션 조기 배치", "rhwp_earlier_than_reference"),
        ("eng-url", "영어 출처 URL 조기 배치", "rhwp_earlier_than_reference"),
        ("citation-url", "법령 URL 조기 배치", "rhwp_earlier_than_reference"),
        ("citation-law", "법령명 조기 배치", "rhwp_earlier_than_reference"),
        ("table-first-row", "표 첫 줄이 다음 쪽에 남고 현재 쪽에 과잉", "rhwp_earlier_than_reference"),
        ("manual-note", "편람 각주 조기", "rhwp_earlier_than_reference"),
        ("korexam-header", "시험 머리글 조기 반복", "rhwp_earlier_than_reference"),
    )
    late_pairs = (
        ("fn-late", "각주가 한 쪽 늦게 SVG 에 나타남", "rhwp_later_than_reference"),
        ("plan-footnote", "각주+URL 이 pN PDF / pN+1 SVG", "rhwp_later_than_reference"),
        ("eng-url", "영어 출처 URL 지연", "rhwp_later_than_reference"),
        ("citation-url", "법령 URL 지연", "rhwp_later_than_reference"),
        ("citation-law", "법령명 지연", "rhwp_later_than_reference"),
        ("manual-note", "편람 각주 지연", "rhwp_later_than_reference"),
        ("issue-3738-tail", "끝쪽 색인 지연", "rhwp_later_than_reference"),
        ("manual-article", "조문 본문 지연", "rhwp_later_than_reference"),
        ("korexam-stem", "지문 도입 지연", "rhwp_later_than_reference"),
        ("plan-goal", "추진목표 문단 지연", "rhwp_later_than_reference"),
    )
    body_keep = "본문고정영역ABCDEFGH이번쪽공통"

    for index, (key, title, direction) in enumerate(early_pairs):
        doc = source_by_key(key)
        add(
            LayerCase(
                f"owner-early-{index:02d}-{key}",
                harness.TEXT_LAYER_EXCESS,
                "owner_shift",
                doc.genre,
                doc.registered_key,
                title,
                f"{doc.why} 방향={direction}.",
                body_keep,
                body_keep + doc.text,
                doc.related_issue,
                "text-owner-shift-candidates 와 sequence 를 함께 본다.",
                direction,
                page=10 + index,
                next_reference=body_keep + doc.text,
                next_svg=body_keep,
            )
        )
    for index, (key, title, direction) in enumerate(late_pairs):
        doc = source_by_key(key)
        add(
            LayerCase(
                f"owner-late-{index:02d}-{key}",
                harness.TEXT_LAYER_LOSS,
                "owner_shift",
                doc.genre,
                doc.registered_key,
                title,
                f"{doc.why} 방향={direction}.",
                body_keep + doc.text,
                body_keep,
                doc.related_issue,
                "PDF 쪽이 한 쪽 앞선 후보. 시트 없이 확정 금지.",
                direction,
                page=30 + index,
                next_reference=body_keep,
                next_svg=body_keep + doc.text,
            )
        )

    overflow = source_by_key("overflow-body")
    add(
        LayerCase(
            "visible-excess-overflow-body",
            harness.TEXT_LAYER_EXCESS,
            "overflow",
            overflow.genre,
            overflow.registered_key,
            overflow.title,
            overflow.why,
            overflow.text,
            overflow.text + ("다음쪽선점문장" * 8),
            overflow.related_issue,
            "visible_svg_only>=48 이고 reference_only 가 작을 때만 후보.",
            "clip_excluded=0",
            clip_excluded=0,
        )
    )
    add(
        LayerCase(
            "visible-excess-with-clip",
            harness.TEXT_LAYER_EXCESS,
            "overflow",
            overflow.genre,
            overflow.registered_key,
            "가시 과잉 + clip 제외 문자",
            "clip 밖 숨은 이전 표 조각은 raw SVG 원장을 부풀리지만 visible 원장은 별도.",
            overflow.text,
            overflow.text + ("가시과잉추가분" * 8),
            overflow.related_issue,
            "clip_excluded_chars 열을 함께 읽는다.",
            "hidden table fragment",
            clip_excluded=36,
        )
    )

    # Mixed substitution that also drops a phrase.
    mixed = source_by_key("plan-goal")
    add(
        LayerCase(
            "sub-mixed-plan-goal",
            harness.TEXT_LAYER_SUBSTITUTION,
            mixed.family,
            mixed.genre,
            mixed.registered_key,
            "목표 문단 소실+전각 혼합",
            "한 쪽에서 단어가 빠지고 다른 쪽에서 숫자가 바뀌면 치환.",
            mixed.text + " 2022",
            to_fullwidth_digits(drop_phrase(mixed.text + " 2022", "체감하는")),
            mixed.related_issue,
            "loss+excess 가 함께 있으면 substitution.",
            "체감하는 소실 + 2022 전각",
        )
    )
    return cases


def build_path_specs(harness: Any) -> list[PathSpec]:
    specs: list[PathSpec] = []
    keys = sorted(harness.REG)
    flag_sets = (
        (False, False, "text-only"),
        (True, False, "text-only+export-all-svg"),
        (False, True, "text-only+layout-ledger"),
        (True, True, "text-only+export-all-svg+layout-ledger"),
    )
    for key in keys:
        for export_all, layout, label in flag_sets:
            argv = [key, "0", "2", "--text-only"]
            if export_all:
                argv.append("--export-all-svg")
            if layout:
                argv.append("--layout-ledger")
            argv.extend(["--out-dir", "/tmp/rhwp-fidelity-" + key])
            specs.append(
                PathSpec(
                    f"path-reg-{key}-{label.replace('+', '-')}",
                    argv,
                    "registered",
                    export_all,
                    layout,
                    False,
                    "",
                    False,
                    True,
                    False,
                    f"등록 키 {key} / {label}",
                    f"{harness.REG[key].reference_grade} 를 --text-only 로 연다.",
                )
            )

    specs.append(
        PathSpec(
            "path-direct-text-only",
            [
                "0",
                "214",
                "--source",
                "samples/입력.hwp",
                "--reference-pdf",
                "pdf/한컴-기준.pdf",
                "--label",
                "issue-3738-hwp",
                "--reference-grade",
                "한컴 2020 기준 PDF",
                "--text-only",
                "--export-all-svg",
                "--layout-ledger",
                "--out-dir",
                "/tmp/rhwp-fidelity-issue-3738",
            ],
            "direct",
            True,
            True,
            False,
            "",
            False,
            True,
            False,
            "direct pair 215쪽 전수 텍스트",
            "README 의 #3738 첫 후보 수집 경로.",
        )
    )
    specs.append(
        PathSpec(
            "path-direct-text-only-min",
            [
                "0",
                "0",
                "--source",
                "samples/a.hwp",
                "--reference-pdf",
                "pdf/a-2022.pdf",
                "--label",
                "pair-a",
                "--text-only",
            ],
            "direct",
            False,
            False,
            False,
            "",
            False,
            True,
            False,
            "direct pair 최소 --text-only",
            "한 쪽만 텍스트 원장.",
        )
    )
    specs.append(
        PathSpec(
            "path-error-unknown-key",
            ["unknown", "0", "1", "--text-only"],
            "error",
            False,
            False,
            True,
            "등록되지 않은 문서 키",
            False,
            False,
            False,
            "미등록 키",
            "등록 fixture positional 검증.",
        )
    )
    specs.append(
        PathSpec(
            "path-error-direct-incomplete",
            ["0", "1", "--source", "a.hwp", "--text-only"],
            "error",
            False,
            False,
            True,
            "direct pair에는 --source, --reference-pdf, --label을 모두",
            False,
            False,
            False,
            "direct pair 불완전",
            "세 플래그 중 일부만 있으면 오류.",
        )
    )
    specs.append(
        PathSpec(
            "path-error-grade-on-registered",
            ["plan", "0", "1", "--reference-grade", "x", "--text-only"],
            "error",
            False,
            False,
            True,
            "--reference-grade는 direct pair에서만",
            False,
            False,
            False,
            "등록 키에 grade",
            "등록 fixture 는 grade 를 REG 가 가진다.",
        )
    )
    specs.append(
        PathSpec(
            "path-error-end-before-start",
            ["plan", "5", "1", "--text-only"],
            "error",
            False,
            False,
            True,
            "끝 쪽이 시작 쪽보다 작을 수 없습니다",
            False,
            False,
            False,
            "쪽 범위 역전",
            "끝 < 시작 이면 오류.",
        )
    )
    specs.append(
        PathSpec(
            "path-error-direct-three-positionals",
            [
                "plan",
                "0",
                "1",
                "--source",
                "a.hwp",
                "--reference-pdf",
                "b.pdf",
                "--label",
                "x",
                "--text-only",
            ],
            "error",
            False,
            False,
            True,
            "direct pair positional은 <시작쪽> <끝쪽> 두 개여야",
            False,
            False,
            False,
            "direct pair 에 키까지",
            "direct 는 positional 2개.",
        )
    )
    specs.append(
        PathSpec(
            "path-error-non-integer-page",
            ["plan", "0", "abc", "--text-only"],
            "error",
            False,
            False,
            True,
            "끝쪽은 정수여야",
            False,
            False,
            False,
            "쪽 번호 비정수",
            "끝쪽은 정수 파싱을 거친다.",
        )
    )
    return specs


def measure_path(harness: Any, spec: PathSpec) -> dict[str, Any]:
    error = ""
    parsed: dict[str, Any] | None = None
    if spec.expects_error:
        stderr = io.StringIO()
        try:
            with contextlib.redirect_stderr(stderr):
                harness.parse_args(spec.argv)
        except SystemExit:
            error = spec.error_needle
        else:
            error = "expected-error-missing"
    else:
        args = harness.parse_args(spec.argv)
        parsed = {
            "key": args.key,
            "startPage": args.start_page,
            "endPage": args.end_page,
            "textOnly": bool(args.text_only),
            "exportAllSvg": bool(args.export_all_svg),
            "layoutLedger": bool(args.layout_ledger),
            "label": args.label,
            "referenceGrade": args.reference_grade,
        }
    artifacts = harness.text_only_artifact_names(
        export_all_svg=spec.export_all_svg,
        layout_ledger=spec.layout_ledger,
    )
    return {
        "id": spec.ident,
        "schema": "rhwp.fidelity_compare.text_only_path.v1",
        "schemaVersion": SCHEMA_VERSION,
        "claim": CLAIM_ID,
        "title": spec.title,
        "why": spec.why,
        "argv": spec.argv,
        "mode": spec.mode,
        "exportAllSvg": spec.export_all_svg,
        "layoutLedger": spec.layout_ledger,
        "expectsError": spec.expects_error,
        "errorNeedle": spec.error_needle,
        "error": error,
        "parsed": parsed,
        "artifacts": list(artifacts),
        "chromeRequired": spec.chrome_required,
        "pypdfRequired": spec.pypdf_required,
        "pypdfium2Required": spec.pypdfium2_required,
        "pixelSheets": False,
        "candidateNotVerdict": True,
    }


def build_svg_fixtures() -> list[dict[str, Any]]:
    fixtures: list[dict[str, Any]] = []
    body_clip = (
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">'
        "<defs><clipPath id=\"body\"><rect x=\"0\" y=\"0\" width=\"100\" height=\"100\"/>"
        "</clipPath><clipPath id=\"cell\"><rect x=\"0\" y=\"40\" width=\"100\" height=\"10\"/>"
        "</clipPath></defs>"
        "<g clip-path=\"url(#body)\">"
        "<text x=\"10\" y=\"-10\" font-size=\"10\">hidden-top</text>"
        "<text x=\"10\" y=\"20\" font-size=\"10\">body-visible</text>"
        "<g clip-path=\"url(#cell)\">"
        "<text x=\"10\" y=\"20\" font-size=\"10\">hidden-cell</text>"
        "<text x=\"10\" y=\"47\" font-size=\"10\">partial-cell</text>"
        "</g></g></svg>"
    )
    fixtures.append(
        {
            "id": "svg-clip-body-cell",
            "title": "body/cell clip 이 이전 표 조각을 가림",
            "why": "raw SVG text 는 hidden 을 세고 visible 원장은 가시 band 만.",
            "svg": body_clip,
            "expectVisible": "body-visiblepartial-cell",
            "expectExcludedAtLeast": len("hidden-tophidden-cell"),
        }
    )
    pua_svg = (
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 40">'
        f'<text x="8" y="24" font-size="16">항목{PUA_CIRCLED[:3]}{FFFD}</text>'
        "</svg>"
    )
    fixtures.append(
        {
            "id": "svg-pua-and-fffd",
            "title": "raw PUA + U+FFFD",
            "why": "glyph-risk 는 PDF 추출과 독립.",
            "svg": pua_svg,
            "expectVisible": f"항목{PUA_CIRCLED[:3]}{FFFD}",
            "expectExcludedAtLeast": 0,
            "expectGlyphs": PUA_CIRCLED[:3] + FFFD,
        }
    )
    hidden_display = (
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 40">'
        '<text x="4" y="20" font-size="12">shown</text>'
        '<text x="4" y="20" font-size="12" display="none">hidden-display</text>'
        '<g visibility="hidden"><text x="4" y="30" font-size="12">hidden-vis</text></g>'
        "</svg>"
    )
    fixtures.append(
        {
            "id": "svg-display-none",
            "title": "display/visibility 숨김",
            "why": "숨김 노드는 visible walker 가 건너뛴다.",
            "svg": hidden_display,
            "expectVisible": "shown",
            "expectExcludedAtLeast": 0,
        }
    )
    no_viewport = (
        '<svg xmlns="http://www.w3.org/2000/svg">'
        '<text x="4" y="20" font-size="12">keep-unknown</text>'
        "</svg>"
    )
    fixtures.append(
        {
            "id": "svg-unknown-viewport",
            "title": "viewport 없는 SVG",
            "why": "좌표를 해석할 수 없으면 보수적으로 포함.",
            "svg": no_viewport,
            "expectVisible": "keep-unknown",
            "expectExcludedAtLeast": 0,
        }
    )
    return fixtures


def emit_cases(bundle: FattenBundle) -> None:
    cases_dir = bundle.out_root / "fixtures" / "text_layer" / "cases"
    index_rows: list[dict[str, Any]] = []
    for case in bundle.cases:
        path = cases_dir / f"{case['id']}.json"
        write_json(path, case)
        record(bundle, path)
        index_rows.append(
            {
                "id": case["id"],
                "kind": case["classification"],
                "family": case["family"],
                "registeredKey": case["registeredKey"],
                "referenceOnly": case["referenceOnly"],
                "svgOnly": case["svgOnly"],
                "title": case["title"],
            }
        )
    write_text(
        record(bundle, bundle.out_root / "fixtures" / "text_layer" / "index.json"),
        json.dumps(
            {
                "schema": "rhwp.fidelity_compare.text_layer_index.v1",
                "claim": CLAIM_ID,
                "caseCount": len(index_rows),
                "cases": index_rows,
            },
            ensure_ascii=False,
            separators=(",", ":"),
        ),
    )
    lines = [
        json.dumps(
            {
                "id": row["id"],
                "kind": row["kind"],
                "family": row["family"],
                "registeredKey": row["registeredKey"],
                "referenceOnly": row["referenceOnly"],
                "svgOnly": row["svgOnly"],
                "title": row["title"],
            },
            ensure_ascii=False,
            separators=(",", ":"),
        )
        for row in index_rows
    ]
    write_text(
        record(bundle, bundle.out_root / "fixtures" / "text_layer" / "index.jsonl"),
        "\n".join(lines),
    )


def emit_tables(bundle: FattenBundle) -> None:
    header = (
        "id\tkind\tfamily\tregistered_key\tgenre\tpage\treference_only\tsvg_only\t"
        "reference_only_chars\tsvg_only_chars\tglyph_risk_count\towner_shift\t"
        "sequence\tvisible_excess\trelated_issue\ttitle\n"
    )
    by_kind: dict[str, list[dict[str, Any]]] = {
        "loss": [],
        "excess": [],
        "substitution": [],
        "match": [],
    }
    for case in bundle.cases:
        by_kind.setdefault(case["classification"], []).append(case)

    def row(case: dict[str, Any]) -> str:
        owner = case["ownerShift"][0]["direction"] if case["ownerShift"] else "-"
        seq = case["sequence"][0]["direction"] if case["sequence"] else "-"
        vis = "yes" if case["visibleExcess"] else "-"
        return (
            f"{case['id']}\t{case['classification']}\t{case['family']}\t"
            f"{case['registeredKey']}\t{case['genre']}\t{case['page1based']}\t"
            f"{case['referenceOnly']}\t{case['svgOnly']}\t"
            f"{case['referenceOnlyChars'] or '-'}\t{case['svgOnlyChars'] or '-'}\t"
            f"{case['glyphRiskCount']}\t{owner}\t{seq}\t{vis}\t"
            f"{case['relatedIssue']}\t{case['title']}\n"
        )

    tables = bundle.out_root / "tables"
    all_rows = header + "".join(row(case) for case in bundle.cases)
    write_text(record(bundle, tables / "text_layer_all.tsv"), all_rows)
    for kind, rows in by_kind.items():
        write_text(
            record(bundle, tables / f"{kind}.tsv"),
            header + "".join(row(case) for case in rows),
        )
    owner_rows = [case for case in bundle.cases if case["ownerShift"] or case["sequence"]]
    write_text(
        record(bundle, tables / "owner_shift.tsv"),
        header + "".join(row(case) for case in owner_rows),
    )
    glyph_rows = [case for case in bundle.cases if case["glyphRiskCount"]]
    write_text(
        record(bundle, tables / "glyph_risk.tsv"),
        header + "".join(row(case) for case in glyph_rows),
    )
    visible_rows = [case for case in bundle.cases if case["visibleExcess"]]
    write_text(
        record(bundle, tables / "visible_excess.tsv"),
        header + "".join(row(case) for case in visible_rows),
    )


def emit_paths(bundle: FattenBundle) -> None:
    path_dir = bundle.out_root / "fixtures" / "text_only_paths"
    for spec in bundle.paths:
        write_json(record(bundle, path_dir / f"{spec['id']}.json"), spec)
    write_json(
        record(bundle, path_dir / "index.json"),
        {
            "schema": "rhwp.fidelity_compare.text_only_path_index.v1",
            "claim": CLAIM_ID,
            "pathCount": len(bundle.paths),
            "paths": [
                {
                    "id": item["id"],
                    "mode": item["mode"],
                    "expectsError": item["expectsError"],
                    "artifacts": item["artifacts"],
                    "title": item["title"],
                }
                for item in bundle.paths
            ],
        },
    )
    header = (
        "id\tmode\texpects_error\texport_all_svg\tlayout_ledger\t"
        "chrome\tpypdf\tpypdfium2\tartifact_count\ttitle\n"
    )
    body = "".join(
        f"{item['id']}\t{item['mode']}\t{int(item['expectsError'])}\t"
        f"{int(item['exportAllSvg'])}\t{int(item['layoutLedger'])}\t"
        f"{int(item['chromeRequired'])}\t{int(item['pypdfRequired'])}\t"
        f"{int(item['pypdfium2Required'])}\t{len(item['artifacts'])}\t"
        f"{item['title']}\n"
        for item in bundle.paths
    )
    write_text(record(bundle, bundle.out_root / "tables" / "text_only_paths.tsv"), header + body)


def emit_svgs(bundle: FattenBundle) -> None:
    svg_dir = bundle.out_root / "fixtures" / "svg"
    for item in bundle.svgs:
        svg_path = svg_dir / f"{item['id']}.svg"
        write_text(svg_path, item["svg"])
        record(bundle, svg_path)
        meta = {key: value for key, value in item.items() if key != "svg"}
        meta["svgPath"] = f"fixtures/svg/{item['id']}.svg"
        write_json(record(bundle, svg_dir / f"{item['id']}.json"), meta)


def emit_reports(bundle: FattenBundle) -> None:
    counts = Counter(case["classification"] for case in bundle.cases)
    family_counts = Counter(case["family"] for case in bundle.cases)
    key_counts = Counter(case["registeredKey"] for case in bundle.cases)
    summary = {
        "schema": "rhwp.fidelity_compare.fatten_summary.v1",
        "claim": CLAIM_ID,
        "generator": GENERATOR,
        "generatedAt": bundle.generated_at,
        "caseCount": len(bundle.cases),
        "pathCount": len(bundle.paths),
        "svgFixtureCount": len(bundle.svgs),
        "kindCounts": dict(counts),
        "familyCounts": dict(family_counts),
        "registeredKeyCounts": dict(key_counts),
        "ownerShiftCases": sum(1 for case in bundle.cases if case["ownerShift"]),
        "sequenceCases": sum(1 for case in bundle.cases if case["sequence"]),
        "glyphRiskCases": sum(1 for case in bundle.cases if case["glyphRiskCount"]),
        "visibleExcessCases": sum(1 for case in bundle.cases if case["visibleExcess"]),
        "constraints": {
            "visualSweepTouched": False,
            "engineTouched": False,
            "gymTouched": False,
            "canvaskitTouched": False,
            "serializerTouched": False,
            "layoutAnomalyTouched": False,
            "renderBackendTouched": False,
            "proptestTouched": False,
        },
        "writtenCount": len(bundle.written),
    }
    write_json(record(bundle, bundle.out_root / "reports" / "fatten_summary.json"), summary)

    def table(rows: Iterable[tuple[Any, ...]], headers: Sequence[str]) -> list[str]:
        out = [
            "| " + " | ".join(headers) + " |",
            "| " + " | ".join("---" for _ in headers) + " |",
        ]
        for row in rows:
            out.append("| " + " | ".join(md_cell(cell) for cell in row) + " |")
        return out

    kind_md = [
        "# 텍스트층 분류 집계",
        "",
        f"생성 시각: `{bundle.generated_at}`",
        "",
        *table(
            ((kind, counts[kind]) for kind in ("loss", "excess", "substitution", "match") if kind in counts),
            ("분류", "건수"),
        ),
        "",
        "## 등록 키",
        "",
        *table(sorted(key_counts.items()), ("키", "건수")),
        "",
        "## 가족",
        "",
        *table(sorted(family_counts.items()), ("가족", "건수")),
        "",
    ]
    write_text(record(bundle, bundle.out_root / "reports" / "kind_counts.md"), "\n".join(kind_md))

    for kind in ("loss", "excess", "substitution", "match"):
        rows = [case for case in bundle.cases if case["classification"] == kind]
        md = [
            f"# text-report 분류표 — {kind}",
            "",
            "후보 검출이다. 최종 시각 판정이 아니다.",
            "",
            f"건수: **{len(rows)}**",
            "",
            *table(
                (
                    (
                        case["id"],
                        case["registeredKey"],
                        case["referenceOnly"],
                        case["svgOnly"],
                        case["glyphRiskCount"],
                        case["title"],
                    )
                    for case in rows
                ),
                ("id", "키", "소실", "과잉", "glyph", "제목"),
            ),
            "",
        ]
        write_text(record(bundle, bundle.out_root / "reports" / f"{kind}_table.md"), "\n".join(md))

    path_md = [
        "# --text-only 경로 카탈로그",
        "",
        "Chrome·pypdfium2 는 `--text-only` 에서 요구하지 않는다. pypdf 만.",
        "",
        *table(
            (
                (
                    item["id"],
                    item["mode"],
                    "error" if item["expectsError"] else "ok",
                    ",".join(item["artifacts"][:3]) + ("…" if len(item["artifacts"]) > 3 else ""),
                    item["title"],
                )
                for item in bundle.paths
            ),
            ("id", "mode", "parse", "artifacts", "제목"),
        ),
        "",
        "## 산출 계약",
        "",
        *table(
            (
                (name, "항상")
                for name in bundle.harness.TEXT_ONLY_CORE_ARTIFACTS
            ),
            ("파일", "조건"),
        ),
        *table(
            ((name, "--layout-ledger") for name in bundle.harness.TEXT_ONLY_LAYOUT_ARTIFACTS),
            ("파일", "조건"),
        ),
        "",
        "| svg/export-svg-manifest.json | --export-all-svg |",
        "",
    ]
    write_text(
        record(bundle, bundle.out_root / "transcripts" / "text_only_paths.md"),
        "\n".join(path_md),
    )
    write_text(
        record(bundle, bundle.out_root / "transcripts" / "text_only_paths.json"),
        json.dumps(
            {
                "schema": "rhwp.fidelity_compare.text_only_transcript.v1",
                "claim": CLAIM_ID,
                "generatedAt": bundle.generated_at,
                "pathCount": len(bundle.paths),
                "paths": [
                    {
                        "id": item["id"],
                        "mode": item["mode"],
                        "expectsError": item["expectsError"],
                        "exportAllSvg": item["exportAllSvg"],
                        "layoutLedger": item["layoutLedger"],
                        "artifactCount": len(item["artifacts"]),
                        "title": item["title"],
                    }
                    for item in bundle.paths
                ],
            },
            ensure_ascii=False,
            indent=2,
        ),
    )

    summary_md = [
        "# M-fid fatten 요약",
        "",
        f"- 클레임: `{CLAIM_ID}`",
        f"- 생성기: `{GENERATOR}`",
        f"- 시각: `{bundle.generated_at}`",
        f"- 텍스트층 케이스: **{len(bundle.cases)}**",
        f"- --text-only 경로: **{len(bundle.paths)}**",
        f"- SVG 픽스처: **{len(bundle.svgs)}**",
        f"- 소실/과잉/치환/일치: {counts.get('loss', 0)}/"
        f"{counts.get('excess', 0)}/{counts.get('substitution', 0)}/{counts.get('match', 0)}",
        "",
        "## 하지 않은 것",
        "",
        "- `scripts/visual_sweep.py` 미수정",
        "- canvaskit_policy · serializer · layout-anomaly · render_backend · proptest 미수정",
        "- gym 미수정",
        "",
        "## 산출물",
        "",
        f"- 파일 수: **{len(bundle.written)}**",
        "- `fixtures/text_layer/cases/` — 쪽별 소실·과잉·치환·일치 픽스처",
        "- `tables/{loss,excess,substitution,match}.tsv` — 분류표",
        "- `fixtures/text_only_paths/` — `--text-only` parse·산출 계약",
        "- `fixtures/svg/` — clip/PUA visible walker",
        "- `WORKING.md` — 작업 기록",
        "",
    ]
    write_text(record(bundle, bundle.out_root / "reports" / "fatten_summary.md"), "\n".join(summary_md))


def emit_working_doc(bundle: FattenBundle) -> None:
    counts = Counter(case["classification"] for case in bundle.cases)
    md = [
        "# M-fid: fidelity_compare 텍스트층·픽스처 고도화",
        "",
        f"날짜: {bundle.generated_at[:10]}",
        "이슈: https://github.com/edwardkim/rhwp/issues/5467",
        "브랜치: `feat/m-fid-fatten` (`upstream/devel` 기준 격리 worktree)",
        "범위: `tools/fidelity_compare/` 만",
        "비범위: `scripts/visual_sweep.py` · canvaskit_policy · serializer ·",
        "layout-anomaly · render_backend · proptest · gym",
        "",
        "## 무엇을",
        "",
        "한컴 기준 PDF 텍스트층과 rhwp SVG `<text>` 를 쪽별로 대조하는",
        "`text-report.tsv` 후보를 **소실 / 과잉 / 치환 / 일치** 로 분류하는",
        "픽스처와 표를 닫는다. `--text-only` 경로의 산출 계약과 parse 오류도",
        "같은 폴더에 고정한다.",
        "",
        f"- 텍스트층 케이스 {len(bundle.cases)}건 "
        f"(loss {counts.get('loss', 0)}, excess {counts.get('excess', 0)}, "
        f"substitution {counts.get('substitution', 0)}, match {counts.get('match', 0)})",
        f"- `--text-only` 경로 {len(bundle.paths)}건",
        f"- SVG clip/PUA 픽스처 {len(bundle.svgs)}건",
        "",
        "## 왜",
        "",
        "픽셀 diff% 는 자간 잡음에 민감하다. 문자 멀티셋은 폰트 대체와 무관한",
        "소실·과잉·치환 후보를 먼저 고른다. 이 루프가 #3385 PUA 원문자 tofu 를",
        "찾았다. 분류는 후보이지 판결이 아니다.",
        "",
        "## 어떻게",
        "",
        "1. `classify_text_layer_delta` / `text_layer_row` / `write_text_report` /",
        "   `text_only_artifact_names` 를 하네스에 명시한다.",
        "2. `fatten_text_layer.py` 가 등록 키·이슈 문장·NFC/전각/PUA/URL/각주",
        "   변이를 라이브 함수로 재분류한다.",
        "3. `tables/loss.tsv` `excess.tsv` `substitution.tsv` `match.tsv` 와",
        "   owner-shift · glyph-risk · visible-excess 표를 방출한다.",
        "4. `fixtures/text_only_paths/` 가 등록 키 4깃발 × 6키 + direct + 오류 경로를 고정한다.",
        "5. Chrome 없이 `test_fatten_text_layer.py` 가 라이브 함수와 픽스처를 대조한다.",
        "",
        "## 분류 규칙",
        "",
        "| 분류 | 조건 | 원장 |",
        "| --- | --- | --- |",
        "| match | reference_only=0 이고 svg_only=0 | text-report |",
        "| loss | reference_only>0 이고 svg_only=0 | text-report |",
        "| excess | svg_only>0 이고 reference_only=0 | text-report |",
        "| substitution | 둘 다 >0 | text-report |",
        "| owner-shift | 인접 쪽 75% 상호 일치, 8자+ | text-owner-shift-candidates |",
        "| sequence | 16자+ 순서 보존 이동 | text-owner-sequence-candidates |",
        "| visible-excess | 가시 과잉 48자+, 소실 작음 | visible-text-excess-candidates |",
        "| glyph-risk | raw PUA 또는 U+FFFD | svg-glyph-risk-report |",
        "",
        "공백은 `str.isspace` 로 제거한다. 한글은 NFC. 순서는 Counter 에서 무시하고",
        "sequence 원장에서만 본다.",
        "",
        "## --text-only",
        "",
        "- pypdf 필요, Chrome·pypdfium2 불필요",
        "- `report.tsv` 의 diff% 는 `not-run`",
        "- `--export-all-svg` 는 SVG cache 한 번",
        "- `--layout-ledger` 는 render-tree 후보 원장",
        "- 픽셀 시트 `cmp-pNNN.png` 를 만들지 않는다",
        "",
        "## 하지 않은 것",
        "",
        "- `scripts/visual_sweep.py` 미수정",
        "- 렌더러·serializer·canvaskit_policy 미수정",
        "- gym pack / 채점기 없음",
        "- 암호화 PDF 우회 없음",
        "",
        "## 검증",
        "",
        "```bash",
        "python tools/fidelity_compare/test_fidelity_compare.py",
        "python tools/fidelity_compare/test_fatten_text_layer.py",
        "python tools/fidelity_compare/fatten_text_layer.py",
        "cargo fmt --all -- --check",
        "```",
        "",
    ]
    write_text(record(bundle, bundle.out_root / "WORKING.md"), "\n".join(md))


def emit_schema(bundle: FattenBundle) -> None:
    schema = {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": CASE_SCHEMA,
        "title": "fidelity_compare text-layer case",
        "type": "object",
        "required": [
            "id",
            "kind",
            "classification",
            "referenceText",
            "svgText",
            "referenceOnly",
            "svgOnly",
            "candidateNotVerdict",
        ],
        "properties": {
            "id": {"type": "string"},
            "kind": {
                "type": "string",
                "enum": ["loss", "excess", "substitution", "match"],
            },
            "classification": {
                "type": "string",
                "enum": ["loss", "excess", "substitution", "match"],
            },
            "referenceText": {"type": "string"},
            "svgText": {"type": "string"},
            "referenceOnly": {"type": "integer", "minimum": 0},
            "svgOnly": {"type": "integer", "minimum": 0},
            "candidateNotVerdict": {"type": "boolean", "const": True},
            "textOnly": {"type": "boolean"},
            "chromeRequired": {"type": "boolean"},
        },
    }
    write_json(record(bundle, bundle.out_root / "schema" / "text_layer_case.v1.json"), schema)
    path_schema = {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "rhwp.fidelity_compare.text_only_path.v1",
        "title": "fidelity_compare --text-only path",
        "type": "object",
        "required": ["id", "argv", "artifacts", "chromeRequired", "pypdfium2Required"],
        "properties": {
            "id": {"type": "string"},
            "argv": {"type": "array", "items": {"type": "string"}},
            "artifacts": {"type": "array", "items": {"type": "string"}},
            "chromeRequired": {"type": "boolean", "const": False},
            "pypdfium2Required": {"type": "boolean", "const": False},
        },
    }
    write_json(
        record(bundle, bundle.out_root / "schema" / "text_only_path.v1.json"),
        path_schema,
    )


def run(out_root: Path) -> dict[str, Any]:
    harness = load_harness()
    bundle = FattenBundle(generated_at=utc_now(), out_root=out_root, harness=harness)
    raw_cases = build_layer_cases(harness)
    seen: set[str] = set()
    for raw in raw_cases:
        measured = measure_case(harness, raw)
        if measured["id"] in seen:
            raise RuntimeError(f"duplicate case id: {measured['id']}")
        seen.add(measured["id"])
        bundle.cases.append(measured)
    for spec in build_path_specs(harness):
        bundle.paths.append(measure_path(harness, spec))
    bundle.svgs = build_svg_fixtures()
    emit_schema(bundle)
    emit_cases(bundle)
    emit_tables(bundle)
    emit_paths(bundle)
    emit_svgs(bundle)
    emit_working_doc(bundle)
    emit_reports(bundle)
    return {
        "claim": CLAIM_ID,
        "generatedAt": bundle.generated_at,
        "caseCount": len(bundle.cases),
        "pathCount": len(bundle.paths),
        "svgFixtureCount": len(bundle.svgs),
        "kindCounts": dict(Counter(case["classification"] for case in bundle.cases)),
        "written": list(bundle.written),
        "constraints": {
            "visualSweepTouched": False,
            "engineTouched": False,
            "gymTouched": False,
        },
    }


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="M-fid 텍스트층·픽스처 고도화")
    parser.add_argument(
        "--out-root",
        type=Path,
        default=None,
        help="산출 루트 (기본: tools/fidelity_compare)",
    )
    parser.add_argument("--json", action="store_true", help="요약을 stdout JSON 으로")
    return parser


def main(argv: list[str] | None = None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    parser = build_parser()
    args = parser.parse_args(argv)
    out = args.out_root.resolve() if args.out_root else HERE
    summary = run(out)
    if args.json:
        print(json.dumps(summary, ensure_ascii=False, indent=2))
    else:
        print(
            f"M-fid 산출 {len(summary['written'])}파일 · "
            f"cases={summary['caseCount']} paths={summary['pathCount']}"
        )
        for rel in summary["written"]:
            print(f"  {rel}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
