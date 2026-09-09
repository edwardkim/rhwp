#!/usr/bin/env python3
"""Generate rhwp-cli skill references, examples, and fixtures.

Does not invent CLI. Authority is mydocs/manual/cli_commands.md and src/main.rs.
Issue: #5316. Run from repo root:

    python .claude/skills/rhwp-cli/references/_gen_pack.py
"""

from __future__ import annotations

import json
import textwrap
from pathlib import Path

HERE = Path(__file__).resolve().parent
SKILL = HERE.parent
REFS = SKILL / "references"
EXAMPLES = SKILL / "examples"
FIXTURES = SKILL / "fixtures"
ENVELOPES = FIXTURES / "envelopes"
TRANSCRIPTS = FIXTURES / "transcripts"
TRACES = FIXTURES / "traces"

ISSUE = 5316
SKILL_NAME = "rhwp-cli"

DEBUG_ORDER = [
    {
        "step": 1,
        "command": "export-svg",
        "flags": ["--debug-overlay", "-p"],
        "why": "문단/표 식별. 라벨은 s{섹션}:pi={인덱스} y={좌표}",
        "output": "SVG overlay",
    },
    {
        "step": 2,
        "command": "dump-pages",
        "flags": ["-p"],
        "why": "해당 페이지 문단/표 배치 목록과 높이(vpos/lh/ls)",
        "output": "pagination dump",
    },
    {
        "step": 3,
        "command": "dump",
        "flags": ["-s", "-p"],
        "why": "ParaShape / LINE_SEG / 표·도형 속성 상세",
        "output": "control dump",
    },
    {
        "step": 4,
        "command": "ir-diff",
        "flags": ["-s", "-p", "--json"],
        "why": "HWPX↔HWP IR 불일치. --json 이면 차이는 exit 3 (판정 데이터)",
        "output": "IR categories",
    },
    {
        "step": 5,
        "command": "export-render-tree",
        "flags": ["-p"],
        "why": "bbox JSON. SVG 문자열 비교보다 좌표 분석에 정확",
        "output": "render_tree_NNN.json",
    },
    {
        "step": 6,
        "command": "hwp5-inventory-diff",
        "flags": ["--report", "--focus"],
        "why": "HWPX→HWP 저장 계약. oracle=한컴 저장본, generated=rhwp 저장본",
        "output": "inventory hints",
    },
]

UNITS = {
    "inch_hwpunit": 7200,
    "inch_mm": 25.4,
    "inch_px_dpi96": 96,
    "px_hwpunit": 75,
    "mm_hwpunit": 283.46,
    "formulas": [
        "1인치 = 7200 HWPUNIT = 25.4mm = 96px (DPI 96)",
        "1px = 75 HWPUNIT",
        "1mm ≈ 283.46 HWPUNIT",
        "페이지 번호는 0부터. PDF/한컴 표기는 1부터.",
        "extract-pages --from/--to 만 1 기준. -p 와 혼동하면 한 쪽 밀린다.",
    ],
}

HWP5 = [
    ("hwp5-inventory", "DocInfo/BodyText record inventory 생성"),
    ("hwp5-inventory-diff", "oracle vs generated inventory + contract 힌트"),
    ("hwp5-contract-analyze", "record-control contract graph 보고서"),
    ("hwp5-ctrl-data-trace", "CTRL_DATA ParameterSet 구조 추적"),
    ("hwp5-contract-probe", "MEMO_SHAPE/ID_MAPPINGS + 누락 CTRL_DATA probe"),
    ("hwp5-table-probe", "TABLE/CTRL_HEADER(Table) field 축 판정"),
    ("hwp5-cell-header-probe", "표 셀 LIST_HEADER/PARA_HEADER 계약"),
    ("hwp5-mel-personnel-probe", "mel-001 인원현황 표 축 판정"),
    ("hwp5-borderfill-diagonal-probe", "BORDER_FILL 대각선 attr/payload"),
    ("hwp5-first-para-control-probe", "첫 문단 control/PARA_TEXT/PARA_CHAR_SHAPE"),
    ("hwp5-anchor-trace", "특정 텍스트 주변 raw HWP5 record 추적"),
    ("hwp5-char-shape-audit", "CHAR_SHAPE sentinel 차이와 PARA_CHAR_SHAPE 사용 위치"),
    ("hwp5-roundtrip", "HWP5 → IR → HWP5 자기 라운드트립 (한컴 호환이 아님)"),
]

COMMANDS = [
    {
        "id": "export-svg",
        "family": "export",
        "request": ["SVG로 내보내", "시각 확인", "debug overlay", "겹침 보이게"],
        "argv": "export-svg <파일> [-p N] [-o 폴더] [--debug-overlay] [--json] [--profile print|screen]",
        "pageZero": True,
        "json": True,
        "notes": [
            "--debug-overlay 는 문단/표 경계와 s{섹션}:pi={인덱스} y={좌표} 라벨을 그린다.",
            "--json 매니페스트: schemaVersion/source/format/outputDir/pageCount/renderedCount/overflowCellLines/pages[].",
            "overflowCellLines > 0 이면 셀 줄이 쪽 하단 밖에 그려져 안 보인다 (#3668).",
            "--profile 생략 시 legacy 경로. 인쇄 등가면 --profile print 를 명시한다.",
            "--profile 과 --font-style/--embed-fonts 는 함께 쓸 수 없다 (exit 2).",
            "페이지 -p 는 0부터. 사용자가 4쪽을 말하면 -p 3.",
        ],
    },
    {
        "id": "export-png",
        "family": "export",
        "request": ["PNG로", "VLM 입력", "스크린샷처럼"],
        "argv": "export-png <파일> [-p N] [--vlm-target claude] [--scale] [--dpi] [--profile]",
        "pageZero": True,
        "json": False,
        "requiresFeature": "native-skia",
        "notes": [
            "native-skia feature 없이 빌드되면 stderr '오류: export-png 명령은 native-skia feature 가 활성화되어야 합니다.' 후 exit 2.",
            "기능 부재는 사용법(2)이다. 0으로 끝내면 스크립트가 성공으로 읽는다.",
            "--vlm-target: claude / gpt4v-low / gpt4v-high / gemini / qwen-vl / llava.",
            "기본 프로필은 high-quality (인쇄 등가). 편집기식 표시는 --profile screen.",
        ],
    },
    {
        "id": "export-pdf",
        "family": "export",
        "request": ["PDF로", "인쇄용으로"],
        "argv": "export-pdf <파일> [-o out.pdf] [-p N] [--backend svg|direct] [--json] [--profile]",
        "pageZero": True,
        "json": True,
        "notes": [
            "--backend direct 는 native-skia 빌드가 필요하다. 없으면 오류와 exit 1.",
            "폰트를 지정하지 않으면 번들 Noto 로 떨어져 글꼴이 바뀐다.",
            "--text-as-paths 는 메모리 절감, 텍스트 선택·검색 상실.",
            "--json 실패 경로의 stdout 은 비운다.",
        ],
    },
    {
        "id": "export-text",
        "family": "export",
        "request": ["텍스트 추출", "본문만", "페이지 텍스트"],
        "argv": "export-text <파일> [-p N] [-o 폴더] [--json] [--max-chars N]",
        "pageZero": True,
        "json": True,
        "notes": [
            "--json 봉투: schemaVersion/source/pageCount/truncated/omittedCount/pages[{page,text}].",
            "page 는 -p 와 같은 0 기준.",
            "--max-chars 는 --json 과 함께만. 파일 저장 모드에 쓰면 exit 2.",
            "조용히 자르지 않는다. truncated:true 와 omittedCount 를 싣는다.",
            "쪽 주소를 보존한다. 예산이 떨어져도 pages[] 항목을 빼지 않는다.",
        ],
    },
    {
        "id": "export-markdown",
        "family": "export",
        "request": ["마크다운으로", "md로 내보내"],
        "argv": "export-markdown <파일> [-p N] [-o 폴더] [--json]",
        "pageZero": True,
        "json": True,
        "notes": [
            "--json 매니페스트: format=markdown, pages[{page,path,bytes}].",
            "실패(부분 저장) 경로의 stdout 은 비운다.",
            "병합 표는 평문/마크다운에서 깨진다. 표는 export-tables / table-to-csv.",
        ],
    },
    {
        "id": "dump-pages",
        "family": "dump",
        "request": ["페이지네이션", "이 페이지 배치", "어느 문단이 어느 쪽"],
        "argv": "dump-pages <파일> [-p N] [--respect-vpos-reset]",
        "pageZero": True,
        "json": False,
        "notes": [
            "페이지별 문단/표 배치 목록 + 높이(vpos/lh/ls).",
            "레이아웃 디버그 2단. overlay 라벨의 인덱스를 여기서 확인한다.",
            "--respect-vpos-reset 은 LINE_SEG vpos=0 리셋을 단/페이지 강제 경계로 본다.",
        ],
    },
    {
        "id": "dump",
        "family": "dump",
        "request": ["조판부호", "문단 속성", "LINE_SEG", "표 속성"],
        "argv": "dump <파일> [-s N] [-p M]",
        "pageZero": False,
        "json": False,
        "notes": [
            "-s/-p 는 구역/문단 인덱스(0부터). dump-pages 의 -p 는 페이지다. 혼동 금지.",
            "ParaShape / LINE_SEG / 표·도형 속성. 상세는 dump_command.md.",
            "문단 헤더 0.3 은 구역 0 문단 3.",
            "용지 크기는 mm 와 HWPUNIT 을 병기한다 (예: 59528×84188 HU).",
        ],
    },
    {
        "id": "dump-records",
        "family": "dump",
        "request": ["raw record", "레코드 트리", "DocInfo 덤프"],
        "argv": "dump-records <파일>",
        "pageZero": False,
        "json": False,
        "notes": [
            "HWP5 raw record 덤프 (DocInfo/BodyText 레코드 트리).",
            "비밀번호 옵션을 받는다. 암호 문서는 --password-stdin.",
            "저장 계약 전에 inventory 가 더 기계친화적이다.",
        ],
    },
    {
        "id": "diag",
        "family": "diag",
        "request": ["번호 진단", "글머리표", "개요 수준"],
        "argv": "diag <파일>",
        "pageZero": False,
        "json": False,
        "notes": [
            "번호/글머리표/개요 분석.",
            "레이아웃 겹침의 1차 도구가 아니다. overlay → dump-pages 가 먼저다.",
        ],
    },
    {
        "id": "info",
        "family": "info",
        "request": ["파일 정보", "버전", "몇 쪽이냐", "암호화냐"],
        "argv": "info <파일> [--json]",
        "pageZero": False,
        "json": True,
        "notes": [
            "--json: schemaVersion/source/format/sizeBytes/version/sections/pageCount/paraCount/fonts.",
            "format 은 hwp5|hwpx|hwp3|hml. HML 이면 version 은 null.",
            "info 의 표 열거는 최상위 controls 만. 글상자·머리말 안 표는 놓친다.",
            "처음 보는 문서는 info 로 규모만 보고 전문 dump 하지 않는다 (rhwp-doc-triage).",
        ],
    },
    {
        "id": "export-render-tree",
        "family": "export",
        "request": ["bbox", "render tree", "좌표 JSON"],
        "argv": "export-render-tree <파일> [-p N] [-o 폴더]",
        "pageZero": True,
        "json": False,
        "notes": [
            "출력 render_tree_{NNN}.json. type + bbox{x,y,w,h} + children.",
            "레이아웃 디버그 5단. SVG 문자열 diff 보다 좌표에 정확.",
            "셀 내부는 절대 y 가 아니라 translate(x,y) 단위로 본다.",
        ],
    },
    {
        "id": "ir-diff",
        "family": "compare",
        "request": ["IR 비교", "HWPX와 HWP 차이", "변환이 같은지"],
        "argv": "ir-diff <a.hwpx> <b.hwp> [-s N] [-p M] [--summary] [--json]",
        "pageZero": False,
        "json": True,
        "notes": [
            "--json 봉투: schemaVersion/a/b/identical/diffCount/categories.",
            "--json 에서 차이 = exit 3 (판정 데이터). 텍스트 모드는 차이가 있어도 exit 0.",
            "읽기·파싱 실패는 exit 1, stdout 0바이트. 인자 부족은 exit 2.",
            "--summary 와 --json 을 같이 주면 JSON 이 이긴다.",
            "알 수 없는 옵션은 현재 조용히 무시될 수 있다. 플래그 철자를 정확히.",
        ],
    },
    {
        "id": "thumbnail",
        "family": "export",
        "request": ["썸네일", "미리보기 이미지"],
        "argv": "thumbnail <파일> [-o 파일] [--data-uri] [--base64]",
        "pageZero": False,
        "json": False,
        "notes": [
            "HWP 내장 썸네일(PrvImage) 추출. 렌더가 아니다.",
            "비밀번호 옵션을 적용하지 않는다 (미리보기만).",
            "기본 출력은 입력명_thumb.png.",
        ],
    },
    {
        "id": "convert",
        "family": "convert",
        "request": ["편집가능하게", "배포용 해제", "HWPX를 HWP로"],
        "argv": "convert <입력> <출력.hwp> [--verify] [--verify-pages]",
        "pageZero": False,
        "json": False,
        "notes": [
            "출력은 항상 .hwp. 다른 확장자면 읽기 전에 exit 2. HWPX 는 export-hwpx.",
            "--verify 차이 시 산출물은 남기고 exit 3.",
            "--verify-pages 불일치 시 산출물은 남기고 exit 4.",
            "자기 검증 통과 ≠ 한컴이 연다. 한컴 수동 검증이 최종 게이트.",
        ],
    },
    {
        "id": "hwp5-inventory-diff",
        "family": "hwp5",
        "request": ["저장 계약", "한컴 저장본과 비교", "oracle vs generated"],
        "argv": "hwp5-inventory-diff <oracle.hwp> <generated.hwp> [--report hints] [--focus table]",
        "pageZero": False,
        "json": False,
        "notes": [
            "oracle = 한컴 저장본, generated = rhwp 저장본. 순서를 뒤집지 말 것.",
            "레이아웃 디버그 6단. IR 이 같아도 record 축이 다를 수 있다.",
            "hwp5-roundtrip 통과는 자기 직렬화 보존이지 한컴 호환이 아니다.",
        ],
    },
]

