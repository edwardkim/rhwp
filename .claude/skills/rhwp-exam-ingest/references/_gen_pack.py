#!/usr/bin/env python3
"""Generate fixtures, examples, and the intent-matrix chapter.

Run from repo root or this directory:

    python3 .claude/skills/rhwp-exam-ingest/references/_gen_pack.py

Idempotent. Overwrites generated JSON/MD under fixtures/ and examples/,
plus references/19_intent_matrix.md. Hand-written references are left alone.
"""

from __future__ import annotations

import json
from pathlib import Path

ISSUE = 5319
SCHEMA = "1.0"
SKILL = "rhwp-exam-ingest"

HERE = Path(__file__).resolve().parent
SKILL_DIR = HERE.parent
FIXT = SKILL_DIR / "fixtures"
EXAMPLES = SKILL_DIR / "examples"
REF = HERE

CHOICES5 = [
    ("①", "환경 보호의 중요성을 강조하는 글"),
    ("②", "도시 생활의 편리함을 설명하는 글"),
    ("③", "전통 음식의 역사를 소개하는 글"),
    ("④", "최신 기술의 발전 동향을 분석하는 글"),
    ("⑤", "청소년 진로 탐색의 필요성을 논하는 글"),
]


def dump(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    text = json.dumps(obj, ensure_ascii=False, indent=2)
    path.write_text(text + "\n", encoding="utf-8", newline="\n")


def choices(pairs=None):
    pairs = pairs or CHOICES5
    return [{"label": a, "text": b} for a, b in pairs]


def q(
    number: int,
    stem: str,
    *,
    auto_number: bool | None = True,
    passage_ref: str | None = None,
    stem_blocks=None,
    media=None,
    extra_choices=None,
):
    item = {
        "number": number,
        "stem": stem,
        "choices": extra_choices or choices(),
    }
    if auto_number is not None:
        item["auto_number"] = auto_number
    if passage_ref:
        item["passage_ref"] = passage_ref
    if stem_blocks is not None:
        item["stem_blocks"] = stem_blocks
    if media is not None:
        item["media"] = media
    return item


def doc(*, questions, passages=None, header="국어 영역", footer="1/20", form="홀수형"):
    out = {
        "version": "1",
        "page_size": {"width_mm": 210.0, "height_mm": 297.0},
        "default_font": "함초롬바탕",
        "questions": questions,
    }
    if header is not None:
        out["header_text"] = header
    if footer is not None:
        out["footer_text"] = footer
    if form is not None:
        out["form_label"] = form
    if passages is not None:
        out["passages"] = passages
    return out


# ---------------------------------------------------------------------------
# Schema fixtures
# ---------------------------------------------------------------------------

PASSAGE_TEXT = (
    "환경 오염은 현대 사회의 중요한 문제 중 하나이다. 특히 미세먼지로 인한 "
    "공기 질 저하는 우리의 건강에 큰 영향을 미친다. 정부는 배출 규제를 강화하고 "
    "대중교통 이용을 장려하고 있으나, 개인의 실천 없이 구조만 바꾸기는 어렵다."
)

SCIENCE_GRAPH_STEM = "다음 그래프에서 알 수 있는 사실로 적절한 것은?"


def schema_fixtures() -> dict:
    """Return {filename: object} for fixtures/schemas/."""
    files = {}

    files["valid_minimal.json"] = {
        "version": "1",
        "questions": [
            q(
                1,
                "다음 글의 주제로 가장 적절한 것은?",
                auto_number=None,
                stem_blocks=[
                    {"type": "text", "text": "다음 글의 주제로 가장 적절한 것은?"},
                    {"type": "text", "text": PASSAGE_TEXT},
                ],
                media=[],
            )
        ],
    }

    files["valid_structured.json"] = doc(
        passages=[
            {
                "id": "p1-2",
                "blocks": [
                    {"type": "text", "text": "[1~2] 다음 글을 읽고 물음에 답하시오."},
                    {"type": "text", "text": PASSAGE_TEXT},
                ],
            }
        ],
        questions=[
            q(
                1,
                "윗글의 중심 내용으로 가장 적절한 것은?",
                passage_ref="p1-2",
                stem_blocks=[
                    {"type": "text", "text": "윗글의 중심 내용으로 가장 적절한 것은?"}
                ],
                media=[],
            ),
            q(
                2,
                "다음 보기의 설명으로 적절한 것은?",
                passage_ref="p1-2",
                stem_blocks=[
                    {"type": "text", "text": "다음 보기의 설명으로 적절한 것은?"},
                    {
                        "type": "boxed",
                        "title": "<보기>",
                        "blocks": [
                            {
                                "type": "text",
                                "text": "보기 박스는 테두리와 배경이 있는 보조 자료입니다.",
                            }
                        ],
                    },
                ],
                extra_choices=choices(
                    [
                        ("①", "보기 블록은 일반 지문과 구분된다"),
                        ("②", "공유 지문은 매 문제마다 중복 출력된다"),
                        ("③", "머리말과 꼬리말은 무시된다"),
                        ("④", "문항 선택지는 사용할 수 없다"),
                        ("⑤", "스키마 버전은 비워 둔다"),
                    ]
                ),
                media=[],
            ),
        ],
    )

    files["valid_shared_passage.json"] = doc(
        passages=[
            {
                "id": "p1-3",
                "blocks": [
                    {"type": "text", "text": "[1~3] 다음 글을 읽고 물음에 답하시오."},
                    {"type": "text", "text": PASSAGE_TEXT},
                ],
            }
        ],
        questions=[
            q(1, "윗글의 주제로 가장 적절한 것은?", passage_ref="p1-3"),
            q(
                2,
                "밑줄 친 부분의 역할로 적절한 것은?",
                passage_ref="p1-3",
                extra_choices=choices(
                    [
                        ("①", "주장의 근거"),
                        ("②", "반박의 제시"),
                        ("③", "화제 전환"),
                        ("④", "정의의 도입"),
                        ("⑤", "비유의 확대"),
                    ]
                ),
            ),
            q(
                3,
                "글쓴이의 태도로 가장 적절한 것은?",
                passage_ref="p1-3",
                extra_choices=choices(
                    [
                        ("①", "비판적"),
                        ("②", "냉소적"),
                        ("③", "예찬적"),
                        ("④", "해학적"),
                        ("⑤", "관조적"),
                    ]
                ),
            ),
        ],
    )

    files["valid_boxed_bogi.json"] = doc(
        header="국어 영역",
        questions=[
            q(
                12,
                "다음 보기를 참고하여 ㉠에 들어갈 말로 적절한 것은?",
                stem_blocks=[
                    {
                        "type": "text",
                        "text": "다음 보기를 참고하여 ㉠에 들어갈 말로 적절한 것은?",
                    },
                    {
                        "type": "boxed",
                        "title": "<보기>",
                        "blocks": [
                            {"type": "text", "text": "ㄱ. 주어와 서술어가 호응한다."},
                            {"type": "text", "text": "ㄴ. 수식어와 피수식어가 가깝다."},
                            {"type": "text", "text": "ㄷ. 접속어가 문맥에 맞다."},
                        ],
                    },
                ],
                extra_choices=choices(
                    [
                        ("①", "ㄱ"),
                        ("②", "ㄴ"),
                        ("③", "ㄷ"),
                        ("④", "ㄱ, ㄴ"),
                        ("⑤", "ㄱ, ㄴ, ㄷ"),
                    ]
                ),
            )
        ],
    )

    def media_doc(placement: str, number: int = 4) -> dict:
        mid = f"img/q{number}_{placement}.png"
        return doc(
            header="과학 탐구",
            form="짝수형",
            questions=[
                q(
                    number,
                    SCIENCE_GRAPH_STEM,
                    stem_blocks=[
                        {"type": "text", "text": SCIENCE_GRAPH_STEM},
                        {"type": "image", "ref": mid, "placement": placement},
                    ],
                    media=[
                        {
                            "id": mid,
                            "natural_w": 900,
                            "natural_h": 520,
                            "target_w_mm": 90.0,
                            "placement": placement,
                        }
                    ],
                    extra_choices=choices(
                        [
                            ("①", "2010년 이후 매출이 꾸준히 증가했다"),
                            ("②", "2015년에 매출이 가장 높았다"),
                            ("③", "2020년 매출은 2010년의 두 배이다"),
                            ("④", "2018년부터 매출 증가율이 둔화되었다"),
                            ("⑤", "2022년에는 전년 대비 감소했다"),
                        ]
                    ),
                )
            ],
        )

    for plc in ("between", "above", "below", "inline"):
        files[f"valid_media_{plc}.json"] = media_doc(plc)

    files["valid_auto_number_true.json"] = doc(
        questions=[
            q(
                1,
                "다음 글의 주제로 가장 적절한 것은?",
                auto_number=True,
                stem_blocks=[
                    {"type": "text", "text": "다음 글의 주제로 가장 적절한 것은?"}
                ],
            )
        ]
    )

    files["valid_auto_number_false.json"] = doc(
        questions=[
            q(
                2,
                "2. ㉠에 해당하는 내용으로 가장 적절한 것은?",
                auto_number=False,
                stem_blocks=[
                    {
                        "type": "text",
                        "text": "2. ㉠에 해당하는 내용으로 가장 적절한 것은?",
                    }
                ],
            )
        ]
    )

    files["valid_header_footer.json"] = doc(
        header="제2외국어/한문 영역",
        footer="8/20",
        form="홀수형",
        questions=[q(21, "밑줄 친 단어의 의미로 적절한 것은?")],
    )

    files["valid_english_passage.json"] = doc(
        header="영어 영역",
        passages=[
            {
                "id": "p20-22",
                "blocks": [
                    {
                        "type": "text",
                        "text": "[20~22] 다음 글을 읽고 물음에 답하시오.",
                    },
                    {
                        "type": "text",
                        "text": (
                            "When cities expand without planning, public transit "
                            "lags behind housing. Commuters then rely on cars, "
                            "which in turn demand more roads."
                        ),
                    },
                    {
                        "type": "text",
                        "text": (
                            "A compact-growth policy tries to reverse this cycle "
                            "by placing homes near existing rail."
                        ),
                    },
                ],
            }
        ],
        questions=[
            q(
                20,
                "위 글의 제목으로 가장 적절한 것은?",
                passage_ref="p20-22",
                extra_choices=choices(
                    [
                        ("①", "Why Rail Always Fails"),
                        ("②", "Compact Growth and Transit"),
                        ("③", "A History of Highways"),
                        ("④", "Rural Housing Trends"),
                        ("⑤", "Airport Design Basics"),
                    ]
                ),
            ),
            q(
                21,
                "밑줄 친 this cycle 이 가리키는 것은?",
                passage_ref="p20-22",
                extra_choices=choices(
                    [
                        ("①", "주택 밀집 → 녹지 확대"),
                        ("②", "도시 확산 → 자가용 → 도로"),
                        ("③", "철도 투자 → 인구 감소"),
                        ("④", "세금 인상 → 이주민 증가"),
                        ("⑤", "항만 개발 → 어업 쇠퇴"),
                    ]
                ),
            ),
        ],
    )

    files["valid_math_as_images.json"] = doc(
        header="수학 영역",
        form=None,
        questions=[
            q(
                15,
                "다음 정적분의 값은?",
                stem_blocks=[
                    {"type": "text", "text": "다음 정적분의 값은?"},
                    {
                        "type": "image",
                        "ref": "img/q15_integral.png",
                        "placement": "between",
                    },
                ],
                media=[
                    {
                        "id": "img/q15_integral.png",
                        "natural_w": 640,
                        "natural_h": 180,
                        "target_w_mm": 70.0,
                        "placement": "between",
                    }
                ],
                extra_choices=choices(
                    [
                        ("①", "0"),
                        ("②", "1"),
                        ("③", "2"),
                        ("④", "e"),
                        ("⑤", "π"),
                    ]
                ),
            )
        ],
    )

    files["valid_table_as_picture.json"] = doc(
        header="사회탐구",
        questions=[
            q(
                7,
                "다음 표에 대한 분석으로 옳은 것은?",
                stem_blocks=[
                    {"type": "text", "text": "다음 표에 대한 분석으로 옳은 것은?"},
                    {
                        "type": "image",
                        "ref": "img/q7_table.png",
                        "placement": "between",
                    },
                ],
                media=[
                    {
                        "id": "img/q7_table.png",
                        "natural_w": 1100,
                        "natural_h": 420,
                        "target_w_mm": 120.0,
                        "placement": "between",
                    }
                ],
            )
        ],
    )

    files["valid_four_choice.json"] = doc(
        header="자격 검정",
        form=None,
        footer=None,
        questions=[
            q(
                1,
                "다음 중 rel=nofollow 속성의 설명으로 옳은 것은?",
                extra_choices=choices(
                    [
                        ("①", "검색엔진이 링크를 따라가지 않는다"),
                        ("②", "새 탭에서 연다"),
                        ("③", "스타일을 제거한다"),
                        ("④", "폼을 전송한다"),
                    ]
                ),
            )
        ],
    )

    # A 30-question mock exam (text only) — real stems, used as size + contract.
    mock_stems = [
        "윗글의 주제로 가장 적절한 것은?",
        "밑줄 친 ㉠과 바꾸어 쓰기에 적절한 것은?",
        "글쓴이의 태도로 가장 적절한 것은?",
        "다음 문장이 들어가기에 가장 적절한 곳은?",
        "빈칸에 들어갈 말로 가장 적절한 것은?",
        "다음 중 사실과 다른 것은?",
        "위 그래프에서 알 수 없는 것은?",
        "실험 결과로 타당하게 추론한 것은?",
        "다음 중 화학 반응이 아닌 것은?",
        "뉴턴의 운동 법칙에 대한 설명으로 옳은 것은?",
        "다음 자료에 대한 분석으로 옳은 것은?",
        "기본권 제한의 원칙으로 옳은 것은?",
        "수요와 공급에 대한 설명으로 옳은 것은?",
        "다음 지도에 대한 설명으로 옳은 것은?",
        "조선 후기 사회 변화에 대한 설명으로 옳은 것은?",
        "다음 정적분의 값은?",
        "수열의 극한에 대한 설명으로 옳은 것은?",
        "다음 중 합성함수의 미분으로 옳은 것은?",
        "확률의 덧셈정리에 대한 설명으로 옳은 것은?",
        "다음 대화의 목적으로 가장 적절한 것은?",
        "밑줄 친 부분이 의미하는 것은?",
        "위 글의 제목으로 가장 적절한 것은?",
        "다음 중 어법상 틀린 것은?",
        "빈칸에 들어갈 말로 가장 적절한 것은?",
        "주어진 문장이 들어가기에 가장 적절한 곳은?",
        "다음 도표의 내용과 일치하지 않는 것은?",
        "다음 글의 요지로 가장 적절한 것은?",
        "밑줄 친 단어의 의미로 적절한 것은?",
        "글의 흐름으로 보아 적절하지 않은 문장은?",
        "다음 중 필자의 주장으로 가장 적절한 것은?",
    ]
    mock_qs = []
    for i, stem in enumerate(mock_stems, start=1):
        mock_qs.append(
            q(
                i,
                stem,
                auto_number=True,
                stem_blocks=[{"type": "text", "text": stem}],
                media=[],
            )
        )
    files["valid_mock_30.json"] = doc(
        header="전국연합학력평가",
        footer="1/8",
        form="홀수형",
        questions=mock_qs,
    )

    # invalids
    files["invalid_missing_version.json"] = {
        "questions": [q(1, "버전 없음", auto_number=None)]
    }
    files["invalid_bad_version.json"] = {
        "version": "2",
        "questions": [q(1, "버전 2는 없음", auto_number=None)],
    }
    files["invalid_missing_questions.json"] = {
        "version": "1",
        "page_size": {"width_mm": 210.0, "height_mm": 297.0},
    }
    files["invalid_unknown_field.json"] = {
        "version": "1",
        "answer_key": [1, 2, 3],
        "questions": [q(1, "미지 필드", auto_number=None)],
    }
    files["invalid_bad_placement.json"] = {
        "version": "1",
        "questions": [
            q(
                1,
                "잘못된 placement",
                stem_blocks=[
                    {"type": "text", "text": "잘못된 placement"},
                    {
                        "type": "image",
                        "ref": "img/a.png",
                        "placement": "beside",
                    },
                ],
                media=[
                    {
                        "id": "img/a.png",
                        "natural_w": 10,
                        "natural_h": 10,
                        "placement": "beside",
                    }
                ],
            )
        ],
    }
    files["invalid_auto_number_type.json"] = {
        "version": "1",
        "questions": [
            {
                "number": 1,
                "stem": "타입 오류",
                "auto_number": "yes",
                "choices": choices(),
            }
        ],
    }
    files["invalid_boxed_text_field.json"] = {
        "version": "1",
        "questions": [
            {
                "number": 1,
                "stem": "boxed.text 사고",
                "stem_blocks": [
                    {"type": "boxed", "text": "소속: 성명:"},
                ],
                "choices": choices(),
            }
        ],
    }
    files["invalid_unknown_block_type.json"] = {
        "version": "1",
        "questions": [
            {
                "number": 1,
                "stem": "latex 블록은 없다",
                "stem_blocks": [{"type": "latex", "text": "\\int x dx"}],
                "choices": choices(),
            }
        ],
    }
    files["invalid_text_with_placement.json"] = {
        "version": "1",
        "questions": [
            {
                "number": 1,
                "stem": "text 에 placement",
                "stem_blocks": [
                    {
                        "type": "text",
                        "text": "안 됨",
                        "placement": "between",
                    }
                ],
                "choices": choices(),
            }
        ],
    }
    files["invalid_media_natural_zero.json"] = {
        "version": "1",
        "questions": [
            q(
                1,
                "natural_w 0",
                media=[
                    {
                        "id": "img/a.png",
                        "natural_w": 0,
                        "natural_h": 10,
                    }
                ],
            )
        ],
    }
    files["invalid_question_number_zero.json"] = {
        "version": "1",
        "questions": [q(0, "번호 0은 최소 1", auto_number=None)],
    }
    files["invalid_choice_missing_label.json"] = {
        "version": "1",
        "questions": [
            {
                "number": 1,
                "stem": "라벨 없음",
                "choices": [{"text": "만 있음"}],
            }
        ],
    }

    return files


# ---------------------------------------------------------------------------
# Envelopes
# ---------------------------------------------------------------------------


def envelopes() -> dict:
    files = {}
    files["check_deps_ok.json"] = {
        "schemaVersion": SCHEMA,
        "helper": "check_deps.sh",
        "ok": True,
        "rhwp": "./target/release/rhwp",
        "imagemagick": "magick",
        "pythonDocx": True,
        "missingRequired": [],
        "missingOptional": [],
        "envelopes": [],
    }
    files["check_deps_miss_rhwp.json"] = {
        "schemaVersion": SCHEMA,
        "helper": "check_deps.sh",
        "ok": False,
        "rhwp": None,
        "imagemagick": "magick",
        "pythonDocx": True,
        "missingRequired": ["rhwp"],
        "missingOptional": [],
        "envelopes": [
            {
                "code": "DEP_MISS_RHWP",
                "severity": "required",
                "tool": "rhwp",
                "exit": 1,
                "hint": "cargo build --release 또는 cargo run --bin rhwp",
                "blocks": ["build-from-ingest"],
            }
        ],
    }
    files["check_deps_miss_imagemagick.json"] = {
        "schemaVersion": SCHEMA,
        "helper": "check_deps.sh",
        "ok": False,
        "rhwp": "./target/release/rhwp",
        "imagemagick": None,
        "pythonDocx": True,
        "missingRequired": ["imagemagick"],
        "missingOptional": [],
        "envelopes": [
            {
                "code": "DEP_MISS_IMAGEMAGICK",
                "severity": "required",
                "tool": "magick|convert",
                "exit": 1,
                "hint": "brew install imagemagick 또는 apt install imagemagick",
                "blocks": ["crop_image.sh", "pdf_to_pngs.sh(magick fallback)"],
            }
        ],
    }
    files["check_deps_miss_poppler.json"] = {
        "schemaVersion": SCHEMA,
        "helper": "check_deps.sh",
        "ok": True,
        "rhwp": "./target/release/rhwp",
        "imagemagick": "magick",
        "pythonDocx": True,
        "missingRequired": [],
        "missingOptional": ["pdftoppm"],
        "envelopes": [
            {
                "code": "DEP_MISS_POPPLER",
                "severity": "pdf_input",
                "tool": "pdftoppm",
                "exit": 0,
                "hint": "brew install poppler 또는 apt install poppler-utils. magick fallback 가능",
                "blocks": ["pdf_to_pngs.sh (preferred)"],
            }
        ],
    }
    files["check_deps_miss_python_docx.json"] = {
        "schemaVersion": SCHEMA,
        "helper": "check_deps.sh",
        "ok": True,
        "rhwp": "./target/release/rhwp",
        "imagemagick": "magick",
        "pythonDocx": False,
        "missingRequired": [],
        "missingOptional": ["python-docx"],
        "envelopes": [
            {
                "code": "DEP_MISS_PYTHON_DOCX",
                "severity": "docx_input_soft",
                "tool": "python-docx",
                "exit": 0,
                "hint": "pip install python-docx. 없어도 zip+정규식 fallback",
                "blocks": [],
                "fallback": "extract_docx.py zip regex",
            }
        ],
    }
    files["check_deps_miss_python3.json"] = {
        "schemaVersion": SCHEMA,
        "helper": "check_deps.sh",
        "ok": True,
        "rhwp": "./target/release/rhwp",
        "imagemagick": "magick",
        "pythonDocx": False,
        "missingRequired": [],
        "missingOptional": ["python3", "python-docx"],
        "envelopes": [
            {
                "code": "DEP_MISS_PYTHON3",
                "severity": "docx_input",
                "tool": "python3",
                "exit": 0,
                "hint": "DOCX 입력이면 python3 필요",
                "blocks": ["extract_docx.py"],
            }
        ],
    }
    files["pdf_ok_dry_run.json"] = {
        "schemaVersion": SCHEMA,
        "helper": "pdf_to_pngs.sh",
        "ok": True,
        "code": "PDF_OK",
        "dryRun": True,
        "engine": "pdftoppm",
        "planned": "pdftoppm -r 300 -png exam.pdf /tmp/out/page -f 1",
        "input": "exam.pdf",
        "outDir": "/tmp/out",
        "dpi": 300,
        "pagePattern": "page_%03d.png",
    }
    files["pdf_src_missing.json"] = {
        "schemaVersion": SCHEMA,
        "helper": "pdf_to_pngs.sh",
        "ok": False,
        "code": "PDF_SRC_MISSING",
        "message": "오류: 입력 PDF가 존재하지 않습니다: /no/such.pdf",
        "input": "/no/such.pdf",
        "outDir": "/tmp/out",
        "dpi": "300",
        "dryRun": True,
    }
    files["pdf_miss_tools.json"] = {
        "schemaVersion": SCHEMA,
        "helper": "pdf_to_pngs.sh",
        "ok": False,
        "code": "PDF_MISS_TOOLS",
        "message": "오류: pdftoppm / magick / convert 중 하나가 필요합니다 (poppler-utils 또는 ImageMagick)",
    }
    files["pdf_dpi_range.json"] = {
        "schemaVersion": SCHEMA,
        "helper": "pdf_to_pngs.sh",
        "ok": False,
        "code": "PDF_DPI_RANGE",
        "message": "오류: DPI 는 72–600 정수여야 합니다 (got 30)",
    }
    files["crop_ok_dry_run.json"] = {
        "schemaVersion": SCHEMA,
        "helper": "crop_image.sh",
        "ok": True,
        "code": "CROP_OK",
        "dryRun": True,
        "engine": "magick",
        "planned": "magick page.png -crop 640x360+120+400 +repage img/q1.png",
        "src": "page.png",
        "bbox": {"x": 120, "y": 400, "w": 640, "h": 360},
        "out": "img/q1.png",
    }
    files["crop_src_missing.json"] = {
        "schemaVersion": SCHEMA,
        "helper": "crop_image.sh",
        "ok": False,
        "code": "CROP_SRC_MISSING",
        "message": "오류: source 이미지가 없습니다: missing.png",
    }
    files["crop_miss_imagemagick.json"] = {
        "schemaVersion": SCHEMA,
        "helper": "crop_image.sh",
        "ok": False,
        "code": "CROP_MISS_IMAGEMAGICK",
        "message": "오류: ImageMagick (magick 또는 convert)이 필요합니다",
    }
    files["crop_bbox_not_uint.json"] = {
        "schemaVersion": SCHEMA,
        "helper": "crop_image.sh",
        "ok": False,
        "code": "CROP_BBOX_NOT_UINT",
        "message": "오류: bbox 는 10진 정수여야 합니다 (x=10.5 y=20 w=100 h=80)",
    }
    files["crop_bbox_empty.json"] = {
        "schemaVersion": SCHEMA,
        "helper": "crop_image.sh",
        "ok": False,
        "code": "CROP_BBOX_EMPTY",
        "message": "오류: bbox 폭/높이는 1 이상이어야 합니다 (w=0 h=80)",
    }
    files["crop_no_output.json"] = {
        "schemaVersion": SCHEMA,
        "helper": "crop_image.sh",
        "ok": False,
        "code": "CROP_NO_OUTPUT",
        "message": "오류: 자르기 실패 — 출력 파일이 생성되지 않음",
    }
    files["docx_ok_dry_run.json"] = {
        "schemaVersion": SCHEMA,
        "helper": "extract_docx.py",
        "ok": True,
        "code": "DOCX_OK",
        "dryRun": True,
        "engine": "python-docx",
        "pythonDocx": True,
        "fallback": None,
        "planned": ["/tmp/out/text.txt", "/tmp/out/img/*"],
    }
    files["docx_fallback.json"] = {
        "schemaVersion": SCHEMA,
        "helper": "extract_docx.py",
        "ok": True,
        "code": "DOCX_OK",
        "dryRun": True,
        "engine": "zip-regex-fallback",
        "pythonDocx": False,
        "fallback": "zip+<w:t> regex",
    }
    files["docx_src_missing.json"] = {
        "schemaVersion": SCHEMA,
        "helper": "extract_docx.py",
        "ok": False,
        "code": "DOCX_SRC_MISSING",
        "message": "오류: 입력 파일이 없습니다: missing.docx",
    }
    files["build_missing_o.json"] = {
        "schemaVersion": SCHEMA,
        "command": "build-from-ingest",
        "ok": False,
        "code": "BUILD_MISSING_O",
        "exit": 2,
        "stderrContains": "오류: -o <출력 경로> 가 누락되었습니다",
        "note": "기존 CLI 사용법. 새 플래그 아님",
    }
    files["build_unknown_field.json"] = {
        "schemaVersion": SCHEMA,
        "command": "build-from-ingest",
        "ok": False,
        "code": "BUILD_UNKNOWN_FIELD",
        "exit": 1,
        "hint": "deny_unknown_fields (#3358). answer/latex/table 키 제거",
    }
    return files


# ---------------------------------------------------------------------------
# Matrices, helpers, transcripts, catalog
# ---------------------------------------------------------------------------

PLACEMENTS = ("between", "above", "below", "inline")


def matrices() -> dict:
    return {
        "placement.json": {
            "schemaVersion": SCHEMA,
            "issue": ISSUE,
            "enum": list(PLACEMENTS),
            "default": "between",
            "rows": [
                {
                    "id": "P-between",
                    "value": "between",
                    "when": "발문과 선택지 사이",
                    "stemOrder": ["text", "image"],
                    "common": True,
                },
                {
                    "id": "P-above",
                    "value": "above",
                    "when": "발문보다 위",
                    "stemOrder": ["image", "text"],
                    "common": False,
                },
                {
                    "id": "P-below",
                    "value": "below",
                    "when": "선택지 다음",
                    "stemOrder": ["text", "choices", "image"],
                    "common": False,
                },
                {
                    "id": "P-inline",
                    "value": "inline",
                    "when": "문장 중간",
                    "stemOrder": ["text+image"],
                    "common": False,
                    "limit": "Picture inline #182",
                },
            ],
        },
        "auto_number.json": {
            "schemaVersion": SCHEMA,
            "issue": ISSUE,
            "default": True,
            "rows": [
                {
                    "id": "AN-true",
                    "auto_number": True,
                    "stem": "다음 글의 주제는?",
                    "printed": "1. 다음 글의 주제는?",
                },
                {
                    "id": "AN-false",
                    "auto_number": False,
                    "stem": "2. ㉠에 해당하는 것은?",
                    "printed": "2. ㉠에 해당하는 것은?",
                },
                {
                    "id": "AN-omit",
                    "auto_number": None,
                    "stem": "빈칸에 들어갈 말",
                    "printed": "3. 빈칸에 들어갈 말",
                    "note": "미지정 시 true",
                },
                {
                    "id": "AN-dup-risk",
                    "auto_number": True,
                    "stem": "3. 밑줄 친",
                    "printed": "3. 3. 밑줄 친",
                    "avoid": True,
                },
            ],
        },
        "input_kind.json": {
            "schemaVersion": SCHEMA,
            "issue": ISSUE,
            "kinds": [
                {
                    "kind": "pdf",
                    "helper": "pdf_to_pngs.sh",
                    "firstOutput": "page_001.png",
                },
                {
                    "kind": "png",
                    "helper": None,
                    "firstOutput": "passthrough",
                },
                {
                    "kind": "jpg",
                    "helper": None,
                    "firstOutput": "passthrough",
                },
                {
                    "kind": "md",
                    "helper": None,
                    "firstOutput": "![alt](path) → media",
                },
                {
                    "kind": "docx",
                    "helper": "extract_docx.py",
                    "firstOutput": "text.txt + img/",
                },
            ],
        },
        "exit_codes.json": {
            "schemaVersion": SCHEMA,
            "issue": ISSUE,
            "rhwp": {"0": "성공", "1": "런타임", "2": "사용법", "3": "verify (해당 없음)"},
            "pdf_to_pngs": {0: "PDF_OK", 1: "PDF_ARGS|PDF_SRC_MISSING", 2: "PDF_MISS_TOOLS", 4: "PDF_DPI_RANGE"},
            "crop_image": {
                0: "CROP_OK",
                1: "CROP_ARGS|CROP_SRC_MISSING",
                2: "CROP_MISS_IMAGEMAGICK",
                3: "CROP_NO_OUTPUT",
                4: "CROP_BBOX_NOT_UINT|CROP_BBOX_EMPTY",
            },
            "extract_docx": {0: "DOCX_OK", 1: "DOCX_ARGS|DOCX_SRC_MISSING", 2: "DOCX_ARGS"},
            "check_deps": {0: "필수 충족", 1: "필수 누락", 2: "사용법"},
        },
        "known_limits.json": {
            "schemaVersion": SCHEMA,
            "issue": ISSUE,
            "limits": [
                {
                    "id": "L-picture",
                    "title": "Picture 직렬화",
                    "issue": 182,
                    "action": "텍스트 우선, 고지",
                    "not": "writer 수정",
                },
                {
                    "id": "L-equation",
                    "title": "수식은 이미지",
                    "action": "crop → image 블록",
                    "not": "latex 필드 발명",
                },
                {
                    "id": "L-table",
                    "title": "표는 그림",
                    "action": "표 bbox crop",
                    "not": "Table IR 발명",
                },
            ],
        },
        "stop_rules.json": {
            "schemaVersion": SCHEMA,
            "issue": ISSUE,
            "rules": [
                {"id": f"F{n:02d}"}
                for n in range(1, 20)
            ],
        },
    }


def helper_contracts() -> dict:
    return {
        "crop_bbox_contract.json": {
            "schemaVersion": SCHEMA,
            "helper": "crop_image.sh",
            "usage": "crop_image.sh [--json] [--dry-run] <source.png> <x> <y> <w> <h> <out.png>",
            "coord": "pixel top-left",
            "types": "decimal integers",
            "min_wh": 1,
            "engine": "magick|convert -crop WxH+X+Y +repage",
            "dryRunValidates": ["uint bbox", "src exists", "engine present"],
            "codes": [
                "CROP_OK",
                "CROP_ARGS",
                "CROP_SRC_MISSING",
                "CROP_MISS_IMAGEMAGICK",
                "CROP_NO_OUTPUT",
                "CROP_BBOX_NOT_UINT",
                "CROP_BBOX_EMPTY",
            ],
        },
        "pdf_to_pngs_contract.json": {
            "schemaVersion": SCHEMA,
            "helper": "pdf_to_pngs.sh",
            "usage": "pdf_to_pngs.sh [--json] [--dry-run] <input.pdf> <out_dir> [<dpi>]",
            "dpiDefault": 300,
            "dpiRange": [72, 600],
            "pagePattern": "page_%03d.png",
            "engines": ["pdftoppm", "magick", "convert"],
            "codes": [
                "PDF_OK",
                "PDF_ARGS",
                "PDF_SRC_MISSING",
                "PDF_MISS_TOOLS",
                "PDF_DPI_RANGE",
            ],
        },
        "extract_docx_contract.json": {
            "schemaVersion": SCHEMA,
            "helper": "extract_docx.py",
            "usage": "extract_docx.py [--json] [--dry-run] <input.docx> <out_dir>",
            "outputs": ["text.txt", "img/"],
            "fallback": "zip+<w:t> regex",
            "fallbackIsFailure": False,
            "codes": ["DOCX_OK", "DOCX_ARGS", "DOCX_SRC_MISSING"],
        },
        "check_deps_matrix.json": {
            "schemaVersion": SCHEMA,
            "helper": "check_deps.sh",
            "usage": "check_deps.sh [--json]",
            "required": ["rhwp", "imagemagick"],
            "optional": ["pdftoppm", "pdftotext", "python3", "python-docx"],
            "codes": [
                "DEP_MISS_RHWP",
                "DEP_MISS_IMAGEMAGICK",
                "DEP_MISS_POPPLER",
                "DEP_MISS_PDFTOTEXT",
                "DEP_MISS_PYTHON3",
                "DEP_MISS_PYTHON_DOCX",
            ],
        },
        "image_passthrough.json": {
            "schemaVersion": SCHEMA,
            "helper": None,
            "extensions": [".png", ".jpg", ".jpeg", ".webp"],
            "resizeHelper": False,
            "exifAutoOrientHelper": False,
            "pageOrder": "numeric, confirm with user",
        },
        "md_image_refs.json": {
            "schemaVersion": SCHEMA,
            "patterns": ["![alt](path)", "<img src>", "[id]: path"],
            "base": "markdown file directory",
            "remoteDownload": False,
            "missingPathStop": "F07",
        },
    }


def transcripts() -> dict:
    steps_t01 = [
        {"cmd": "check_deps.sh --json", "expect": "ok true"},
        {"cmd": "pdf_to_pngs.sh 2024_수능_국어.pdf $TMP 300", "expect": "pages>=1"},
        {"cmd": "Read page_001.png", "expect": "header/form_label"},
        {"cmd": "Write ingest.json", "expect": "passages p1-3"},
        {
            "cmd": "rhwp build-from-ingest $TMP/ingest.json -o output/exam/2024_국어.hwpx",
            "expect": "exit 0",
        },
        {"cmd": "rhwp export-text …", "expect": "지문 1회, 번호 중복 없음"},
    ]
    return {
        "pdf_to_hwpx.json": {
            "id": "T01",
            "title": "PDF 수능 국어",
            "issue": ISSUE,
            "notGym": True,
            "steps": steps_t01,
        },
        "png_graph.json": {
            "id": "T02",
            "title": "PNG 그래프",
            "issue": ISSUE,
            "notGym": True,
            "steps": [
                {"cmd": "Read desk.jpg", "expect": "bbox 240,410,980,620"},
                {"cmd": "crop_image.sh desk.jpg 240 410 980 620 $MEDIA/img/q1.png", "expect": "CROP_OK"},
                {
                    "cmd": "rhwp build-from-ingest ingest.json --media-dir $MEDIA -o out.hwpx",
                    "expect": "exit 0",
                },
            ],
            "limit": "L-picture",
        },
        "md_to_hwpx.json": {
            "id": "T03",
            "title": "MD + 이미지 ref",
            "issue": ISSUE,
            "notGym": True,
            "auto_number": False,
            "steps": [
                {"cmd": "Read quiz.md", "expect": "![plot](figures/pm10.png)"},
                {"cmd": "copy figures/pm10.png → $MEDIA/img/pm10.png", "expect": "exists"},
                {"cmd": "build-from-ingest --media-dir", "expect": "2. 한 번"},
            ],
        },
        "docx_fallback.json": {
            "id": "T04",
            "title": "DOCX zip fallback",
            "issue": ISSUE,
            "notGym": True,
            "steps": [
                {"cmd": "check_deps.sh --json", "expect": "DEP_MISS_PYTHON_DOCX ok true"},
                {"cmd": "extract_docx.py 학원.docx $TMP", "expect": "engine zip-regex-fallback"},
            ],
        },
        "dep_failure_pdf.json": {
            "id": "T06",
            "title": "PDF 도구 전무",
            "issue": ISSUE,
            "notGym": True,
            "stop": "F03",
            "steps": [
                {"cmd": "pdf_to_pngs.sh exam.pdf $TMP", "expect": "PDF_MISS_TOOLS exit 2"},
            ],
        },
        "bbox_retry.json": {
            "id": "T08",
            "title": "bbox 소수 재시도",
            "issue": ISSUE,
            "notGym": True,
            "steps": [
                {"cmd": "crop … 120.4 …", "expect": "CROP_BBOX_NOT_UINT exit 4"},
                {"cmd": "crop … 120 … --dry-run", "expect": "CROP_OK"},
            ],
        },
        "auto_number_fix.json": {
            "id": "T09",
            "title": "중복 번호 수정",
            "issue": ISSUE,
            "notGym": True,
            "steps": [
                {"cmd": "export-text", "expect": "3. 3. 밑줄 친"},
                {"cmd": "auto_number false 또는 stem 수정 후 rebuild", "expect": "3. 한 번"},
            ],
        },
        "equation_images.json": {
            "id": "T10",
            "title": "수식 이미지",
            "issue": ISSUE,
            "notGym": True,
            "limit": "L-equation",
            "steps": [
                {"cmd": "crop 적분 bbox", "expect": "image 블록"},
                {"cmd": "고지 Equation IR 없음", "expect": "F16"},
            ],
        },
    }


INTENTS_RAW = [
    # (id, utterance, action, reference, stop)
    ("I001", "이 PDF를 HWPX로 만들어줘", "pdf_to_pngs.sh → Vision → build-from-ingest", "02_pdf_to_pngs.md", "F05"),
    ("I002", "수능 국어 PDF 한글 문서로", "pdf_to_pngs.sh → passages → build-from-ingest", "07_passages_questions.md", "F05"),
    ("I003", "/rhwp-exam-ingest exam.pdf", "같은 사다리", "00_tree.md", "F01"),
    ("I004", "이 스캔 사진 한 장 변환", "image passthrough", "04_image_passthrough.md", "F06"),
    ("I005", "JPG 여러 장 페이지 순서대로", "numeric sort + confirm", "04_image_passthrough.md", "F06"),
    ("I006", "quiz.md 를 시험지로", "MD + ![alt](path)", "05_md_image_refs.md", "F07"),
    ("I007", "학원.docx 변환", "extract_docx.py", "03_extract_docx.md", "F08"),
    ("I008", "의존성 있니", "check_deps.sh --json", "13_check_deps.md", "F01"),
    ("I009", "poppler 없이 PDF 가능하냐", "magick fallback", "02_pdf_to_pngs.md", "F03"),
    ("I010", "python-docx 없는데 DOCX", "zip fallback, 중단 금지", "03_extract_docx.md", "F04"),
    ("I011", "그래프를 지문과 선택지 사이에", "placement between", "09_media_placement.md", "F14"),
    ("I012", "그림이 발문보다 위", "placement above + 블록 순서", "09_media_placement.md", "F14"),
    ("I013", "그림이 선택지 다음", "placement below", "09_media_placement.md", "F14"),
    ("I014", "문장 가운데 작은 도형", "placement inline, #182 고지", "09_media_placement.md", "F18"),
    ("I015", "공유 지문 1~3", "passages + passage_ref", "07_passages_questions.md", "F19"),
    ("I016", "<보기> 상자 살려줘", "stem_blocks boxed", "08_stem_blocks_boxed.md", "F12"),
    ("I017", "번호가 이미 지문에 있어", "auto_number false", "10_auto_number.md", "F13"),
    ("I018", "번호 자동으로 붙여", "auto_number true, stem 에 번호 없음", "10_auto_number.md", "F13"),
    ("I019", "이 그래프만 잘라서 넣어", "crop_image.sh bbox", "11_crop_bbox.md", "F14"),
    ("I020", "수식도 한글로 편집 가능하게", "거절, 이미지로. Equation IR 없음", "15_known_limits.md", "F16"),
    ("I021", "표를 한글 표로", "거절, Picture. Table IR 없음", "15_known_limits.md", "F17"),
    ("I022", "OCR 엔진 깔아줘", "거절, Vision 사용", "16_pitfalls.md", "F19"),
    ("I023", "exam-from-pdf 명령 있어?", "없다. 발명 금지", "00_tree.md", "F19"),
    ("I024", "-o 빼도 돼?", "안 됨. -o 필수", "12_build_from_ingest.md", "F15"),
    ("I025", "media-dir 없이 그림 넣기", "불가. --media-dir", "12_build_from_ingest.md", "F15"),
    ("I026", "정답도 JSON 에 넣어", "answer 키 금지 deny_unknown", "06_ingest_schema_v1.md", "F11"),
    ("I027", "흐린 스캔인데", "DPI 400 또는 원본 재요청", "02_pdf_to_pngs.md", "F10"),
    ("I028", "한 페이지에 문제 40개", "사분면 분할 Vision", "00_tree.md", "F09"),
    ("I029", "산출 확인은?", "export-text / dump / unzip -l", "18_verify_gate.md", "F19"),
    ("I030", "한컴에서 그림이 안 보여", "#182 한계 고지. writer 수정 금지", "15_known_limits.md", "F18"),
]


def more_intents():
    """Expand to 80+ real utterances covering subjects and failure paths."""
    extra = []
    n = 31
    subjects = [
        ("국어", "지문", "07_passages_questions.md"),
        ("영어", "장문", "07_passages_questions.md"),
        ("수학", "적분", "15_known_limits.md"),
        ("과학", "그래프", "09_media_placement.md"),
        ("사회", "통계 표", "15_known_limits.md"),
        ("한국사", "연표", "04_image_passthrough.md"),
        ("한문", "구결", "10_auto_number.md"),
        ("제2외국어", "대화문", "05_md_image_refs.md"),
    ]
    verbs = [
        "HWPX로",
        "한글 시험지로",
        "ingest 해줘",
        "변환해 줘",
        "만들어 줘",
    ]
    for subj, thing, ref in subjects:
        for verb in verbs:
            extra.append(
                (
                    f"I{n:03d}",
                    f"{subj} {thing} {verb}",
                    "사다리 동일, 과목은 Vision 힌트일 뿐",
                    ref,
                    "F19",
                )
            )
            n += 1
    failures = [
        ("PDF 경로가 틀려", "PDF_SRC_MISSING", "02_pdf_to_pngs.md", "F05"),
        ("이미지 경로가 틀려", "CROP_SRC_MISSING", "11_crop_bbox.md", "F14"),
        ("DOCX 가 없어", "DOCX_SRC_MISSING", "03_extract_docx.md", "F08"),
        ("DPI 30으로", "PDF_DPI_RANGE", "02_pdf_to_pngs.md", "F05"),
        ("bbox 에 12.7 썼어", "CROP_BBOX_NOT_UINT", "11_crop_bbox.md", "F14"),
        ("폭 0으로 crop", "CROP_BBOX_EMPTY", "11_crop_bbox.md", "F14"),
        ("ImageMagick 없는데 crop", "CROP_MISS_IMAGEMAGICK", "13_check_deps.md", "F02"),
        ("rhwp 바이너리 없는데", "DEP_MISS_RHWP", "13_check_deps.md", "F01"),
        ("boxed 에 text 필드", "F12 rebuild", "08_stem_blocks_boxed.md", "F12"),
        ("version 2 로", "스키마 const 1", "06_ingest_schema_v1.md", "F11"),
    ]
    for utt, action, ref, stop in failures:
        extra.append((f"I{n:03d}", utt, action, ref, stop))
        n += 1
    # MD variants
    for i, name in enumerate(
        [
            "상대 경로 이미지",
            "img 태그",
            "참조 링크 이미지",
            "원격 URL 이미지",
            "깨진 이미지 경로",
        ],
        start=1,
    ):
        extra.append(
            (
                f"I{n:03d}",
                f"MD 에서 {name}",
                "05장 규약. URL 은 다운로드 금지" if "URL" in name else "경로 확인",
                "05_md_image_refs.md",
                "F07",
            )
        )
        n += 1
    return extra


def all_intents():
    rows = []
    for item in INTENTS_RAW + more_intents():
        iid, utt, action, ref, stop = item
        rows.append(
            {
                "id": iid,
                "utterance": utt,
                "command": action,
                "reference": ref,
                "stop": stop,
                "notGym": True,
            }
        )
    return rows


def catalog(schema_files, env_files, trans_files) -> dict:
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "skill": SKILL,
        "notGym": True,
        "noNewCli": True,
        "noNewExamPaperLogic": True,
        "existingCommand": "build-from-ingest",
        "helpers": [
            "pdf_to_pngs.sh",
            "extract_docx.py",
            "crop_image.sh",
            "check_deps.sh",
        ],
        "references": [
            "00_tree.md",
            "01_input_normalize.md",
            "02_pdf_to_pngs.md",
            "03_extract_docx.md",
            "04_image_passthrough.md",
            "05_md_image_refs.md",
            "06_ingest_schema_v1.md",
            "07_passages_questions.md",
            "08_stem_blocks_boxed.md",
            "09_media_placement.md",
            "10_auto_number.md",
            "11_crop_bbox.md",
            "12_build_from_ingest.md",
            "13_check_deps.md",
            "14_failure_envelopes.md",
            "15_known_limits.md",
            "16_pitfalls.md",
            "17_sample_transcripts.md",
            "18_verify_gate.md",
            "19_intent_matrix.md",
            "20_exit_codes.md",
            "README.md",
        ],
        "schemas": sorted(schema_files),
        "envelopes": sorted(env_files),
        "transcripts": sorted(trans_files),
        "placements": list(PLACEMENTS),
        "depCodes": [
            "DEP_MISS_RHWP",
            "DEP_MISS_IMAGEMAGICK",
            "DEP_MISS_POPPLER",
            "DEP_MISS_PYTHON_DOCX",
        ],
    }


