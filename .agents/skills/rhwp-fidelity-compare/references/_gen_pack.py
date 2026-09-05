#!/usr/bin/env python3
"""Emit rhwp-fidelity-compare fixtures (JSON, TSV, traces, transcripts).

References and examples are hand-written siblings of this script. This
generator only fills fixtures/ so contract tests can lock the machine-
readable surface without inventing a CLI.
"""

from __future__ import annotations

import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
SKILL = HERE.parent
FIXT = SKILL / "fixtures"
ISSUE = 5329
SCHEMA = "1.0"

STOP_RULES = [
    ("F01", "no_independent_pdf", "독립 한컴 PDF 없음", "rhwp-visual-regression 인계"),
    ("F02", "text_only_candidates", "--text-only 후보만", "시트 없이 원장. 확정 금지"),
    ("F03", "worst_first_rank", "report.tsv 상위", "최악 쪽부터 사람 감사"),
    ("F04", "glyph_risk", "PUA/FFFD/□", "글꼴 별칭·경로 먼저"),
    ("F05", "maintainer_verdict", "시트 감사 끝", "유지자 최종 판정"),
    ("F06", "no_new_cli", "gym/새 CLI", "거절"),
    ("F07", "no_rewrite_visual", "visual-regression 재작성", "거절"),
    ("F08", "no_rewrite_bug_hunter", "bug-hunter 재작성", "거절"),
    ("F09", "missing_venv", "venv 없음", "저장소 venv 재생성"),
    ("F10", "missing_chrome", "Chrome 없음", "--text-only 또는 정지"),
    ("F11", "page_count_mismatch", "쪽수 불일치", "후보 기록, 전역 패치 금지"),
    ("F12", "incomplete_run", "run-state incomplete", "누락 쪽 먼저"),
    ("F13", "encrypted_pdf", "암호화 PDF", "정지, 우회 금지"),
    ("F14", "tofu_harness", "두부 시트", "하네스 오염, 글꼴 후 재실행"),
    ("F15", "break_system_packages", "--break-system-packages", "거절"),
    ("F16", "question_already_answered", "질문이 이미 답", "다음 단 금지"),
    ("F17", "companion_pdf_promotion", "동반 PDF 승격", "provenance 확인 전 금지"),
    ("F18", "overwrite_source", "원본 덮어쓰기", "금지"),
]

REG_KEYS = [
    {
        "key": "plan",
        "source": "samples/2022* *.hwp",
        "pdf": "pdf/2022* *-2022.pdf",
        "grade": "한컴 2022 기준 PDF",
        "pages": 35,
        "note": "보고서 — 표·도해·강조",
    },
    {
        "key": "manual",
        "source": "samples/2025 *.hwpx",
        "pdf": "pdf/2025 *-2024.pdf",
        "grade": "한컴 2024 기준 PDF",
        "pages": None,
        "note": "장문 편람",
    },
    {
        "key": "bunjang",
        "source": "samples/21868765*.hwp",
        "pdf": "samples/21868765*.pdf",
        "grade": "참고 PDF — 버전·provenance 별도 확인",
        "pages": None,
        "note": "표 중심. 최종 기준 승격 금지",
    },
    {
        "key": "korexam",
        "source": "samples/21_*.hwp",
        "pdf": "pdf/21_*-2022.pdf",
        "grade": "한컴 2022 기준 PDF",
        "pages": 15,
        "note": "A3 2단 법학적성시험",
    },
    {
        "key": "math",
        "source": "samples/exam_math.hwp",
        "pdf": "pdf/exam_math-2022.pdf",
        "grade": "한컴 2022 기준 PDF",
        "pages": 20,
        "note": "수식. 실측 diff 6~11%",
    },
    {
        "key": "eng",
        "source": "samples/exam_eng.hwp",
        "pdf": "pdf/exam_eng-2022.pdf",
        "grade": "한컴 2022 기준 PDF",
        "pages": 8,
        "note": "영어 시험지, 라틴 혼합",
    },
]

OUTPUTS = [
    ("cmp-pNNN.png", "page sheet", "사람 눈", False),
    ("report.tsv", "pixel diff% ranking worst-first", "후보 순위", False),
    ("text-report.tsv", "NFC multiset loss/excess/substitution", "후보", True),
    ("svg-glyph-risk-report.tsv", "PUA/U+FFFD", "두부 후보", True),
    ("text-owner-shift-candidates.tsv", "adjacent page owner move", "후보", True),
    ("text-owner-sequence-candidates.tsv", "order-preserving string move", "후보", True),
    ("page-boundary-fidelity-candidates.tsv", "combined boundary queue", "후보", True),
    ("visible-text-excess-candidates.tsv", "clip-visible SVG excess", "후보", True),
    ("float-owner-shift-candidates.tsv", "text owner + successor float", "후보", False),
    ("page-count-ledger.tsv", "PDF/SVG/tree page counts", "후보", True),
    ("provenance.tsv", "source/oracle path+grade", "재현", True),
    ("run-state.tsv", "requested/completed/missing", "완전성", True),
    ("svg/export-svg-manifest.json", "export-svg cache", "재사용", True),
    ("layout-candidates.tsv", "body/footnote/table/footer", "후보", False),
    ("table-fragment-candidates.tsv", "same (pi,ci) across pages", "후보", False),
    ("svg-table-border-clip-candidates.tsv", "vertical border clip", "후보", False),
    ("svg-table-horizontal-border-clip-candidates.tsv", "horizontal border clip", "후보", False),
    ("table-cell-text-overlap-candidates.tsv", "duplicate paint in cell", "후보", False),
    ("table-cell-text-boundary-candidates.tsv", "text crossing cell edge", "후보", False),
    ("svg-text-band-clip-candidates.tsv", "glyph band clip", "후보", False),
]


