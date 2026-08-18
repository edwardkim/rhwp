#!/usr/bin/env python3
"""[#5324] bug-hunter 레퍼런스·픽스처 생성기.

playbook 이 유일한 루브릭이다. 새 CLI 를 발명하지 않는다.
명령은 cli_commands.md 와 tools/fidelity_compare 가 이미 고정한 표면만
복제한다. gym 경로가 아니다. DocumentCore 를 고치지 않는다.
"""

from __future__ import annotations

import json
from pathlib import Path

SKILL = Path(__file__).resolve().parents[1]
REF = SKILL / "references"
EX = SKILL / "examples"
FIXT = SKILL / "fixtures"

ISSUE = 5324
SCHEMA = "1.0"
PLAYBOOK = "mydocs/manual/bug_hunting_playbook.md"
FIDELITY = "tools/fidelity_compare"

REQUIRED_REFS = [
    "00_tree.md",
    "01_playbook_authority.md",
    "02_judgment_traps.md",
    "03_journey_selection.md",
    "04_ground_truth.md",
    "05_hangul_pdf_provenance.md",
    "06_self_consistency_limit.md",
    "07_run_to_final.md",
    "08_pixel_visual.md",
    "09_text_multiset.md",
    "10_reread_values.md",
    "11_exit_json_contract.md",
    "12_fidelity_compare.md",
    "13_issue_template.md",
    "14_no_filing.md",
    "15_utf8_console.md",
    "16_pitfalls.md",
    "17_journeys.md",
    "18_worked_traces.md",
    "19_intent_matrix.md",
    "20_classification.md",
    "21_handoff.md",
    "22_failure_signals.md",
    "23_gate_recipes.md",
    "24_existing_cli.md",
    "README.md",
]

REQUIRED_EXAMPLES = [
    "01_kstartup_form.md",
    "02_hangul_pdf_compare.md",
    "03_gianmun_legal.md",
    "04_roundtrip_ir.md",
    "05_cli_contract.md",
    "06_float_margin_leet.md",
    "07_seoul_hwpx_zip.md",
    "08_full_table_fill.md",
    "09_rag_citation.md",
    "10_batch_archive.md",
    "11_official_notice.md",
    "12_pii_mask.md",
    "13_edit_format_preserve.md",
    "14_no_baseline.md",
    "15_console_encoding.md",
    "16_oracle_pass_not_lossless.md",
    "17_check_devel_first.md",
    "18_dont_generalize.md",
    "19_reject_hypothesis.md",
    "20_issue_from_finding.md",
    "README.md",
]

# 기존 샘플·공개 서식. 새 HWP 바이너리를 만들지 않는다.
SAMPLES = {
    "kstartup": {
        "path": "(외부) K-Startup 방산 특화 창업중심대학 사업계획서.hwp",
        "pages": 12,
        "tables": 39,
        "fields": 0,
        "note": "playbook 예시 1. 누름틀 0 · 표 양식. 가상 데이터만.",
        "groundTruth": "제출 요건 문구 + 재독 값",
    },
    "plan": {
        "path": "samples/업무계획.hwp",
        "pages": 35,
        "note": "playbook 예시 2. fidelity_compare 키 plan.",
        "groundTruth": "한컴 출력 PDF (도구·버전·폰트 기록)",
    },
    "math": {
        "path": "samples/수학.hwp",
        "pages": 20,
        "note": "playbook 예시 2 고난도 수식.",
        "groundTruth": "한컴 출력 PDF",
    },
    "leet": {
        "path": "samples/21_언어_기출_편집가능본.hwp",
        "pages": 15,
        "sha256": "905454045ca2e236839a7cab59750678116d08af3db31dbf846819af355b8d15",
        "note": "playbook 예시 6. A3 법학적성시험 언어이해.",
        "groundTruth": "pdf/21_언어_기출_편집가능본-2022.pdf",
        "referenceSha256": "f2d858d7974393661d91a658e6b384b951114ef52783379f426a963effd97b72",
        "creator": "Hwp 2022 12.0.0.4426",
        "producer": "Hancom PDF 1.3.0.550",
    },
    "gianmun": {
        "path": "(외부) 행정 업무 편람 별지 제1호 일반기안문",
        "pages": None,
        "note": "playbook 예시 3. 법정 서식.",
        "groundTruth": "편람 별지 제1·2호",
    },
    "seoul": {
        "path": "(외부) 서울시 정보소통광장 보고서 작성 서식 안내 첨부 HWPX",
        "pages": None,
        "note": "playbook 예시 7. 인터넷 배포 실물.",
        "groundTruth": "원본 ZIP 엔트리 이름 집합·태그 개수",
        "url": "https://opengov.seoul.go.kr/sanction/11678326",
    },
    "form01": {
        "path": "samples/form-01.hwp",
        "pages": 1,
        "note": "정답지 없을 때 자기 일관성 표본.",
        "groundTruth": None,
    },
    "field01": {
        "path": "samples/field-01.hwp",
        "pages": 3,
        "note": "누름틀 재독 표본.",
        "groundTruth": "fields --json 기록값",
    },
}

ALLOWED_COMMANDS = [
    "info",
    "fields",
    "export-tables",
    "export-svg",
    "export-png",
    "export-pdf",
    "export-text",
    "export-hwpx",
    "export-render-tree",
    "edit set-cell",
    "edit fill-fields",
    "edit replace-text",
    "ir-diff",
    "render-diff",
    "dump",
    "dump-pages",
    "capabilities",
    "search",
    "convert",
    "thumbnail",
    "inspect",
]

INVENTED_COMMANDS = [
    "bug-hunt",
    "oracle-check",
    "fidelity-diff",
    "ground-truth",
    "hunt-bugs",
    "compare-oracle",
    "text-multiset",
    "gym-hunt",
]

COMMANDS = [
    {
        "id": "info-json",
        "argv": ["info", "--json", "<파일>"],
        "writes": False,
        "when": "쪽수·형식·폰트 파악",
    },
    {
        "id": "fields-json",
        "argv": ["fields", "--json", "<파일>"],
        "writes": False,
        "when": "누름틀 목록. 0 이면 표 양식 후보",
    },
    {
        "id": "export-tables-json",
        "argv": ["export-tables", "--json", "<파일>"],
        "writes": False,
        "when": "표 좌표·재독 대조",
    },
    {
        "id": "set-cell",
        "argv": [
            "edit",
            "set-cell",
            "<파일>",
            "--table",
            "N",
            "--row",
            "R",
            "--col",
            "C",
            "--text",
            "<값>",
            "-o",
            "<산출>",
            "--json",
        ],
        "writes": True,
        "when": "누름틀 0 인 표 양식 칸 채움",
    },
    {
        "id": "fill-fields",
        "argv": ["edit", "fill-fields", "<파일>", "--data", "<JSON>", "-o", "<산출>", "--json"],
        "writes": True,
        "when": "누름틀 있는 서식 채움",
    },
    {
        "id": "export-svg",
        "argv": ["export-svg", "<파일>", "-o", "<svg/>"],
        "writes": True,
        "when": "fidelity_compare 입력 SVG",
    },
    {
        "id": "export-pdf",
        "argv": ["export-pdf", "<파일>", "-o", "<제출용.pdf>"],
        "writes": True,
        "when": "제출 직전 산출물. 실제 접수는 하지 않음",
    },
    {
        "id": "export-hwpx-verify",
        "argv": ["export-hwpx", "<원본>", "<변환>", "--verify", "--verify-pages"],
        "writes": True,
        "when": "IR 오라클. 통과해도 ZIP 대조를 멈취지 않음",
    },
    {
        "id": "ir-diff-json",
        "argv": ["ir-diff", "<A>", "<B>", "--json"],
        "writes": False,
        "when": "IR 차이. 차이 = exit 3",
    },
    {
        "id": "render-diff-self",
        "argv": ["render-diff", "<파일>", "--via", "hwpx"],
        "writes": False,
        "when": "자기 일관성. 한컴 충실도가 아님",
    },
    {
        "id": "dump-pages",
        "argv": ["dump-pages", "<파일>", "--json"],
        "writes": False,
        "when": "쪽 갈라짐 좁힘",
    },
    {
        "id": "capabilities",
        "argv": ["capabilities"],
        "writes": False,
        "when": "자기서술 vs 실제 가용성",
    },
    {
        "id": "fidelity-compare",
        "argv": [
            "venv/bin/python",
            "tools/fidelity_compare/fidelity_compare.py",
            "<키>",
            "<시작>",
            "<끝>",
            "--out-dir",
            "<외부>",
        ],
        "writes": True,
        "when": "한컴 PDF ↔ SVG 픽셀 + 문자 멀티셋",
        "notCli": True,
    },
    {
        "id": "fidelity-text-only",
        "argv": [
            "venv/bin/python",
            "tools/fidelity_compare/fidelity_compare.py",
            "<시작>",
            "<끝>",
            "--source",
            "<hwp>",
            "--reference-pdf",
            "<pdf>",
            "--text-only",
            "--export-all-svg",
            "--layout-ledger",
            "--out-dir",
            "<외부>",
        ],
        "writes": True,
        "when": "등록 키 없는 임의 쌍. Chrome 없이 문자만",
        "notCli": True,
    },
]

