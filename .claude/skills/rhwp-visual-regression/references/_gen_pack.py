#!/usr/bin/env python3
"""[#5312] rhwp-visual-regression 레퍼런스·픽스처 생성기.

새 CLI 를 발명하지 않는다. 명령·봉투·종료 코드는 cli_commands.md 와
src/diagnostics/render_geom_diff.rs · ir_diff_command.md 가 이미 고정한
표면만 복제한다. gym 경로가 아니다.
"""

from __future__ import annotations

import json
from pathlib import Path

SKILL = Path(__file__).resolve().parents[1]
REF = SKILL / "references"
EX = SKILL / "examples"
FIXT = SKILL / "fixtures"

ISSUE = 5312
SCHEMA = "1.0"

# 기존 샘플. 새 HWP 바이너리를 만들지 않는다.
SAMPLES = {
    "form01": {
        "path": "samples/form-01.hwp",
        "pages": 1,
        "note": "레시피 06 실측 표본. 자기 라운드트립 PASS, 채움 후 STRUCT.",
    },
    "form02": {
        "path": "samples/form-02.hwp",
        "pages": 1,
        "note": "배치 실측 형제. --via hwpx 도 PASS.",
    },
    "field01": {
        "path": "samples/field-01.hwp",
        "pages": 3,
        "note": "누름틀 3쪽. fill-fields 전후 비교 대상.",
    },
    "hwp3": {
        "path": "samples/hwp3-sample.hwp",
        "pages": 1,
        "note": "HWP3. ir-diff 자기비교 identical.",
    },
    "sosueop": {
        "path": "samples/SO-SUEOP.hwp",
        "pages": None,
        "note": "ir-diff 서로 다른 문서 쌍. 차이 = exit 3.",
    },
}

COMMANDS = [
    {
        "id": "render-diff-self",
        "argv": ["render-diff", "<파일>", "--via", "hwpx"],
        "writes": False,
        "when": "포맷 왕복이 레이아웃을 깨뜨리는지",
    },
    {
        "id": "render-diff-self-hwp",
        "argv": ["render-diff", "<파일>", "--via", "hwp"],
        "writes": False,
        "when": "HWP 어댑터 경로 자기 라운드트립",
    },
    {
        "id": "render-diff-pair",
        "argv": ["render-diff", "<전>", "<후>", "--max-disp", "1.0"],
        "writes": False,
        "when": "편집 전 vs 후 두 파일",
    },
    {
        "id": "render-diff-batch",
        "argv": ["render-diff", "--batch", "<폴더>", "-o", "<출력>"],
        "writes": True,
        "when": "폴더 전수 → geom_inventory.tsv",
        "artifact": "geom_inventory.tsv",
    },
    {
        "id": "render-diff-json",
        "argv": ["render-diff", "<A>", "<B>", "--json"],
        "writes": False,
        "when": "기계 봉투. 하드 실패는 exit 3",
    },
    {
        "id": "ir-diff-json",
        "argv": ["ir-diff", "<A>", "<B>", "--json"],
        "writes": False,
        "when": "IR 구조 차이. 차이 = exit 3",
    },
    {
        "id": "ir-diff-text",
        "argv": ["ir-diff", "<A>", "<B>"],
        "writes": False,
        "when": "사람용 텍스트. 차이가 있어도 exit 0",
    },
    {
        "id": "thumbnail",
        "argv": ["thumbnail", "<파일>", "--data-uri"],
        "writes": False,
        "when": "저장 시점 PrvImage. 재렌더가 아님",
    },
    {
        "id": "export-png",
        "argv": ["export-png", "<파일>", "-p", "0"],
        "writes": True,
        "when": "현재 IR 을 재렌더한 PNG",
    },
    {
        "id": "export-render-tree",
        "argv": ["export-render-tree", "<파일>", "-p", "0"],
        "writes": False,
        "when": "정밀 bbox JSON (선택 후속)",
    },
]

STOP_RULES = [
    ("F01", "status PASS", "끝. 다음 단 금지"),
    ("F02", "A==A 가 PASS 아님", "도구 비결정성. 중단"),
    ("F03", "STRUCT + 경로가 편집 위치", "정상. 실패로 읽지 않음"),
    ("F04", "STRUCT + 경로가 무관", "진짜 회귀"),
    ("F05", "PAGE_MISMATCH", "dump-pages 로 좁힘"),
    ("F06", "OVER", "worst_page 로 좁힘"),
    ("F07", "LOAD_FAIL", "info 로 그 파일만"),
    ("F08", "ir-diff --json exit 3", "차이 검출은 데이터"),
    ("F09", "눈 검증 필요", "export-png. thumbnail 은 저장본"),
    ("F10", "배치 TSV 혼합", "행별 status 로 격리"),
    ("F11", "질문이 이미 답", "다음 단 금지"),
    ("F12", "WARN_TEXTRUN", "하드 실패 아님"),
]

STATUSES = [
    {
        "id": "PASS",
        "hard": False,
        "textExit": 0,
        "jsonExit": 0,
        "meaning": "쪽 수 동일, 변위 ≤ 임계, 하드 구조 불일치 없음",
    },
    {
        "id": "WARN_TEXTRUN",
        "hard": False,
        "textExit": 0,
        "jsonExit": 0,
        "meaning": "TextRun ±1 만 (#1773). 변위도 임계 이내",
    },
    {
        "id": "OVER",
        "hard": True,
        "textExit": 1,
        "jsonExit": 3,
        "meaning": "구조 동일, maxDisp > --max-disp",
    },
    {
        "id": "STRUCT_MISMATCH",
        "hard": True,
        "textExit": 1,
        "jsonExit": 3,
        "meaning": "노드 삽입·삭제. 임계와 무관. 경로부터 읽는다",
    },
    {
        "id": "PAGE_MISMATCH",
        "hard": True,
        "textExit": 1,
        "jsonExit": 3,
        "meaning": "pageCountA != pageCountB",
    },
    {
        "id": "LOAD_FAIL",
        "hard": True,
        "textExit": 1,
        "jsonExit": 1,
        "meaning": "파일을 못 열었음. 측정 실패이지 회귀 검출이 아님",
    },
]

PITFALLS = [
    {
        "id": "P01",
        "trap": "STRUCT_MISMATCH 를 경로도 안 읽고 롤백",
        "signal": "status: STRUCT_MISMATCH, exit 1 또는 3",
        "fix": "변위 노드 경로가 편집한 필드 위치와 일치하는지 읽는다",
    },
    {
        "id": "P02",
        "trap": "thumbnail 을 편집 후 재렌더로 착각",
        "signal": "채운 문서의 썸네일이 빈 서식과 같다",
        "fix": "thumbnail 은 PrvImage. 눈 검증은 export-png",
    },
    {
        "id": "P03",
        "trap": "A==A 실패를 문서 회귀로 오진",
        "signal": "같은 파일을 두 번 비교했는데 PASS 가 아님",
        "fix": "렌더 파이프라인 비결정성. 문서보다 도구가 먼저",
    },
    {
        "id": "P04",
        "trap": "--max-disp 를 키워 STRUCT 를 숨기려 함",
        "signal": "임계 100px 인데도 STRUCT_MISMATCH",
        "fix": "구조 불일치는 임계와 무관하게 항상 플래그",
    },
    {
        "id": "P05",
        "trap": "render-diff 텍스트 모드를 --json 종료 코드로 읽음",
        "signal": "차이인데 exit 1 (텍스트) vs 3 (json)",
        "fix": "사람 모드는 1, --json 만 3. ir-diff 도 같은 축",
    },
    {
        "id": "P06",
        "trap": "ir-diff 텍스트 모드 차이를 실패로 읽음",
        "signal": "텍스트 비교는 차이가 있어도 exit 0",
        "fix": "게이트는 반드시 --json. 차이 = exit 3",
    },
    {
        "id": "P07",
        "trap": "페이지를 1부터 셈",
        "signal": "-p 1 이 두 번째 쪽",
        "fix": "0 기준. 한컴/PDF 표기(1부터)와 혼동 금지",
    },
    {
        "id": "P08",
        "trap": "배치 요약 줄만 보고 통과",
        "signal": "STRUCT 1건이 있는데 총 파일 수만 봄",
        "fix": "geom_inventory.tsv 행별 status",
    },
    {
        "id": "P09",
        "trap": "자기 라운드트립 PASS 를 한컴 충실도로 읽음",
        "signal": "내부 왕복은 맞는데 한컴 PDF 와 다름",
        "fix": "render-diff 는 내부 회귀 방지. 한컴은 수동 검증",
    },
    {
        "id": "P10",
        "trap": "래스터 픽셀 diff 로 착각",
        "signal": "색만 바뀌었는데 PASS",
        "fix": "render tree 노드 위치·구조 비교. 색은 export-png",
    },
    {
        "id": "P11",
        "trap": "--batch 폴더 경로 오타",
        "signal": "오류: 폴더 읽기 실패, exit 2",
        "fix": "단건 파일 없음은 exit 1 과 구분",
    },
    {
        "id": "P12",
        "trap": "새 비교 하위명령을 만들어 쓴다",
        "signal": "알 수 없는 하위명령, exit 2",
        "fix": "기존 네 명령만. 별칭을 만들지 않는다",
    },
    {
        "id": "P13",
        "trap": "LOAD_FAIL 을 회귀 검출(exit 3)로 접음",
        "signal": "배치 --json 에서 error 키가 있는 행",
        "fix": "로드 실패는 측정 실패(exit 1). 회귀가 아님",
    },
    {
        "id": "P14",
        "trap": "같은 글자 수 메일머지 행을 서로 비교하지 않음",
        "signal": "특정 값에서만 레이아웃이 깨짐",
        "fix": "산출물끼리도 render-diff. 글자 수 같으면 PASS 가 정상",
    },
]

HANDOFF = [
    {
        "when": "누름틀을 채운 뒤 레이아웃을 본다",
        "to": "rhwp-form-fill 에서 돌아와 이 스킬",
        "cmd": "render-diff <빈서식> <채움산출>",
    },
    {
        "when": "표 CSV 왕복 후 칸 너비",
        "to": "rhwp-table-exchange",
        "cmd": "render-diff 전후 후 필요하면 export-png",
    },
    {
        "when": "원본을 계획서로 여러 번 고침",
        "to": "rhwp-safe-edit",
        "cmd": "run --verify 후 render-diff",
    },
    {
        "when": "배포 전 숨은 글·주입",
        "to": "rhwp-security-sweep",
        "cmd": "inspect. 비교 전에 먼저",
    },
    {
        "when": "문서가 뭔지만",
        "to": "rhwp-doc-triage",
        "cmd": "info / explain. 비교하지 않음",
    },
    {
        "when": "작업 영수증이 필요",
        "to": "rhwp-work-receipt",
        "cmd": "replay. 이 스킬은 레이아웃만",
    },
]