# ---------------------------------------------------------------------------
# Examples
# ---------------------------------------------------------------------------

EXAMPLES_META = [
    ("01_pdf_suneung.md", "PDF 수능 국어", "pdf_to_pngs.sh", "T01"),
    ("02_png_single_page.md", "PNG 한 장 패스스루", "Read + crop", "T02"),
    ("03_md_with_images.md", "MD + 이미지 ref", "![alt](path)", "T03"),
    ("04_docx_extract.md", "DOCX 추출", "extract_docx.py", "T04"),
    ("05_shared_passage.md", "공유 지문 passages", "passage_ref", "T01"),
    ("06_boxed_bogi.md", "boxed <보기>", "stem_blocks boxed", "F12"),
    ("07_media_between.md", "placement between", "between", "P-between"),
    ("08_media_above.md", "placement above", "above", "P-above"),
    ("09_media_below.md", "placement below", "below", "P-below"),
    ("10_media_inline.md", "placement inline", "inline + #182", "P-inline"),
    ("11_auto_number_true.md", "auto_number true", "prefix 자동", "AN-true"),
    ("12_auto_number_false.md", "auto_number false", "원본 번호 유지", "AN-false"),
    ("13_crop_bbox.md", "bbox crop dry-run", "crop_image.sh --dry-run", "T08"),
    ("14_missing_poppler.md", "poppler 없음", "DEP_MISS_POPPLER / magick", "T05"),
    ("15_missing_imagemagick.md", "ImageMagick 없음", "DEP_MISS_IMAGEMAGICK", "F02"),
    ("16_missing_python_docx.md", "python-docx 없음", "fallback exit 0", "T07"),
    ("17_picture_serialization_limit.md", "Picture #182", "텍스트 우선 고지", "L-picture"),
    ("18_equation_as_image.md", "수식은 이미지", "crop 적분", "L-equation"),
    ("19_table_as_picture.md", "표는 그림", "표 bbox", "L-table"),
    ("20_build_from_ingest.md", "build-from-ingest -o", "--media-dir -o", "F15"),
    ("21_dense_page_split.md", "빽빽한 페이지", "사분면 Vision", "F09"),
    ("22_scan_quality.md", "흐린 스캔", "DPI 400 또는 F10", "F10"),
    ("23_header_footer_form.md", "머리말·홀수형", "header_text form_label", "valid_header_footer"),
    ("24_verify_export_text.md", "export-text 게이트", "중복 번호 검사", "T09"),
]