STOP_RULES = [
    ("F01", "실물 정답지 없는 무작위 스윕", "playbook 카탈로그로 되돌림"),
    ("F02", "정답지를 아직 안 확보", "여정 실행 금지"),
    ("F03", "한컴 PDF provenance 미기록", "비교 시트를 이슈 근거로 쓰지 않음"),
    ("F04", "독립 기준 없음", "render-diff 만 + 한계 기록. 충실도 이슈 금지"),
    ("F05", "중간 단계에서 멈춤", "최종 산출물까지 이어서 실행"),
    ("F06", "reference_only 문자", "소실 후보. 단독 최종 판정 금지"),
    ("F07", "svg_only 문자", "과잉 후보. 단독 최종 판정 금지"),
    ("F08", "같은 쪽 양쪽 차이", "치환 후보. 사람 감사"),
    ("F09", "--verify 4/4 통과", "ZIP 엔트리 이름 집합·태그 개수 대조"),
    ("F10", "콘솔 한글 깨짐", "결함 아님. UTF-8 파일 재비교"),
    ("F11", "증상만 있는 이슈 초안", "재현·파일:라인·정답지 근거가 생길 때까지 금지"),
    ("F12", "이 스킬 안에서 수정 시작", "별도 작업. DocumentCore 금지"),
    ("F13", "실제 접수·로그인·실명인증 자동화", "즉시 거부"),
    ("F14", "devel 에서 이미 고쳐짐", "새 이슈 금지"),
    ("F15", "표본 1건으로 계약 단정", "N중 M·반례 수까지 가설"),
    ("F16", "가설만 있고 구현 기각 없음", "음성 결과도 이슈에 남김"),
]

CLASSIFICATION = [
    {
        "id": "C01",
        "observation": "reference_only",
        "labelKo": "소실",
        "labelEn": "loss",
        "final": False,
        "axis": "text-multiset",
        "issueReady": False,
        "note": "기준 PDF 텍스트층에만 있는 문자. 후보.",
    },
    {
        "id": "C02",
        "observation": "svg_only",
        "labelKo": "과잉",
        "labelEn": "excess",
        "final": False,
        "axis": "text-multiset",
        "issueReady": False,
        "note": "SVG <text> 에만 있는 문자. 숨김 대상 과잉 출력 의심.",
    },
    {
        "id": "C03",
        "observation": "both_delta_same_page",
        "labelKo": "치환",
        "labelEn": "substitution",
        "final": False,
        "axis": "text-multiset",
        "issueReady": False,
        "note": "같은 쪽에 양쪽 차이. PUA·폰트 대체 후보.",
    },
    {
        "id": "C04",
        "observation": "reread_mismatch",
        "labelKo": "기록값 불일치",
        "labelEn": "reread",
        "final": True,
        "axis": "reread",
        "issueReady": True,
        "note": "export-tables/fields 재독이 쓴 값과 다름. 기계 확정.",
    },
    {
        "id": "C05",
        "observation": "exit_or_json_contract",
        "labelKo": "계약 위반",
        "labelEn": "contract",
        "final": True,
        "axis": "exit-json",
        "issueReady": True,
        "note": "종료 코드·JSON 봉투가 cli_commands.md 와 다름.",
    },
    {
        "id": "C06",
        "observation": "pixel_diff_rank",
        "labelKo": "픽셀 후보",
        "labelEn": "pixel-candidate",
        "final": False,
        "axis": "pixel",
        "issueReady": False,
        "note": "diff% 랭킹. 절대 오라클 아님. 사람 감사.",
    },
    {
        "id": "C07",
        "observation": "zip_name_set_missing",
        "labelKo": "엔트리 소실",
        "labelEn": "zip-loss",
        "final": False,
        "axis": "zip",
        "issueReady": False,
        "note": "개수가 아니라 이름 집합. 추가가 소실을 상쇄할 수 있다.",
    },
    {
        "id": "C08",
        "observation": "constant_byte_shrink",
        "labelKo": "상수 블록 소실 신호",
        "labelEn": "constant-shrink",
        "final": False,
        "axis": "zip",
        "issueReady": False,
        "note": "여러 문서에서 같은 바이트 수만큼 줄어들면 구조 소실 의심.",
    },
    {
        "id": "C09",
        "observation": "self_render_diff_only",
        "labelKo": "자기 일관성",
        "labelEn": "self-consistency",
        "final": False,
        "axis": "render-diff",
        "issueReady": False,
        "note": "정답지 없을 때 한계. 한컴 충실도 이슈로 승격 금지.",
    },
    {
        "id": "C10",
        "observation": "console_mojibake",
        "labelKo": "콘솔 착시",
        "labelEn": "not-a-defect",
        "final": True,
        "axis": "encoding",
        "issueReady": False,
        "note": "cp949 콘솔. UTF-8 파일 비교만. 이슈 금지.",
    },
    {
        "id": "C11",
        "observation": "pdf_path_only_glyphs",
        "labelKo": "텍스트층 손상 후보",
        "labelEn": "pdf-path-text",
        "final": False,
        "axis": "text-multiset",
        "issueReady": False,
        "note": "PDF 가 글자를 path 로 그린 경우. 단독 최종 판정 금지.",
    },
    {
        "id": "C12",
        "observation": "layout_ledger_square_wrap",
        "labelKo": "그림 침범 후보",
        "labelEn": "square-wrap",
        "final": False,
        "axis": "layout-ledger",
        "issueReady": False,
        "note": "square_wrap_text_overlap. PDF raster 없이 후보화.",
    },
]