# 레시피 06 실측 전사.
TRANSCRIPTS = {
    "self_form01": """페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
임계 초과 페이지: 0 / 구조 불일치 페이지: 0 (임계 1.00px)
status: PASS
""",
    "pair_fill": """페이지 수: A=1 B=1
최대 변위: 495.93 px (page 0)
임계 초과 페이지: 1 / 구조 불일치 페이지: 1 (임계 1.00px)
  page   0: max= 495.93 mean= 13.40 nodes=39/37  [STRUCT]
       495.93px  Page/Body2/Column0/TextLine10/TextRun0
         0.00px  Page
         0.00px  Page/PageBg0
      Δ TextRun: 15→13 (-2)
status: STRUCT_MISMATCH
""",
    "pair_fill_tight": """페이지 수: A=1 B=1
최대 변위: 495.93 px (page 0)
임계 초과 페이지: 1 / 구조 불일치 페이지: 1 (임계 0.05px)
  page   0: max= 495.93 mean= 13.40 nodes=39/37  [STRUCT]
       495.93px  Page/Body2/Column0/TextLine10/TextRun0
         0.00px  Page
         0.00px  Page/PageBg0
      Δ TextRun: 15→13 (-2)
status: STRUCT_MISMATCH
""",
    "pair_same_len": """페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
임계 초과 페이지: 0 / 구조 불일치 페이지: 0 (임계 1.00px)
status: PASS
""",
    "aa_determinism": """페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
임계 초과 페이지: 0 / 구조 불일치 페이지: 0 (임계 1.00px)
status: PASS
""",
    "batch_pass": """[           PASS] max_disp=   0.00 struct=0 over=0      5ms  form-01.hwp
[           PASS] max_disp=   0.00 struct=0 over=0      4ms  form-02.hwp

TSV 저장: rd_out\\geom_inventory.tsv

=== render-diff 요약 ===
  총 파일         : 2
  PASS            : 2
  WARN_TEXTRUN    : 0
  OVER            : 0
  STRUCT_MISMATCH : 0
  PAGE_MISMATCH   : 0
  LOAD_FAIL       : 0
  전체 최대 변위  : 0.00 px
""",
}

TSV_PASS = """sample	status	pages_a	pages_b	max_disp	worst_page	struct_pages	over_pages	elapsed_ms	error	struct_delta
form-01.hwp	PASS	1	1	0.000	-	0	0	5		
form-02.hwp	PASS	1	1	0.000	-	0	0	4		
"""

TSV_MIXED = """sample	status	pages_a	pages_b	max_disp	worst_page	struct_pages	over_pages	elapsed_ms	error	struct_delta
form-01.hwp	PASS	1	1	0.000	-	0	0	5		
filled-0001.hwp	STRUCT_MISMATCH	1	1	495.930	0	1	1	12		TextRun:-2
long-replace.hwp	OVER	3	3	279.000	1	0	1	18		
broken.hwp	LOAD_FAIL	0	0	0.000	-	0	0	1	parse failed	
paged.hwp	PAGE_MISMATCH	2	3	0.000	-	0	0	9		
noise.hwp	WARN_TEXTRUN	1	1	0.400	0	1	0	7		TextRun:+1
"""

REQUIRED_REFS = [
    "00_tree.md",
    "01_render_diff_self.md",
    "02_render_diff_two_file.md",
    "03_render_diff_batch.md",
    "04_struct_mismatch.md",
    "05_status_codes.md",
    "06_ir_diff.md",
    "07_thumbnail_vs_png.md",
    "08_determinism.md",
    "09_max_disp.md",
    "10_envelopes.md",
    "11_pitfalls.md",
    "12_journeys.md",
    "13_handoff.md",
    "14_failure_signals.md",
    "15_node_paths.md",
    "16_worked_traces.md",
    "17_intent_matrix.md",
    "18_tsv_schema.md",
    "19_gate_recipes.md",
    "20_exit_codes.md",
    "21_page_mismatch.md",
    "22_load_fail.md",
    "23_over_status.md",
    "24_export_render_tree.md",
    "README.md",
]

REQUIRED_EXAMPLES = [
    "01_self_roundtrip_form01.md",
    "02_self_roundtrip_via_hwp.md",
    "03_two_file_fill.md",
    "04_two_file_same_length.md",
    "05_aa_determinism.md",
    "06_batch_pass_folder.md",
    "07_batch_mixed_status.md",
    "08_struct_intended.md",
    "09_struct_unrelated.md",
    "10_page_mismatch.md",
    "11_load_fail.md",
    "12_over_threshold.md",
    "13_ir_diff_json.md",
    "14_thumbnail_stale.md",
    "15_export_png_rerender.md",
    "16_max_disp_struct_independent.md",
    "17_text_mode_exit1.md",
    "18_json_mode_exit3.md",
    "19_geom_inventory_gate.md",
    "20_warn_textrun.md",
    "README.md",
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


def skill_index() -> dict:
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "skill": "rhwp-visual-regression",
        "notGym": True,
        "noNewCli": True,
        "noNewEditLogic": True,
        "forbiddenSkillsTouch": [
            "rhwp-onboarding",
            "rhwp-mcp-session",
            "rhwp-safe-edit",
            "rhwp-provenance",
            "rhwp-doc-triage",
            "rhwp-form-fill",
        ],
        "forbiddenTrees": ["gym/"],
        "references": REQUIRED_REFS,
        "examples": REQUIRED_EXAMPLES,
        "coreTopics": [
            "render-diff self-roundtrip",
            "render-diff two-file",
            "render-diff --batch geom_inventory.tsv",
            "STRUCT_MISMATCH node path",
            "PAGE_MISMATCH",
            "OVER",
            "LOAD_FAIL",
            "PASS",
            "ir-diff --json exit 3",
            "thumbnail vs export-png",
            "A==A determinism",
            "--max-disp 1.0px",
        ],
        "allowedCommands": [
            "render-diff",
            "ir-diff",
            "thumbnail",
            "export-png",
            "export-svg",
            "export-render-tree",
            "dump-pages",
            "info",
        ],
        "inventedCommandsForbidden": [
            "visual-diff",
            "pixel-diff",
            "layout-diff",
            "screenshot-compare",
            "render-compare",
            "gym-render",
        ],
        "authority": [
            "mydocs/manual/cli_commands.md",
            "mydocs/manual/recipes/06_visual_regression_before_after.md",
            "mydocs/manual/ir_diff_command.md",
            "mydocs/manual/export_png_command.md",
            "src/diagnostics/render_geom_diff.rs",
        ],
    }


def tree() -> dict:
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "notGym": True,
        "noNewCli": True,
        "noNewEditLogic": True,
        "coreReuse": [
            "diagnostics::render_geom_diff::run",
            "ir-diff --json",
            "thumbnail PrvImage",
            "export-png native-skia",
        ],
        "ladder": [
            "render-diff self",
            "render-diff pair",
            "render-diff --batch",
            "ir-diff --json",
            "export-png",
        ],
        "defaultMaxDispPx": 1.0,
        "structIgnoresThreshold": True,
        "aaMustPass": True,
        "thumbnailIsStoredPreview": True,
    }


def stop_rules() -> dict:
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "rules": [{"id": i, "when": w, "action": a} for i, w, a in STOP_RULES],
    }


def status_catalog() -> dict:
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "defaultMaxDispPx": 1.0,
        "hardFailurePredicate": "status not in {PASS, WARN_TEXTRUN}",
        "statuses": STATUSES,
    }


def envelope_keys() -> dict:
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "exitCodes": {
            "0": "성공 또는 차이 없음(텍스트 ir-diff 는 차이가 있어도 0)",
            "1": "런타임·로드 실패. render-diff 텍스트 모드 하드 실패도 1",
            "2": "사용법. --batch 폴더 읽기 실패",
            "3": "판정 데이터. ir-diff --json 차이, render-diff --json 하드 실패",
        },
        "commands": {
            "render-diff-single": {
                "required": [
                    "schemaVersion",
                    "mode",
                    "sourceA",
                    "sourceB",
                    "via",
                    "threshold",
                    "pageCountA",
                    "pageCountB",
                    "maxDisp",
                    "status",
                    "regression",
                    "pages",
                ],
                "mode": ["pair", "roundtrip"],
                "stdout": "한 줄 JSON",
                "hardExit": 3,
                "textHardExit": 1,
            },
            "render-diff-batch": {
                "required": [
                    "schemaVersion",
                    "mode",
                    "source",
                    "status",
                    "maxDisp",
                    "regression",
                ],
                "stdout": "NDJSON",
                "artifact": "geom_inventory.tsv",
                "loadFailHasErrorKey": True,
                "loadFailExit": 1,
                "regressionExit": 3,
            },
            "ir-diff": {
                "required": [
                    "schemaVersion",
                    "a",
                    "b",
                    "identical",
                    "diffCount",
                    "categories",
                ],
                "invariant": "identical ⇔ diffCount==0 ⇔ categories 비어 있음",
                "diffExit": 3,
                "textDiffExit": 0,
            },
        },
    }


def command_ladder() -> dict:
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "commands": COMMANDS,
        "livingVerbs": [
            "render-diff",
            "ir-diff",
            "thumbnail",
            "export-png",
        ],
        "optionalFollowups": ["export-svg", "export-render-tree", "dump-pages"],
    }


def pitfalls() -> dict:
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "pitfalls": PITFALLS,
    }


def handoff() -> dict:
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "handoff": HANDOFF,
    }


def samples() -> dict:
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "samples": SAMPLES,
        "noNewBinaries": True,
    }


def tsv_schema() -> dict:
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "file": "geom_inventory.tsv",
        "separator": "\t",
        "columns": [
            "sample",
            "status",
            "pages_a",
            "pages_b",
            "max_disp",
            "worst_page",
            "struct_pages",
            "over_pages",
            "elapsed_ms",
            "error",
            "struct_delta",
        ],
        "notes": {
            "max_disp": "소수점 3자리",
            "worst_page": "없으면 -",
            "struct_delta": "예: Line:-4;RawSvg:-1. 음수=손실",
            "error": "LOAD_FAIL 행만 채움",
        },
    }