EXCEPTIONS = [
    {
        "id": "missing_file",
        "kind": "missing-file",
        "command": "export-svg",
        "argv": ["export-svg", "없는파일.hwp", "-p", "0"],
        "exitCode": 1,
        "exitClass": "runtime",
        "stderrContains": "오류: 파일을 읽을 수 없습니다",
        "stdoutEmpty": True,
        "source": "src/main.rs fs::read → EXIT_RUNTIME",
        "doNot": "없는 경로를 성공으로 읽지 않는다. exit 0 이 아니다.",
    },
    {
        "id": "missing_file_info",
        "kind": "missing-file",
        "command": "info",
        "argv": ["info", "없는파일.hwp", "--json"],
        "exitCode": 1,
        "exitClass": "runtime",
        "stderrContains": "오류: 파일을 읽을 수 없습니다",
        "stdoutEmpty": True,
        "source": "같은 fs::read 계약",
        "doNot": "--json 실패 경로의 stdout 은 비운다.",
    },
    {
        "id": "bad_page_index",
        "kind": "bad-page-index",
        "command": "export-svg",
        "argv": ["export-svg", "sample.hwp", "-p", "99"],
        "exitCode": 2,
        "exitClass": "usage",
        "stderrContains": "오류: 페이지 번호가 범위를 벗어났습니다 (0~",
        "stdoutEmpty": True,
        "source": "src/main.rs page >= page_count → EXIT_USAGE",
        "doNot": "페이지 범위 초과는 런타임(1)이 아니라 사용법(2).",
    },
    {
        "id": "bad_page_png",
        "kind": "bad-page-index",
        "command": "export-png",
        "argv": ["export-png", "sample.hwp", "-p", "99"],
        "exitCode": 2,
        "exitClass": "usage",
        "stderrContains": "오류: 페이지 번호가 범위를 벗어났습니다 (0~",
        "stdoutEmpty": True,
        "source": "export-png 동일 검사",
        "doNot": "한컴 4쪽을 -p 4 로 넣으면 5번째를 찾고 범위 초과가 난다.",
    },
    {
        "id": "native_skia_missing",
        "kind": "native-skia-missing",
        "command": "export-png",
        "argv": ["export-png", "sample.hwp"],
        "exitCode": 2,
        "exitClass": "usage",
        "stderrContains": "오류: export-png 명령은 native-skia feature 가 활성화되어야 합니다.",
        "stdoutEmpty": True,
        "source": "src/main.rs #[cfg(not(feature = \"native-skia\"))] stub",
        "doNot": "기능 부재를 성공(0)이나 런타임(1)으로 읽지 않는다. 사용법(2).",
        "repair": "cargo build --release --features native-skia",
    },
    {
        "id": "native_skia_direct_pdf",
        "kind": "native-skia-missing",
        "command": "export-pdf",
        "argv": ["export-pdf", "sample.hwp", "--backend", "direct"],
        "exitCode": 1,
        "exitClass": "runtime",
        "stderrContains": "direct PDF backend requires a build with the native-skia feature",
        "stdoutEmpty": True,
        "source": "export-pdf --backend direct 스텁은 RenderError → exit 1",
        "doNot": "export-png 스텁(exit 2)과 코드를 섞지 말 것. direct PDF 는 1.",
    },
    {
        "id": "load_fail",
        "kind": "load-fail",
        "command": "info",
        "argv": ["info", "truncated.hwp"],
        "exitCode": 1,
        "exitClass": "runtime",
        "stderrContains": "오류: 문서 파싱 실패 -",
        "stdoutEmpty": True,
        "source": "LoadError::Other → EXIT_RUNTIME",
        "doNot": "손상 OLE/잘린 파일을 빈 문서로 성공 처리하지 않는다.",
    },
    {
        "id": "load_fail_export",
        "kind": "load-fail",
        "command": "export-text",
        "argv": ["export-text", "not_hwp.txt", "--json"],
        "exitCode": 1,
        "exitClass": "runtime",
        "stderrContains": "오류: 문서 파싱 실패 -",
        "stdoutEmpty": True,
        "source": "detect_format + load_document 실패",
        "doNot": "확장자만 .hwp 인 텍스트를 본문으로 읽지 않는다.",
    },
    {
        "id": "need_password",
        "kind": "need-password",
        "command": "info",
        "argv": ["info", "protected.hwp"],
        "exitCode": 2,
        "exitClass": "usage",
        "stderrContains": "오류: 비밀번호가 필요한 암호 문서입니다",
        "stdoutEmpty": True,
        "source": "LoadError::NeedPassword → EXIT_USAGE",
        "doNot": "암호 문서를 로드 실패(1)와 같은 봉투로 묶지 말 것. 비밀번호 없음은 2.",
    },
    {
        "id": "wrong_password",
        "kind": "wrong-password",
        "command": "info",
        "argv": ["info", "protected.hwp", "--password", "wrong"],
        "exitCode": 1,
        "exitClass": "runtime",
        "stderrContains": "오류: 비밀번호가 일치하지 않거나 암호화 데이터가 손상되었습니다.",
        "stdoutEmpty": True,
        "source": "LoadError::WrongPassword → EXIT_RUNTIME",
        "doNot": "틀린 비밀번호를 사용법(2)으로 재해석하지 않는다.",
    },
    {
        "id": "no_input",
        "kind": "no-input",
        "command": "export-svg",
        "argv": ["export-svg"],
        "exitCode": 2,
        "exitClass": "usage",
        "stderrContains": "오류: 문서 파일 경로를 지정해주세요.",
        "stdoutEmpty": True,
        "source": "positional 누락",
        "doNot": "인자 없음을 런타임으로 읽지 않는다.",
    },
    {
        "id": "ir_diff_mismatch",
        "kind": "ir-diff-data",
        "command": "ir-diff",
        "argv": ["ir-diff", "a.hwpx", "b.hwp", "--json"],
        "exitCode": 3,
        "exitClass": "ir-diff",
        "stderrContains": "",
        "stdoutEmpty": False,
        "source": "#3274 --json 차이 = exit 3",
        "doNot": "exit 3 을 크래시로 읽지 않는다. identical:false 가 데이터다.",
        "stdoutKeys": ["schemaVersion", "a", "b", "identical", "diffCount", "categories"],
    },
]