ISSUE_TEMPLATE_FIELDS = [
    {
        "id": "repro",
        "required": True,
        "title": "재현 명령",
        "hint": "복붙하면 같은 산출이 나오는 rhwp / fidelity_compare 명령",
    },
    {
        "id": "codePath",
        "required": True,
        "title": "코드 경로",
        "hint": "파일:라인. devel HEAD 에서 확인",
    },
    {
        "id": "groundTruth",
        "required": True,
        "title": "정답지 대비 근거",
        "hint": "한컴 PDF provenance / 법정 서식 / 제출 요건 문구 / 재독 표",
    },
    {
        "id": "classification",
        "required": True,
        "title": "비교 분류",
        "hint": "소실·과잉·치환·재독·계약·픽셀 후보 중 하나",
    },
    {
        "id": "limitations",
        "required": True,
        "title": "한계",
        "hint": "오라클이 안 보는 축, 텍스트층 path, 자기 일관성만인지",
    },
    {
        "id": "notAFix",
        "required": True,
        "title": "수정 아님",
        "hint": "이 이슈는 헌팅 산출. 패치는 별도 PR",
    },
]

PITFALLS = [
    {
        "id": "P01",
        "trap": "오라클 통과를 무손실로 읽음",
        "playbook": 1,
        "signal": "--verify 4/4 인데 tabItem 절반 (#3551)",
        "fix": "ZIP 엔트리 이름 집합·태그 개수까지",
    },
    {
        "id": "P02",
        "trap": "devel 확인 없이 이슈를 다시 염",
        "playbook": 2,
        "signal": "열려 있던 53건 중 17건이 이미 고쳐짐",
        "fix": "현재 devel 파일:라인으로 생존 확인",
    },
    {
        "id": "P03",
        "trap": "표본 1건을 포맷 계약으로 일반화",
        "playbook": 3,
        "signal": "한 파일 21쌍으로 halving 단정 (#3368)",
        "fix": "코퍼스 전량 N중 M + 반례 수",
    },
    {
        "id": "P04",
        "trap": "가설을 구현하지 않고 원인으로 씀",
        "playbook": 4,
        "signal": "provenance 가드가 원인이라 썼으나 페이지 수 그대로 (#3518)",
        "fix": "구현해서 기각. 음성 결과도 남김",
    },
    {
        "id": "P05",
        "trap": "정답지 없이 그럴듯하다로 통과",
        "playbook": None,
        "signal": "스크린샷만 있고 한컴 PDF 없음",
        "fix": "F04. render-diff 한계를 기록",
    },
    {
        "id": "P06",
        "trap": "여정을 중간에서 끊음",
        "playbook": None,
        "signal": "info 만 보고 채움·내보내기를 안 함",
        "fix": "F05. 최종 산출물까지",
    },
    {
        "id": "P07",
        "trap": "콘솔 깨짐을 결함으로 이슈화",
        "playbook": None,
        "signal": "cp949 콘솔에서 한글 물음표",
        "fix": "F10. UTF-8 파일 비교",
    },
    {
        "id": "P08",
        "trap": "실제 접수를 자동화",
        "playbook": None,
        "signal": "로그인·실명인증 스크립트",
        "fix": "F13. 제출 직전 산출물까지만",
    },
    {
        "id": "P09",
        "trap": "자기 라운드트립 PASS 를 한컴 충실도로 읽음",
        "playbook": None,
        "signal": "render-diff PASS 인데 한컴 PDF 와 다름",
        "fix": "fidelity_compare. render-diff 는 내부 회귀",
    },
    {
        "id": "P10",
        "trap": "엔트리 개수로 소실을 판정",
        "playbook": 1,
        "signal": "12→12 인데 ole1.ole 소실 (#3557)",
        "fix": "이름 집합. 추가가 소실을 상쇄",
    },
    {
        "id": "P11",
        "trap": "문자 멀티셋을 최종 판정으로 씀",
        "playbook": None,
        "signal": "PDF path 글자를 소실로 이슈화",
        "fix": "후보. 사람 감사 + 시각 거버넌스",
    },
    {
        "id": "P12",
        "trap": "두 번째 헌팅 루브릭을 만듦",
        "playbook": None,
        "signal": "스킬 안에 독자 점수표",
        "fix": "playbook 만. 이 스킬은 실행 계약",
    },
    {
        "id": "P13",
        "trap": "증상만 적고 파일:라인이 없음",
        "playbook": None,
        "signal": "렌더가 깨진다 한 줄",
        "fix": "F11. 재현·경로·정답지",
    },
    {
        "id": "P14",
        "trap": "한컴 PDF 를 보편 절대 오라클로 취급",
        "playbook": None,
        "signal": "도구·버전·폰트 미기록",
        "fix": "F03. provenance 필수",
    },
    {
        "id": "P15",
        "trap": "새 비교 CLI 를 발명",
        "playbook": None,
        "signal": "존재하지 않는 헌팅 전용 하위명령",
        "fix": "기존 CLI + tools/fidelity_compare",
    },
    {
        "id": "P16",
        "trap": "검출 94.6% 를 손실 94.6% 로 씀",
        "playbook": 1,
        "signal": "정규화(fwSpace)를 데이터 손실로 과장",
        "fix": "검출과 판정을 나눔. 값 손실이 아니면 아니라고 씀",
    },
]

HANDOFF = [
    {
        "when": "누름틀을 채우는 수단 자체",
        "to": "rhwp-form-fill",
        "back": "채운 산출을 정답지와 대조하러 여기로",
    },
    {
        "when": "전후 레이아웃 px 숫자만",
        "to": "rhwp-visual-regression",
        "back": "한컴 충실도는 여기. render-diff 는 자기 일관성",
    },
    {
        "when": "표 CSV 왕복",
        "to": "rhwp-table-exchange",
        "back": "되돌린 표를 재독·정답지 대조",
    },
    {
        "when": "배포 전 숨은 글·주입",
        "to": "rhwp-security-sweep",
        "back": "스윕 후 제출 직전 산출 대조",
    },
    {
        "when": "미지 문서 파악만",
        "to": "rhwp-doc-triage",
        "back": "파악 후 여정을 여기로",
    },
    {
        "when": "수정 PR 절차",
        "to": "rhwp-contributor",
        "back": "요청받은 뒤에만. 이 스킬은 헌팅",
    },
    {
        "when": "폴더 수백 건 일괄",
        "to": "rhwp-bulk-pipeline",
        "back": "실패한 한 건을 여정으로 승격",
    },
]

PROVENANCE_KEYS = [
    "tool",
    "version",
    "outputPath",
    "fonts",
    "sourcePath",
    "referencePdfPath",
    "sourceSha256",
    "referenceSha256",
    "creator",
    "producer",
    "paper",
    "recordedAt",
]