def max_disp() -> dict:
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "defaultPx": 1.0,
        "structMismatchIgnoresThreshold": True,
        "cases": [
            {
                "id": "D01",
                "threshold": 1.0,
                "maxDisp": 0.0,
                "hardStruct": False,
                "status": "PASS",
            },
            {
                "id": "D02",
                "threshold": 1.0,
                "maxDisp": 1.01,
                "hardStruct": False,
                "status": "OVER",
            },
            {
                "id": "D03",
                "threshold": 100.0,
                "maxDisp": 495.93,
                "hardStruct": True,
                "status": "STRUCT_MISMATCH",
                "note": "임계를 키워도 STRUCT 는 남는다",
            },
            {
                "id": "D04",
                "threshold": 0.05,
                "maxDisp": 495.93,
                "hardStruct": True,
                "status": "STRUCT_MISMATCH",
                "note": "레시피 06 --max-disp 0.05 실측. 판정 동일",
            },
            {
                "id": "D05",
                "threshold": 1.0,
                "maxDisp": 0.4,
                "hardStruct": False,
                "textrunPm1": True,
                "status": "WARN_TEXTRUN",
            },
        ],
    }


def determinism() -> dict:
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "rule": "A==A 는 항상 PASS",
        "why": "회귀 도구의 결정성 기준선. 실패는 문서가 아니라 파이프라인",
        "command": ["render-diff", "<A>", "<A>"],
        "expectedStatus": "PASS",
        "expectedMaxDisp": 0.0,
        "expectedExit": 0,
        "ciBaseline": True,
    }


def thumbnail_vs_png() -> dict:
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "thumbnail": {
            "command": "thumbnail",
            "source": "HWP 내장 PrvImage",
            "rerender": False,
            "flags": ["-o", "--base64", "--data-uri"],
            "use": "저장 시점 미리보기. 편집 후 눈이 아님",
        },
        "exportPng": {
            "command": "export-png",
            "source": "현재 IR 재렌더 (native-skia)",
            "rerender": True,
            "flags": ["-o", "-p", "--scale", "--dpi", "--vlm-target", "--profile"],
            "use": "전후 눈 검증의 기준",
        },
    }


def node_paths() -> dict:
    cases = []
    intended = [
        (
            "Page/Body2/Column0/TextLine10/TextRun0",
            "form-01 myMsg01 채움",
            True,
            "레시피 06 실측 495.93px",
        ),
        (
            "Page/Body2/Column0/TextLine10/TextRun1",
            "같은 줄 이웃 런",
            True,
            "같은 필드 줄이면 의도 범위",
        ),
        (
            "Page/Body/Column0/Table0/Cell[2,3]/TextLine0/TextRun0",
            "set-cell 대상 칸",
            True,
            "표 칸 편집",
        ),
        (
            "Page/Body/Column0/TextLine0/TextRun0",
            "replace-text 첫 줄",
            True,
            "본문 치환",
        ),
    ]
    unrelated = [
        (
            "Page/Header/TextLine0/TextRun0",
            "본문만 편집했는데 머리말",
            False,
            "진짜 회귀",
        ),
        (
            "Page/Footer/TextLine0/TextRun0",
            "꼬리말",
            False,
            "쪽 번호·꼬리 밀림",
        ),
        (
            "Page/Body2/Column1/TextLine0/TextRun0",
            "다른 단",
            False,
            "다단 문서에서 옆 단이 움직임",
        ),
        (
            "Page/Body/Column0/Image0",
            "로고",
            False,
            "상단 로고가 밀리면 회귀",
        ),
        (
            "Page/PageBg0",
            "배경",
            False,
            "0.00px 이면 틀은 유지",
        ),
    ]
    for i, (path, edit, ok, note) in enumerate(intended + unrelated, 1):
        cases.append(
            {
                "id": f"N{i:02d}",
                "path": path,
                "edit": edit,
                "intended": ok,
                "note": note,
                "pageZeroBased": True,
            }
        )
    # expand with synthetic but realistic paths for catalog coverage
    for page in range(0, 8):
        for col in range(0, 2):
            for line in range(0, 6):
                cases.append(
                    {
                        "id": f"N{len(cases)+1:02d}",
                        "path": f"Page/Body{page}/Column{col}/TextLine{line}/TextRun0",
                        "edit": f"page {page} col {col} line {line}",
                        "intended": line >= 4,
                        "note": "쪽·단·줄 좌표로 편집 위치와 대조",
                        "pageZeroBased": True,
                    }
                )
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "zeroBasedPage": True,
        "readPathFirst": True,
        "doNotReflexFail": True,
        "cases": cases,
    }


UTTERANCES = [
    ("편집 전후 화면 비교", "render-diff-pair", "02_render_diff_two_file.md", "F03"),
    ("레이아웃 깨졌는지", "render-diff-pair", "02_render_diff_two_file.md", "F04"),
    ("라운드트립 시각 검증", "render-diff-self", "01_render_diff_self.md", "F01"),
    ("render-diff 돌려줘", "render-diff-pair", "02_render_diff_two_file.md", "F01"),
    ("바뀐 게 의도한 것뿐인지", "render-diff-pair", "04_struct_mismatch.md", "F03"),
    ("포맷 왕복 안전한지", "render-diff-self", "01_render_diff_self.md", "F01"),
    ("HWP 어댑터로 왕복", "render-diff-self-hwp", "01_render_diff_self.md", "F01"),
    ("폴더 전체 회귀 게이트", "render-diff-batch", "03_render_diff_batch.md", "F10"),
    ("geom_inventory 뽑아줘", "render-diff-batch", "18_tsv_schema.md", "F10"),
    ("STRUCT 떴는데 어쩌지", "render-diff-pair", "04_struct_mismatch.md", "F03"),
    ("노드 경로 읽어줘", "render-diff-pair", "15_node_paths.md", "F03"),
    ("쪽 수가 달라졌어", "render-diff-pair", "21_page_mismatch.md", "F05"),
    ("파일을 못 열었어", "render-diff-pair", "22_load_fail.md", "F07"),
    ("변위만 임계 초과", "render-diff-pair", "23_over_status.md", "F06"),
    ("IR 구조 차이", "ir-diff-json", "06_ir_diff.md", "F08"),
    ("ir-diff 제이슨", "ir-diff-json", "06_ir_diff.md", "F08"),
    ("변환 파이프라인 게이트", "ir-diff-json", "06_ir_diff.md", "F08"),
    ("빨리 눈으로 확인", "export-png", "07_thumbnail_vs_png.md", "F09"),
    ("썸네일 뽑아줘", "thumbnail", "07_thumbnail_vs_png.md", "F09"),
    ("저장 미리보기만", "thumbnail", "07_thumbnail_vs_png.md", "F09"),
    ("재렌더 PNG", "export-png", "07_thumbnail_vs_png.md", "F09"),
    ("자기 자신이랑 비교", "render-diff-pair", "08_determinism.md", "F02"),
    ("결정성 기준선", "render-diff-pair", "08_determinism.md", "F02"),
    ("임계 0.05 로", "render-diff-pair", "09_max_disp.md", "F06"),
    ("max-disp 기본값", "render-diff-pair", "09_max_disp.md", "F01"),
    ("기계 봉투로", "render-diff-json", "10_envelopes.md", "F08"),
    ("json 모드 exit", "render-diff-json", "20_exit_codes.md", "F08"),
    ("정밀 bbox", "export-render-tree", "24_export_render_tree.md", "F11"),
    ("메일머지 산출물끼리", "render-diff-pair", "02_render_diff_two_file.md", "F01"),
    ("채운 자리만 움직였는지", "render-diff-pair", "04_struct_mismatch.md", "F03"),
    ("로고가 밀렸는지", "render-diff-pair", "04_struct_mismatch.md", "F04"),
    ("CI 에 배치 심어", "render-diff-batch", "19_gate_recipes.md", "F10"),
    ("TSV 컬럼이 뭐야", "render-diff-batch", "18_tsv_schema.md", "F10"),
    ("WARN_TEXTRUN 은 실패야?", "render-diff-pair", "05_status_codes.md", "F12"),
    ("페이지 0만", "render-diff-pair", "02_render_diff_two_file.md", "F11"),
    ("한컴이랑 같은지", "export-png", "11_pitfalls.md", "F09"),
    ("색이 바뀌었는지", "export-png", "11_pitfalls.md", "F09"),
    ("래스터 diff", "export-png", "11_pitfalls.md", "F09"),
    ("배치 폴더 없음", "render-diff-batch", "22_load_fail.md", "F07"),
    ("없는 파일 단건", "render-diff-pair", "22_load_fail.md", "F07"),
]


def more_utterances() -> list[tuple[str, str, str, str]]:
    extra = []
    verbs = [
        "비교해",
        "재줘",
        "확인해",
        "돌려",
        "재봐",
        "측정해",
        "게이트 걸어",
        "숫자로 봐",
        "판정해",
        "좁혀줘",
    ]
    subjects = [
        ("빈 서식과 채움본", "render-diff-pair", "02_render_diff_two_file.md", "F03"),
        ("같은 길이 두 산출물", "render-diff-pair", "02_render_diff_two_file.md", "F01"),
        ("자기 라운드트립", "render-diff-self", "01_render_diff_self.md", "F01"),
        ("폴더 스윕", "render-diff-batch", "03_render_diff_batch.md", "F10"),
        ("IR 카테고리", "ir-diff-json", "06_ir_diff.md", "F08"),
        ("쪽 0 PNG", "export-png", "07_thumbnail_vs_png.md", "F09"),
        ("내장 썸네일", "thumbnail", "07_thumbnail_vs_png.md", "F09"),
        ("결정성", "render-diff-pair", "08_determinism.md", "F02"),
        ("임계", "render-diff-pair", "09_max_disp.md", "F06"),
        ("구조 경로", "render-diff-pair", "15_node_paths.md", "F03"),
        ("쪽 수", "render-diff-pair", "21_page_mismatch.md", "F05"),
        ("로드 실패", "render-diff-pair", "22_load_fail.md", "F07"),
        ("OVER 페이지만", "render-diff-pair", "23_over_status.md", "F06"),
        ("json 봉투", "render-diff-json", "10_envelopes.md", "F08"),
        ("텍스트 모드", "render-diff-pair", "20_exit_codes.md", "F01"),
    ]
    for subj, cmd, ref, stop in subjects:
        for v in verbs:
            extra.append((f"{subj} {v}", cmd, ref, stop))
    return extra