def envelope(extra: dict) -> dict:
    out = {"schemaVersion": SCHEMA, "issue": ISSUE, "notGym": True, "noNewCli": True}
    out.update(extra)
    return out


def dump(name: str, data: dict) -> None:
    path = FIXT / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def write_skill_index() -> None:
    refs = sorted(p.name for p in HERE.glob("*.md"))
    examples = sorted(p.name for p in (SKILL / "examples").glob("*.md"))
    dump(
        "skill_index.json",
        envelope(
            {
                "skill": "rhwp-fidelity-compare",
                "forbiddenSkillsTouch": [
                    "rhwp-visual-regression",
                    "rhwp-cli",
                    "rhwp-onboarding",
                    "rhwp-mcp-session",
                    "rhwp-safe-edit",
                    "rhwp-provenance",
                    "rhwp-doc-triage",
                    "rhwp-form-fill",
                ],
                "forbiddenTrees": ["gym/"],
                "coreReuse": [
                    "tools/fidelity_compare/fidelity_compare.py",
                    "rhwp export-svg",
                    "rhwp export-svg --font-style",
                    "rhwp export-render-tree",
                ],
                "inventedCommandsForbidden": [
                    "fidelity-diff",
                    "pdf-compare",
                    "hangul-diff",
                    "pixel-diff",
                    "oracle-diff",
                    "hancom-compare",
                ],
                "references": refs,
                "examples": examples,
                "coreTopics": [
                    "independent Hangul PDF vs render-diff only",
                    "venv + pypdf/pypdfium2/pillow",
                    "Windows venv\\Scripts\\python.exe",
                    "never --break-system-packages",
                    "page sheets cmp-pNNN.png",
                    "pixel diff% ranking worst first",
                    "text-report.tsv multiset",
                    "--font-style local face aliases",
                    "tofu contamination",
                    "RHWP_FONT_PATH_DIR",
                    "provenance record",
                    "maintainer visual verdict",
                    "missing chrome",
                    "missing venv",
                    "page-count mismatch",
                    "encrypted PDF",
                    "tofu-filled harness",
                ],
            }
        ),
    )


def write_tree() -> None:
    dump(
        "tree.json",
        envelope(
            {
                "aaMustPass": False,
                "defaultMode": "pixel-and-text",
                "textOnlySkipsChrome": True,
                "rankingIsCandidate": True,
                "verdictIsMaintainer": True,
                "windowsPython": r"venv\Scripts\python.exe",
                "posixPython": "venv/bin/python",
                "breakSystemPackages": False,
                "coreReuse": [
                    "tools/fidelity_compare/fidelity_compare.py",
                    "export-svg",
                    "export-svg --font-style",
                ],
                "axes": [
                    {"when": "no official PDF", "go": "rhwp-visual-regression", "stop": "F01"},
                    {"when": "text candidates first", "go": "--text-only", "stop": "F02"},
                    {"when": "pixel sheets", "go": "default mode + Chrome", "stop": "F03"},
                    {"when": "tofu", "go": "font-style + RHWP_FONT_PATH_DIR", "stop": "F04/F14"},
                    {"when": "encrypted", "go": "stop", "stop": "F13"},
                ],
            }
        ),
    )


def write_stop_rules() -> None:
    dump(
        "stop_rules.json",
        envelope(
            {
                "rules": [
                    {
                        "id": rid,
                        "slug": slug,
                        "when": when,
                        "action": action,
                        "hard": rid
                        in {
                            "F06",
                            "F07",
                            "F08",
                            "F09",
                            "F13",
                            "F15",
                            "F18",
                        },
                    }
                    for rid, slug, when, action in STOP_RULES
                ]
            }
        ),
    )


def write_registered_keys() -> None:
    dump("registered_keys.json", envelope({"keys": REG_KEYS, "asciiGlobOnly": True}))