def playbook_journeys() -> list[dict]:
    """playbook 예시 1–7 + 카탈로그 후보를 기계 가독으로."""
    core = [
        {
            "id": "J01",
            "title": "정부 실공고 양식 채움 (K-Startup)",
            "playbookExample": 1,
            "groundTruthKind": "submission-requirement",
            "steps": ["info-json", "fields-json", "export-tables-json", "set-cell", "export-tables-json", "export-pdf"],
            "stop": "F13",
            "sample": "kstartup",
            "findings": ["#3381", "#3391", "#3395", "#3358"],
            "notGym": True,
        },
        {
            "id": "J02",
            "title": "한컴 출력 PDF 페이지별 대규모 대조",
            "playbookExample": 2,
            "groundTruthKind": "hangul-pdf",
            "steps": ["export-svg", "fidelity-compare"],
            "stop": "F06",
            "sample": "plan",
            "findings": ["#3385", "#3382", "#3389"],
            "notGym": True,
        },
        {
            "id": "J03",
            "title": "법정 서식 생성 (기안문)",
            "playbookExample": 3,
            "groundTruthKind": "legal-form",
            "steps": ["export-svg", "fill-fields", "fidelity-compare"],
            "stop": "F02",
            "sample": "gianmun",
            "findings": ["#3372", "#3375"],
            "notGym": True,
        },
        {
            "id": "J04",
            "title": "형식 변환·무손실 라운드트립 (IR 오라클)",
            "playbookExample": 4,
            "groundTruthKind": "ir-oracle",
            "steps": ["export-hwpx-verify", "ir-diff-json"],
            "stop": "F09",
            "sample": "form01",
            "findings": ["#3367", "#3368", "#3383"],
            "notGym": True,
        },
        {
            "id": "J05",
            "title": "에이전트 계약 정합",
            "playbookExample": 5,
            "groundTruthKind": "cli-contract",
            "steps": ["capabilities"],
            "stop": "F11",
            "sample": None,
            "findings": ["#3349", "#3353", "#3355", "#3357", "#3359", "#3366"],
            "notGym": True,
        },
        {
            "id": "J06",
            "title": "부동 개체 본문 여백 (법학적성시험)",
            "playbookExample": 6,
            "groundTruthKind": "hangul-pdf",
            "steps": ["export-svg", "fidelity-compare"],
            "stop": "F03",
            "sample": "leet",
            "findings": ["#3402"],
            "notGym": True,
        },
        {
            "id": "J07",
            "title": "인터넷 배포 실물 무손실 왕복 (서울시)",
            "playbookExample": 7,
            "groundTruthKind": "zip-structure",
            "steps": ["info-json", "export-tables-json", "fields-json", "export-svg", "export-hwpx-verify"],
            "stop": "F09",
            "sample": "seoul",
            "findings": ["#3551"],
            "notGym": True,
        },
    ]
    catalog = [
        ("J08", "실물 표 양식 전 항목 채움(46칸)", "submission-requirement", ["export-tables-json", "set-cell", "export-tables-json"], "F05"),
        ("J09", "공고 검색 → 근거 조항 위치 → 해당 쪽만 렌더", "legal-form", ["info-json", "export-svg"], "F02"),
        ("J10", "대량 아카이브 대장화", "cli-contract", ["info-json"], "F11"),
        ("J11", "시행문 법정 서식 생성", "legal-form", ["export-svg", "fill-fields"], "F02"),
        ("J12", "공고문 법정 서식 생성", "legal-form", ["export-svg", "fill-fields"], "F02"),
        ("J13", "회의록 법정 서식 생성", "legal-form", ["export-svg", "fill-fields"], "F02"),
        ("J14", "개인정보 탐지 → 마스킹", "submission-requirement", ["inspect"], "F13"),
        ("J15", "인터넷 배포 실물 수집 확장", "zip-structure", ["export-hwpx-verify"], "F09"),
        ("J16", "edit 3종 형식 보존 전수", "ir-oracle", ["export-hwpx-verify", "ir-diff-json"], "F09"),
        ("J17", "정답지 없는 내부 문서 자기 일관성", None, ["render-diff-self"], "F04"),
        ("J18", "수학 20쪽 한컴 PDF 대조", "hangul-pdf", ["fidelity-compare"], "F06"),
        ("J19", "업무계획 35쪽 전수 랭킹", "hangul-pdf", ["fidelity-compare"], "F06"),
        ("J20", "A3 수식·언어이해 고난도", "hangul-pdf", ["fidelity-compare"], "F03"),
    ]
    extra_titles = [
        "누름틀 0 표 양식에서 set-cell 가능 여부",
        "채움 값 스타일이 검정 글씨 요건을 만족하는가",
        "체크박스가 글머리표라 텍스트 밖인지",
        "잘못된 입력 침묵 유실",
        "한컴 PUA 원문자 tofu",
        "제어문자 불법 XML",
        "빈 누름틀 안내문 인쇄 프로필 출력",
        "구역 시작 secd/cold 순서 뒤집힘",
        "ParaShape 좌측여백 왕복 표류",
        "edit 계열 HWPX→HWP5 강제 산출",
        "search --limit 총량 은폐",
        "옵션 선행 파싱",
        "릴리스 바이너리 export-png 부재",
        "export 계열 파싱",
        "thumbnail 계약 밖",
        "바탕쪽 부동 개체 본문 여백",
        "tabPr hp:switch 소실",
        "엔트리 수 동일·이름 집합 소실",
        "fwSpace 정규화 vs 데이터 손실",
        "페이지 수 64→65 char_shapes -2",
        "한컴 PDF 도구 버전 미기록 거부",
        "UTF-8 파일 vs cp949 콘솔",
        "가상 데이터로 작성·실제 접수 거부",
        "playbook 4단 예시 초안 작성",
        "fidelity_compare --text-only 전수",
        "fidelity_compare --layout-ledger",
        "text-report.tsv 소실 랭킹",
        "text-report.tsv 과잉 랭킹",
        "같은 쪽 치환 후보 사람 감사",
        "run-state.tsv 누락 쪽 재실행",
        "provenance.tsv 기록 확인",
        "ZIP 태그 개수 상수 감소",
        "devel HEAD 파일:라인 재확인",
        "가설 구현 기각 로그",
        "코퍼스 N중 M 표",
        "제출 직전 PDF 재독",
        "fields --json 재독 100%",
        "export-tables 재독 100%",
        "ir-diff --json exit 3 데이터",
        "render-diff A==A 결정성",
        "한컴 Creator/Producer 메타",
        "폰트 설치 vs @font-face local",
        "RHWP_FONT_PATH_DIR 계약",
        "Chrome raster 없이 text-only",
        "임의 HWP/PDF 쌍 --source",
        "외부 --out-dir 로 worktree 청결",
        "비교 시트 스케일 착시 재확인",
        "숨김 대상 과잉 출력",
        "쪽번호·채움점 소실",
        "PUA 치환 후보",
        "값 손실이 아니면 아니라고 씀",
        "수정은 별도 PR",
        "gym 경로 거부",
        "새 CLI 발명 거부",
        "DocumentCore 손대지 않음",
        "이웃 스킬 재작성 거부",
        "간이기안문 결재란 표",
        "시행문 두문·결문",
        "공고문 항목체계",
        "회의록 출석 표",
        "46칸 전 항목 완결",
        "RAG 인용 쪽만 export-svg -p",
        "batch 봉투 제목 필드 부재 #3407",
        "형식 보존 #3383 전수",
        "한셀OLE 엔트리 상쇄 #3557",
        "samples/ 치우침 → 인터넷 실물",
        "정답지 렌더 후 서식 제작 루프",
        "자기검증 info/fields/export-svg",
        "픽셀 상위 + 문자 소실 교집합",
        "사람 감사 큐 page-boundary",
        "visible-text-excess 후보",
        "float-owner-shift 후보",
        "table-fragment 후보",
        "svg-glyph-risk PUA/U+FFFD",
    ]
    out = list(core)
    for row in catalog:
        jid, title, gtk, steps, stop = row
        out.append(
            {
                "id": jid,
                "title": title,
                "playbookExample": None,
                "groundTruthKind": gtk,
                "steps": steps,
                "stop": stop,
                "sample": None,
                "findings": [],
                "notGym": True,
            }
        )
    for i, title in enumerate(extra_titles, start=21):
        gtk = [
            "hangul-pdf",
            "legal-form",
            "submission-requirement",
            "zip-structure",
            "ir-oracle",
            "cli-contract",
            None,
        ][i % 7]
        stop = ["F02", "F03", "F04", "F05", "F06", "F09", "F11", "F13"][i % 8]
        out.append(
            {
                "id": f"J{i:02d}",
                "title": title,
                "playbookExample": None,
                "groundTruthKind": gtk,
                "steps": ["info-json", "export-svg"] if gtk else ["render-diff-self"],
                "stop": stop,
                "sample": None,
                "findings": [],
                "notGym": True,
            }
        )
    return out