def intent_matrix() -> dict:
    rows = []
    seen = set()
    for utt, cmd, ref, stop in UTTERANCES + more_utterances():
        if utt in seen:
            continue
        seen.add(utt)
        rows.append(
            {
                "id": f"I{len(rows)+1:03d}",
                "utterance": utt,
                "command": cmd,
                "reference": ref,
                "stop": stop,
                "notGym": True,
            }
        )
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "count": len(rows),
        "intents": rows,
    }


JOURNEY_SPECS = [
    ("J01", "자기 라운드트립만", ["render-diff-self"], "F01", "form-01"),
    ("J02", "전후 비교 PASS", ["render-diff-pair"], "F01", "form-01"),
    ("J03", "채움 후 STRUCT 의도", ["render-diff-pair"], "F03", "form-01"),
    ("J04", "STRUCT 무관 자리", ["render-diff-pair", "export-png"], "F04", "field-01"),
    ("J05", "쪽 수 불일치", ["render-diff-pair", "dump-pages"], "F05", "field-01"),
    ("J06", "OVER 만", ["render-diff-pair"], "F06", "field-01"),
    ("J07", "로드 실패", ["render-diff-pair", "info"], "F07", "missing"),
    ("J08", "배치 전원 PASS", ["render-diff-batch"], "F01", "rd_batch"),
    ("J09", "배치 혼합 격리", ["render-diff-batch"], "F10", "rd_batch"),
    ("J10", "A==A 기준선", ["render-diff-pair"], "F02", "form-01"),
    ("J11", "ir-diff 동일", ["ir-diff-json"], "F01", "hwp3"),
    ("J12", "ir-diff 차이 데이터", ["ir-diff-json"], "F08", "hwp3+sosueop"),
    ("J13", "눈 검증 PNG", ["export-png"], "F09", "form-01"),
    ("J14", "썸네일 함정", ["thumbnail", "export-png"], "F09", "form-01"),
    ("J15", "임계 조임", ["render-diff-pair"], "F06", "form-01"),
    ("J16", "json 모드 게이트", ["render-diff-json"], "F08", "form-01"),
    ("J17", "메일머지 산출물끼리", ["render-diff-pair"], "F01", "batch_out"),
    ("J18", "정밀 bbox", ["export-render-tree"], "F11", "form-01"),
    ("J19", "WARN_TEXTRUN 통과", ["render-diff-pair"], "F12", "form-01"),
    ("J20", "질문이 이미 답", ["render-diff-self"], "F11", "form-01"),
]


def more_journeys() -> list[dict]:
    items = []
    for spec in JOURNEY_SPECS:
        items.append(
            {
                "id": spec[0],
                "title": spec[1],
                "steps": spec[2],
                "stop": spec[3],
                "sample": spec[4],
                "notGym": True,
            }
        )
    templates = [
        ("자기 왕복 {n}번째 표본", ["render-diff-self"], "F01"),
        ("전후 비교 {n}", ["render-diff-pair"], "F03"),
        ("배치 행 {n} 격리", ["render-diff-batch"], "F10"),
        ("ir-diff 표본 {n}", ["ir-diff-json"], "F08"),
        ("PNG 쪽 {n}", ["export-png"], "F09"),
        ("결정성 반복 {n}", ["render-diff-pair"], "F02"),
        ("STRUCT 경로 {n}", ["render-diff-pair"], "F03"),
        ("무관 자리 {n}", ["render-diff-pair", "export-png"], "F04"),
        ("OVER 행 {n}", ["render-diff-pair"], "F06"),
        ("LOAD 행 {n}", ["render-diff-pair"], "F07"),
        ("PAGE 행 {n}", ["render-diff-pair"], "F05"),
        ("json 봉투 {n}", ["render-diff-json"], "F08"),
    ]
    n = 21
    for i in range(1, 61):
        title_t, steps, stop = templates[i % len(templates)]
        items.append(
            {
                "id": f"J{n:02d}",
                "title": title_t.format(n=i),
                "steps": steps,
                "stop": stop,
                "sample": "form-01" if i % 2 else "form-02",
                "notGym": True,
            }
        )
        n += 1
    return items


def journeys() -> dict:
    items = more_journeys()
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "count": len(items),
        "journeys": items,
    }


def failure_signals() -> dict:
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "signals": [
            {
                "signal": "status: PASS",
                "stop": "F01",
                "prescription": "끝",
            },
            {
                "signal": "A==A 가 PASS 아님",
                "stop": "F02",
                "prescription": "도구 비결정성",
            },
            {
                "signal": "STRUCT + 편집 위치 경로",
                "stop": "F03",
                "prescription": "정상",
            },
            {
                "signal": "STRUCT + 무관 경로",
                "stop": "F04",
                "prescription": "진짜 회귀",
            },
            {
                "signal": "PAGE_MISMATCH",
                "stop": "F05",
                "prescription": "dump-pages",
            },
            {
                "signal": "OVER",
                "stop": "F06",
                "prescription": "worst_page",
            },
            {
                "signal": "LOAD_FAIL / 파일 읽기 실패",
                "stop": "F07",
                "prescription": "info",
            },
            {
                "signal": "ir-diff --json exit 3",
                "stop": "F08",
                "prescription": "categories 읽기",
            },
            {
                "signal": "눈으로 봐야 함",
                "stop": "F09",
                "prescription": "export-png",
            },
            {
                "signal": "배치 혼합",
                "stop": "F10",
                "prescription": "TSV 행",
            },
            {
                "signal": "질문 종료",
                "stop": "F11",
                "prescription": "다음 단 금지",
            },
            {
                "signal": "WARN_TEXTRUN",
                "stop": "F12",
                "prescription": "하드 실패 아님",
            },
        ],
    }


TRACE_SPECS = [
    ("T01", "self_form01", "render-diff samples/form-01.hwp --via hwpx", "PASS", 0),
    ("T02", "pair_fill", "render-diff samples/form-01.hwp batch_out/0001.hwp", "STRUCT_MISMATCH", 1),
    ("T03", "pair_fill_tight", "render-diff samples/form-01.hwp batch_out/0001.hwp --max-disp 0.05", "STRUCT_MISMATCH", 1),
    ("T04", "pair_same_len", "render-diff batch_out/0001.hwp batch_out/0002.hwp", "PASS", 0),
    ("T05", "aa_determinism", "render-diff batch_out/0001.hwp batch_out/0001.hwp", "PASS", 0),
    ("T06", "batch_pass", "render-diff --batch rd_batch --via hwpx -o rd_out", "PASS", 0),
]


def traces() -> list[dict]:
    items = []
    for tid, key, cmd, status, exit_ in TRACE_SPECS:
        items.append(
            {
                "id": tid,
                "issue": ISSUE,
                "command": cmd,
                "status": status,
                "exit": exit_,
                "stdout": TRANSCRIPTS[key],
                "source": "recipes/06_visual_regression_before_after.md",
                "notGym": True,
            }
        )
    # synthetic but schema-stable traces for catalog size
    statuses_cycle = [
        ("PASS", 0, TRANSCRIPTS["self_form01"]),
        ("STRUCT_MISMATCH", 1, TRANSCRIPTS["pair_fill"]),
        ("OVER", 1, "페이지 수: A=3 B=3\n최대 변위: 279.00 px (page 1)\n임계 초과 페이지: 1 / 구조 불일치 페이지: 0 (임계 1.00px)\nstatus: OVER\n"),
        ("PAGE_MISMATCH", 1, "페이지 수: A=2 B=3\n⚠ 페이지 수 불일치 — 시각 회귀 강신호\n최대 변위: 0.00 px (page -)\nstatus: PAGE_MISMATCH\n"),
        ("LOAD_FAIL", 1, "오류: 파일 읽기 실패 samples/no-such.hwp\n"),
        ("WARN_TEXTRUN", 0, "페이지 수: A=1 B=1\n최대 변위: 0.40 px (page 0)\n임계 초과 페이지: 0 / 구조 불일치 페이지: 1 (임계 1.00px)\n  page   0: max=   0.40 mean=  0.05 nodes=40/41  [STRUCT:TextRun±1]\nstatus: WARN_TEXTRUN\n"),
    ]
    n = 7
    for i in range(24):
        st, ex, out = statuses_cycle[i % len(statuses_cycle)]
        items.append(
            {
                "id": f"T{n:02d}",
                "issue": ISSUE,
                "command": "render-diff <A> <B>" if st != "PASS" else "render-diff <A> --via hwpx",
                "status": st,
                "exit": ex,
                "stdout": out,
                "source": "skill catalog",
                "notGym": True,
            }
        )
        n += 1
    return items


def ir_diff_envelopes() -> dict:
    return {
        "identical": {
            "schemaVersion": SCHEMA,
            "a": "samples/hwp3-sample.hwp",
            "b": "samples/hwp3-sample.hwp",
            "identical": True,
            "diffCount": 0,
            "categories": {},
        },
        "different": {
            "schemaVersion": SCHEMA,
            "a": "samples/hwp3-sample.hwp",
            "b": "samples/SO-SUEOP.hwp",
            "identical": False,
            "diffCount": 12,
            "categories": {"text": 4, "char_count": 3, "controls": 5},
        },
    }


def render_diff_envelopes() -> dict:
    return {
        "pass_roundtrip": {
            "schemaVersion": SCHEMA,
            "mode": "roundtrip",
            "sourceA": "samples/form-01.hwp",
            "sourceB": None,
            "via": "hwpx",
            "pageFilter": None,
            "threshold": 1.0,
            "pageCountA": 1,
            "pageCountB": 1,
            "pageCountMismatch": False,
            "maxDisp": 0.0,
            "worstPage": None,
            "overPages": 0,
            "structPages": 0,
            "hardStructPages": 0,
            "status": "PASS",
            "regression": False,
            "pages": [],
        },
        "struct_pair": {
            "schemaVersion": SCHEMA,
            "mode": "pair",
            "sourceA": "samples/form-01.hwp",
            "sourceB": "batch_out/0001.hwp",
            "via": None,
            "threshold": 1.0,
            "pageCountA": 1,
            "pageCountB": 1,
            "pageCountMismatch": False,
            "maxDisp": 495.93,
            "worstPage": 0,
            "overPages": 1,
            "structPages": 1,
            "hardStructPages": 1,
            "status": "STRUCT_MISMATCH",
            "regression": True,
            "pages": [
                {
                    "page": 0,
                    "nodeCountA": 39,
                    "nodeCountB": 37,
                    "maxDisp": 495.93,
                    "meanDisp": 13.40,
                    "structureMismatch": True,
                    "structTextrunPm1": False,
                    "topDeltas": [
                        {
                            "path": "Page/Body2/Column0/TextLine10/TextRun0",
                            "nodeType": "TextRun",
                            "disp": 495.93,
                        }
                    ],
                    "typeDeltas": [
                        {"nodeType": "TextRun", "countA": 15, "countB": 13, "net": -2}
                    ],
                }
            ],
        },
    }