def write_outputs() -> None:
    dump(
        "outputs.json",
        envelope(
            {
                "artifacts": [
                    {
                        "file": name,
                        "means": means,
                        "authority": auth,
                        "textOnly": text_only,
                    }
                    for name, means, auth, text_only in OUTPUTS
                ],
                "reportTsvHeader": "page\tdiff%\tnote",
                "textReportTsvHeader": (
                    "page\treference_only\tsvg_only\t"
                    "reference_only_chars\tsvg_only_chars\tnote"
                ),
                "provenanceTsvHeader": "role\tpath\tgrade",
                "runStateTsvHeader": "field\tvalue",
                "sort": "report.tsv rows sorted by descending diff%",
            }
        ),
    )


def write_exceptions() -> None:
    dump(
        "exception_catalog.json",
        envelope(
            {
                "exceptions": [
                    {
                        "id": "E-CHROME",
                        "stop": "F10",
                        "signal": "Chrome/Chromium을 찾을 수 없습니다",
                        "exit": 2,
                        "fix": "CHROME_BIN or --text-only",
                    },
                    {
                        "id": "E-VENV",
                        "stop": "F09",
                        "signal": "pypdf가 필요합니다 / pypdfium2가 필요합니다",
                        "exit": 2,
                        "fix": "repo venv, never --break-system-packages",
                    },
                    {
                        "id": "E-PAGECOUNT",
                        "stop": "F11",
                        "signal": "page-count-ledger.tsv delta != 0",
                        "exit": None,
                        "fix": "candidate only, no global page-break patch",
                    },
                    {
                        "id": "E-ENCRYPT",
                        "stop": "F13",
                        "signal": "encrypted / password required",
                        "exit": None,
                        "fix": "stop; do not invent decrypt CLI",
                    },
                    {
                        "id": "E-TOFU",
                        "stop": "F14",
                        "signal": "sheet filled with U+25A1 / .notdef",
                        "exit": None,
                        "fix": "RHWP_FONT_PATH_DIR + --font-style rerun",
                    },
                    {
                        "id": "E-RHGP",
                        "stop": "F12",
                        "signal": "rhwp 실행 파일을 찾을 수 없습니다",
                        "exit": 2,
                        "fix": "RHWP_BIN or release-test build",
                    },
                    {
                        "id": "E-RANGE",
                        "stop": "F12",
                        "signal": "요청 끝 쪽이 기준 PDF 마지막 index를 넘습니다",
                        "exit": 2,
                        "fix": "clamp end page to PDF last index",
                    },
                ]
            }
        ),
    )


def write_fonts() -> None:
    dump(
        "font_aliases.json",
        envelope(
            {
                "defaultExportFlag": "--font-style",
                "embedDefault": False,
                "envFontDir": "RHWP_FONT_PATH_DIR",
                "envFontMode": "RHWP_SVG_FONT_MODE",
                "fontModes": {"style": "--font-style", "subset": "--embed-fonts", "full": "--embed-fonts=full"},
                "legacyFaces": [
                    {
                        "docFace": "한양중고딕",
                        "localAlias": True,
                        "note": "installed family/full name may differ",
                    },
                    {
                        "docFace": "휴먼명조",
                        "chromeFace": "HMKMM",
                        "ebdtNotdef": True,
                        "preferOutline": True,
                    },
                    {
                        "docFace": "휴먼고딕",
                        "chromeFace": "HMKMG",
                        "ebdtNotdef": True,
                        "preferOutline": True,
                    },
                    {
                        "docFace": "HY신명조",
                        "preserveOriginalPriority": True,
                    },
                ],
                "tofuCodepoints": ["U+25A1", "U+FFFD", "U+F02B1-U+F02C4", "U+F02FB"],
                "knownFind": "#3385 CharOverlap PUA tofu",
            }
        ),
    )


def write_provenance() -> None:
    dump(
        "provenance_schema.json",
        envelope(
            {
                "requiredFields": [
                    "hangulTool",
                    "hangulVersion",
                    "exportPath",
                    "fonts",
                    "originalPath",
                    "oraclePath",
                    "referenceGrade",
                ],
                "tsvColumns": ["role", "path", "grade"],
                "roles": ["source", "reference_pdf"],
                "grades": [
                    "한컴 2022 기준 PDF",
                    "한컴 2024 기준 PDF",
                    "한컴 2020 기준 PDF",
                    "참고 PDF — 버전·provenance 별도 확인",
                    "사용자 지정 기준 PDF (provenance는 출력 파일 참조)",
                ],
                "referenceGradeFlagDirectPairOnly": True,
            }
        ),
    )