def intent_rows() -> list[dict]:
    seeds = [
        ("버그 찾아줘 실사용 기준", "playbook 여정 하나 선택", "03_journey_selection.md", "F01"),
        ("playbook 여정 실행", "카탈로그 J01–J07 중 하나", "17_journeys.md", "F01"),
        ("정답지부터 잡아", "한컴 PDF/법정 서식/제출 요건", "04_ground_truth.md", "F02"),
        ("한컴 공식 PDF 와 비교", "fidelity_compare.py", "12_fidelity_compare.md", "F03"),
        ("법정 서식이랑 맞춰", "편람 별지를 정답지로", "03_journey_selection.md", "F02"),
        ("제출 요건 문구 대조", "재독 + 스타일", "10_reread_values.md", "F11"),
        ("정답지가 없는데", "render-diff 만 + 한계", "06_self_consistency_limit.md", "F04"),
        ("여정 끝까지 돌려", "최종 산출물까지", "07_run_to_final.md", "F05"),
        ("픽셀 diff 상위 쪽", "report.tsv 랭킹", "08_pixel_visual.md", "F06"),
        ("글자 빠졌나", "text-report reference_only", "09_text_multiset.md", "F06"),
        ("글자가 더 나왔나", "text-report svg_only", "09_text_multiset.md", "F07"),
        ("글자가 바뀌었나", "같은 쪽 양쪽 차이", "09_text_multiset.md", "F08"),
        ("값이 맞나 다시 읽어", "export-tables --json", "10_reread_values.md", "F11"),
        ("누름틀 값 재독", "fields --json", "10_reread_values.md", "F11"),
        ("종료 코드가 이상해", "cli_commands.md 계약", "11_exit_json_contract.md", "F11"),
        ("JSON 봉투 깨짐", "schemaVersion 확인", "11_exit_json_contract.md", "F11"),
        ("이슈로 남겨", "템플릿 3필수", "13_issue_template.md", "F11"),
        ("접수까지 자동화해", "거부", "14_no_filing.md", "F13"),
        ("로그인해서 제출", "거부", "14_no_filing.md", "F13"),
        ("콘솔이 깨져 보여", "UTF-8 파일", "15_utf8_console.md", "F10"),
        ("오라클 통과했는데", "ZIP 이름 집합", "02_judgment_traps.md", "F09"),
        ("이미 고친 거 아냐", "devel 파일:라인", "02_judgment_traps.md", "F14"),
        ("한 파일만 보고 단정", "N중 M", "02_judgment_traps.md", "F15"),
        ("이 가설이 원인이야", "구현해서 기각", "02_judgment_traps.md", "F16"),
        ("K-Startup 양식", "J01 set-cell", "17_journeys.md", "F13"),
        ("기안문 만들어", "J03 별지", "17_journeys.md", "F02"),
        ("왕복이 무손실이야", "J04 후 J07", "02_judgment_traps.md", "F09"),
        ("서울시 공개 서식", "J07 ZIP", "17_journeys.md", "F09"),
        ("언어이해 8쪽 머리말", "J06 provenance", "05_hangul_pdf_provenance.md", "F03"),
        ("fidelity_compare 쓰는 법", "README 복제", "12_fidelity_compare.md", "F03"),
        ("--text-only 로 전수", "Chrome 없이", "12_fidelity_compare.md", "F06"),
        ("--layout-ledger", "square_wrap 후보", "12_fidelity_compare.md", "F06"),
        ("새 비교 명령 만들어", "거부", "24_existing_cli.md", "F12"),
        ("gym 과제로 바꿔", "거부", "01_playbook_authority.md", "F01"),
        ("DocumentCore 고쳐", "거부. 별도 PR", "13_issue_template.md", "F12"),
        ("두 번째 채점표", "거부. playbook 만", "01_playbook_authority.md", "F01"),
        ("samples/ 를 전수 스윕", "카탈로그 실물이 우선", "03_journey_selection.md", "F01"),
        ("한컴 버전 뭐로 찍었어", "provenance 키", "05_hangul_pdf_provenance.md", "F03"),
        ("폰트 기록해야 해", "fonts + RHWP_FONT_PATH_DIR", "05_hangul_pdf_provenance.md", "F03"),
        ("중간 info 만 보고 끝", "최종 산출물", "07_run_to_final.md", "F05"),
        ("증상만 이슈에 적어", "3필수", "13_issue_template.md", "F11"),
        ("가상 데이터로 작성", "허용. 접수 금지", "14_no_filing.md", "F13"),
        ("실명인증 통과시켜", "거부", "14_no_filing.md", "F13"),
        ("cp949 가 버그야", "아님", "15_utf8_console.md", "F10"),
        ("검출 94% 를 손실로", "과장 금지", "02_judgment_traps.md", "F15"),
        ("엔트리 수 같으니 무손실", "이름 집합", "02_judgment_traps.md", "F09"),
        ("render-diff PASS 면 한컴과 같다", "아님", "06_self_consistency_limit.md", "F04"),
        ("시각 최종 판정은 누가", "작업지시자/maintainer", "08_pixel_visual.md", "F06"),
        ("문자 멀티셋으로 이슈 확정", "후보만", "09_text_multiset.md", "F06"),
        ("재독 100% 면 기계 확정", "이슈화 가능", "10_reread_values.md", "F11"),
        ("exit 3 이 실패야", "판정은 데이터", "11_exit_json_contract.md", "F11"),
        ("playbook 에 예시 추가", "4단 구조", "01_playbook_authority.md", "F01"),
        ("다음 여정 후보", "카탈로그", "17_journeys.md", "F01"),
        ("46칸 전부 채워", "J08", "17_journeys.md", "F05"),
        ("근거 조항만 렌더", "J09 -p", "17_journeys.md", "F02"),
        ("아카이브 대장", "J10 + #3407", "17_journeys.md", "F11"),
        ("시행문 서식", "J11", "17_journeys.md", "F02"),
        ("PII 마스킹", "J14 inspect", "17_journeys.md", "F13"),
        ("형식 보존 전수", "J16 #3383", "17_journeys.md", "F09"),
        ("비교 시트 밖으로", "--out-dir", "12_fidelity_compare.md", "F03"),
        ("등록 키 없는 쌍", "--source --reference-pdf", "12_fidelity_compare.md", "F03"),
        ("run-state 누락", "종료 코드 비0", "12_fidelity_compare.md", "F05"),
        ("사람 감사 큐", "page-boundary TSV", "08_pixel_visual.md", "F06"),
        ("수정 PR 열어", "contributor 로 인계", "21_handoff.md", "F12"),
        ("이웃 스킬 고쳐", "금지", "21_handoff.md", "F12"),
        ("함정 4개 먼저", "02장", "02_judgment_traps.md", "F14"),
        ("코드 경로 찾아", "Grep/Read 파일:라인", "13_issue_template.md", "F11"),
        ("음성 결과도 남겨", "함정 4", "02_judgment_traps.md", "F16"),
        ("반례 수 적어", "함정 3", "02_judgment_traps.md", "F15"),
        ("한컴을 절대 오라클로", "금지. provenance", "05_hangul_pdf_provenance.md", "F03"),
        ("워크트리에 시트 남김", "--out-dir 외부", "12_fidelity_compare.md", "F03"),
        ("samples 동반 PDF 만", "provenance 전엔 참고", "05_hangul_pdf_provenance.md", "F03"),
        ("제출용 PDF 만들어", "export-pdf. 접수 금지", "07_run_to_final.md", "F13"),
        ("누름틀 0 인데", "set-cell 대상", "07_run_to_final.md", "F05"),
        ("파란 안내문 스타일", "#3391 검정 요건", "10_reread_values.md", "F11"),
        ("체크박스 못 바꿔", "#3395 글머리표", "13_issue_template.md", "F11"),
        ("값이 침묵 유실", "#3358", "10_reread_values.md", "F11"),
        ("PUA tofu", "#3385", "09_text_multiset.md", "F08"),
        ("불법 XML", "#3382", "11_exit_json_contract.md", "F11"),
        ("빈 누름틀 인쇄", "#3375", "08_pixel_visual.md", "F06"),
        ("기안문 ingest 격차", "#3372", "03_journey_selection.md", "F02"),
        ("tabItem 절반", "#3551", "02_judgment_traps.md", "F09"),
        ("OLE 상쇄", "#3557", "02_judgment_traps.md", "F09"),
        ("char_shapes -2", "#3518", "02_judgment_traps.md", "F16"),
        ("머리말 홀수형", "#3402", "05_hangul_pdf_provenance.md", "F03"),
        ("search 총량 은폐", "#3353", "11_exit_json_contract.md", "F11"),
        ("옵션 선행 파싱", "#3349", "11_exit_json_contract.md", "F11"),
        ("edit 형식 미보존", "#3383", "17_journeys.md", "F09"),
        ("여백 왕복 표류", "#3368", "02_judgment_traps.md", "F15"),
        ("secd/cold 뒤집힘", "#3367", "11_exit_json_contract.md", "F11"),
        ("thumbnail 계약", "#3366", "11_exit_json_contract.md", "F11"),
        ("export-png 부재", "#3357", "11_exit_json_contract.md", "F11"),
        ("텍스트 상자 ingest", "#3355", "11_exit_json_contract.md", "F11"),
        ("set-cell 부재였던 격차", "#3381", "07_run_to_final.md", "F05"),
        ("비교 분류표 보여줘", "20장", "20_classification.md", "F06"),
        ("이슈 템플릿 필드", "13장", "13_issue_template.md", "F11"),
        ("정지 규칙 표", "SKILL + 22장", "22_failure_signals.md", "F01"),
        ("게이트 레시피", "23장", "23_gate_recipes.md", "F09"),
        ("기존 CLI 화이트리스트", "24장", "24_existing_cli.md", "F12"),
        ("의도 행렬", "19장", "19_intent_matrix.md", "F01"),
        ("트레이스 T01", "18장", "18_worked_traces.md", "F05"),
        ("UTF-8 만 비교", "15장", "15_utf8_console.md", "F10"),
        ("접수 경계", "14장", "14_no_filing.md", "F13"),
        ("자기 일관성 한계 문구", "06장", "06_self_consistency_limit.md", "F04"),
        ("픽셀은 랭킹이다", "08장", "08_pixel_visual.md", "F06"),
        ("쪽별 멀티셋 정규화", "NFC·공백", "09_text_multiset.md", "F06"),
        ("재독 기계 판정", "10장", "10_reread_values.md", "F11"),
        ("JSON 계약 기계 판정", "11장", "11_exit_json_contract.md", "F11"),
        ("권위는 playbook", "01장", "01_playbook_authority.md", "F01"),
        ("판단 트리 상자", "00장", "00_tree.md", "F01"),
        ("함정 P12 두 번째 루브릭", "16장", "16_pitfalls.md", "F01"),
        ("인계 form-fill", "21장", "21_handoff.md", "F12"),
        ("인계 visual-regression", "21장", "21_handoff.md", "F04"),
        ("인계 contributor", "21장", "21_handoff.md", "F12"),
        ("능력 등록 CAP-5324", "working 문서", "01_playbook_authority.md", "F01"),
        ("포인터 스킬", ".claude/skills/rhwp-bug-hunter", "01_playbook_authority.md", "F01"),
        ("에이전트 정의", ".claude/agents/bug-hunter.md", "01_playbook_authority.md", "F01"),
        ("작업 기록", "mydocs/working/agent_bug_hunter.md", "01_playbook_authority.md", "F01"),
        ("한컴 Creator 메타", "Hwp 2022 …", "05_hangul_pdf_provenance.md", "F03"),
        ("A3 841×1190", "leet provenance", "05_hangul_pdf_provenance.md", "F03"),
        ("가상 기업 데이터", "(주)시연용가상기업", "14_no_filing.md", "F13"),
        ("사업자번호 전부 0", "허위 신청 방지", "14_no_filing.md", "F13"),
        ("제출 직전이 경계", "export-pdf 까지", "07_run_to_final.md", "F13"),
        ("4단 예시 구조", "문제/실물/흐름/방식", "01_playbook_authority.md", "F01"),
        ("사용자 가치 순", "카탈로그", "03_journey_selection.md", "F01"),
        ("samples 보다 지금 배포 실물", "playbook 요약 2", "03_journey_selection.md", "F01"),
        ("통과해도 멈추지 않는다", "playbook 요약 3", "02_judgment_traps.md", "F09"),
        ("재현·실측 표·반례 수", "playbook 요약 4", "13_issue_template.md", "F11"),
        ("전/후 실측 표는 수정 PR", "playbook 요약 5", "21_handoff.md", "F12"),
        ("시각 거버넌스", "최종 판정은 maintainer", "08_pixel_visual.md", "F06"),
        ("폰트 대체 픽셀 흔들림", "문자 수로 보완", "09_text_multiset.md", "F06"),
        ("숨김 대상 과잉", "svg_only", "09_text_multiset.md", "F07"),
        ("쪽번호 채움점 소실", "reference_only", "09_text_multiset.md", "F06"),
        ("배치·줄바꿈은 픽셀", "멀티셋은 순서 무시", "08_pixel_visual.md", "F06"),
        ("IR 없는 XML 래퍼", "함정 1", "02_judgment_traps.md", "F09"),
        ("CLOSED PR 이 반영됐을 수 있음", "함정 2 cherry-pick", "02_judgment_traps.md", "F14"),
        ("증상 대신 IR 차이", "함정 4", "02_judgment_traps.md", "F16"),
        ("--verify-pages exit 4 우회", "--verify", "11_exit_json_contract.md", "F09"),
        ("헤더 6737B 상수 감소", "#3551 신호", "02_judgment_traps.md", "F09"),
        ("default == case × 2", "구조 소실이지 데이터 손실 아님", "02_judgment_traps.md", "F15"),
        ("정규화 fwSpace/nbSpace", "검출 ≠ 손실", "20_classification.md", "F15"),
        ("값 손실이 아니면 아니라고 쓴다", "판정의 일부", "13_issue_template.md", "F11"),
        ("헌팅이지 픽스가 아니다", "F12", "13_issue_template.md", "F12"),
        ("기존 CLI 만", "24장 화이트리스트", "24_existing_cli.md", "F12"),
        ("fidelity_compare 는 도구", "rhwp 하위명령 아님", "12_fidelity_compare.md", "F12"),
        ("Windows python 경로", "venv\\Scripts\\python.exe", "12_fidelity_compare.md", "F03"),
        ("RHWP_BIN CHROME_BIN", "자동 탐색 실패 시", "12_fidelity_compare.md", "F03"),
        ("release-test 프로파일", "비교 전 빌드", "12_fidelity_compare.md", "F03"),
        ("worktree 청결", "--out-dir /tmp", "12_fidelity_compare.md", "F03"),
    ]
    rows = []
    for i, (utt, cmd, ref, stop) in enumerate(seeds, start=1):
        rows.append(
            {
                "id": f"I{i:03d}",
                "utterance": utt,
                "command": cmd,
                "reference": ref,
                "stop": stop,
                "notGym": True,
            }
        )
    return rows