def write_fixtures() -> None:
    FIXT.mkdir(parents=True, exist_ok=True)
    dump(FIXT / "skill_index.json", skill_index())
    dump(FIXT / "tree.json", tree())
    dump(FIXT / "stop_rules.json", stop_rules())
    dump(FIXT / "status_catalog.json", status_catalog())
    dump(FIXT / "envelope_keys.json", envelope_keys())
    dump(FIXT / "command_ladder.json", command_ladder())
    dump(FIXT / "pitfalls.json", pitfalls())
    dump(FIXT / "handoff.json", handoff())
    dump(FIXT / "samples.json", samples())
    dump(FIXT / "tsv_schema.json", tsv_schema())
    dump(FIXT / "max_disp.json", max_disp())
    dump(FIXT / "determinism.json", determinism())
    dump(FIXT / "thumbnail_vs_png.json", thumbnail_vs_png())
    dump(FIXT / "node_paths.json", node_paths())
    dump(FIXT / "intent_matrix.json", intent_matrix())
    dump(FIXT / "journeys.json", journeys())
    dump(FIXT / "failure_signals.json", failure_signals())
    tr = traces()
    dump(FIXT / "traces_index.json", {"schemaVersion": SCHEMA, "issue": ISSUE, "ids": [t["id"] for t in tr]})
    for t in tr:
        dump(FIXT / "traces" / f"{t['id']}.json", t)
    for name, body in TRANSCRIPTS.items():
        write_text(FIXT / "transcripts" / f"{name}.txt", body)
    write_text(FIXT / "tsv" / "geom_inventory_pass.tsv", TSV_PASS)
    write_text(FIXT / "tsv" / "geom_inventory_mixed.tsv", TSV_MIXED)
    env = ir_diff_envelopes()
    dump(FIXT / "envelopes" / "ir_diff_identical.json", env["identical"])
    dump(FIXT / "envelopes" / "ir_diff_different.json", env["different"])
    rd = render_diff_envelopes()
    dump(FIXT / "envelopes" / "render_diff_pass.json", rd["pass_roundtrip"])
    dump(FIXT / "envelopes" / "render_diff_struct.json", rd["struct_pair"])
    # extra TSV rows for CI-style inventory (real column contract, many samples)
    rows = [
        "sample\tstatus\tpages_a\tpages_b\tmax_disp\tworst_page\tstruct_pages\tover_pages\telapsed_ms\terror\tstruct_delta"
    ]
    names = [
        "form-01.hwp",
        "form-02.hwp",
        "field-01.hwp",
        "hwp3-sample.hwp",
        "blank.hwp",
        "letter.hwpx",
        "notice.hwpx",
        "table-01.hwp",
        "table-02.hwp",
        "memo.hwp",
    ]
    for i, name in enumerate(names * 3):
        status = ["PASS", "PASS", "PASS", "WARN_TEXTRUN", "OVER", "STRUCT_MISMATCH", "PAGE_MISMATCH", "LOAD_FAIL"][i % 8]
        pages_a = 1 + (i % 5)
        pages_b = pages_a if status != "PAGE_MISMATCH" else pages_a + 1
        disp = 0.0 if status in ("PASS", "LOAD_FAIL", "PAGE_MISMATCH") else (0.4 if status == "WARN_TEXTRUN" else 12.5 + i)
        err = "parse failed" if status == "LOAD_FAIL" else ""
        delta = "TextRun:-2" if status == "STRUCT_MISMATCH" else ("TextRun:+1" if status == "WARN_TEXTRUN" else "")
        worst = "-" if disp == 0.0 else str(i % pages_a)
        rows.append(
            f"{name.replace('.hwp', f'-{i:03d}.hwp')}\t{status}\t{pages_a}\t{pages_b}\t{disp:.3f}\t{worst}\t{1 if status=='STRUCT_MISMATCH' else 0}\t{1 if status=='OVER' else 0}\t{4+i}\t{err}\t{delta}"
        )
    write_text(FIXT / "tsv" / "geom_inventory_catalog.tsv", "\n".join(rows) + "\n")


# ---------------------------------------------------------------------------
# Markdown chapters
# ---------------------------------------------------------------------------


def md_tree() -> str:
    return """# 00 — 시각 회귀 판단 트리

이 장은 에이전트가 **어느 명령을 먼저 칠지**만 고른다. 사다리는 강제 순회가
아니다. 질문이 이미 답이면 멈춘다.

gym 경로가 아니다. 새 CLI 도 없다. 아래 상자는
`mydocs/manual/cli_commands.md` 와 레시피 06, 그리고
`src/diagnostics/render_geom_diff.rs` 가 이미 고정한 명령이다.

```
render-diff <파일> [--via hwpx|hwp] [-p N] [--max-disp PX] [--json]
  │
  ├─ 포맷 왕복만 묻는가
  │     --via hwpx (기본) 또는 --via hwp
  │     PASS → 끝 (F01)
  │
render-diff <A> <B> [-p N] [--max-disp PX] [--json]
  │
  ├─ A 와 B 가 같은 경로인가
  │     항상 PASS 여야 한다 (F02). 아니면 도구 비결정성
  │
  ├─ PASS / WARN_TEXTRUN → 끝 (F01 / F12)
  ├─ STRUCT_MISMATCH → 노드 경로를 읽는다 (F03/F04)
  ├─ PAGE_MISMATCH → dump-pages (F05)
  ├─ OVER → worst_page (F06)
  └─ LOAD_FAIL → info (F07)

render-diff --batch <폴더> [-o 출력] [--via hwpx] [--json]
  │     산출: geom_inventory.tsv
  │     요약 줄만 보지 말고 행별 status (F10)

ir-diff <A> <B> --json
  │     0=동일 / 3=차이(데이터) / 1=로드 / 2=사용법 (F08)

thumbnail <파일>          저장 시점 PrvImage. 재렌더 아님 (F09)
export-png <파일> [-p N]  현재 IR 재렌더. 눈 검증 기준
```

## 축을 고르는 한 줄

| 관찰 | 축 |
| --- | --- |
| 포맷 왕복이 레이아웃을 깨나 | `render-diff <파일> --via hwpx` |
| 편집 전후 | `render-diff <전> <후>` |
| 폴더 CI | `render-diff --batch` |
| IR 구조(텍스트·표 필드) | `ir-diff --json` |
| 색·폰트 래스터 | `export-png` (render-diff 가 아님) |
| 저장 미리보기 | `thumbnail` |

## 명령 상자 (발명 금지)

살아 있는 동사는 이 넷이다.

1. `render-diff`
2. `ir-diff`
3. `thumbnail`
4. `export-png`

후속(이미 있는 명령, 이 스킬이 발명하지 않음): `export-svg --debug-overlay`,
`export-render-tree`, `dump-pages`, `info`.

없는 것: 픽셀 비교 전용 하위명령, 레이아웃 별칭, 스크린샷 비교 동사,
gym 렌더 러너. 오타 난 하위명령은 exit 2.

코어 재사용:

- 기하 = 기존 `diagnostics::render_geom_diff`
- IR = 기존 `ir-diff`
- 미리보기 = 기존 `thumbnail` (PrvImage)
- 재렌더 = 기존 `export-png` (native-skia)

## 원본 불변

비교 명령은 입력을 덮어쓰지 않는다. `--batch -o` 는 측정 TSV 만 쓴다.
편집은 다른 스킬(`rhwp-form-fill` / `rhwp-safe-edit`)이 `-o` 로 산출을
분리한 뒤에야 이 스킬이 전후를 잰다.

## 에이전트가 하지 말 것

- STRUCT 빨간불을 경로도 안 읽고 롤백
- thumbnail 을 채운 화면으로 제출
- A==A 실패를 문서 탓으로 돌림
- `--max-disp` 로 STRUCT 를 숨기려 함
- gym/ 아래에 과제를 만들기
"""


def md_self() -> str:
    return """# 01 — render-diff 자기 라운드트립

가장 싼 점검이다. 파일을 하나 주고, 원본 IR 과 직렬화→재로드 IR 을
같은 렌더러로 그린 뒤 노드 bbox 를 비교한다.

```bash
rhwp render-diff samples/form-01.hwp --via hwpx
```

레시피 06 실측:

```
페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
임계 초과 페이지: 0 / 구조 불일치 페이지: 0 (임계 1.00px)
status: PASS
```

종료 코드 0. `--via hwpx` 는 HWP5 원본을 HWPX 로 변환했다가 다시
렌더링한다. `--via hwp` 는 HWP 어댑터 경로다. 기본은 hwpx.

특정 페이지만:

```bash
rhwp render-diff samples/form-01.hwp -p 0 --via hwpx
```

1쪽 문서라 결과는 같다. `-p` 가 비교 범위 밖이면 **exit 2** 다.
빈 PASS 로 위장하지 않는다.

## 이것이 재는 것 / 안 재는 것

재는 것: rhwp 가 그린 원본 vs rhwp 가 그린 왕복. 내부 회귀.

안 재는 것: 한컴 PDF 충실도. 자기 라운드트립 PASS ≠ 한컴과 같다.

## JSON

```bash
rhwp render-diff samples/form-01.hwp --via hwpx --json
```

`mode` 는 `"roundtrip"`, `sourceB` 는 null, `via` 는 `"hwpx"` 또는
`"hwp"`. `status: PASS` 이면 exit 0.

## 언제 쓰나

- 변환 파이프라인에 넣기 전 싼 스모크
- CI 상시 기준선 (A==A 와 함께)
- 편집 전후를 보기 전에 "도구 자체가 흔들리지 않는가"
"""