def write_command_ladder() -> None:
    dump(
        "command_ladder.json",
        envelope(
            {
                "steps": [
                    {
                        "id": "S0",
                        "ask": "독립 한컴 PDF?",
                        "yes": "S1",
                        "no": "handoff-visual-regression",
                    },
                    {
                        "id": "S1",
                        "ask": "venv + deps?",
                        "yes": "S2",
                        "no": "F09",
                    },
                    {
                        "id": "S2",
                        "ask": "pixel sheets needed?",
                        "yes": "S3-chrome",
                        "no": "S3-text",
                    },
                    {
                        "id": "S3-text",
                        "cmd": "--text-only --export-all-svg --layout-ledger",
                        "next": "S4",
                    },
                    {
                        "id": "S3-chrome",
                        "cmd": "default pixel+text",
                        "need": "Chrome",
                        "missing": "F10",
                        "next": "S4",
                    },
                    {
                        "id": "S4",
                        "read": ["report.tsv", "text-report.tsv", "provenance.tsv", "run-state.tsv"],
                        "next": "S5",
                    },
                    {
                        "id": "S5",
                        "do": "audit worst pages / tofu / page-count",
                        "verdict": "maintainer",
                    },
                ]
            }
        ),
    )


def write_intent_matrix() -> None:
    intents = [
        ("한컴 PDF랑 비교해", "fidelity_compare registered or direct pair", "F03"),
        ("공식 출력 기준", "need provenance then compare", "F17"),
        ("글자만 빨리", "--text-only", "F02"),
        ("최악 쪽", "report.tsv sort", "F03"),
        ("□ 로 보여", "font-style / font path", "F14"),
        ("윈도에서", r"venv\Scripts\python.exe", "F15"),
        ("venv 깔아 pip --break-system-packages", "refuse", "F15"),
        ("Chrome 없는데 시트", "--text-only or stop", "F10"),
        ("암호화 PDF 열어", "stop", "F13"),
        ("쪽수가 달라", "page-count-ledger candidate", "F11"),
        ("편집 전후만", "handoff visual-regression", "F01"),
        ("버그 헌팅 여정", "handoff bug-hunter", "F08"),
        ("gym pack 만들어", "refuse", "F06"),
        ("fidelity-diff 명령 추가", "refuse", "F06"),
        ("render-diff 로 한컴 PDF", "wrong axis, this skill", "F01"),
        ("A==A 결정성", "visual-regression not here", "F01"),
        ("korexam A3", "key korexam", "F03"),
        ("math 수식", "key math", "F03"),
        ("임의 HWP+PDF", "--source --reference-pdf --label", "F03"),
        ("provenance 남겨", "provenance.tsv + grade", "F05"),
        ("유지자 판정", "governance", "F05"),
        ("두부 하네스", "rerun fonts", "F14"),
        ("RHWP_FONT_PATH_DIR", "pass dir list", "F04"),
        ("--font-style", "default already", "F04"),
        ("export-all-svg 전수", "long doc text-only", "F02"),
        ("layout-ledger", "table/footnote candidates", "F02"),
        ("missing pages", "run-state incomplete", "F12"),
        ("samples 옆 PDF 를 기준으로", "check grade first", "F17"),
        ("원본 덮어써", "refuse", "F18"),
        ("시스템 pip", "refuse", "F09"),
    ]
    dump(
        "intent_matrix.json",
        envelope(
            {
                "intents": [
                    {"utterance": u, "action": a, "stop": s, "notGym": True}
                    for u, a, s in intents
                ]
            }
        ),
    )