INTENTS = [
    ("svg로 빼줘", "export-svg", "시각 확인 1단"),
    ("3쪽을 svg", "export-svg", "-p 2 (0 기준)"),
    ("debug overlay", "export-svg", "--debug-overlay"),
    ("겹침 보이게", "export-svg", "--debug-overlay"),
    ("png로 비전 모델에", "export-png", "--vlm-target claude"),
    ("인쇄용 pdf", "export-pdf", "--profile print 권장"),
    ("본문만 텍스트", "export-text", "--json --max-chars 로 예산"),
    ("마크다운으로", "export-markdown", "표 병합은 별도"),
    ("이 페이지 배치", "dump-pages", "디버그 2단"),
    ("문단 속성", "dump", "-s -p 는 구역/문단"),
    ("raw record", "dump-records", "HWP5 트리"),
    ("번호가 이상해", "diag", "개요/글머리표"),
    ("몇 쪽이야", "info", "--json pageCount"),
    ("bbox 좌표", "export-render-tree", "디버그 5단"),
    ("hwpx랑 hwp 비교", "ir-diff", "--json 이면 exit 3=차이"),
    ("썸네일만", "thumbnail", "PrvImage, 렌더 아님"),
    ("배포용 풀고 편집", "convert", "출력은 .hwp"),
    ("한컴 저장이랑 달라", "hwp5-inventory-diff", "oracle vs generated"),
    ("표 저장 계약", "hwp5-table-probe", "hwp5 가족"),
    ("특정 글자 주변 record", "hwp5-anchor-trace", "--needle"),
    ("CHAR_SHAPE 차이", "hwp5-char-shape-audit", "--out 필수"),
    ("자기 라운드트립", "hwp5-roundtrip", "한컴 호환이 아님"),
    ("레이아웃 버그", "export-svg", "디버그 1단부터"),
    ("간격이 이상해", "dump-pages", "2단 높이"),
    ("셀이 잘려", "export-svg", "overflowCellLines"),
]