def md_pair() -> str:
    return """# 02 — render-diff 두 파일

편집 전 vs 후, 또는 같은 종류의 산출물끼리.

```bash
rhwp render-diff samples/form-01.hwp batch_out/0001.hwp
```

레시피 06 실측 (빈 서식 vs `myMsg01`="김철수 귀하"):

```
페이지 수: A=1 B=1
최대 변위: 495.93 px (page 0)
임계 초과 페이지: 1 / 구조 불일치 페이지: 1 (임계 1.00px)
  page   0: max= 495.93 mean= 13.40 nodes=39/37  [STRUCT]
       495.93px  Page/Body2/Column0/TextLine10/TextRun0
         0.00px  Page
         0.00px  Page/PageBg0
      Δ TextRun: 15→13 (-2)
status: STRUCT_MISMATCH
```

텍스트 모드 종료 코드 1. **버그가 아니다.** 빈 누름틀이 실제 값으로
바뀌면 그 자리 텍스트런 구조가 달라진다. 핵심은 경로
`Page/Body2/Column0/TextLine10/TextRun0` 가 편집한 필드와 맞는가다.

같은 글자 수 산출물끼리:

```bash
rhwp render-diff batch_out/0001.hwp batch_out/0002.hwp
```

```
status: PASS
최대 변위: 0.00 px
```

값은 달라도("김철수 귀하" vs "이영희 귀하") 글자 수가 같으면 구조가
유지된다. 메일머지에서 특정 값만 레이아웃이 깨지는 행을 찾을 때 쓴다.

## JSON

```bash
rhwp render-diff 전.hwp 후.hwp --json
```

`mode` 는 `"pair"`, `via` 는 null, `sourceA`/`sourceB` 가 두 경로.
하드 실패는 exit **3** (`regression: true`).

## 페이지 필터

`-p N` 은 0 부터. 한컴 쪽번호 1 과 혼동하지 않는다.
"""


def md_batch() -> str:
    return """# 03 — render-diff --batch 와 geom_inventory.tsv

폴더를 전수 왕복 비교하고 TSV 를 남긴다. CI 아티팩트다.

```bash
mkdir -p rd_batch
cp samples/form-01.hwp samples/form-02.hwp rd_batch/
rhwp render-diff --batch rd_batch --via hwpx -o rd_out
```

레시피 06 실측:

```
[           PASS] max_disp=   0.00 struct=0 over=0      5ms  form-01.hwp
[           PASS] max_disp=   0.00 struct=0 over=0      4ms  form-02.hwp

TSV 저장: rd_out\\geom_inventory.tsv
```

## TSV 컬럼

`sample status pages_a pages_b max_disp worst_page struct_pages over_pages elapsed_ms error struct_delta`

픽스처: `fixtures/tsv/geom_inventory_pass.tsv`.

- `max_disp` 는 소수점 3자리
- `worst_page` 없으면 `-`
- `struct_delta` 예: `Line:-4;RawSvg:-1` (음수=손실)
- `error` 는 LOAD_FAIL 행만

## 종료 코드

- 사람 모드: 하드 실패(OVER/STRUCT/PAGE/LOAD)가 하나라도 있으면 1
- `--json`: 로드 실패가 있으면 1 우선, 아니면 회귀 검출 3
- 폴더를 못 읽으면 2 (`오류: 폴더 읽기 실패`)
- 폴더에 .hwp/.hwpx 가 없으면 2

## JSON 배치

stdout 은 NDJSON. 한 파일 한 줄. `error` 키는 실패 행에만 있다.
TSV 저장 안내는 stderr 로 빠진다(stdout 순수성).

요약 줄("총 파일 2 / PASS 2")만 보고 통과시키지 않는다. 행별 status 로
게이트한다. [19_gate_recipes.md](19_gate_recipes.md).
"""


def md_struct() -> str:
    return """# 04 — STRUCT_MISMATCH 는 데이터다

`status: STRUCT_MISMATCH` 는 종료 코드 1(텍스트) 또는 3(`--json`)을
낸다. 자동화는 그걸 **실패 신호**로 받을 수 있다. 에이전트는 받아서
**경로를 읽는다.** 반사적으로 롤백하지 않는다.

## 읽는 순서

1. `Δ TextRun: 15→13 (-2)` 같은 타입 증감. 음수=손실, 양수=추가.
2. 변위 큰 노드 경로 상위 몇 개 (`495.93px  Page/Body2/...`).
3. 그 경로가 **방금 편집한 위치**와 같은가.
4. 상위 틀(`Page`, `Page/PageBg0`)이 0.00px 인가.

편집 위치와 같으면 F03 — 정상. 값이 바뀌면 그 자리 구조도 바뀐다.
무관한 머리말·다른 단·로고면 F04 — 진짜 회귀.

## 임계와 무관

`--max-disp 100` 을 줘도 하드 구조 불일치는 STRUCT 로 남는다.
임계는 변위(OVER)만 가른다.

## TextRun ±1

한 페이지의 구조 차이가 TextRun 삽입·삭제 각 최대 1개뿐이면
`WARN_TEXTRUN` (#1773). 하드 실패가 아니다. 다른 페이지에 일반
STRUCT 가 있으면 문서는 여전히 STRUCT_MISMATCH.

## JSON 에서

`pages[].topDeltas[].path`, `pages[].typeDeltas`, `hardStructPages`.
`regression: true` 여도 경로부터 대조한다.
"""


def md_status() -> str:
    return """# 05 — 상태 코드

`status_str` (`render_geom_diff.rs`) 우선순위:

1. 쪽 수가 다르면 `PAGE_MISMATCH`
2. 하드 구조 불일치 페이지가 있으면 `STRUCT_MISMATCH`
3. maxDisp > 임계면 `OVER`
4. TextRun ±1 구조만 있으면 `WARN_TEXTRUN`
5. 그 외 `PASS`

`LOAD_FAIL` 은 비교 전에 파일을 못 연 배치 행의 상태다.

| status | hard | 텍스트 exit | --json exit |
| --- | --- | --- | --- |
| PASS | 아니오 | 0 | 0 |
| WARN_TEXTRUN | 아니오 | 0 | 0 |
| OVER | 예 | 1 | 3 |
| STRUCT_MISMATCH | 예 | 1 | 3 |
| PAGE_MISMATCH | 예 | 1 | 3 |
| LOAD_FAIL | 예 | 1 | 1 (측정 실패 우선) |

`status_is_hard_failure` = PASS/WARN_TEXTRUN 이 아닌 것.

에이전트 판정은 hard 여부와 별개다. STRUCT 는 hard 이지만 F03 이면
문서 회귀가 아니다.
"""


def md_ir() -> str:
    return """# 06 — ir-diff --json

레이아웃(px)이 아니라 IR 구조(텍스트, 문단 모양, 표 필드, 컨트롤)를
비교한다. 변환 파이프라인의 내용 보존 게이트다.

```bash
rhwp ir-diff A.hwpx B.hwp --json
```

봉투 한 줄:

```
{"schemaVersion":"1.0","a","b","identical","diffCount","categories":{…}}
```

불변식: `identical` ⇔ `diffCount == 0` ⇔ `categories` 가 비어 있음.

## 종료 코드

| 상황 | --json | 텍스트 |
| --- | --- | --- |
| 동일 | 0 | 0 |
| 차이 | **3** | **0** (기존 소비자 보호) |
| 읽기·파싱 실패 | 1, stdout 0바이트 | 1 |
| 사용법 | 2 | 2 |

게이트는 반드시 `--json` 이다.

```bash
rhwp ir-diff 원본.hwp 변환본.hwpx --json || 격리처리
```

exit 3 은 실패가 아니라 **차이 검출 데이터**다. `categories` 로
어느 축(text, char_count, controls, …)인지 읽는다.

`--summary` / `--max-lines` 와 `--json` 을 같이 주면 JSON 이 이긴다.
stdout 순수성.

알 수 없는 옵션은 현재 조용히 무시된다(#3178). 게이트 스크립트는
플래그 철자를 정확히 쓴다.
"""


def md_thumb() -> str:
    return """# 07 — thumbnail vs export-png

두 명령 모두 PNG 비슷한 것을 내지만 **같은 눈이 아니다.**

## thumbnail

HWP **내장** 썸네일(PrvImage)을 추출한다.

```bash
rhwp thumbnail 문서.hwp -o 문서_thumb.png
rhwp thumbnail 문서.hwp --data-uri
```

저장 시점의 미리보기다. `edit fill-fields` 직후 다시 뽑아도, 한컴이
PrvImage 를 갱신하지 않은 파일이면 **빈 서식 그림**이 나온다.
편집 후 눈 검증의 기준이 될 수 없다.

## export-png

현재 IR 을 Skia 로 **재렌더**한다. `native-skia` feature.

```bash
rhwp export-png 문서.hwp -p 0 -o 문서_p0.png
rhwp export-png 문서.hwp --vlm-target claude
```

전후 눈 검증, VLM 입력, 래스터 품질은 이쪽이다.

## 선택 규칙

| 질문 | 명령 |
| --- | --- |
| 저장본에 들어 있는 미리보기 | thumbnail |
| 지금 레이아웃이 어떻게 그려지나 | export-png |
| 문단/표 경계를 겹쳐 보기 | export-svg --debug-overlay |
| px 숫자 | render-diff |

thumbnail 을 채움 확인에 쓰면 F09 함정이다.
"""


def md_det() -> str:
    return """# 08 — A==A 결정성

```bash
rhwp render-diff 산출.hwp 산출.hwp
```

레시피 06 실측:

```
페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
status: PASS
```

항상 PASS, maxDisp 0.00 이어야 한다. 같은 바이트를 두 번 렌더했는데
노드가 움직이면 **문서 회귀가 아니라 도구 비결정성**이다.

CI 에 상시 기준선으로 심는다. 이 한 줄이 깨지면 전후 비교 숫자를
믿을 수 없다.

자기 라운드트립(`render-diff <파일> --via hwpx`)과 혼동하지 않는다.
왕복은 직렬화 경로를 한 번 탄다. A==A 는 그 경로조차 타지 않는다.

CI 한 줄 예:

```bash
rhwp render-diff "$OUT" "$OUT"; test $? -eq 0
```

`--json` 이어도 `status` 는 PASS, `maxDisp` 는 0.0, `regression` 은 false.
같은 입력을 두 번 연속으로 돌려 봉투가 바이트 단위로 같은지까지 보면
게이트의 결정성 전제가 닫힌다.
"""


def md_max() -> str:
    return """# 09 — --max-disp 기본 1.0px

`DEFAULT_MAX_DISP = 1.0`. 대응 노드 변위가 이 값을 **초과**하면
OVER 후보가 된다 (`>` , `>=` 가 아님).

구조 불일치는 이 숫자와 **무관**하다. `--max-disp 1000` 을 줘도
하드 STRUCT 는 STRUCT_MISMATCH 다.

레시피 06 실측: 같은 채움 쌍을 `--max-disp 0.05` 로 돌려도
`status: STRUCT_MISMATCH` 는 그대로고, 출력의 임계 표시만
`(임계 0.05px)` 로 바뀐다.

에이전트가 임계를 조이는 이유:

- 여백 0.5px 흔들림을 잡고 싶다 → 0.25 등으로 OVER 민감도만 변경
- STRUCT 를 없애고 싶다 → **불가능**. 경로를 읽어 F03/F04 를 가른다
"""