JOURNEY_SEEDS = [
    ("J01", "plan 전수 35쪽", ["venv", "plan 0 34", "report.tsv", "audit top"], "F03", True),
    ("J02", "독립 PDF 없음", ["ask oracle", "none"], "F01", True),
    ("J03", "text-only 215쪽", ["direct pair", "--text-only", "text-report"], "F02", True),
    ("J04", "Chrome 부재", ["pixel requested", "find_chrome fail"], "F10", True),
    ("J05", "venv 부재", ["ImportError pypdf"], "F09", True),
    ("J06", "Windows 경로", [r"venv\Scripts\python.exe", "plan 0 9"], "F03", True),
    ("J07", "break-system-packages 요청", ["refuse"], "F15", True),
    ("J08", "암호화 PDF", ["pypdf password"], "F13", True),
    ("J09", "쪽수 PDF 35 SVG 37", ["page-count-ledger"], "F11", True),
    ("J10", "두부 시트", ["□ majority", "set FONT_PATH", "rerun"], "F14", True),
    ("J11", "korexam A3 창 크기", ["auto viewport", "not fixed window"], "F03", True),
    ("J12", "math 수식 6-11%", ["rank not absolute"], "F03", True),
    ("J13", "bunjang 참고 PDF", ["grade 참고", "do not promote"], "F17", True),
    ("J14", "direct pair 누락 플래그", ["--source only"], "F12", True),
    ("J15", "PUA #3385", ["svg-glyph-risk", "issue escalate"], "F04", True),
    ("J16", "owner shift 각주", ["text-owner-shift-candidates"], "F02", True),
    ("J17", "table fragment p81-p82", ["page-boundary ledger"], "F02", True),
    ("J18", "HMKMM .notdef", ["prefer outline"], "F14", True),
    ("J19", "HY신명조 우선순위 보존", ["do not swap"], "F04", True),
    ("J20", "run-state incomplete", ["missing pages"], "F12", True),
    ("J21", "편집 전후 오인", ["handoff visual-regression"], "F01", True),
    ("J22", "여정 방법론 요청", ["handoff bug-hunter"], "F08", True),
    ("J23", "gym pack 요청", ["refuse"], "F06", True),
    ("J24", "새 CLI 요청", ["refuse"], "F06", True),
    ("J25", "visual-regression 스킬 수정", ["refuse"], "F07", True),
    ("J26", "provenance 빈칸", ["fill tool version path fonts"], "F05", True),
    ("J27", "유지자 판정 대기", ["do not self-merge"], "F05", True),
    ("J28", "export-all-svg cache 재사용", ["same --out-dir pixel"], "F03", True),
    ("J29", "layout-ledger square wrap", ["candidate not defect"], "F02", True),
    ("J30", "encrypted + text-only", ["still stop"], "F13", True),
    ("J31", "missing RHWP_BIN", ["exit 2"], "F12", True),
    ("J32", "end page overflow", ["clamp"], "F12", True),
    ("J33", "cp949 한글 argv", ["use ASCII key"], "F03", True),
    ("J34", "worktree 청결", ["--out-dir /tmp"], "F18", True),
    ("J35", "release-test stale binary", ["rebuild pr-review"], "F03", True),
    ("J36", "Linux fontconfig", ["FONTCONFIG_PATH from FONT_PATH_DIR"], "F04", True),
    ("J37", "Windows 설치 글꼴", ["native, no fontconfig"], "F04", True),
    ("J38", "macOS Applications Chrome", ["find_chrome darwin"], "F10", True),
    ("J39", "visible-text-excess", ["clip-aware candidate"], "F02", True),
    ("J40", "float owner p118-p119", ["float-owner-shift"], "F02", True),
    ("J41", "border clip p4", ["svg-table-border-clip"], "F02", True),
    ("J42", "horizontal clip p9-p14", ["svg-table-horizontal-border-clip"], "F02", True),
    ("J43", "cell overlap p2", ["table-cell-text-overlap"], "F02", True),
    ("J44", "cell boundary p34", ["table-cell-text-boundary"], "F02", True),
    ("J45", "text band clip first line", ["svg-text-band-clip"], "F02", True),
    ("J46", "sequence URL move p52-p53", ["text-owner-sequence"], "F02", True),
    ("J47", "companion pdf/ vs samples/", ["grade table"], "F17", True),
    ("J48", "질문 이미 답", ["stop ladder"], "F16", True),
    ("J49", "원본 덮어쓰기 거부", ["out-dir only"], "F18", True),
    ("J50", "시스템 pip 거부", ["venv only"], "F09", True),
    ("J51", "eng 라틴 혼합", ["key eng 0 7"], "F03", True),
    ("J52", "manual 장문 편람", ["key manual window"], "F03", True),
    ("J53", "한컴 2024 grade", ["manual key"], "F05", True),
    ("J54", "한컴 2020 사용자 PDF", ["--reference-grade"], "F05", True),
    ("J55", "인쇄→PDF vs 파일→PDF", ["record export path"], "F05", True),
    ("J56", "맞춰찍기 축소 PDF", ["hangul_pdf_baseline warning"], "F17", True),
    ("J57", "PageCount 불일치 생성 PDF", ["do not use as oracle"], "F17", True),
    ("J58", "diff% 0 발표 금지", ["candidate only"], "F05", True),
    ("J59", "top 8 stdout", ["then open tsv"], "F03", True),
    ("J60", "missing svg page", ["run-state incomplete"], "F12", True),
    ("J61", "Chrome retry stderr", ["surface exit"], "F10", True),
    ("J62", "A3 auto window", ["do not fix 1920x1080"], "F03", True),
    ("J63", "subset/full embed 요청", ["RHWP_SVG_FONT_MODE, default style"], "F04", True),
    ("J64", "라이선스 글꼴 embed 금지 기본", ["--font-style"], "F04", True),
    ("J65", "bug-hunter 에 원장 전달", ["handoff not rewrite"], "F08", True),
    ("J66", "render-diff 와 동시", ["different axis, do not mix verdict"], "F01", True),
    ("J67", "thumbnail 을 한컴 기준으로", ["wrong tool"], "F01", True),
    ("J68", "ir-diff 로 PDF 대조", ["wrong tool"], "F01", True),
    ("J69", "DocumentCore 수정 요청", ["out of scope"], "F06", True),
    ("J70", "새 rhwp 하위명령", ["refuse"], "F06", True),
    ("J71", "CI 에서 diff% 게이트", ["not merge gate"], "F05", True),
    ("J72", "incomplete 를 전수로 포장", ["forbid"], "F12", True),
    ("J73", "tofu 를 문서 회귀로", ["forbid"], "F14", True),
    ("J74", "암호화 우회 스크립트", ["forbid"], "F13", True),
    ("J75", "neighbor skill rewrite", ["forbid"], "F07", True),
    ("J76", "gym/ 아래 과제", ["forbid"], "F06", True),
    ("J77", "plan 0 9 창 샘플", ["then expand"], "F03", True),
    ("J78", "worst page issue 승격", ["after eyes"], "F05", True),
    ("J79", "font path list pathsep", ["multiple dirs"], "F04", True),
    ("J80", "CHROME_BIN override", ["when auto-discover fails"], "F10", True),
]