def write_text(path: Path, body: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not body.endswith("\n"):
        body += "\n"
    path.write_text(body.replace("\r\n", "\n"), encoding="utf-8", newline="\n")


def write_json(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    text = json.dumps(obj, ensure_ascii=False, indent=2) + "\n"
    path.write_text(text.replace("\r\n", "\n"), encoding="utf-8", newline="\n")


def wrap(s: str) -> str:
    return textwrap.dedent(s).strip() + "\n"


def emit_tree() -> str:
    lines = [
        "# 분석·디버깅 판단 트리",
        "",
        "권위는 `mydocs/manual/cli_commands.md` 와 `src/main.rs` 디스패치다.",
        "이 장은 기존 CLI 를 설명하고 소비 규칙을 고정한다. 새 명령·새 플래그를 발명하지 않는다.",
        "",
        "이 스킬은 **gym 이 아니다.** 실사용 에이전트가 HWP/HWPX 를 내보내고 레이아웃을 좁히는 경로다.",
        "",
        "## 한 줄",
        "",
        "요청을 명령으로 매핑하고, 겹침·간격은 overlay → dump-pages → dump → ir-diff → render-tree → hwp5-inventory-diff 순으로 좁힌다.",
        "",
        "페이지는 0부터. 자기 라운드트립 통과는 한컴 호환이 아니다.",
        "",
        "## 요청 트리",
        "",
        "```",
        "사용자가 파일을 준다",
        "  ├─ 없다 / 깨졌다 ──▶ 예외 봉투 (missing-file · load-fail). 명령을 발명하지 않음",
        "  ├─ 규모만 ──▶ info --json",
        "  ├─ 본문 ──▶ export-text [--json --max-chars]",
        "  ├─ 그림/인쇄 ──▶ export-svg / export-png / export-pdf",
        "  ├─ 레이아웃·겹침 ──▶ 디버그 6단 (아래)",
        "  ├─ 두 파일 내용 ──▶ ir-diff --json",
        "  └─ 한컴 저장과 다름 ──▶ hwp5-inventory-diff oracle generated",
        "```",
        "",
        "## 레이아웃 디버그 6단 (강제 순회가 아님, 기본 순서)",
        "",
        "```",
        "1 export-svg --debug-overlay -p N",
        "    └─ 라벨 s{섹션}:pi={인덱스} y={좌표}",
        "2 dump-pages -p N",
        "    └─ 배치 목록 + vpos/lh/ls",
        "3 dump -s N -p M",
        "    └─ ParaShape / LINE_SEG / 표 속성",
        "4 ir-diff a.hwpx b.hwp [-s N -p M] [--json]",
        "    └─ 형식 쌍이 있을 때만. 차이는 데이터",
        "5 export-render-tree -p N",
        "    └─ bbox JSON. translate 단위",
        "6 hwp5-inventory-diff oracle.hwp generated.hwp",
        "    └─ 저장 계약. oracle=한컴, generated=rhwp",
        "```",
        "",
        "답이 나오면 다음 단으로 내려가지 않는다. overlay 라벨만으로 문단이 보이면 dump 로 점프해도 된다.",
        "순서를 건너뛰어 render-tree 부터 여는 것은 금지 기본값이 아니다 — 다만 인덱스 없이 bbox 를 읽으면 좌표만 떠다닌다.",
        "",
        "## 분기 필드",
        "",
        "| 단계 | 보는 것 | 다음 |",
        "|---|---|---|",
        "| info | pageCount, format, version | 규모만이면 정지 |",
        "| export-svg --json | overflowCellLines | >0 이면 셀 소실 |",
        "| dump-pages | vpos/lh/ls | 높이 이상이면 dump |",
        "| ir-diff --json | identical, diffCount, categories | exit 3 = 데이터 |",
        "| convert --verify | exit 3, 산출물 잔류 | 한컴 검증은 별도 |",
        "| hwp5-* | oracle vs generated | 순서를 뒤집지 않음 |",
        "",
        "## 페이지와 단위",
        "",
        "- `-p` / export-text `pages[].page` / search matches 는 **0부터**.",
        "- 사용자가 \"4쪽\" 이라고 하면 한컴·PDF 표기이므로 `-p 3`.",
        "- `extract-pages --from/--to` 만 **1부터**. 이 스킬의 기본 축이 아니다.",
        "- 1인치=7200 HWPUNIT=96px, 1px=75 HWPUNIT, 1mm≈283.46 HWPUNIT.",
        "",
        "## 금지 진입",
        "",
        "- 새 CLI 하위명령·플래그 발명",
        "- gym/ 팩으로 이 트리를 대체",
        "- 자기 hwp5-roundtrip 통과를 한컴 호환으로 보고",
        "- oracle/generated 순서를 추측으로 뒤집기",
        "- 페이지 1부터를 `-p` 에 그대로 넣기",
        "- 없는 파일·깨진 파일을 빈 성공으로 삼키기",
        "- DocumentCore 편집 로직을 이 스킬에서 고치기",
        "",
        "## 관련",
        "",
        "[01_request_command_map.md](01_request_command_map.md) · [17_layout_debug_order.md](17_layout_debug_order.md) · [21_exception_envelopes.md](21_exception_envelopes.md)",
        "",
        "## 질문 카드",
        "",
        "| 질문 | 첫 명령 | 정지 |",
        "|---|---|---|",
        "| 이 파일 뭐야 | info --json | 규모만이면 정지 |",
        "| 1쪽 그림으로 | export-svg -p 0 | 파일 생성 |",
        "| 겹친다 | export-svg --debug-overlay | 라벨 |",
        "| 몇 문단이 이 쪽 | dump-pages -p N | 목록 |",
        "| 줄간격 숫자 | dump -s -p | LINE_SEG |",
        "| 두 파일이 같나 | ir-diff --json | identical |",
        "| 한컴이 안 연다 | hwp5-inventory-diff | 축 힌트 |",
        "| PNG 가 거절 | native-skia 봉투 | 재빌드 안내 |",
        "",
    ]
    return "\n".join(lines) + "\n"


def emit_command_ref(cmd: dict, index: int) -> str:
    title = cmd["id"]
    lines = [
        f"# {title}",
        "",
        f"권위: `mydocs/manual/cli_commands.md` 의 `{title}` 절, `src/main.rs` 디스패치.",
        "새 플래그를 발명하지 않는다. 여기 없는 옵션은 `--help` 와 매뉴얼을 본다.",
        "",
        "## 한 줄",
        "",
        f"`rhwp {cmd['argv']}`",
        "",
        "## 요청 매핑",
        "",
    ]
    for req in cmd["request"]:
        lines.append(f"- \"{req}\" → `{title}`")
    lines += [
        "",
        "## 페이지",
        "",
        (
            "이 명령의 `-p` 는 **0부터**다. 사용자가 한컴 쪽번호를 말하면 1을 뺀다."
            if cmd["pageZero"]
            else "이 명령의 `-p`/`-s` 는 페이지가 아니라 문단/구역 인덱스(0부터)일 수 있다. dump-pages 의 `-p`(페이지)와 섞지 말 것."
        ),
        "",
        "## 계약 메모",
        "",
    ]
    for n in cmd["notes"]:
        lines.append(f"- {n}")
    if cmd.get("requiresFeature"):
        lines += [
            "",
            "## feature 게이트",
            "",
            f"이 명령은 `{cmd['requiresFeature']}` 가 필요하다.",
            "없으면 stderr 에 기능 부재를 알리고 exit 2 (export-png) 또는 해당 백엔드는 exit 1 (pdf direct).",
            "`capabilities` 의 `requiresFeature`/`available` 을 먼저 본다 (#3357).",
        ]
    lines += [
        "",
        "## 예외",
        "",
        "공통 네 봉투:",
        "",
        "1. 파일 없음 — `오류: 파일을 읽을 수 없습니다 - {path}: {os}` , exit 1",
        "2. 페이지 범위 — `오류: 페이지 번호가 범위를 벗어났습니다 (0~{max})` , exit 2",
        "3. native-skia 부재 — export-png 스텁 exit 2",
        "4. 로드 실패 — `오류: 문서 파싱 실패 - {msg}` , exit 1",
        "",
        "실패 경로의 `--json` stdout 은 비운다. 부분 JSON 을 파싱하지 말 것.",
        "",
        "## 실측 레시피",
        "",
        "```bash",
        f"rhwp {cmd['argv'].split()[0]} samples/basic/KTX.hwp"
        + (" -p 0" if cmd["pageZero"] else ""),
        "```",
        "",
        "산출은 `output/poc/agent-cli/` 아래로 분리한다. 원본은 읽기 전용 명령에서 불변이다.",
        "convert 만 출력을 쓰며, 입력과 같은 경로를 거부하는 명령은 그 계약을 따른다.",
        "",
        "## 하지 않는 것",
        "",
        "- 이 명령의 새 별칭을 만들지 않는다.",
        "- gym 과제로 이 명령을 대체하지 않는다.",
        "- DocumentCore 를 열어 렌더를 고치지 않는다. 분석만 한다.",
        "",
        f"관련: [00_tree.md](00_tree.md) · [01_request_command_map.md](01_request_command_map.md) · 장 {index:02d}",
        "",
    ]
    # Extra worked rows so each chapter is a real cookbook, not a stub.
    extra_cases = [
        ("없는 경로", f"rhwp {title} 없는파일.hwp", "exit 1, missing-file"),
        ("인자 없음", f"rhwp {title}", "exit 2, no-input (인자가 필수인 명령)"),
        ("한컴 4쪽", f"rhwp {title} doc.hwp -p 3" if cmd["pageZero"] else f"rhwp {title} doc.hwp -s 0 -p 3", "0 기준"),
        ("JSON 실패", f"rhwp {title} broken.hwp --json" if cmd["json"] else "(json 없음)", "stdout 0바이트"),
    ]
    lines += ["## 호출 카드", "", "| 상황 | 명령 | 읽는 것 |", "|---|---|---|"]
    for a, b, c in extra_cases:
        lines.append(f"| {a} | `{b}` | {c} |")
    lines += ["", "## 소비 규칙", ""]
    if cmd["json"]:
        lines.append("`--json` 성공 시 stdout 순수 JSON 한 줄. stderr 진행 메시지와 섞지 않는다.")
        lines.append("파이프는 `jq` 로 필드만 고른다. 실패면 jq 를 돌리지 말고 exit 를 본다.")
    else:
        lines.append("사람용 텍스트. 자동화 게이트가 필요하면 형제 명령의 `--json` 을 쓴다.")
        lines.append("이 명령에 `--json` 을 발명해서 붙이지 않는다.")
    lines += [
        "",
        "## 인계",
        "",
        "- 긴 문서 파악만 → `rhwp-doc-triage` (info/digest/search). 이 스킬은 분석·디버그.",
        "- 시각 회귀 숫자 판정 → `rhwp-visual-regression` (render-diff). 여기선 overlay 와 tree.",
        "- 편집 → `rhwp-safe-edit`. 이 스킬은 원본을 고치지 않는다 (convert 제외, 출력 분리).",
        "",
    ]
    return "\n".join(lines) + "\n"


def emit_hwp5_ref() -> str:
    lines = [
        "# hwp5-* 가족 — HWPX→HWP 저장 계약",
        "",
        "권위: `cli_commands.md` §4. oracle = 한컴 저장본, generated = rhwp 저장본.",
        "#178 어댑터로 HWPX 를 열어 HWP 로 썼을 때 한컴이 다르게 여는 경우의 record 축 분석.",
        "문서를 고치지 않는 **진단 전용**이다. 새 hwp5 명령을 발명하지 않는다.",
        "",
        "## 왜 자기 라운드트립이 부족한가",
        "",
        "`hwp5-roundtrip` 은 HWP5 → IR → HWP5 자기 직렬화 보존이다.",
        "통과해도 한컴이 같은 파일을 연다는 뜻이 아니다.",
        "한컴이 쓰는 record 순·CTRL_DATA·CHAR_SHAPE sentinel 은 oracle 과 비교해야 보인다.",
        "",
        "## 명령 표",
        "",
        "| 명령 | 용도 |",
        "|---|---|",
    ]
    for name, why in HWP5:
        lines.append(f"| `{name}` | {why} |")
    lines += [
        "",
        "## 기본 레시피",
        "",
        "```bash",
        "# 1. 한컴이 저장한 정본과 rhwp 가 저장한 산출을 나란히 둔다",
        "rhwp hwp5-inventory-diff oracle.hwp generated.hwp --report hints --focus table",
        "",
        "# 2. 표 축이 의심되면 probe",
        "rhwp hwp5-table-probe oracle.hwp generated.hwp --out-dir output/poc/probe/",
        "rhwp hwp5-cell-header-probe oracle.hwp generated.hwp --out-dir output/poc/probe/",
        "",
        "# 3. 특정 글자 주변 raw record",
        "rhwp hwp5-anchor-trace generated.hwp --needle \"특정텍스트\" --section 0",
        "",
        "# 4. CHAR_SHAPE sentinel",
        "rhwp hwp5-char-shape-audit oracle.hwp generated.hwp --out output/char-shape-audit.md",
        "```",
        "",
        "## 순서 계약",
        "",
        "첫 positional 은 항상 oracle(한컴). 둘째는 generated(rhwp).",
        "뒤집으면 힌트가 \"한컴이 과다\" / \"rhwp 가 과다\" 를 반대로 말한다.",
        "픽스처와 예제는 파일명에 `oracle` / `generated` 를 박아 순서를 고정한다.",
        "",
        "## 성공·실패",
        "",
        "hwp5-char-shape-audit: 성공 0, 읽기/쓰기 실패 1, 인자 누락 2.",
        "성공 시 stdout 은 `written: <보고서 경로>` 한 줄.",
        "`--out` 은 이 명령에서 필수다.",
        "",
        "inventory-diff 의 차이 보고는 **진단 데이터**다. 차이 자체를 크래시로 승격하지 않는다.",
        "",
        "## 레이아웃 사다리에서의 위치",
        "",
        "6단. IR(4단) 과 bbox(5단) 다음에 온다.",
        "화면은 같은데 한컴이 안 열리면 1–5 를 건너뛰고 여기로 와도 된다.",
        "화면이 다른데 inventory 만 보면 좌표를 놓친다 — overlay 가 먼저다.",
        "",
        "## 하지 않는 것",
        "",
        "- Hancom record 를 runtime serializer 에 주입하는 기능을 여기서 설계하지 않는다.",
        "- equivalent 논리 payload 만 보고 canonicalization 을 적용하지 않는다.",
        "- PARA_LINE_SEG bit 0 누적 쪽수를 한컴 PDF 쪽번호와 같다고 가정하지 않는다.",
        "- gym 저장 계약 팩을 이 장의 정본으로 쓰지 않는다.",
        "",
    ]
    return "\n".join(lines) + "\n"


def emit_debug_order() -> str:
    lines = [
        "# 레이아웃·겹침 디버그 순서",
        "",
        "코드 무수정으로 결함을 좁힌다. 순서는 이슈 #5316 과 스킬 본문이 같은 단어를 쓴다.",
        "",
        "1. `export-svg --debug-overlay`",
        "2. `dump-pages`",
        "3. `dump`",
        "4. `ir-diff`",
        "5. `export-render-tree`",
        "6. `hwp5-inventory-diff`",
        "",
        "`cli_commands.md` §6 은 5·6 이 뒤바뀐 참고 순서가 있다. **이 스킬의 계약 순서는 위 여섯**이다.",
        "render-tree(좌표)를 inventory(저장 record)보다 먼저 본다. 화면 버그는 좌표가 먼저다.",
        "",
        "## 단별 산출",
        "",
    ]
    for step in DEBUG_ORDER:
        lines += [
            f"### {step['step']}. `{step['command']}`",
            "",
            f"- 플래그: {', '.join('`'+f+'`' for f in step['flags'])}",
            f"- 이유: {step['why']}",
            f"- 산출: {step['output']}",
            "",
        ]
    lines += [
        "## 겹침 여정 (실측 순서)",
        "",
        "```bash",
        "# 사용자가 \"3쪽이 겹친다\" — 한컴 3쪽 = -p 2",
        "rhwp export-svg 보고서.hwp --debug-overlay -p 2 -o output/poc/overlap/",
        "# SVG 라벨에서 s0:pi=14 y=... 를 읽는다",
        "rhwp dump-pages 보고서.hwp -p 2",
        "# 문단 14 의 vpos/lh 가 옆 표와 겹치면",
        "rhwp dump 보고서.hwp -s 0 -p 14",
        "# HWPX 원본이 있으면",
        "rhwp ir-diff 보고서.hwpx 보고서.hwp -s 0 -p 14 --json",
        "# 좌표를 숫자로",
        "rhwp export-render-tree 보고서.hwp -p 2 -o output/poc/overlap/tree/",
        "# 한컴 저장본이 있으면",
        "rhwp hwp5-inventory-diff oracle.hwp generated.hwp --focus table",
        "```",
        "",
        "## 정지",
        "",
        "| 단 | 멈추는 때 |",
        "|---|---|",
        "| 1 | overlay 라벨만으로 문단/표가 특정됨 |",
        "| 2 | 높이 숫자가 겹침을 설명함 |",
        "| 3 | ParaShape/LINE_SEG 가 원인 후보 |",
        "| 4 | identical:true 이면 형식 쌍 문제는 아님 |",
        "| 5 | bbox 가 두 파일에서 갈라짐 |",
        "| 6 | record 힌트가 저장 축을 가리킴 |",
        "",
        "구현 수정은 별도 이슈다. 이 스킬은 좁히기만 한다.",
        "",
        "## 보정 전/후",
        "",
        "레이아웃 변경의 회귀는 테스트 golden 통과로 안 잡힐 수 있다.",
        "기준 브랜치 SVG 와 변경 SVG 를 `output/poc/before|after` 에 두고 페이지별로 본다.",
        "좌표 분석은 render-tree JSON diff 가 SVG 문자열보다 정확하다.",
        "셀 내부는 `translate(x,y)` 단위다.",
        "",
        "시각 회귀의 **숫자 게이트**(render-diff max-disp)는 `rhwp-visual-regression` 스킬.",
        "여기서는 그 명령을 발명하지 않고, 사람이 overlay 를 읽는 사다리만 닫는다.",
        "",
    ]
    return "\n".join(lines) + "\n"


def emit_page_units() -> str:
    lines = [
        "# 페이지 번호와 HWPUNIT",
        "",
        "페이지 번호는 **0부터**. 단위는 HWPUNIT.",
        "이 두 가지를 틀리면 올바른 명령을 잘못된 쪽에 쏜다. 결함이 아닌데 결함처럼 보인다.",
        "",
        "## 0 기준 축",
        "",
        "| 표면 | 필드/플래그 | 기준 |",
        "|---|---|---|",
        "| export-svg/png/pdf/text/markdown | `-p`, pages[].page | 0 |",
        "| dump-pages | `-p` | 0 (페이지) |",
        "| dump | `-s` 구역, `-p` 문단 | 0 (페이지 아님) |",
        "| export-render-tree | `-p` | 0 |",
        "| ir-diff | `-s` 구역, `-p` 문단 | 0 (페이지 아님) |",
        "| export-text --json | pages[].page | 0 |",
        "| search --json | matches[].page | 0 |",
        "| digest --pages a..b | 0, 양끝 포함, a<=b | 0 |",
        "",
        "## 1 기준 예외 (이 스킬 기본 축 아님)",
        "",
        "`extract-pages --from/--to` 는 **1부터**. `search` 가 `page: 1` 을 주면 여기서는 `--from 2 --to 2`.",
        "차트 `--chart` 도 문서 순서 1부터. 표 `--table` 은 0부터.",
        "",
        "## 환산",
        "",
    ]
    for f in UNITS["formulas"]:
        lines.append(f"- {f}")
    lines += [
        "",
        f"- 1인치 = {UNITS['inch_hwpunit']} HWPUNIT = {UNITS['inch_mm']}mm = {UNITS['inch_px_dpi96']}px",
        f"- 1px = {UNITS['px_hwpunit']} HWPUNIT",
        f"- 1mm ≈ {UNITS['mm_hwpunit']} HWPUNIT",
        "",
        "## 계산 카드",
        "",
        "| 입력 | 연산 | 결과 |",
        "|---|---|---|",
        "| 한컴 1쪽 | 1-1 | `-p 0` |",
        "| 한컴 4쪽 | 4-1 | `-p 3` |",
        "| 10mm | 10 × 283.46 | 2834.6 HU |",
        "| 96px (1인치) | 96 × 75 | 7200 HU |",
        "| A4 가로 | 210mm × 283.46 | ≈ 59526 HU (덤프는 59528) |",
        "| A4 세로 | 297mm × 283.46 | ≈ 84188 HU |",
        "",
        "덤프 헤더 예: `용지: 210.0mm × 297.0mm (59528×84188 HU)`.",
        "반올림이 있으니 HU 를 mm 로 역산한 뒤 다시 곱해 같기를 기대하지 말 것.",
        "",
        "## 함정",
        "",
        "- 사용자가 \"페이지 0\" 이라고 하면 이미 0 기준인지 한 번 확인한다.",
        "- dump 의 `-p` 를 dump-pages 의 `-p` 로 읽으면 문단 3 을 페이지 3 으로 연다.",
        "- PDF 뷰어 쪽번호는 1부터. export-pdf `-p 0` 은 PDF 의 첫 쪽.",
        "- ir-diff `-p` 는 문단이다. 페이지를 좁히려면 먼저 dump-pages 로 문단 번호를 얻는다.",
        "",
    ]
    return "\n".join(lines) + "\n"


def emit_roundtrip() -> str:
    lines = [
        "# 자기 라운드트립 ≠ 한컴 호환",
        "",
        "이 명제는 스킬 전역 불변식이다. 테스트가 golden 을 통과하고,",
        "`hwp5-roundtrip` / `hwpx-roundtrip` / `render-diff` 자기 비교가 PASS 여도",
        "한컴이 같은 화면을 보여주거나 같은 파일을 연다는 뜻이 아니다.",
        "",
        "## 세 층",
        "",
        "| 층 | 명령 | 무엇을 말하나 | 말하지 않는 것 |",
        "|---|---|---|---|",
        "| 구조 보존 | hwpx-roundtrip, hwp5-roundtrip | 우리 IR↔직렬화가 닫힘 | 한컴 파서 호환 |",
        "| 자기 시각 | render-diff (한 파일) | 직렬화 전후 bbox 변위 | 한컴 PDF 충실 |",
        "| 한컴 계약 | hwp5-inventory-diff + 한컴 수동 | oracle record / 한컴 화면 | 우리 테스트 초록불 |",
        "",
        "## 에이전트가 하면 안 되는 보고",
        "",
        "- \"라운드트립 통과했으니 한컴에서 열립니다\"",
        "- \"render-diff PASS 이니 간격이 맞습니다\"",
        "- \"IR identical 이니 저장본이 같습니다\"",
        "",
        "올바른 보고:",
        "",
        "- \"자기 직렬화는 닫혔다. 한컴 검증은 남아 있다.\"",
        "- \"IR 카테고리 X 가 N 건. 화면은 overlay 로 따로 봤다.\"",
        "- \"oracle/generated inventory 힌트는 TABLE 축. 한컴 열기 여부는 미확인.\"",
        "",
        "## convert --verify",
        "",
        "`--verify` 는 저장 후 재파싱 IR 과 어댑터 적용 후 IR 을 비교한다.",
        "차이 시 산출물은 남기고 exit 3. 이것도 **자기** 검증이다.",
        "`--verify-pages` 는 쪽수 비교, 불일치 시 exit 4. 한컴 쪽수가 아니다.",
        "",
        "## render-diff 주의",
        "",
        "매뉴얼 문구: 자기 roundtrip 통과 ≠ 한컴 충실. 내부 회귀 방지용.",
        "한컴 PDF 기준은 `tools/fidelity_compare` 등 별도 경로이며 이 스킬의 기본 축이 아니다.",
        "",
        "## 최종 게이트",
        "",
        "저장·렌더 결함의 최종 게이트는 한컴 수동 검증이다.",
        "이 스킬은 그 전에 기계로 좁힌다. 좁힌 결과를 최종 합격으로 승격하지 않는다.",
        "",
    ]
    return "\n".join(lines) + "\n"


def emit_save_contract() -> str:
    lines = [
        "# HWPX→HWP 저장 계약 (oracle vs generated)",
        "",
        "HWPX 로 연 문서를 HWP 로 저장(#178 어댑터)했을 때 한컴이 다르게 여는 경우.",
        "",
        "## 이름",
        "",
        "| 이름 | 누구의 손 | 파일 예 |",
        "|---|---|---|",
        "| oracle | 한컴이 저장한 HWP | `oracle.hwp`, `hancom-saved.hwp` |",
        "| generated | rhwp 가 저장한 HWP | `generated.hwp`, `rhwp-saved.hwp` |",
        "| source | 원본 HWPX | `source.hwpx` |",
        "",
        "명령 인자는 항상 `oracle generated` 순이다.",
        "",
        "## 언제 이 축인가",
        "",
        "- \"한컴에서 안 열려요\" / \"표가 한컴이랑 달라요\" / \"저장하니까 깨져요\"",
        "- convert 산출을 한컴이 거부",
        "- IR 은 같은데 한컴만 실패",
        "",
        "화면 겹침만 있고 한컴 저장본이 없으면 1–5단으로 남는다. 6단을 가짜 oracle 로 채우지 말 것.",
        "",
        "## 최소 세트",
        "",
        "```bash",
        "rhwp hwp5-inventory-diff oracle.hwp generated.hwp --report hints --focus table",
        "rhwp hwp5-table-probe oracle.hwp generated.hwp --out-dir output/poc/probe/",
        "rhwp hwp5-anchor-trace generated.hwp --needle \"문제문장\" --section 0",
        "```",
        "",
        "원본 HWPX 가 있으면:",
        "",
        "```bash",
        "rhwp hwp5-contract-analyze source.hwpx oracle.hwp generated.hwp --out-dir output/poc/contract/",
        "rhwp hwp5-char-shape-audit oracle.hwp generated.hwp --source-hwpx source.hwpx --out output/audit.md",
        "```",
        "",
        "## 읽기 규칙",
        "",
        "- inventory 힌트는 축 후보다. serializer 패치를 여기서 쓰지 않는다.",
        "- CHAR_SHAPE equivalent 는 비활성 underline/strike/shadow sentinel 제거 비교다.",
        "- PARA_LINE_SEG 쪽수 표식이 0 일 수 있다. 한컴 PDF 쪽번호와 같지 않다.",
        "- 같은 source charPr signature 가 서로 다른 raw 분류에 나타나면 선택 기준으로 쓰지 않는다.",
        "",
        "## 자기 라운드트립과의 관계",
        "",
        "`hwp5-roundtrip generated.hwp` 가 통과해도 oracle 과 다를 수 있다.",
        "자기 닫힘과 한컴 계약은 다른 명제다. [19_roundtrip_vs_hangul.md](19_roundtrip_vs_hangul.md).",
        "",
    ]
    return "\n".join(lines) + "\n"


def emit_exceptions() -> str:
    lines = [
        "# 예외 봉투",
        "",
        "실패는 추측하지 않고 아래 네 종류로 먼저 분류한다.",
        "메시지는 `src/main.rs` 문자열을 그대로 인용한다. 의역하지 않는다.",
        "",
        "| kind | 대표 stderr | exit | class |",
        "|---|---|---|---|",
        "| missing-file | `오류: 파일을 읽을 수 없습니다 - {path}: {os}` | 1 | runtime |",
        "| bad-page-index | `오류: 페이지 번호가 범위를 벗어났습니다 (0~{max})` | 2 | usage |",
        "| native-skia-missing | `오류: export-png 명령은 native-skia feature 가 활성화되어야 합니다.` | 2 | usage |",
        "| load-fail | `오류: 문서 파싱 실패 - {msg}` | 1 | runtime |",
        "",
        "## 부가 봉투 (같은 장에서 혼동 방지)",
        "",
        "| kind | stderr | exit |",
        "|---|---|---|",
        "| no-input | `오류: 문서 파일 경로를 지정해주세요.` | 2 |",
        "| need-password | `오류: 비밀번호가 필요한 암호 문서입니다` | 2 |",
        "| wrong-password | `오류: 비밀번호가 일치하지 않거나 암호화 데이터가 손상되었습니다.` | 1 |",
        "| ir-diff-data | (stdout JSON, identical:false) | 3 |",
        "| pdf-direct-no-skia | `direct PDF backend requires a build with the native-skia feature` | 1 |",
        "",
        "export-png 기능 부재는 **2**, export-pdf `--backend direct` 기능 부재는 **1**.",
        "코드를 하나로 합치지 말 것.",
        "",
        "## 소비",
        "",
        "- 실패 경로 stdout 은 0바이트. 빈 JSON 을 만들지 않는다.",
        "- `--json` 을 붙였어도 실패면 jq 하지 않는다.",
        "- missing-file 과 load-fail 은 둘 다 1 이지만 메시지가 갈라진다. 경로 vs 바이트.",
        "- bad-page-index 는 한컴 쪽번호를 그대로 `-p` 에 넣은 실수가 흔하다.",
        "",
        "## 픽스처",
        "",
        "각 봉투는 `fixtures/envelopes/` 에 같은 id 로 있다. 시험이 stderrContains 와 exitCode 를 고정한다.",
        "",
    ]
    for ex in EXCEPTIONS:
        lines += [
            f"### `{ex['id']}`",
            "",
            f"- kind: `{ex['kind']}`",
            f"- argv: `rhwp {' '.join(ex['argv'])}`",
            f"- exit: {ex['exitCode']} ({ex['exitClass']})",
            f"- stderr: `{ex['stderrContains']}`" if ex["stderrContains"] else "- stderr: (비거나 부수)",
            f"- 출처: {ex['source']}",
            f"- 금지: {ex['doNot']}",
            "",
        ]
    return "\n".join(lines) + "\n"


def emit_exit_codes() -> str:
    return wrap(
        """
        # 종료 코드 (#2707)

        | 코드 | 의미 | 이 스킬에서 |
        |---:|---|---|
        | 0 | 성공 | 요청한 페이지를 모두 내보냄. ir-diff **텍스트** 모드는 차이가 있어도 0 |
        | 1 | 런타임 | 파일 없음, 파싱 실패, 쓰기 실패, 틀린 비밀번호, pdf direct 부재 |
        | 2 | 사용법 | 인자 없음, 페이지 범위 초과, export-png native-skia 부재, 비밀번호 없음 |
        | 3 | IR 차이 | `ir-diff --json`, `convert --verify` |
        | 4 | 쪽수 불일치 | `convert --verify-pages` 전용 |

        ## 판정은 데이터

        exit 3 은 크래시가 아니다. `--json` 봉투의 `identical:false` / `diffCount` 를 읽는다.
        convert 는 산출물을 남긴다. 지우고 실패로 뭉개지 말 것.

        ## 기능 부재

        export-png 스텁은 2 다. 0 이면 스크립트가 PNG 가 생겼다고 믿는다.
        안내 둘째 줄: `cargo build --release --features native-skia`.

        ## 수복 줄 (#4220)

        사용법(2) 중 다음 호출이 결정론적인 부류는 stderr 마지막 줄에
        `수복: {"nextCall":...}` 이 붙을 수 있다. 런타임(1)에는 없다.
        소비자는 마지막 `수복: ` 줄 하나만 파싱한다.

        ## 페이지 완료 메시지

        "N개 … 완료" 는 **저장에 성공한 개수**다. 한 장이라도 실패하면 종료 코드는 1.
        """
    )


def emit_pitfalls() -> str:
    items = [
        ("P01", "페이지 1부터를 -p 에 그대로", "한컴 4쪽 → -p 3. -p 4 는 범위 초과(2) 또는 다음 쪽."),
        ("P02", "dump -p 와 dump-pages -p 혼동", "전자는 문단, 후자는 페이지."),
        ("P03", "자기 라운드트립 = 한컴 호환", "세 층이 다르다. 19장."),
        ("P04", "oracle/generated 순서 뒤집기", "힌트가 반대로 나온다."),
        ("P05", "export-png 부재를 성공으로", "exit 2. 재빌드."),
        ("P06", "ir-diff 텍스트 모드 차이 = 실패", "텍스트는 0, --json 만 3."),
        ("P07", "실패 JSON 을 jq", "stdout 0바이트. exit 먼저."),
        ("P08", "info 표 개수 = 실제 표", "글상자·머리말 안 표는 놓친다. export-tables."),
        ("P09", "export-markdown 으로 병합 표", "빈 칸이 생긴다. export-tables."),
        ("P10", "thumbnail 을 렌더로", "PrvImage. 화면과 다를 수 있다."),
        ("P11", "convert 출력을 .hwpx", "exit 2. export-hwpx."),
        ("P12", "extract-pages --from 에 0 기준", "그 명령만 1 기준."),
        ("P13", "profile 없이 인쇄 PDF", "legacy 는 editor_only 를 보여 줄 수 있다."),
        ("P14", "--profile + --embed-fonts", "exit 2."),
        ("P15", "overflowCellLines 무시", "셀 줄이 쪽 밖에 있다."),
        ("P16", "HU 와 px 를 1:1", "1px=75 HU, 1인치=96px=7200 HU."),
        ("P17", "없는 명령을 스킬에 추가", "새 CLI 금지."),
        ("P18", "gym 점수로 레이아웃 판정", "이 스킬은 gym 이 아니다."),
        ("P19", "암호 없음(2)과 틀림(1) 혼동", "NeedPassword vs WrongPassword."),
        ("P20", "pdf --backend direct 부재를 2로", "그 경로는 1."),
    ]
    lines = ["# 함정", "", "실측에서 에이전트가 반복하는 실수다.", ""]
    for pid, title, body in items:
        lines += [f"## {pid} — {title}", "", body, ""]
    return "\n".join(lines) + "\n"


def emit_anti() -> str:
    return wrap(
        """
        # 금지 패턴

        | id | 패턴 | 대신 |
        |---|---|---|
        | A01 | 새 서브커맨드 `rhwp layout-debug` | 기존 6단 |
        | A02 | DocumentCore 패치를 이 스킬에서 | 이슈→브랜치 |
        | A03 | gym/ 팩 실행 | 실파일 + 기존 CLI |
        | A04 | 다른 스킬 SKILL.md 수정 | 인계만 |
        | A05 | 페이지 기본값을 1로 문서화 | 0 |
        | A06 | 한컴 호환을 테스트 초록으로 단정 | 19장 |
        | A07 | oracle 없이 generated 두 번 비교 | 한컴 저장본을 받기 |
        | A08 | 실패 경로를 빈 성공 봉투로 합성 | stdout 0바이트 |
        | A09 | export-png 를 skia 없이 재시도 루프 | 재빌드 안내 |
        | A10 | ir-diff 차이를 예외 throw | 데이터 |
        | A11 | dump 전체 문서를 컨텍스트에 덤프 | -s -p 로 좁히기 |
        | A12 | 편집 fill/redact 를 이 스킬에 흡수 | 해당 스킬 |

        이 표의 id 는 픽스처 `anti_patterns.json` 과 같다.
        """
    )


def emit_journeys() -> str:
    journeys = [
        ("J01", "3쪽 겹침", "export-svg --debug-overlay -p 2", "라벨로 문단 특정"),
        ("J02", "인쇄 PDF", "export-pdf --profile print", "폰트 경로 명시"),
        ("J03", "VLM 입력", "export-png --vlm-target claude -p 0", "skia 게이트"),
        ("J04", "본문 예산", "export-text --json --max-chars 4000", "truncated 읽기"),
        ("J05", "마크다운 초안", "export-markdown -p 0", "표는 따로"),
        ("J06", "쪽 배치", "dump-pages -p 0", "vpos"),
        ("J07", "문단 속성", "dump -s 0 -p 3", "LINE_SEG"),
        ("J08", "raw 트리", "dump-records", "암호면 stdin"),
        ("J09", "번호 이상", "diag", "개요"),
        ("J10", "규모", "info --json", "pageCount"),
        ("J11", "bbox", "export-render-tree -p 0", "translate"),
        ("J12", "형식 쌍", "ir-diff a.hwpx b.hwp --json", "exit 3 데이터"),
        ("J13", "썸네일", "thumbnail --data-uri", "PrvImage"),
        ("J14", "배포용 해제", "convert in.hwp out.hwp --verify", "exit 3 자기검증"),
        ("J15", "저장 계약", "hwp5-inventory-diff oracle generated", "순서"),
        ("J16", "표 probe", "hwp5-table-probe", "out-dir"),
        ("J17", "글자 주변", "hwp5-anchor-trace --needle", "section 0"),
        ("J18", "CHAR_SHAPE", "hwp5-char-shape-audit --out", "written:"),
        ("J19", "없는 파일", "export-svg missing.hwp", "exit 1"),
        ("J20", "쪽 초과", "export-svg -p 99", "exit 2"),
        ("J21", "PNG 부재", "export-png", "exit 2 메시지"),
        ("J22", "깨진 OLE", "info truncated.hwp", "파싱 실패"),
        ("J23", "셀 소실", "export-svg --json", "overflowCellLines"),
        ("J24", "전후 SVG", "export-svg before/after", "tree diff"),
    ]
    lines = ["# 실사용 여정", "", "에이전트가 닫는 짧은 길이다. 전부 기존 CLI.", ""]
    for jid, title, first, stop in journeys:
        lines += [
            f"## {jid} — {title}",
            "",
            f"첫 명령: `{first}`",
            f"정지: {stop}",
            "gym 경로 없음. 새 플래그 없음.",
            "",
        ]
    return "\n".join(lines) + "\n"


def emit_surface() -> str:
    names = [c["id"] for c in COMMANDS] + [h[0] for h in HWP5]
    lines = [
        "# 기존 CLI 표면만",
        "",
        "이 스킬이 호출을 생성하는 이름 목록이다. 여기 없는 이름은 발명이 아니라 매뉴얼 확인 대상이다.",
        "확인 후에도 없으면 호출하지 않는다.",
        "",
        "## 핵심 명령",
        "",
    ]
    for n in names:
        lines.append(f"- `{n}`")
    lines += [
        "",
        "## 명시적으로 이 스킬의 1차 축이 아닌 것",
        "",
        "- `edit *` / `batch fill` / `inspect *` / `replay` / `audit` / `lineage`",
        "- `explore` / `digest` / `search` (트리아지 스킬)",
        "- `render-diff` (시각 회귀 스킬). 언급은 하되 여기서 고도화하지 않음",
        "- `test-*` / `gen-*` (내부 개발)",
        "",
        "새 rhwp CLI 명령을 이 PR 에서 추가하지 않는다.",
        "",
    ]
    return "\n".join(lines) + "\n"


def emit_fields() -> str:
    return wrap(
        """
        # 봉투 필드 카탈로그

        권위 스키마는 cli_commands.md 와 tests/cli_json_contract.rs 다.
        필드를 지어내지 않는다.

        ## export-svg --json

        `schemaVersion, source, format=svg, outputDir, pageCount, renderedCount, overflowCellLines, pages[{page,path,bytes,overflowCellLines}]`

        ## export-pdf --json

        `schemaVersion, source, format=pdf, backend, output, bytes, pageCount, renderedCount`

        ## export-text --json

        `schemaVersion, source, pageCount, truncated, omittedCount, pages[{page,text,truncated?,omittedCount?}]`

        ## export-markdown --json

        `schemaVersion, source, format=markdown, outputDir, pageCount, renderedCount, imageCount, pages[{page,path,bytes}]`

        ## info --json

        `schemaVersion, source, format, sizeBytes, version, sections, pageCount, paraCount, fonts`

        ## ir-diff --json

        `schemaVersion, a, b, identical, diffCount, categories`

        ## 실패

        stdout 없음. 필드 카탈로그를 적용하지 않는다.
        """
    )


def emit_traces() -> str:
    lines = ["# 재현 트레이스", "", "fixtures/traces/ 와 같은 id. argv 는 실명령.", ""]
    for i, (req, cmd, note) in enumerate(INTENTS, 1):
        lines += [
            f"## T{i:02d} — {req}",
            "",
            f"명령: `{cmd}`",
            f"메모: {note}",
            "페이지가 있으면 0 기준. 실패면 21장 봉투.",
            "",
        ]
    return "\n".join(lines) + "\n"


def emit_map() -> str:
    lines = [
        "# 요청 → 명령 매핑",
        "",
        "사용자 말을 기존 CLI 이름 하나로 접는다. 새 이름을 만들지 않는다.",
        "",
        "| 사용자 요청 | 명령 | 페이지 | 레퍼런스 |",
        "|---|---|---|---|",
    ]
    ref_for = {
        "export-svg": "02_export_svg.md",
        "export-png": "03_export_png.md",
        "export-pdf": "04_export_pdf.md",
        "export-text": "05_export_text.md",
        "export-markdown": "06_export_markdown.md",
        "dump-pages": "07_dump_pages.md",
        "dump": "08_dump.md",
        "dump-records": "09_dump_records.md",
        "diag": "10_diag.md",
        "info": "11_info.md",
        "export-render-tree": "12_export_render_tree.md",
        "ir-diff": "13_ir_diff.md",
        "thumbnail": "14_thumbnail.md",
        "convert": "15_convert.md",
        "hwp5-inventory-diff": "16_hwp5_family.md",
        "hwp5-table-probe": "16_hwp5_family.md",
        "hwp5-anchor-trace": "16_hwp5_family.md",
        "hwp5-char-shape-audit": "16_hwp5_family.md",
        "hwp5-roundtrip": "16_hwp5_family.md",
    }
    for req, cmd, note in INTENTS:
        page = "0-based -p" if cmd in {
            "export-svg", "export-png", "export-pdf", "export-text",
            "export-markdown", "dump-pages", "export-render-tree",
        } else "n/a"
        lines.append(f"| {req} | `{cmd}` | {page} | {ref_for.get(cmd, '16_hwp5_family.md')} |")
        _ = note
    lines += [
        "",
        "## 레이아웃 요청은 명령 하나가 아니다",
        "",
        "\"간격/겹침/잘림 디버깅\" 은 6단 사다리다. 첫 명령은 항상 `export-svg --debug-overlay`.",
        "",
        "## 매핑 규칙",
        "",
        "1. 요청에 쪽번호가 있으면 한컴 표기로 가정하고 1을 뺀다. 사용자가 0 기준이라고 밝히면 그대로.",
        "2. \"비교\" 가 두 파일이면 ir-diff. 한 파일+한컴 저장본이면 hwp5-inventory-diff.",
        "3. \"고쳐서 저장\" 은 이 스킬이 아니다. rhwp-safe-edit.",
        "",
    ]
    return "\n".join(lines) + "\n"


EXAMPLE_SPECS = [
    ("01_export_svg_page0", "export-svg", "첫 쪽 SVG", "rhwp export-svg 공문.hwp -p 0 -o output/poc/cli/svg"),
    ("02_export_svg_overlay", "export-svg", "겹침 overlay", "rhwp export-svg 공문.hwp --debug-overlay -p 2 -o output/poc/cli/overlay"),
    ("03_export_png_vlm", "export-png", "VLM PNG", "rhwp export-png 공문.hwp -p 0 --vlm-target claude -o output/poc/cli/png"),
    ("04_export_pdf_print", "export-pdf", "인쇄 PDF", "rhwp export-pdf 공문.hwp -o output/poc/cli/공문.pdf --profile print"),
    ("05_export_text_budget", "export-text", "예산 텍스트", "rhwp export-text 편람.hwp --json --max-chars 4000"),
    ("06_export_markdown", "export-markdown", "MD", "rhwp export-markdown 공문.hwp -p 0 -o output/poc/cli/md"),
    ("07_dump_pages", "dump-pages", "쪽 배치", "rhwp dump-pages 공문.hwp -p 2"),
    ("08_dump_para", "dump", "문단 덤프", "rhwp dump 공문.hwp -s 0 -p 14"),
    ("09_dump_records", "dump-records", "raw records", "rhwp dump-records 공문.hwp"),
    ("10_diag", "diag", "번호 진단", "rhwp diag 공문.hwp"),
    ("11_info_json", "info", "info JSON", "rhwp info 공문.hwp --json"),
    ("12_render_tree", "export-render-tree", "bbox", "rhwp export-render-tree 공문.hwp -p 2 -o output/poc/cli/tree"),
    ("13_ir_diff_json", "ir-diff", "IR 판정", "rhwp ir-diff 공문.hwpx 공문.hwp --json"),
    ("14_thumbnail", "thumbnail", "썸네일", "rhwp thumbnail 공문.hwp --data-uri"),
    ("15_convert_verify", "convert", "배포용 해제", "rhwp convert 배포.hwp 편집.hwp --verify --verify-pages"),
    ("16_hwp5_inventory_diff", "hwp5-inventory-diff", "oracle 비교", "rhwp hwp5-inventory-diff oracle.hwp generated.hwp --report hints --focus table"),
    ("17_hwp5_table_probe", "hwp5-table-probe", "표 probe", "rhwp hwp5-table-probe oracle.hwp generated.hwp --out-dir output/poc/probe"),
    ("18_hwp5_anchor", "hwp5-anchor-trace", "needle", "rhwp hwp5-anchor-trace generated.hwp --needle \"별표\" --section 0"),
    ("19_missing_file", "export-svg", "없는 파일", "rhwp export-svg 없는파일.hwp -p 0"),
    ("20_bad_page", "export-svg", "쪽 초과", "rhwp export-svg 공문.hwp -p 99"),
    ("21_native_skia", "export-png", "skia 부재", "rhwp export-png 공문.hwp"),
    ("22_load_fail", "info", "파싱 실패", "rhwp info truncated.hwp"),
    ("23_overflow_cell", "export-svg", "셀 소실", "rhwp export-svg 표문서.hwp --json"),
    ("24_debug_ladder", "export-svg", "6단 사다리", "rhwp export-svg 보고서.hwp --debug-overlay -p 2"),
]


def emit_example(stem: str, cmd: str, title: str, argv: str) -> str:
    return wrap(
        f"""
        # 예제 — {title}

        명령: `{argv}`

        이 예제는 기존 `{cmd}` 만 쓴다. 새 CLI 없음. gym 없음.

        ## 언제

        사용자가 "{title}" 에 해당하는 말을 할 때. 페이지가 있으면 0 기준인지 확인한다.

        ## 절차

        ```bash
        cargo build --release
        {argv}
        ```

        ## 읽는 것

        - exit 0 이면 산출 경로 또는 JSON 필드.
        - exit 1 이면 missing-file / load-fail / 쓰기 실패. stderr 첫 줄.
        - exit 2 이면 사용법·페이지 범위·png skia 부재.
        - exit 3 이면 ir-diff --json 또는 convert --verify. 산출물은 남아 있을 수 있다.

        ## 페이지·단위

        - 한컴 N쪽 → `-p N-1`.
        - dump 의 `-p` 는 문단. dump-pages 의 `-p` 는 페이지.
        - 1px = 75 HWPUNIT. overlay 의 y 와 dump 의 HU 를 1:1 로 두지 말 것.

        ## 자기 왕복

        이 예제가 성공해도 한컴 호환을 선언하지 않는다.

        ## 저장 계약

        oracle/generated 가 필요하면 한컴 저장본을 사용자에게 받는다. 가짜 oracle 금지.

        ## 다음

        레이아웃이면 [17_layout_debug_order.md](../references/17_layout_debug_order.md) 다음 단.
        예외면 [21_exception_envelopes.md](../references/21_exception_envelopes.md).
        """
    )


def emit_example_readme() -> str:
    lines = ["# rhwp-cli 예제", "", "실측 레시피 24개. 모두 기존 CLI.", "", "| 파일 | 명령 |", "|---|---|"]
    for stem, cmd, title, _ in EXAMPLE_SPECS:
        lines.append(f"| [{stem}.md]({stem}.md) | `{cmd}` — {title} |")
    lines.append("")
    return "\n".join(lines) + "\n"


def extra_worked_body(kind: str) -> str:
    """Longer cookbook sections so chapters stay real, not stubs."""
    samples = [
        ("samples/basic/KTX.hwp", "기본 표·본문"),
        ("samples/basic/treatise sample.hwp", "info 표 1개 vs export-tables 3개"),
        ("공문.hwp", "사용자가 준 경로. 상대 경로 함정"),
        ("편람.hwp", "대형. --max-chars 없이 export-text 금지 기본"),
        ("oracle.hwp", "한컴 저장본"),
        ("generated.hwp", "rhwp 저장본"),
        ("source.hwpx", "HWPX 원본"),
    ]
    lines = ["## 표본 경로", ""]
    for p, why in samples:
        lines.append(f"- `{p}` — {why}")
    lines += ["", "## 대화 예", ""]
    dialogues = {
        "export": [
            ("3쪽 SVG 로 빼줘", "export-svg -p 2", "한컴 3 = 인덱스 2"),
            ("인쇄 PDF", "export-pdf --profile print", "legacy 기본 금지"),
            ("텍스트만 조금", "export-text --json --max-chars 2000", "truncated"),
        ],
        "dump": [
            ("이 쪽 뭐가 있나", "dump-pages -p N", "페이지"),
            ("그 문단 속성", "dump -s 0 -p M", "문단"),
            ("바이너리 레코드", "dump-records", "HWP5"),
        ],
        "compare": [
            ("두 파일 같아?", "ir-diff --json", "exit 3 = 데이터"),
            ("한컴이 안 연다", "hwp5-inventory-diff", "oracle 먼저"),
        ],
        "except": [
            ("파일 없는데?", "그대로 실행", "exit 1 메시지"),
            ("99쪽", "-p 99", "exit 2 범위"),
        ],
    }
    for user, cmd, note in dialogues.get(kind, dialogues["export"]):
        lines += [f"- 사용자: {user}", f"  - 명령: `{cmd}`", f"  - 메모: {note}"]
    lines += ["", "## 재시도", "", "같은 실패 봉투가 나오면 플래그를 발명하지 말고 입력을 고친다.", ""]
    return "\n".join(lines)


def main() -> None:
    for d in (REFS, EXAMPLES, FIXTURES, ENVELOPES, TRANSCRIPTS, TRACES):
        d.mkdir(parents=True, exist_ok=True)

    refs = {
        "00_tree.md": emit_tree(),
        "01_request_command_map.md": emit_map(),
        "02_export_svg.md": emit_command_ref(COMMANDS[0], 2) + extra_worked_body("export"),
        "03_export_png.md": emit_command_ref(COMMANDS[1], 3) + extra_worked_body("export"),
        "04_export_pdf.md": emit_command_ref(COMMANDS[2], 4) + extra_worked_body("export"),
        "05_export_text.md": emit_command_ref(COMMANDS[3], 5) + extra_worked_body("export"),
        "06_export_markdown.md": emit_command_ref(COMMANDS[4], 6) + extra_worked_body("export"),
        "07_dump_pages.md": emit_command_ref(COMMANDS[5], 7) + extra_worked_body("dump"),
        "08_dump.md": emit_command_ref(COMMANDS[6], 8) + extra_worked_body("dump"),
        "09_dump_records.md": emit_command_ref(COMMANDS[7], 9) + extra_worked_body("dump"),
        "10_diag.md": emit_command_ref(COMMANDS[8], 10) + extra_worked_body("dump"),
        "11_info.md": emit_command_ref(COMMANDS[9], 11) + extra_worked_body("export"),
        "12_export_render_tree.md": emit_command_ref(COMMANDS[10], 12) + extra_worked_body("export"),
        "13_ir_diff.md": emit_command_ref(COMMANDS[11], 13) + extra_worked_body("compare"),
        "14_thumbnail.md": emit_command_ref(COMMANDS[12], 14) + extra_worked_body("export"),
        "15_convert.md": emit_command_ref(COMMANDS[13], 15) + extra_worked_body("compare"),
        "16_hwp5_family.md": emit_hwp5_ref() + extra_worked_body("compare"),
        "17_layout_debug_order.md": emit_debug_order(),
        "18_page_units.md": emit_page_units(),
        "19_roundtrip_vs_hangul.md": emit_roundtrip(),
        "20_hwpx_hwp_save_contract.md": emit_save_contract(),
        "21_exception_envelopes.md": emit_exceptions() + extra_worked_body("except"),
        "22_exit_codes.md": emit_exit_codes(),
        "23_pitfalls.md": emit_pitfalls(),
        "24_anti_patterns.md": emit_anti(),
        "25_journeys.md": emit_journeys(),
        "26_cli_surface.md": emit_surface(),
        "27_field_catalog.md": emit_fields(),
        "28_worked_traces.md": emit_traces(),
    }
    for name, body in refs.items():
        write_text(REFS / name, body)

    write_text(EXAMPLES / "README.md", emit_example_readme())
    for stem, cmd, title, argv in EXAMPLE_SPECS:
        write_text(EXAMPLES / f"{stem}.md", emit_example(stem, cmd, title, argv))

    # --- fixtures ---
    write_json(
        FIXTURES / "skill_index.json",
        {
            "skill": SKILL_NAME,
            "issue": ISSUE,
            "gym": False,
            "newCli": False,
            "pageZeroBased": True,
            "selfRoundTripIsNotHangul": True,
            "debugOrder": [s["command"] for s in DEBUG_ORDER],
            "oracleName": "oracle",
            "generatedName": "generated",
            "references": list(refs.keys()),
            "examples": [f"{s[0]}.md" for s in EXAMPLE_SPECS],
            "commands": [c["id"] for c in COMMANDS],
            "hwp5": [h[0] for h in HWP5],
            "exceptionKinds": [
                "missing-file",
                "bad-page-index",
                "native-skia-missing",
                "load-fail",
            ],
            "forbiddenSkillsTouch": [
                "rhwp-onboarding",
                "rhwp-mcp-session",
                "rhwp-provenance",
                "rhwp-safe-edit",
                "rhwp-doc-triage",
                "rhwp-security-sweep",
                "rhwp-work-receipt",
                "rhwp-form-fill",
                "rhwp-table-exchange",
                "rhwp-visual-regression",
            ],
            "workingDoc": "mydocs/working/archives/agent_cli.md",
            "capabilityNote": "CAP-5316 expands existing rhwp-cli (LEGACY-d86c935bc), no new capability id",
        },
    )
    write_json(
        FIXTURES / "command_map.json",
        {
            "skill": SKILL_NAME,
            "commands": [
                {
                    "id": c["id"],
                    "family": c["family"],
                    "request": c["request"],
                    "argv": c["argv"],
                    "pageZero": c["pageZero"],
                    "json": c["json"],
                    "requiresFeature": c.get("requiresFeature"),
                    "notes": c["notes"],
                }
                for c in COMMANDS
            ],
        },
    )
    write_json(
        FIXTURES / "debug_order.json",
        {
            "skill": SKILL_NAME,
            "order": DEBUG_ORDER,
            "note": "export-svg --debug-overlay → dump-pages → dump → ir-diff → export-render-tree → hwp5-inventory-diff",
            "pageZeroBased": True,
        },
    )
    write_json(FIXTURES / "page_units.json", {"skill": SKILL_NAME, **UNITS, "pageZeroBased": True})
    write_json(
        FIXTURES / "hwp5_family.json",
        {
            "skill": SKILL_NAME,
            "oracle": "한컴 저장본",
            "generated": "rhwp 저장본",
            "argumentOrder": ["oracle", "generated"],
            "commands": [{"id": n, "why": w} for n, w in HWP5],
        },
    )
    write_json(
        FIXTURES / "exit_codes.json",
        {
            "0": "success",
            "1": "runtime",
            "2": "usage",
            "3": "ir-diff-or-verify",
            "4": "verify-pages",
            "pngMissingFeature": 2,
            "pdfDirectMissingFeature": 1,
            "irDiffJsonMismatch": 3,
            "irDiffTextMismatch": 0,
        },
    )
    write_json(
        FIXTURES / "anti_patterns.json",
        {
            "patterns": [
                {"id": f"A{i:02d}", "forbidden": True}
                for i in range(1, 13)
            ]
        },
    )
    write_json(
        FIXTURES / "intents.json",
        {
            "intents": [
                {"utterance": u, "command": c, "note": n} for u, c, n in INTENTS
            ]
        },
    )
    write_json(
        FIXTURES / "journeys.json",
        {
            "journeys": [
                {"id": f"J{i:02d}", "example": EXAMPLE_SPECS[i - 1][0], "command": EXAMPLE_SPECS[i - 1][1]}
                for i in range(1, 25)
            ]
        },
    )
    write_json(
        FIXTURES / "envelope_keys.json",
        {
            "export-svg": ["schemaVersion", "source", "format", "pageCount", "overflowCellLines", "pages"],
            "export-pdf": ["schemaVersion", "source", "format", "backend", "output", "bytes", "pageCount"],
            "export-text": ["schemaVersion", "source", "pageCount", "truncated", "omittedCount", "pages"],
            "export-markdown": ["schemaVersion", "source", "format", "outputDir", "pageCount", "pages"],
            "info": ["schemaVersion", "source", "format", "sizeBytes", "version", "sections", "pageCount"],
            "ir-diff": ["schemaVersion", "a", "b", "identical", "diffCount", "categories"],
        },
    )

    for ex in EXCEPTIONS:
        write_json(
            ENVELOPES / f"{ex['id']}.json",
            {
                "id": ex["id"],
                "kind": ex["kind"],
                "command": ex["command"],
                "argv": ex["argv"],
                "exitCode": ex["exitCode"],
                "exitClass": ex["exitClass"],
                "stderrContains": ex["stderrContains"],
                "stdoutEmpty": ex["stdoutEmpty"],
                "source": ex["source"],
                "doNot": ex["doNot"],
                "repair": ex.get("repair"),
                "stdoutKeys": ex.get("stdoutKeys"),
                "_skillMeta": {"exit": ex["exitCode"], "issue": ISSUE},
            },
        )

    success_envs = [
        {
            "id": "export_svg_ok",
            "command": "export-svg",
            "exitCode": 0,
            "schemaVersion": "1.0",
            "format": "svg",
            "pageCount": 3,
            "renderedCount": 1,
            "overflowCellLines": 0,
            "pages": [{"page": 0, "path": "output/poc/cli/svg/doc_001.svg", "bytes": 12000, "overflowCellLines": 0}],
        },
        {
            "id": "export_svg_overflow",
            "command": "export-svg",
            "exitCode": 0,
            "schemaVersion": "1.0",
            "format": "svg",
            "pageCount": 1,
            "renderedCount": 1,
            "overflowCellLines": 4,
            "pages": [{"page": 0, "path": "output/poc/cli/svg/table_001.svg", "bytes": 8000, "overflowCellLines": 4}],
            "note": "overflowCellLines>0 은 성공(0)이지만 셀 소실 신호",
        },
        {
            "id": "export_text_truncated",
            "command": "export-text",
            "exitCode": 0,
            "schemaVersion": "1.0",
            "pageCount": 12,
            "truncated": True,
            "omittedCount": 8800,
            "pages": [
                {"page": 0, "text": "제1장", "truncated": False, "omittedCount": 0},
                {"page": 1, "text": "…", "truncated": True, "omittedCount": 8800},
            ],
        },
        {
            "id": "info_ok",
            "command": "info",
            "exitCode": 0,
            "schemaVersion": "1.0",
            "format": "hwp5",
            "sizeBytes": 1423360,
            "version": "5.1.1.0",
            "sections": 1,
            "pageCount": 8,
            "paraCount": 120,
        },
        {
            "id": "ir_diff_identical",
            "command": "ir-diff",
            "exitCode": 0,
            "schemaVersion": "1.0",
            "a": "a.hwpx",
            "b": "b.hwp",
            "identical": True,
            "diffCount": 0,
            "categories": {},
        },
        {
            "id": "ir_diff_mismatch",
            "command": "ir-diff",
            "exitCode": 3,
            "schemaVersion": "1.0",
            "a": "a.hwpx",
            "b": "b.hwp",
            "identical": False,
            "diffCount": 6,
            "categories": {"text": 2, "line_segs": 3, "controls": 1},
            "note": "판정 데이터. 크래시 아님",
        },
    ]
    for env in success_envs:
        env["_skillMeta"] = {"exit": env["exitCode"], "issue": ISSUE}
        write_json(ENVELOPES / f"{env['id']}.json", env)

    for i, (req, cmd, note) in enumerate(INTENTS, 1):
        write_json(
            TRANSCRIPTS / f"T{i:02d}.json",
            {
                "id": f"T{i:02d}",
                "utterance": req,
                "command": cmd,
                "note": note,
                "pageZeroBased": True,
                "newCli": False,
                "gym": False,
            },
        )
        write_json(
            TRACES / f"T{i:02d}.json",
            {
                "id": f"T{i:02d}",
                "utterance": req,
                "steps": [{"command": cmd, "pageZeroBased": True}],
                "stop": note,
            },
        )

    scenarios = []
    docs = [
        "공문.hwp",
        "편람.hwp",
        "보고서.hwpx",
        "서식.hwp",
        "표문서.hwp",
        "oracle.hwp",
        "generated.hwp",
        "source.hwpx",
        "배포.hwp",
        "KTX.hwp",
    ]
    for i in range(1, 101):
        cmd = COMMANDS[i % len(COMMANDS)]
        doc = docs[i % len(docs)]
        page = i % 5
        hangul_page = page + 1
        scenarios.append(
            {
                "id": f"S{i:03d}",
                "utterance": f"{doc} 한컴 {hangul_page}쪽 {cmd['request'][0]}",
                "command": cmd["id"],
                "file": doc,
                "hangulPage": hangul_page,
                "cliPage": page if cmd["pageZero"] else None,
                "argvHint": cmd["argv"],
                "debugStep": next((s["step"] for s in DEBUG_ORDER if s["command"] == cmd["id"]), None),
                "family": cmd["family"],
                "selfRoundTripIsHangul": False,
                "newCli": False,
                "gym": False,
            }
        )
    write_json(
        FIXTURES / "scenario_catalog.json",
        {
            "skill": SKILL_NAME,
            "issue": ISSUE,
            "count": len(scenarios),
            "scenarios": scenarios,
        },
    )

    write_text(
        REFS / "README.md",
        wrap(
            """
            # rhwp-cli references

            SKILL.md 의 자식 장. 생성기: `_gen_pack.py`.
            권위는 생성기가 아니라 `cli_commands.md` 와 `src/main.rs`.
            """
        ),
    )
    print(f"wrote {len(refs)} refs, {len(EXAMPLE_SPECS)} examples, fixtures ok")


if __name__ == "__main__":
    main()