def traces() -> list[dict]:
    base = [
        {
            "id": "T01",
            "journey": "J01",
            "title": "K-Startup 표 양식 채움 끝까지",
            "commands": [
                "rhwp info --json 양식.hwp",
                "rhwp fields --json 양식.hwp",
                "rhwp export-tables --json 양식.hwp",
                "rhwp edit set-cell 양식.hwp --table 5 --row 0 --col 1 --text 시연용 -o 작성본.hwp --json",
                "rhwp export-tables --json 작성본.hwp",
                "rhwp export-pdf 작성본.hwp -o 제출용.pdf",
            ],
            "stop": "F13",
            "filed": False,
            "note": "실제 접수 없음. 가상 데이터.",
        },
        {
            "id": "T02",
            "journey": "J02",
            "title": "plan 0–34 fidelity_compare",
            "commands": [
                "venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 34 --out-dir /tmp/rhwp-fidelity-plan",
                "sort -t $'\\t' -k2,2nr -k3,3nr /tmp/rhwp-fidelity-plan/text-report.tsv | head",
            ],
            "stop": "F06",
            "filed": False,
            "note": "픽셀 상위 + 문자 소실 교집합 후 사람 감사.",
        },
        {
            "id": "T03",
            "journey": "J07",
            "title": "서울시 HWPX verify 통과 후 ZIP 이름 집합",
            "commands": [
                "rhwp export-hwpx 서식.hwpx out.hwpx --verify --verify-pages",
                "python zip_name_set_compare.py 서식.hwpx out.hwpx",
            ],
            "stop": "F09",
            "filed": False,
            "note": "4/4 통과해도 멈추지 않음.",
        },
        {
            "id": "T04",
            "journey": "J17",
            "title": "정답지 없는 form-01 자기 일관성",
            "commands": ["rhwp render-diff samples/form-01.hwp --via hwpx"],
            "stop": "F04",
            "filed": False,
            "note": "한계를 기록. 한컴 충실도 이슈 금지.",
        },
        {
            "id": "T05",
            "journey": "J06",
            "title": "언어이해 8쪽 provenance 기록 후 대조",
            "commands": [
                "기록: Creator Hwp 2022 12.0.0.4426 Producer Hancom PDF 1.3.0.550",
                "venv/bin/python tools/fidelity_compare/fidelity_compare.py 7 7 --source samples/21_언어_기출_편집가능본.hwp --reference-pdf pdf/21_언어_기출_편집가능본-2022.pdf --out-dir /tmp/rhwp-fidelity-leet-p8",
            ],
            "stop": "F03",
            "filed": False,
            "note": "페이지 0 기준이므로 8쪽은 -p 7.",
        },
    ]
    extras = []
    for i in range(6, 41):
        extras.append(
            {
                "id": f"T{i:02d}",
                "journey": f"J{((i - 1) % 20) + 1:02d}",
                "title": f"재현 트레이스 {i:02d} — playbook 실행 계약",
                "commands": [
                    "rhwp info --json <파일>",
                    "정답지 provenance 기록 또는 F04 한계",
                    "rhwp export-svg <파일> -o svg/",
                    "최종 산출물까지 기존 CLI",
                ],
                "stop": ["F02", "F04", "F05", "F06", "F09", "F11"][i % 6],
                "filed": False,
                "note": "증상만 남기지 않음. 파일:라인은 devel 에서.",
            }
        )
    return base + extras