def write_journeys() -> None:
    dump(
        "journeys.json",
        envelope(
            {
                "journeys": [
                    {
                        "id": jid,
                        "title": title,
                        "steps": steps,
                        "stop": stop,
                        "notGym": True,
                    }
                    for jid, title, steps, stop, _ in JOURNEY_SEEDS
                ]
            }
        ),
    )


def write_pitfalls() -> None:
    dump(
        "pitfalls.json",
        envelope(
            {
                "items": [
                    "고정 Chrome 창은 A3 를 크롭해 가짜 diff 를 만든다",
                    "diff% 는 랭킹용이다. 자간 미세 차가 픽셀로 누적된다",
                    "text-report 는 순서·좌표를 모른다",
                    "배경 셸 한글 argv 는 cp949 로 깨진다",
                    "Chrome 실패는 한 번 재시도하고 stderr 를 표면화한다",
                    "samples/ 동반 PDF 는 미확인 시 참고 등급",
                    "맞춰찍기/배율 PDF 는 PageCount 가드가 필요하다",
                    "두부 시트는 하네스 오염일 수 있다",
                    "HMKMM/HMKMG EBDT 는 Chrome .notdef",
                    "--break-system-packages 금지",
                    "Windows 는 venv\\Scripts\\python.exe",
                    "누락 쪽이 있으면 종료 코드 0 이 아니다",
                    "page-count 차이는 전역 page-break 패치 근거가 아니다",
                    "최종 판정은 유지자",
                ]
            }
        ),
    )


def write_handoff() -> None:
    dump(
        "handoff.json",
        envelope(
            {
                "peers": [
                    {
                        "skill": "rhwp-visual-regression",
                        "when": "공식 PDF 없이 전후/왕복 레이아웃",
                        "rewriteHere": False,
                    },
                    {
                        "skill": "bug-hunter",
                        "when": "원인 미확정 실사용 여정",
                        "rewriteHere": False,
                    },
                    {
                        "skill": "rhwp-doc-triage",
                        "when": "미지 문서 파악만",
                        "rewriteHere": False,
                    },
                    {
                        "skill": "rhwp-cli",
                        "when": "export-svg 단건",
                        "rewriteHere": False,
                    },
                    {
                        "skill": "rhwp-safe-edit",
                        "when": "편집 자체",
                        "rewriteHere": False,
                    },
                ]
            }
        ),
    )