def example_body(fname, title, action, xref) -> str:
    stem = fname.replace(".md", "")
    lines = [
        f"# 예제 {stem} — {title}",
        "",
        f"이 워크스루는 gym 과제가 아니다. 기존 helper 와 `rhwp build-from-ingest` 만 쓴다.",
        f"교차: `{xref}`. 동작: {action}.",
        "",
        "## 입력",
        "",
        "사용자가 시험지 원본(또는 그 경로)을 준다. 원본은 읽기만 한다.",
        "",
        "## 명령",
        "",
        "```bash",
        "bash .claude/skills/rhwp-exam-ingest/helpers/check_deps.sh --json",
    ]
    if "pdf" in fname or "suneung" in fname or "poppler" in fname or "scan" in fname:
        lines += [
            "bash .claude/skills/rhwp-exam-ingest/helpers/pdf_to_pngs.sh \\",
            "    --json --dry-run \"$PDF\" \"$TMP\" 300",
            "bash .claude/skills/rhwp-exam-ingest/helpers/pdf_to_pngs.sh \\",
            "    \"$PDF\" \"$TMP\" 300",
        ]
    if "docx" in fname or "python_docx" in fname:
        lines += [
            "python3 .claude/skills/rhwp-exam-ingest/helpers/extract_docx.py \\",
            "    --json --dry-run \"$DOCX\" \"$TMP\"",
            "python3 .claude/skills/rhwp-exam-ingest/helpers/extract_docx.py \\",
            "    \"$DOCX\" \"$TMP\"",
        ]
    if "crop" in fname or "media" in fname or "equation" in fname or "table" in fname or "png" in fname:
        lines += [
            "bash .claude/skills/rhwp-exam-ingest/helpers/crop_image.sh \\",
            "    --json --dry-run \"$PAGE\" 180 620 2100 880 \"$MEDIA/img/q1.png\"",
            "bash .claude/skills/rhwp-exam-ingest/helpers/crop_image.sh \\",
            "    \"$PAGE\" 180 620 2100 880 \"$MEDIA/img/q1.png\"",
        ]
    lines += [
        "rhwp build-from-ingest \"$TMP/ingest.json\" --media-dir \"$MEDIA\" -o \"$OUT\"",
        "rhwp export-text \"$OUT\" -o \"$TMP/txt\"",
        "rhwp dump \"$OUT\" > \"$TMP/dump.txt\"",
        "```",
        "",
        "## ingest 요지",
        "",
        f"- 스키마 `version: \"1\"`. 미지 필드 없음.",
        f"- 교차 픽스처/정지를 `{xref}` 로 확인.",
        f"- `auto_number` 정책을 10장에 맞게 고정.",
        "",
        "## 정지",
        "",
        "실패 봉투는 `references/14_failure_envelopes.md`. 성공이 아니면 `-o` 산출을 사용자에게 주지 않는다.",
        "",
        "## 한계",
        "",
        "Picture #182, 수식 이미지, 표 Picture. 새 CLI / exam_paper 수정 없음.",
        "",
    ]
    return "\n".join(lines) + "\n"