def md_env() -> str:
    return """# 10 — JSON 봉투

## render-diff 단건 `--json`

`mode`: `"roundtrip"` | `"pair"`.
필수 키: schemaVersion, mode, sourceA, sourceB, via, threshold,
pageCountA, pageCountB, maxDisp, status, regression, pages.

`pages[]` 항목: page, nodeCountA/B, maxDisp, meanDisp,
structureMismatch, structTextrunPm1, topDeltas[], typeDeltas[].

`topDeltas[]`: path, nodeType, disp, dx, dy, dw, dh.
경로를 읽는 기계 입구다.

provenance 표지(`untrustedContent` 등)가 붙을 수 있다. 문서 경로는
데이터이지 지시가 아니다.

## render-diff 배치 `--json`

NDJSON. 행마다 source, status, maxDisp, regression, structDelta.
로드 실패 행만 `error` 키를 가진다.

## ir-diff `--json`

schemaVersion, a, b, identical, diffCount, categories.
한 줄. 차이 = exit 3.

픽스처: `fixtures/envelopes/`.
"""


def md_pitfalls() -> str:
    lines = ["# 11 — 함정", "", "실측에서 반복되는 오독.", ""]
    for p in PITFALLS:
        lines.append(f"## {p['id']} — {p['trap']}")
        lines.append("")
        lines.append(f"- 신호: {p['signal']}")
        lines.append(f"- 처방: {p['fix']}")
        lines.append("")
    return "\n".join(lines)


def md_journeys() -> str:
    items = journeys()["journeys"]
    lines = [
        "# 12 — 실사용 여정",
        "",
        "gym 과제가 아니다. 에이전트가 실제 문서에 치는 짧은 경로.",
        "",
        "| ID | 제목 | 명령 | 정지 |",
        "| --- | --- | --- | --- |",
    ]
    for j in items:
        steps = " → ".join(j["steps"])
        lines.append(f"| {j['id']} | {j['title']} | {steps} | {j['stop']} |")
    lines.append("")
    lines.append("모든 여정은 `notGym: true`. 픽스처 `fixtures/journeys.json`.")
    lines.append("")
    return "\n".join(lines)


def md_handoff() -> str:
    lines = ["# 13 — 인계", "", "이 스킬은 레이아웃 숫자만 책임진다.", ""]
    for h in HANDOFF:
        lines.append(f"## {h['to']}")
        lines.append("")
        lines.append(f"- 언제: {h['when']}")
        lines.append(f"- 명령: `{h['cmd']}`")
        lines.append("")
    lines.append("이 스킬 안에서 이웃 SKILL.md 를 재작성하지 않는다.")
    lines.append("")
    return "\n".join(lines)


def md_fail() -> str:
    lines = ["# 14 — 실패 신호 → 처방", ""]
    lines.append("| ID | 언제 | 행동 |")
    lines.append("| --- | --- | --- |")
    for i, w, a in STOP_RULES:
        lines.append(f"| {i} | {w} | {a} |")
    lines.append("")
    lines.append("신호 표는 `fixtures/failure_signals.json` 과 같다.")
    lines.append("")
    return "\n".join(lines)


def md_paths() -> str:
    return """# 15 — 노드 경로를 읽는 법

경로는 render tree 의 안정 문자열이다. 예:

`Page/Body2/Column0/TextLine10/TextRun0`

- `Page` — 쪽 루트. 보통 0.00px 이면 전체 틀은 유지
- `PageBg0` — 쪽 배경
- `Body` / `Body2` — 본문 흐름 (마스터/본문 층)
- `ColumnN` — 단. N 이 바뀌면 다른 단
- `TextLineN` — 줄. 0 부터
- `TextRunN` — 같은 줄의 런
- `Table` / `Cell` — 표와 칸
- `Header` / `Footer` — 머리말·꼬리말. 본문 편집과 무관하면 회귀
- `Image` — 그림. 로고가 여기로 잡힌다

페이지 번호는 명령 `-p` 와 출력 `page 0` 모두 0 부터다.

대조 절차:

1. 편집 명령이 건드린 쪽·칸·필드 이름을 적는다.
2. STRUCT/OVER 가 가리키는 경로를 적는다.
3. 둘의 쪽·단·줄이 같은가.
4. 상위 `Page`/`PageBg0` 가 0px 인가.

카탈로그: `fixtures/node_paths.json`.
"""


def md_traces() -> str:
    lines = [
        "# 16 — 재현 트레이스",
        "",
        "레시피 06 실측과 상태별 카탈로그. 원문은 `fixtures/traces/`.",
        "",
    ]
    for t in traces():
        lines.append(f"## {t['id']} — {t['status']} (exit {t['exit']})")
        lines.append("")
        lines.append(f"`{t['command']}`")
        lines.append("")
        lines.append("```")
        lines.append(t["stdout"].rstrip())
        lines.append("```")
        lines.append("")
    return "\n".join(lines)


def md_intents() -> str:
    rows = intent_matrix()["intents"]
    lines = [
        "# 17 — 발화 → 명령",
        "",
        f"발화 {len(rows)}건. 발명 명령 없음.",
        "",
        "| ID | 발화 | 명령 | 정지 |",
        "| --- | --- | --- | --- |",
    ]
    for r in rows:
        lines.append(f"| {r['id']} | {r['utterance']} | {r['command']} | {r['stop']} |")
    lines.append("")
    return "\n".join(lines)


def md_tsv() -> str:
    return """# 18 — geom_inventory.tsv 스키마

`--batch -o <폴더>` 가 쓰는 파일 이름은 항상 `geom_inventory.tsv` 다.

탭 구분, 헤더 1행. 컬럼 11개:

1. sample
2. status
3. pages_a
4. pages_b
5. max_disp (%.3f)
6. worst_page (`-` 또는 정수)
7. struct_pages
8. over_pages
9. elapsed_ms
10. error
11. struct_delta

실측 PASS 2행은 레시피 06 과 `fixtures/tsv/geom_inventory_pass.tsv`.
혼합 카탈로그는 `fixtures/tsv/geom_inventory_mixed.tsv`.

게이트는 헤더를 건너뛰고 `$2` (status) 를 읽는다. PASS 와
WARN_TEXTRUN 만 통과로 둘지, STRUCT 를 경로 대조 큐로 보낼지는
소비자가 정한다. 스킬 기본은 STRUCT 를 즉시 실패로 접지 않는 것이다.
"""


def md_gate() -> str:
    return """# 19 — 게이트 레시피

## 결정성

```bash
rhwp render-diff "$OUT" "$OUT"
test $? -eq 0
```

## 배치 TSV (사람 모드)

```bash
rhwp render-diff --batch "$DIR" -o "$OUT"
awk -F'\\t' 'NR>1 && $2!="PASS" && $2!="WARN_TEXTRUN" {print; n++}
             END{exit n?1:0}' "$OUT/geom_inventory.tsv"
```

STRUCT 를 즉시 실패에서 빼려면 `$2=="STRUCT_MISMATCH"` 를 별도 큐로
보낸다. 그 행의 `struct_delta`($11)와 단건 `--json` 의 path 를 읽는다.

## ir-diff 변환

```bash
rhwp ir-diff "$A" "$B" --json
case $? in
  0) echo identical ;;
  3) echo diff; jq .categories ;;
  1) echo load-fail ;;
  2) echo usage ;;
esac
```

## render-diff --json 단건

```bash
rhwp render-diff "$A" "$B" --json > env.json
# exit 3 = regression true. path 를 읽는다
jq -r '.status, .maxDisp, .pages[].topDeltas[].path' env.json
```

stdout 은 순수 JSON/NDJSON. 진행 메시지는 stderr.
"""


def md_exit() -> str:
    return """# 20 — 종료 코드

전역 계약 #2707: 0 성공 · 1 런타임 · 2 사용법 · 3 검증 단언(판정).

## render-diff

- 텍스트: 하드 실패(OVER/STRUCT/PAGE/LOAD) = **1** (기존 CI)
- `--json`: 하드 실패 = **3**, 로드 실패 = 1, 사용법 = 2
- 배치 `--json`: 로드 실패가 하나라도 있으면 1 이 회귀 3 보다 우선
- `-p` 범위 밖 = 2
- `--batch` 폴더 읽기 실패 / 빈 폴더 = 2
- 단건 파일 없음 = 1

## ir-diff

- `--json` 차이 = **3**
- 텍스트 차이 = **0**
- 로드 = 1 (json 이면 stdout 0바이트)
- 사용법 = 2

exit 3 을 "프로그램이 죽었다"로 읽지 않는다. 판정 데이터다.
"""


def md_page() -> str:
    return """# 21 — PAGE_MISMATCH

`pageCountA != pageCountB`. 시각 회귀의 가장 명백한 신호라
우선순위 1위다. 변위나 구조를 보기 전에 쪽 수가 갈라진다.

텍스트 출력에 `⚠ 페이지 수 불일치 — 시각 회귀 강신호` 가 붙는다.

의도한 경우: 긴 값을 넣어 한 쪽이 늘어난 메일머지, 페이지 나누기
편집. 그때는 F05 를 "정상, 기록만"으로 닫는다.

의도하지 않은 경우: 같은 길이 치환인데 쪽이 늘거나 줄었다.
`dump-pages --json` 으로 어느 쪽에서 갈라지는지 좁힌다.

JSON: `pageCountMismatch: true`, `status: PAGE_MISMATCH`,
`regression: true`.

배치 TSV 에서는 `pages_a` 와 `pages_b` 가 다르고 `status` 가
`PAGE_MISMATCH` 다. `max_disp` 는 0 일 수 있다 — 쪽 수가 갈라지면
변위를 재기 전에 끝난다. 같은 폴더의 다른 행은 계속 측정된다.
"""


def md_load() -> str:
    return """# 22 — LOAD_FAIL

비교 대상 바이트를 파싱하지 못했다. **측정 실패**이지 회귀 검출이
아니다.

- 단건 없는 파일: `오류: 파일 읽기 실패`, exit 1
- 배치 한 행: status `LOAD_FAIL`, TSV `error` 컬럼에 이유, 다른 행은 계속
- 배치 `--json`: 그 줄에 `error` 키. 전건 중 하나라도 있으면 전체 exit 1
- 배치 폴더 자체 없음: 비교 시작 전 exit 2

처방: `rhwp info <파일> --json` 으로 그 파일만 연다. 암호·손상·확장자
사칭을 가른다. 이웃 스킬 `rhwp-doc-triage` 로 넘길 수 있다.

LOAD_FAIL 을 exit 3 으로 접지 않는다. 3 은 "재봤다, 차이가 있다"이다.

JSON 배치 행에 `error` 키가 있으면 `regression` 은 false 다. 측정하지
못했으므로 회귀를 검출했다고 말할 수 없다. TSV 의 `error` 컬럼과
같은 축이다.
"""