TRANSCRIPTS = {
    "plan_text_only.txt": """\
$ venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 34 --text-only --out-dir /tmp/rhwp-fidelity-plan
기준 PDF: /repo/pdf/2022-업무계획-2022.pdf
등급: 기준 PDF: pdf/ 보존 한컴 2022 출력
요청: 35쪽, 완료: 35쪽, 누락: 0쪽
pixel report: /tmp/rhwp-fidelity-plan/report.tsv
text report: /tmp/rhwp-fidelity-plan/text-report.tsv
run state: /tmp/rhwp-fidelity-plan/run-state.tsv
""",
    "plan_pixel_top8.txt": """\
$ venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 34 --out-dir /tmp/rhwp-fidelity-plan
기준 PDF: /repo/pdf/2022-업무계획-2022.pdf
등급: 기준 PDF: pdf/ 보존 한컴 2022 출력
요청: 35쪽, 완료: 35쪽, 누락: 0쪽
diff 랭킹(top 8):
  p12: 4.82%
  p7: 3.91%
  p28: 3.40%
  p3: 2.11%
  p19: 1.88%
  p1: 1.02%
  p4: 0.91%
  p35: 0.77%
pixel report: /tmp/rhwp-fidelity-plan/report.tsv
""",
    "missing_chrome.txt": """\
$ venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 2 --out-dir /tmp/rhwp-fidelity-nochrome
Chrome/Chromium을 찾을 수 없습니다. CHROME_BIN을 지정하세요.
$ echo $?
2
# F10 — --text-only 로 내리거나 Chrome 을 설치한다. 새 CLI 를 만들지 않는다.
""",
    "missing_venv.txt": """\
$ python3 tools/fidelity_compare/fidelity_compare.py plan 0 2 --text-only
pypdf가 필요합니다: python -m pip install pypdf
$ echo $?
2
# F09 — 저장소 venv 를 만들고 venv/bin/python 또는 venv\\Scripts\\python.exe 를 쓴다.
# --break-system-packages 금지 (F15).
""",
    "encrypted_pdf.txt": """\
$ venv/bin/python tools/fidelity_compare/fidelity_compare.py 0 0 \\
    --source samples/secret.hwp --reference-pdf samples/secret.pdf --label secret-enc --text-only
# pypdf.errors.FileNotDecryptedError / password required
# F13 — 정지. 암호 제거 CLI 를 발명하지 않는다. 잠금 해제된 공식 PDF 를 다시 받는다.
""",
    "page_count_mismatch.txt": """\
$ cat /tmp/rhwp-fidelity-issue/page-count-ledger.tsv
source	pages	delta_vs_reference	scope	note
reference_pdf	35	0	full PDF	comparison baseline
rhwp_svg	37	2	full export	page-count difference is a candidate, not a global-break fix
rhwp_render_tree	37	2	full render tree	page-count difference is a candidate, not a global-break fix
# F11 — 전역 page-break 패치를 열지 않는다. owner 쪽을 조사한다.
""",
    "tofu_harness.txt": """\
$ file /tmp/rhwp-fidelity-plan/cmp-p001.png
# 시트가 거의 □. svg-glyph-risk-report 에 U+25A1 다수, 본문 PUA 없음.
# F14 — 문서 회귀가 아니라 하네스 글꼴 오염.
$ RHWP_FONT_PATH_DIR=/Library/Fonts:/opt/hancom/fonts \\
    venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 2 --out-dir /tmp/rhwp-fidelity-plan-fonts
""",
    "windows_venv.txt": """\
PS> venv\\Scripts\\python.exe tools\\fidelity_compare\\fidelity_compare.py plan 0 9 --out-dir $env:TEMP\\rhwp-fidelity-plan
기준 PDF: C:\\repo\\pdf\\2022-plan-2022.pdf
등급: 기준 PDF: pdf/ 보존 한컴 2022 출력
요청: 10쪽, 완료: 10쪽, 누락: 0쪽
diff 랭킹(top 8):
  p3: 2.40%
  p1: 1.10%
# F15 — pip install --break-system-packages 를 쓰지 않았다.
""",
    "direct_pair_text.txt": """\
$ RHWP_BIN=target/release-test/rhwp venv/bin/python tools/fidelity_compare/fidelity_compare.py 0 214 \\
    --source samples/input.hwp --reference-pdf pdf/oracle-2020.pdf \\
    --label issue-3738-hwp --reference-grade '한컴 2020 기준 PDF' \\
    --text-only --export-all-svg --layout-ledger --out-dir /tmp/rhwp-fidelity-issue-3738
기준 PDF: /repo/pdf/oracle-2020.pdf
등급: 한컴 2020 기준 PDF
요청: 215쪽, 완료: 215쪽, 누락: 0쪽
text report: /tmp/rhwp-fidelity-issue-3738/text-report.tsv
page-count ledger: /tmp/rhwp-fidelity-issue-3738/page-count-ledger.tsv
""",
    "run_state_incomplete.txt": """\
$ cat /tmp/rhwp-fidelity-plan/run-state.tsv
field	value
mode	pixel-and-text
requested_pages_1based	1,2,3,4,5
completed_pages_1based	1,2,4
missing_pages_1based	3,5
run_state	incomplete
$ echo $?
1
# F12 — 부분 랭킹을 전수로 포장하지 않는다.
""",
}


TSV_FILES = {
    "report_ranked.tsv": """\
page	diff%	note
12	4.82	-
7	3.91	-
28	3.40	-
3	2.11	-
19	1.88	-
1	1.02	-
4	0.91	-
35	0.77	-
""",
    "text_report_mixed.tsv": """\
page	reference_only	svg_only	reference_only_chars	svg_only_chars	note
1	0	0			-
12	6	6	①②③	□□□	substitution-candidate
18	24	0	각주본문누락		loss-candidate
19	0	24		각주본문과잉	excess-candidate
""",
    "provenance_plan.tsv": """\
role	path	grade
source	/repo/samples/2022-업무계획.hwp	원본 입력
reference_pdf	/repo/pdf/2022-업무계획-2022.pdf	기준 PDF: pdf/ 보존 한컴 2022 출력
""",
    "run_state_complete.tsv": """\
field	value
mode	text-only
requested_pages_1based	1,2,3
completed_pages_1based	1,2,3
missing_pages_1based	-
run_state	complete
""",
    "page_count_drift.tsv": """\
source	pages	delta_vs_reference	scope	note
reference_pdf	35	0	full PDF	comparison baseline
rhwp_svg	37	2	full export	page-count difference is a candidate, not a global-break fix
rhwp_render_tree	37	2	full render tree	page-count difference is a candidate, not a global-break fix
""",
    "glyph_risk_pua.tsv": """\
page	risk_count	glyphs	note
1	0	-	-
12	14	U+F02B1×7,U+F02C4×7	raw PUA 또는 U+FFFD — 공개 글꼴에서 두부 후보
""",
}