def issue_templates() -> list[dict]:
    return [
        {
            "id": "IT01",
            "title": "[헌팅] <한 줄 격차>",
            "fields": [f["id"] for f in ISSUE_TEMPLATE_FIELDS],
            "body": [
                "## 재현 명령",
                "```bash",
                "<복붙 가능한 명령>",
                "```",
                "## 코드 경로",
                "`path/to/file.rs:LINE` (devel HEAD `<sha>` 에서 확인)",
                "## 정답지 대비 근거",
                "- 종류: 한컴 PDF / 법정 서식 / 제출 요건 / 재독 / 계약",
                "- provenance: 도구·버전·경로·폰트",
                "- 분류: 소실 / 과잉 / 치환 / 재독 / 계약 / 픽셀 후보",
                "## 한계",
                "- 이 오라클이 안 보는 축",
                "## 수정",
                "이 이슈는 헌팅 산출이다. 패치는 별도 PR.",
            ],
        },
        {
            "id": "IT02",
            "title": "[헌팅] 정답지 없음 — 자기 일관성만",
            "fields": ["repro", "limitations", "notAFix"],
            "body": [
                "독립 기준을 확보하지 못했다.",
                "수행: `rhwp render-diff <파일> --via hwpx`",
                "한컴 충실도 결함으로 승격하지 않는다 (F04).",
            ],
        },
        {
            "id": "IT03",
            "title": "[헌팅] 음성 결과 — 가설 기각",
            "fields": ["repro", "codePath", "limitations"],
            "body": [
                "가설: …",
                "구현/재현 결과: 증상 그대로 또는 사라짐",
                "다음 사람이 같은 길을 다시 파지 않도록 남긴다 (함정 4).",
            ],
        },
    ]