def write_intent_md(rows) -> None:
    lines = [
        "# 19 — 발화 → 동작 행렬",
        "",
        "에이전트가 사용자 문장을 기존 helper/CLI 로만 매핑한다.",
        "새 동사를 만들지 않는다. gym 발화가 아니다.",
        "",
        f"행 수: {len(rows)}. 기계본: `fixtures/matrices/intent_matrix.json`.",
        "",
        "| ID | 발화 | 동작 | 장 | 정지 |",
        "| --- | --- | --- | --- | --- |",
    ]
    for r in rows:
        lines.append(
            f"| {r['id']} | {r['utterance']} | {r['command']} | {r['reference']} | {r['stop']} |"
        )
    lines += [
        "",
        "## 읽는 법",
        "",
        "- `command` 열에 `rhwp exam-from-pdf` 가 있으면 발명된 금지 명령이다. 이 표가 틀린 것이다.",
        "- 정지 열의 Fxx 는 SKILL.md 정지 표와 같아야 한다.",
        "- 과목명(국어/수학)은 Vision 힌트일 뿐 다른 파이프라인이 아니다.",
        "",
    ]
    (REF / "19_intent_matrix.md").write_text(
        "\n".join(lines), encoding="utf-8", newline="\n"
    )


def write_examples_readme() -> None:
    lines = [
        "# rhwp-exam-ingest examples",
        "",
        "실사용 워크스루. gym pack 이 아니다. 각 파일은 같은 사다리를 한 장면만 확대한다.",
        "",
        "| 파일 | 장면 |",
        "| --- | --- |",
    ]
    for fname, title, action, xref in EXAMPLES_META:
        lines.append(f"| [{fname}]({fname}) | {title} (`{xref}`) |")
    lines.append("")
    (EXAMPLES / "README.md").write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")