TRACE_SPECS = [
    ("T01", "plan registered 0-34", "F03", ["venv", "plan 0 34", "rank"]),
    ("T02", "no oracle PDF", "F01", ["ask", "handoff visual-regression"]),
    ("T03", "text-only direct pair", "F02", ["--source", "--text-only"]),
    ("T04", "chrome missing", "F10", ["find_chrome", "exit 2"]),
    ("T05", "venv missing", "F09", ["ImportError", "exit 2"]),
    ("T06", "windows python", "F03", [r"venv\Scripts\python.exe"]),
    ("T07", "break-system-packages", "F15", ["refuse"]),
    ("T08", "encrypted pdf", "F13", ["stop"]),
    ("T09", "page count 35 vs 37", "F11", ["ledger"]),
    ("T10", "tofu sheet", "F14", ["FONT_PATH rerun"]),
    ("T11", "korexam A3", "F03", ["auto viewport"]),
    ("T12", "math 6-11%", "F03", ["rank"]),
    ("T13", "bunjang companion", "F17", ["do not promote"]),
    ("T14", "pua 3385", "F04", ["glyph-risk"]),
    ("T15", "owner shift footnote", "F02", ["adjacent pages"]),
    ("T16", "incomplete run", "F12", ["exit 1"]),
    ("T17", "maintainer verdict", "F05", ["governance"]),
    ("T18", "gym refuse", "F06", ["no pack"]),
    ("T19", "rewrite visual refuse", "F07", ["peer exists"]),
    ("T20", "rewrite hunter refuse", "F08", ["handoff only"]),
    ("T21", "stale binary", "F03", ["release-test rebuild"]),
    ("T22", "end overflow", "F12", ["exit 2"]),
    ("T23", "cp949 argv", "F03", ["ASCII key"]),
    ("T24", "out-dir tmp", "F18", ["source untouched"]),
    ("T25", "HMKMM notdef", "F14", ["outline fallback"]),
    ("T26", "sequence URL", "F02", ["p52-p53"]),
    ("T27", "border clip", "F02", ["candidate"]),
    ("T28", "print-scale PDF", "F17", ["not oracle"]),
    ("T29", "CHROME_BIN", "F10", ["override"]),
    ("T30", "multi font dirs", "F04", ["pathsep"]),
]


def write_traces() -> None:
    index = []
    for tid, title, stop, steps in TRACE_SPECS:
        rec = envelope(
            {
                "id": tid,
                "title": title,
                "stop": stop,
                "steps": steps,
                "tool": "tools/fidelity_compare/fidelity_compare.py",
                "notGym": True,
            }
        )
        dump(f"traces/{tid}.json", rec)
        index.append({"id": tid, "title": title, "stop": stop})
    dump("traces_index.json", envelope({"traces": index}))


def write_envelopes() -> None:
    dump(
        "envelopes/text_only_complete.json",
        envelope(
            {
                "mode": "text-only",
                "run_state": "complete",
                "requested": 35,
                "completed": 35,
                "missing": 0,
                "exit": 0,
            }
        ),
    )
    dump(
        "envelopes/pixel_ranked.json",
        envelope(
            {
                "mode": "pixel-and-text",
                "run_state": "complete",
                "top": [["p12", 4.82], ["p7", 3.91], ["p28", 3.40]],
                "absoluteVerdict": False,
            }
        ),
    )
    dump(
        "envelopes/missing_chrome.json",
        envelope({"mode": "pixel-and-text", "error": "chrome missing", "exit": 2, "stop": "F10"}),
    )
    dump(
        "envelopes/incomplete.json",
        envelope(
            {
                "mode": "pixel-and-text",
                "run_state": "incomplete",
                "missing": [3, 5],
                "exit": 1,
                "stop": "F12",
            }
        ),
    )


def write_samples() -> None:
    dump(
        "samples.json",
        envelope(
            {
                "optionalLive": [
                    "samples/form-01.hwp",
                    "samples/exam_math.hwp",
                    "samples/exam_eng.hwp",
                ],
                "liveCompareNotRequiredForContract": True,
                "reason": "공식 PDF 와 Chrome 은 CI 에 없을 수 있다. 계약은 파일 존재·스키마.",
            }
        ),
    )


def write_text_files() -> None:
    tdir = FIXT / "transcripts"
    tdir.mkdir(parents=True, exist_ok=True)
    for name, body in TRANSCRIPTS.items():
        (tdir / name).write_text(body, encoding="utf-8")
    sdir = FIXT / "tsv"
    sdir.mkdir(parents=True, exist_ok=True)
    for name, body in TSV_FILES.items():
        (sdir / name).write_text(body, encoding="utf-8")


def main() -> None:
    FIXT.mkdir(parents=True, exist_ok=True)
    write_stop_rules()
    write_tree()
    write_registered_keys()
    write_outputs()
    write_exceptions()
    write_fonts()
    write_provenance()
    write_command_ladder()
    write_intent_matrix()
    write_journeys()
    write_pitfalls()
    write_handoff()
    write_traces()
    write_envelopes()
    write_samples()
    write_text_files()
    write_skill_index()
    print(f"wrote fixtures under {FIXT}")


if __name__ == "__main__":
    main()