def md_over() -> str:
    return """# 23 — OVER

구조(노드 개수·경로)는 같고, 대응 노드 변위가 `--max-disp` 를 넘었다.

같은 글자 수 치환인데 줄바꿈이 달라져 아래 문단이 밀린 경우가 전형이다.
STRUCT 가 아니므로 경로 개수는 맞다. `worst_page` 와 상위 `topDeltas`
의 disp 를 본다.

임계를 헐겁게 하면 OVER 는 사라질 수 있다. 그게 맞는지(폰트 힌팅
노이즈) 실제 여백 회귀인지는 `export-png` 로 그 쪽을 본다.

채움처럼 구조가 바뀌면 OVER 가 아니라 STRUCT 가 먼저 붙는다
(우선순위).

JSON: `status: OVER`, `regression: true`, `hardStructPages: 0`,
`overPages >= 1`. 텍스트 모드 exit 1, `--json` exit 3.
"""


def md_rtree() -> str:
    return """# 24 — export-render-tree (후속)

`render-diff` 가 이미 같은 트리로 변위를 잰다. 사람이 전후 JSON 을
직접 diff 하고 싶을 때만 후속으로 친다. 새 명령이 아니다.

```bash
rhwp export-render-tree 전.hwp -p 0 > before.json
rhwp export-render-tree 후.hwp -p 0 > after.json
```

bbox 좌표가 들어 있다. 자동화 게이트의 1차는 여전히 `render-diff`
종료 코드와 TSV 다. 이 덤프는 좁힌 뒤의 정밀 대조다.

`export-svg --debug-overlay -p N` 은 문단/표 경계를 그림으로 겹친다.
숫자 다음의 눈 검증.

이 스킬이 이 명령을 발명한 것이 아니다. 이미 CLI 에 있다. 1차 판정은
항상 `render-diff` 의 status 와 TSV 다.
"""


def md_ref_readme() -> str:
    lines = [
        "# rhwp-visual-regression references",
        "",
        "실 에이전트가 render-diff / ir-diff / thumbnail / export-png 로",
        "레이아웃 회귀를 숫자로 판정하는 장. gym 아님. 새 CLI 없음.",
        "",
        f"이슈 #{ISSUE}. 픽스처는 상위 `fixtures/`.",
        "",
    ]
    for name in REQUIRED_REFS:
        if name != "README.md":
            lines.append(f"- [{name}]({name})")
    lines.append("")
    return "\n".join(lines)


EX_BODIES = {
    "01_self_roundtrip_form01.md": (
        "자기 라운드트립 form-01",
        "render-diff samples/form-01.hwp --via hwpx",
        "self_form01",
        "F01 PASS. 포맷 왕복이 레이아웃을 안 건드렸다.",
    ),
    "02_self_roundtrip_via_hwp.md": (
        "자기 라운드트립 --via hwp",
        "render-diff samples/form-01.hwp --via hwp",
        "self_form01",
        "어댑터 경로. 출력 형식은 같고 via 만 hwp.",
    ),
    "03_two_file_fill.md": (
        "빈 서식 vs 채움 산출",
        "render-diff samples/form-01.hwp batch_out/0001.hwp",
        "pair_fill",
        "F03. 경로 TextLine10 이 myMsg01 자리. 실패로 읽지 않는다.",
    ),
    "04_two_file_same_length.md": (
        "같은 글자 수 산출물끼리",
        "render-diff batch_out/0001.hwp batch_out/0002.hwp",
        "pair_same_len",
        "F01. 값이 달라도 구조가 같으면 PASS.",
    ),
    "05_aa_determinism.md": (
        "A==A 결정성",
        "render-diff batch_out/0001.hwp batch_out/0001.hwp",
        "aa_determinism",
        "F02 의 기준선. PASS 가 아니면 도구 문제.",
    ),
    "06_batch_pass_folder.md": (
        "배치 전원 PASS",
        "render-diff --batch rd_batch --via hwpx -o rd_out",
        "batch_pass",
        "TSV 두 행 모두 PASS. 요약 줄과 행이 일치하는지 본다.",
    ),
    "07_batch_mixed_status.md": (
        "배치 혼합 상태",
        "render-diff --batch mixed -o rd_out",
        "batch_pass",
        "F10. fixtures/tsv/geom_inventory_mixed.tsv 를 행별로 읽는다.",
    ),
    "08_struct_intended.md": (
        "의도된 STRUCT",
        "render-diff samples/form-01.hwp batch_out/0001.hwp --json",
        "pair_fill",
        "path 가 편집 위치. F03. --json 이면 exit 3 이지만 데이터.",
    ),
    "09_struct_unrelated.md": (
        "무관한 STRUCT",
        "render-diff 전.hwp 후.hwp",
        "pair_fill",
        "경로가 Header/Image0 이면 F04. export-png 로 그 쪽을 본다.",
    ),
    "10_page_mismatch.md": (
        "쪽 수 불일치",
        "render-diff short.hwp long.hwp",
        "pair_fill",
        "F05. dump-pages --json 으로 갈라지는 쪽을 좁힌다.",
    ),
    "11_load_fail.md": (
        "없는 파일",
        "render-diff samples/no-such.hwp",
        "self_form01",
        "F07. 단건 exit 1. 배치 폴더 오류는 exit 2.",
    ),
    "12_over_threshold.md": (
        "OVER — 구조는 동일",
        "render-diff a.hwp b.hwp --max-disp 1.0",
        "pair_fill",
        "F06. worst_page 로 좁히고 임계가 빡센지 확인.",
    ),
    "13_ir_diff_json.md": (
        "ir-diff --json 차이",
        "ir-diff samples/hwp3-sample.hwp samples/SO-SUEOP.hwp --json",
        "self_form01",
        "F08. exit 3, identical false. 텍스트 모드면 같은 차이가 0.",
    ),
    "14_thumbnail_stale.md": (
        "thumbnail 은 저장 미리보기",
        "thumbnail filled.hwp --data-uri",
        "self_form01",
        "F09. 채운 뒤에도 PrvImage 가 옛것이면 빈 서식 그림.",
    ),
    "15_export_png_rerender.md": (
        "export-png 재렌더",
        "export-png filled.hwp -p 0 -o filled_p0.png",
        "self_form01",
        "눈 검증 기준. native-skia. thumbnail 을 대체하지 않는다.",
    ),
    "16_max_disp_struct_independent.md": (
        "임계를 조여도 STRUCT",
        "render-diff samples/form-01.hwp batch_out/0001.hwp --max-disp 0.05",
        "pair_fill_tight",
        "판정은 그대로 STRUCT. 임계 표시만 0.05px.",
    ),
    "17_text_mode_exit1.md": (
        "텍스트 모드 하드 실패는 1",
        "render-diff samples/form-01.hwp batch_out/0001.hwp",
        "pair_fill",
        "기존 CI 가 1 을 실패로 읽는다. 에이전트는 경로를 읽는다.",
    ),
    "18_json_mode_exit3.md": (
        "JSON 모드 하드 실패는 3",
        "render-diff samples/form-01.hwp batch_out/0001.hwp --json",
        "pair_fill",
        "ir-diff --json 과 같은 판정=데이터 축.",
    ),
    "19_geom_inventory_gate.md": (
        "TSV 게이트",
        "render-diff --batch rd_batch -o rd_out",
        "batch_pass",
        "awk 로 status 열을 읽는다. 19_gate_recipes.md.",
    ),
    "20_warn_textrun.md": (
        "WARN_TEXTRUN 은 하드 실패 아님",
        "render-diff a.hwp b.hwp",
        "self_form01",
        "F12. TextRun ±1 만. exit 0.",
    ),
}


def write_example(name: str) -> None:
    if name == "README.md":
        lines = [
            "# rhwp-visual-regression examples",
            "",
            "레시피 06 실측을 에이전트가 그대로 따라 치는 짧은 예.",
            "새 명령 없음. gym 없음.",
            "",
        ]
        for n in REQUIRED_EXAMPLES:
            if n != "README.md":
                lines.append(f"- [{n}]({n})")
        write_text(EX / name, "\n".join(lines) + "\n")
        return
    title, cmd, key, note = EX_BODIES[name]
    body = f"""# 예제 — {title}

이슈 #{ISSUE}. 실 에이전트 경로. gym 아님.

## 명령

```bash
rhwp {cmd}
```

## 실측·카탈로그 출력

```
{TRANSCRIPTS[key].rstrip()}
```

## 읽는 법

{note}

관련: `references/` 같은 번호 장, `fixtures/transcripts/{key}.txt`.
"""
    write_text(EX / name, body)


def write_markdown() -> None:
    chapters = {
        "00_tree.md": md_tree,
        "01_render_diff_self.md": md_self,
        "02_render_diff_two_file.md": md_pair,
        "03_render_diff_batch.md": md_batch,
        "04_struct_mismatch.md": md_struct,
        "05_status_codes.md": md_status,
        "06_ir_diff.md": md_ir,
        "07_thumbnail_vs_png.md": md_thumb,
        "08_determinism.md": md_det,
        "09_max_disp.md": md_max,
        "10_envelopes.md": md_env,
        "11_pitfalls.md": md_pitfalls,
        "12_journeys.md": md_journeys,
        "13_handoff.md": md_handoff,
        "14_failure_signals.md": md_fail,
        "15_node_paths.md": md_paths,
        "16_worked_traces.md": md_traces,
        "17_intent_matrix.md": md_intents,
        "18_tsv_schema.md": md_tsv,
        "19_gate_recipes.md": md_gate,
        "20_exit_codes.md": md_exit,
        "21_page_mismatch.md": md_page,
        "22_load_fail.md": md_load,
        "23_over_status.md": md_over,
        "24_export_render_tree.md": md_rtree,
        "README.md": md_ref_readme,
    }
    for name, fn in chapters.items():
        write_text(REF / name, fn())
    for name in REQUIRED_EXAMPLES:
        write_example(name)


def main() -> None:
    write_fixtures()
    write_markdown()
    print(f"wrote fixtures+refs+examples for #{ISSUE}")


if __name__ == "__main__":
    main()