def dump(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    text = json.dumps(obj, ensure_ascii=False, indent=2) + "\n"
    path.write_text(text, encoding="utf-8", newline="\n")


def write_text(path: Path, body: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not body.endswith("\n"):
        body += "\n"
    path.write_text(body.replace("\r\n", "\n"), encoding="utf-8", newline="\n")


def envelope(name: str, **fields) -> dict:
    base = {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "skill": "bug-hunter",
        "notGym": True,
        "noNewCli": True,
    }
    base.update(fields)
    return base


def skill_index() -> dict:
    return envelope(
        "skill_index",
        skill="bug-hunter",
        claudePointer=".claude/skills/rhwp-bug-hunter/SKILL.md",
        playbook=PLAYBOOK,
        secondRubricForbidden=True,
        noNewEditLogic=True,
        huntingNotFix=True,
        forbiddenSkillsTouch=[
            "rhwp-onboarding",
            "rhwp-mcp-session",
            "rhwp-safe-edit",
            "rhwp-provenance",
            "rhwp-doc-triage",
            "rhwp-form-fill",
            "rhwp-visual-regression",
        ],
        forbiddenTrees=["gym/"],
        references=REQUIRED_REFS,
        examples=REQUIRED_EXAMPLES,
        coreTopics=[
            "playbook is the only rubric",
            "journey selection with ground truth",
            "Hangul PDF / legal form / submission first",
            "provenance tool/version/path/fonts",
            "self-consistency limit when no baseline",
            "run to final artifact",
            "pixel/visual candidate",
            "PDF text vs SVG text per-page multiset",
            "missing=loss extra=excess both=substitution",
            "reread recorded values",
            "exit codes and JSON contracts",
            "finding = issue with repro + file:line + ground truth",
            "never automate filing/login/identity",
            "UTF-8 file compare only",
            "tools/fidelity_compare",
        ],
        allowedCommands=ALLOWED_COMMANDS,
        inventedCommandsForbidden=INVENTED_COMMANDS,
        authority=[
            PLAYBOOK,
            "tools/fidelity_compare/README.md",
            "mydocs/manual/cli_commands.md",
            "mydocs/manual/verification/visual_verification_governance.md",
        ],
    )


def tree() -> dict:
    return envelope(
        "tree",
        ladder=[
            "select-journey",
            "obtain-ground-truth",
            "record-provenance-or-limit",
            "run-to-final-artifact",
            "compare-axes",
            "file-issue",
        ],
        livingVerbs=["rhwp existing CLI", FIDELITY],
        playbook=PLAYBOOK,
        secondRubricForbidden=True,
        notGym=True,
        noNewCli=True,
        huntingNotFix=True,
        aaIsNotHangulFidelity=True,
        coreReuse=[
            "tools/fidelity_compare/fidelity_compare.py",
            "rhwp export-svg",
            "rhwp export-png",
            "rhwp export-pdf",
            "rhwp render-diff",
            "rhwp ir-diff",
            "rhwp dump",
            "rhwp info",
            "rhwp fields",
            "rhwp export-tables",
        ],
    )


def classification_tsv() -> str:
    header = "id\tobservation\tlabel_ko\tlabel_en\tfinal\taxis\tissue_ready\n"
    rows = []
    for c in CLASSIFICATION:
        rows.append(
            "\t".join(
                [
                    c["id"],
                    c["observation"],
                    c["labelKo"],
                    c["labelEn"],
                    "yes" if c["final"] else "no",
                    c["axis"],
                    "yes" if c["issueReady"] else "no",
                ]
            )
        )
    return header + "\n".join(rows) + "\n"


def write_all() -> None:
    journeys = playbook_journeys()
    intents = intent_rows()
    trs = traces()
    dump(FIXT / "skill_index.json", skill_index())
    dump(FIXT / "tree.json", tree())
    dump(
        FIXT / "stop_rules.json",
        envelope(
            "stop_rules",
            rules=[{"id": a, "when": b, "action": c} for a, b, c in STOP_RULES],
        ),
    )
    dump(
        FIXT / "command_ladder.json",
        envelope("command_ladder", commands=COMMANDS),
    )
    dump(
        FIXT / "samples.json",
        envelope("samples", samples=SAMPLES),
    )
    dump(
        FIXT / "journeys.json",
        envelope("journeys", count=len(journeys), journeys=journeys),
    )
    dump(
        FIXT / "intent_matrix.json",
        envelope("intent_matrix", count=len(intents), intents=intents),
    )
    dump(
        FIXT / "pitfalls.json",
        envelope("pitfalls", pitfalls=PITFALLS),
    )
    dump(
        FIXT / "handoff.json",
        envelope("handoff", handoff=HANDOFF),
    )
    dump(
        FIXT / "classification.json",
        envelope(
            "classification",
            count=len(CLASSIFICATION),
            rules=CLASSIFICATION,
            missing="loss",
            extra="excess",
            both="substitution",
        ),
    )
    dump(
        FIXT / "issue_templates.json",
        envelope(
            "issue_templates",
            requiredFields=["repro", "codePath", "groundTruth"],
            fields=ISSUE_TEMPLATE_FIELDS,
            templates=issue_templates(),
        ),
    )
    dump(
        FIXT / "provenance_keys.json",
        envelope("provenance_keys", keys=PROVENANCE_KEYS),
    )
    dump(
        FIXT / "envelope_keys.json",
        envelope(
            "envelope_keys",
            commands={
                "info": {"json": True},
                "fields": {"json": True},
                "export-tables": {"json": True},
                "ir-diff": {"diffExit": 3, "textDiffExit": 0},
                "render-diff": {"selfIsNotHangulFidelity": True},
                "export-hwpx": {"verifyDoesNotProveZip": True},
            },
        ),
    )
    dump(
        FIXT / "traces_index.json",
        envelope("traces_index", ids=[t["id"] for t in trs], count=len(trs)),
    )
    for t in trs:
        dump(FIXT / "traces" / f"{t['id']}.json", envelope(t["id"], **t))

    dump(
        FIXT / "envelopes" / "text_report_loss.json",
        envelope(
            "text_report_loss",
            page=3,
            reference_only=["쪽", "3"],
            svg_only=[],
            class_="loss",
        ),
    )
    dump(
        FIXT / "envelopes" / "text_report_excess.json",
        envelope(
            "text_report_excess",
            page=5,
            reference_only=[],
            svg_only=["안", "내", "문"],
            class_="excess",
        ),
    )
    dump(
        FIXT / "envelopes" / "text_report_substitution.json",
        envelope(
            "text_report_substitution",
            page=8,
            reference_only=["\uf000"],
            svg_only=["\ufffd"],
            class_="substitution",
        ),
    )
    dump(
        FIXT / "envelopes" / "reread_mismatch.json",
        envelope(
            "reread_mismatch",
            written="시연용가상기업",
            reread="(빈 칸)",
            class_="reread",
            issueReady=True,
        ),
    )
    dump(
        FIXT / "envelopes" / "verify_pass_zip_loss.json",
        envelope(
            "verify_pass_zip_loss",
            verifyPages="4/4",
            entryCountSame=True,
            missingNames=["BinData/ole1.ole"],
            addedNames=["Preview/PrvImage.png"],
            class_="zip-loss",
        ),
    )
    dump(
        FIXT / "envelopes" / "console_mojibake.json",
        envelope(
            "console_mojibake",
            console="????",
            utf8File="한글",
            defect=False,
            class_="not-a-defect",
        ),
    )

    write_text(FIXT / "tsv" / "classification.tsv", classification_tsv())
    write_text(
        FIXT / "tsv" / "text_report_sample.tsv",
        "page\treference_only\tsvg_only\tclass\n"
        "3\t2\t0\tloss\n"
        "5\t0\t3\texcess\n"
        "8\t1\t1\tsubstitution\n",
    )
    write_text(
        FIXT / "tsv" / "provenance_sample.tsv",
        "key\tvalue\n"
        "tool\tHwp 2022\n"
        "version\t12.0.0.4426\n"
        "outputPath\t인쇄>PDF\n"
        "fonts\t함초롬바탕,HY신명조\n"
        "creator\tHwp 2022 12.0.0.4426\n"
        "producer\tHancom PDF 1.3.0.550\n"
        "paper\tA3 841x1190pt\n",
    )
    write_text(
        FIXT / "issue_template.md",
        "# 헌팅 이슈 템플릿 (IT01)\n\n"
        "필수: 재현 명령 · 코드 경로(파일:라인) · 정답지 대비 근거.\n"
        "증상만 있는 초안은 올리지 않는다 (F11).\n\n"
        + "\n".join(issue_templates()[0]["body"])
        + "\n",
    )
    write_text(
        FIXT / "transcripts" / "kstartup_reread.txt",
        "fields: 0\n"
        "tables: 39\n"
        "set-cell table=5 row=0 col=1 written=시연용가상기업\n"
        "reread table=5 row=0 col=1 value=시연용가상기업\n"
        "match: true\n"
        "filing: refused\n",
    )
    write_text(
        FIXT / "transcripts" / "verify_then_zip.txt",
        "export-hwpx --verify --verify-pages: 4/4 exit 0\n"
        "zip name set missing: Contents/header.xml size 6737B shrink on 3 docs\n"
        "tabItem 480->240\n"
        "do not stop at oracle pass\n",
    )
    write_text(
        FIXT / "transcripts" / "self_only_limit.txt",
        "ground truth: none\n"
        "rhwp render-diff samples/form-01.hwp --via hwpx\n"
        "status: PASS\n"
        "limit: self-consistency only. not Hangul fidelity. F04\n",
    )


if __name__ == "__main__":
    write_all()
    print("wrote", FIXT)