def write_md_sample() -> None:
    body = """# 국어 영역

[1~2] 다음 글을 읽고 물음에 답하시오.

환경 오염은 현대 사회의 중요한 문제 중 하나이다. 특히 미세먼지로 인한
공기 질 저하는 우리의 건강에 큰 영향을 미친다.

## 1. 윗글의 주제로 가장 적절한 것은?

① 환경 보호의 중요성을 강조하는 글
② 도시 생활의 편리함을 설명하는 글
③ 전통 음식의 역사를 소개하는 글
④ 최신 기술의 발전 동향을 분석하는 글
⑤ 청소년 진로 탐색의 필요성을 논하는 글

## 2. 다음 그래프에서 알 수 있는 사실은?

![미세먼지](figures/pm10.png)

① 2010년 이후 증가
② 2015년이 최고
③ 2020년은 2010년의 두 배
④ 2018년부터 둔화
⑤ 2022년 감소
"""
    path = FIXT / "md" / "sample_exam.md"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body, encoding="utf-8", newline="\n")


def main() -> None:
    schemas = schema_fixtures()
    for name, obj in schemas.items():
        dump(FIXT / "schemas" / name, obj)
    dump(FIXT / "schemas" / "catalog.json", {"files": sorted(schemas), "count": len(schemas)})

    envs = envelopes()
    for name, obj in envs.items():
        dump(FIXT / "envelopes" / name, obj)

    for name, obj in matrices().items():
        dump(FIXT / "matrices" / name, obj)

    for name, obj in helper_contracts().items():
        dump(FIXT / "helpers" / name, obj)

    trans = transcripts()
    for name, obj in trans.items():
        dump(FIXT / "transcripts" / name, obj)

    intents = all_intents()
    dump(
        FIXT / "matrices" / "intent_matrix.json",
        {"schemaVersion": SCHEMA, "issue": ISSUE, "count": len(intents), "intents": intents},
    )
    write_intent_md(intents)

    dump(
        FIXT / "catalog.json",
        catalog(list(schemas), list(envs), list(trans)),
    )
    dump(
        FIXT / "skill_index.json",
        {
            "schemaVersion": SCHEMA,
            "issue": ISSUE,
            "skill": SKILL,
            "notGym": True,
            "noNewCli": True,
            "noNewExamPaperLogic": True,
            "forbiddenSkillsTouch": [
                "rhwp-form-fill",
                "rhwp-table-exchange",
                "rhwp-onboarding",
                "rhwp-safe-edit",
                "rhwp-doc-triage",
            ],
            "references": catalog(list(schemas), list(envs), list(trans))["references"],
        },
    )

    EXAMPLES.mkdir(parents=True, exist_ok=True)
    for fname, title, action, xref in EXAMPLES_META:
        (EXAMPLES / fname).write_text(
            example_body(fname, title, action, xref), encoding="utf-8", newline="\n"
        )
    write_examples_readme()
    write_md_sample()

    # loops — multi-step contracts
    dump(
        FIXT / "loops" / "pdf_success.json",
        {
            "id": "loop-pdf",
            "notGym": True,
            "steps": ["check_deps", "pdf_to_pngs", "vision", "ingest", "build", "export-text"],
            "stopOn": ["DEP_MISS_RHWP", "PDF_MISS_TOOLS", "PDF_SRC_MISSING"],
        },
    )
    dump(
        FIXT / "loops" / "media_crop.json",
        {
            "id": "loop-crop",
            "notGym": True,
            "steps": ["vision-bbox", "crop --dry-run", "crop", "build --media-dir"],
            "stopOn": ["CROP_BBOX_NOT_UINT", "CROP_MISS_IMAGEMAGICK", "CROP_SRC_MISSING"],
        },
    )
    dump(
        FIXT / "loops" / "docx_fallback.json",
        {
            "id": "loop-docx",
            "notGym": True,
            "steps": ["check_deps", "extract_docx", "vision-img", "ingest", "build"],
            "pythonDocxMissingIsOk": True,
        },
    )

    print(f"generated schemas={len(schemas)} envelopes={len(envs)} intents={len(intents)}")


if __name__ == "__main__":
    main()
