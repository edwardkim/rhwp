#!/usr/bin/env python3
"""[#5342] rhwp-knowledge-map 레퍼런스·픽스처·예제 생성기.

문서 진입점 라우터다. 새 CLI / 편집 로직 / gym 과제를 발명하지 않는다.
정본은 llms.txt 와 mydocs/manual/agent_knowledge_map.md 이다.
지도 행을 다시 쓰지 않는다. 필드 이름은 지도 §2 에서만 추출한다.
지도와 canonical 이 다르면 canonical 을 따른다.
"""

from __future__ import annotations

import json
import re
from datetime import date, datetime
from pathlib import Path

SKILL = Path(__file__).resolve().parents[1]
REF = SKILL / "references"
FIXT = SKILL / "fixtures"
EXAMPLES = SKILL / "examples"
CARDS_DIR = FIXT / "cards"
TRANS = FIXT / "transcripts"
TRACES = FIXT / "traces"
REPO = Path(__file__).resolve().parents[4]
MAP_PATH = REPO / "mydocs" / "manual" / "agent_knowledge_map.md"
LLMS_PATH = REPO / "llms.txt"

ISSUE = 5342
SCHEMA = "1.0"
STALE_DAYS = 30
AS_OF = date(2026, 8, 18)
TODAY = "2026-08-18"
SKILL_NAME = "rhwp-knowledge-map"
CANONICAL_MAP = "mydocs/manual/agent_knowledge_map.md"
CANONICAL_LLMS = "llms.txt"

# 이 스킬은 문서 진입점이다. 대전 항해·3층 계약 스킬을 여기서 다시 쓰지 않는다.
FORBIDDEN_REWRITE = [
    "rhwp-codex",
    "rhwp-agent-surface",
]

# devel 에 이미 있는 이웃. 존재 확인은 하되 본문을 고치지 않는다.
PEER_SKILLS_ON_DEVEL = [
    "rhwp-cli",
    "rhwp-codex",
    "rhwp-contributor",
    "rhwp-doc-triage",
    "rhwp-exam-ingest",
    "rhwp-form-fill",
    "rhwp-mcp-session",
    "rhwp-onboarding",
    "rhwp-provenance",
    "rhwp-safe-edit",
    "rhwp-security-sweep",
    "rhwp-table-exchange",
    "rhwp-visual-regression",
    "rhwp-work-receipt",
    "rhwp-bulk-pipeline",
]

INVENTED_COMMANDS = [
    "rhwp knowledge-map",
    "rhwp knowledge_map",
    "rhwp map",
    "rhwp docs-index",
    "rhwp agent-map",
    "rhwp field-dict",
    "rhwp remap",
    "rhwp lookup-field",
    "rhwp open-map",
    "rhwp first-read",
]

REMEASURE_COMMANDS = [
    {
        "id": "RM01",
        "argv": ["rhwp", "capabilities"],
        "why": "명령·플래그·recordFields·종료 코드 자기서술",
        "source": "mydocs/manual/agent_knowledge_map.md#0-이-지도의-실측-기준",
    },
    {
        "id": "RM02",
        "argv": ["rhwp", "capabilities", "--mcp"],
        "why": "MCP 무상태 도구 선언",
        "source": "mydocs/manual/agent_knowledge_map.md#0-이-지도의-실측-기준",
    },
    {
        "id": "RM03",
        "argv": ["rhwp", "mcp-serve"],
        "method": "tools/list",
        "why": "세션 포함 실제 도구 목록. capabilities --mcp 에 없는 세션 도구가 여기 있다",
        "source": "mydocs/manual/agent_knowledge_map.md#0-이-지도의-실측-기준",
    },
]

# 요청 → 지도 절 → 정본 하나 → (필요하면) 스킬. 지도 행을 복제하지 않는다.
REQUEST_ROWS = [
    ("이 저장소 문서는 어디서 시작하나", "S00", "llms.txt", None, "R01"),
    ("지식 지도가 뭐야", "S00", CANONICAL_MAP, None, "R01"),
    ("llms.txt 다음에 뭘 읽나", "S00", CANONICAL_MAP, None, "R01"),
    ("이 바이너리가 뭘 할 수 있나", "S00", CANONICAL_MAP, None, "R02"),
    ("capabilities 다시 찍어", "S00", CANONICAL_MAP, None, "R02"),
    ("tools/list 로 재측정", "S06", "mydocs/manual/mcp_integration_guide.md", "rhwp-mcp-session", "R08"),
    ("info 로 쪽수 보고 싶어", "S11A", "mydocs/manual/cli_commands.md", "rhwp-doc-triage", "R08"),
    ("문서 규모부터", "S11A", "mydocs/manual/cli_commands.md", "rhwp-doc-triage", "R08"),
    ("digest 한 줄 요약", "S11A", "mydocs/tech/tiny_model_macro_tools.md", "rhwp-doc-triage", "R08"),
    ("본문 전체 뽑기", "S11B", "mydocs/manual/cli_commands.md", "rhwp-cli", "R08"),
    ("표를 CSV 로", "S11B", "mydocs/manual/cli_commands.md", "rhwp-table-exchange", "R08"),
    ("search 로 쪽 주소", "S11B", "mydocs/manual/cli_commands.md", "rhwp-doc-triage", "R08"),
    ("누름틀 이름 목록", "S11B", "mydocs/manual/form_filling_guide.md", "rhwp-form-fill", "R08"),
    ("쪽을 PNG 로", "S11C", "mydocs/manual/export_png_command.md", "rhwp-cli", "R08"),
    ("제출용 PDF", "S11C", "mydocs/manual/cli_commands.md", "rhwp-cli", "R08"),
    ("서식 채워", "S11D", "mydocs/manual/form_filling_guide.md", "rhwp-form-fill", "R08"),
    ("표 칸 기록", "S11D", "mydocs/manual/form_filling_guide.md", "rhwp-table-exchange", "R08"),
    ("CSV 로 표 덮어쓰기", "S11D", "mydocs/manual/cli_commands.md", "rhwp-table-exchange", "R08"),
    ("문구 일괄 치환", "S11D", "mydocs/manual/form_filling_guide.md", "rhwp-safe-edit", "R08"),
    ("개인정보 마스킹", "S11D", "mydocs/tech/agent_security/consumer_guide.md", "rhwp-security-sweep", "R08"),
    ("여러 편집을 원자로", "S11D", "mydocs/manual/cli_commands.md", "rhwp-safe-edit", "R08"),
    ("HWP 를 HWPX 로", "S11E", "mydocs/manual/cli_commands.md", "rhwp-cli", "R08"),
    ("은닉 텍스트", "S11F", "mydocs/tech/agent_security/hidden_content.md", "rhwp-security-sweep", "R08"),
    ("프롬프트 주입 신호", "S11F", "mydocs/tech/agent_security/indirect_prompt_injection.md", "rhwp-security-sweep", "R08"),
    ("유니코드 기만", "S11F", "mydocs/tech/agent_security/unicode_deception.md", "rhwp-security-sweep", "R08"),
    ("ir-diff 로 비교", "S11G", "mydocs/manual/ir_diff_command.md", "rhwp-visual-regression", "R08"),
    ("render-diff 레이아웃", "S11G", "mydocs/manual/cli_commands.md", "rhwp-visual-regression", "R08"),
    ("폴더 일괄", "S11H", "mydocs/manual/cli_json_pipeline_guide.md", "rhwp-bulk-pipeline", "R08"),
    ("세션으로 재파싱 피하기", "S11I", "mydocs/manual/mcp_integration_guide.md", "rhwp-mcp-session", "R08"),
    ("UTF-8 깨짐", "S12", "mydocs/manual/agent_troubleshooting_guide.md", None, "R01"),
    ("exit 2 사용법", "S12", "mydocs/manual/agent_troubleshooting_guide.md", None, "R01"),
    ("verify 가 exit 3", "S12", "mydocs/manual/agent_troubleshooting_guide.md", None, "R01"),
    ("표면을 더하고 싶다", "S13", "mydocs/manual/agent_surface_playbook.md", "rhwp-agent-surface", "R09"),
    ("레시피 처음부터 끝까지", "S131", "mydocs/manual/recipes/01_fill_form_and_submit.md", "rhwp-form-fill", "R08"),
    ("프로필로 도구 좁히기", "S15", "mydocs/manual/mcp_integration_guide.md", "rhwp-mcp-session", "R08"),
    ("이 필드 이름이 뭐 뜻이야", "S22", CANONICAL_MAP, None, "R03"),
    ("untrustedContent 가 뭐야", "S21", "mydocs/tech/envelope_provenance.md", "rhwp-provenance", "R08"),
    ("changedPages 기준", "S22", CANONICAL_MAP, "rhwp-safe-edit", "R03"),
    ("페이지는 0부터인가", "S31", CANONICAL_MAP, None, "R01"),
    ("extract-pages 쪽 기준", "S33", "mydocs/manual/cli_commands.md", None, "R01"),
    ("isError 만 보면 되나", "S40", "mydocs/manual/mcp_integration_guide.md", "rhwp-mcp-session", "R08"),
    ("명령 전수 목록", "S50", CANONICAL_MAP, "rhwp-codex", "R09"),
    ("MCP 도구 이름", "S61", "mydocs/manual/mcp_integration_guide.md", "rhwp-mcp-session", "R08"),
    ("어떤 샘플을 쓰나", "S71", CANONICAL_MAP, None, "R01"),
    ("어느 계약 테스트가 잡나", "S80", CANONICAL_MAP, None, "R01"),
    ("대전 장 항해", "S50", "mydocs/manual/agent_codex/00_서문.md", "rhwp-codex", "R09"),
    ("3층 계약 규칙", "S13", "mydocs/manual/agent_surface_playbook.md", "rhwp-agent-surface", "R09"),
    ("작업 영수증", "S22", "mydocs/manual/cli_commands.md", "rhwp-work-receipt", "R08"),
    ("온보딩 닥터", "S00", "mydocs/manual/agent_onboarding.md", "rhwp-onboarding", "R08"),
    ("기여 절차", "S90", "CONTRIBUTING.md", "rhwp-contributor", "R08"),
    ("시험지 ingest", "S11E", "mydocs/manual/cli_commands.md", "rhwp-exam-ingest", "R08"),
    ("last_verified 낡은 지도", "S00", CANONICAL_MAP, None, "R04"),
    ("바이너리 버전이 지도와 다름", "S00", CANONICAL_MAP, None, "R05"),
    ("지도 숫자와 CLI 매뉴얼이 다름", "S90", "mydocs/manual/cli_commands.md", None, "R06"),
    ("스키마Version 철자로 필드 만들어", "S22", CANONICAL_MAP, None, "R07"),
    ("gym 벤치에서 지도 재현", "S00", CANONICAL_MAP, None, "R12"),
    ("지식지도 하위명령 추가", "S13", "mydocs/manual/agent_surface_playbook.md", None, "R11"),
]

# 발화 행렬. command 칸은 기존 표면이거나 "없음/재측정/정본" 이다.
INTENTS = [
    ("I001", "지식 지도부터 읽어", "없음 — 첫 문서는 llms.txt", "00_first_read.md", "R01"),
    ("I002", "llms.txt 다음 문서", "없음 — agent_knowledge_map.md", "00_first_read.md", "R01"),
    ("I003", "어디에 무엇이 있는지 표", "없음 — 지도 §1-1", "19_three_questions.md", "R01"),
    ("I004", "이 필드 이름 뜻이 뭐야", "없음 — 지도 §2 조회, 발명 금지", "05_envelope_dict.md", "R03"),
    ("I005", "schemaVersion 이 뭐야", "없음 — 지도 §2-1", "18_field_lookup.md", "R03"),
    ("I006", "untrustedFields 경로", "rhwp export-provenance-map --json", "05_envelope_dict.md", "R08"),
    ("I007", "capabilities 다시 찍자", "rhwp capabilities", "01_remeasure.md", "R02"),
    ("I008", "MCP 도구 선언 재측정", "rhwp capabilities --mcp", "01_remeasure.md", "R02"),
    ("I009", "세션 포함 tools/list", "rhwp mcp-serve", "22_mcp_remeasure.md", "R02"),
    ("I010", "지도 last_verified 오래됨", "없음 — 날짜를 보여 주고 중단", "10_stale_last_verified.md", "R04"),
    ("I011", "바이너리가 v0.8.4 인데 지도는 v0.8.3", "rhwp capabilities", "11_version_mismatch.md", "R05"),
    ("I012", "지도와 cli_commands 가 숫자가 다름", "없음 — cli_commands.md 를 따른다", "12_map_vs_canonical.md", "R06"),
    ("I013", "recordField 이름을 내가 지어", "없음 — §2 에 없는 이름은 쓰지 않는다", "05_envelope_dict.md", "R07"),
    ("I014", "서식 채워줘", "rhwp fields <file> --json", "08_jump_to_skill.md", "R08"),
    ("I015", "표를 CSV", "rhwp export-tables <file> --json", "08_jump_to_skill.md", "R08"),
    ("I016", "배포 전 마스킹", "rhwp edit redact <file> --dry-run", "08_jump_to_skill.md", "R08"),
    ("I017", "폴더 일괄 추출", "rhwp batch info --json", "08_jump_to_skill.md", "R08"),
    ("I018", "레이아웃 숫자 비교", "rhwp render-diff <file> --json", "08_jump_to_skill.md", "R08"),
    ("I019", "MCP 호스트에 붙여", "rhwp mcp-serve", "08_jump_to_skill.md", "R08"),
    ("I020", "대전 교본 장 순서", "없음 — rhwp-codex 로 인계", "04_boundary.md", "R09"),
    ("I021", "3층 계약으로 도구 추가", "없음 — rhwp-agent-surface 로 인계", "04_boundary.md", "R09"),
    ("I022", "지식지도 명령 만들어줘", "없음 — 새 CLI 금지", "13_stop_conditions.md", "R11"),
    ("I023", "gym pack 으로 지도 검증", "없음 — gym 금지", "13_stop_conditions.md", "R12"),
    ("I024", "지도를 처음부터 끝까지 읽어", "없음 — 필요한 절만", "00_first_read.md", "R10"),
    ("I025", "exit 2 가 났어", "없음 — 실패 사전 앵커", "19_three_questions.md", "R01"),
    ("I026", "identical false 는 오류인가", "없음 — 지도 §4, 데이터다", "19_three_questions.md", "R01"),
    ("I027", "페이지 번호 0부터?", "없음 — 지도 §3-1, extract-pages 만 1", "19_three_questions.md", "R01"),
    ("I028", "어떤 샘플로 fields 시험", "없음 — 지도 §7-1 표본 표", "20_samples_index.md", "R01"),
    ("I029", "provenance 계약 테스트", "없음 — 지도 §8-1", "21_contract_tests_index.md", "R01"),
    ("I030", "온보딩 첫 5분", "rhwp capabilities", "08_jump_to_skill.md", "R08"),
    ("I031", "작업 영수증 발급", "rhwp replay --json", "08_jump_to_skill.md", "R08"),
    ("I032", "안전 편집 dry-run", "rhwp run <plan> --dry-run --json", "08_jump_to_skill.md", "R08"),
    ("I033", "출처 표지 읽기", "rhwp export-provenance-map --json", "08_jump_to_skill.md", "R08"),
    ("I034", "문서 트리아지", "rhwp info <file> --json", "08_jump_to_skill.md", "R08"),
    ("I035", "기여하려면", "없음 — rhwp-contributor", "08_jump_to_skill.md", "R08"),
    ("I036", "시험지 PDF 를 HWPX 로", "rhwp build-from-ingest --json", "08_jump_to_skill.md", "R08"),
    ("I037", "메일머지 N행", "rhwp batch fill --json", "08_jump_to_skill.md", "R08"),
    ("I038", "은닉 텍스트 스윕", "rhwp inspect hidden-text --json", "08_jump_to_skill.md", "R08"),
    ("I039", "세션 hwp_open", "rhwp mcp-serve", "22_mcp_remeasure.md", "R08"),
    ("I040", "프로필 행정서식", "rhwp capabilities --mcp --profile 행정서식", "19_three_questions.md", "R08"),
    ("I041", "스키마 코드 생성", "rhwp export-capabilities-schema --json", "19_three_questions.md", "R01"),
    ("I042", "IR 스키마", "rhwp export-ir-schema --json", "19_three_questions.md", "R01"),
    ("I043", "filledCount 성공인데 빈칸", "없음 — 실패 사전 편집 오독", "19_three_questions.md", "R01"),
    ("I044", "한글 파일명 깨짐", "없음 — 실패 사전 입력·인코딩", "19_three_questions.md", "R01"),
    ("I045", "export-png 없다", "rhwp capabilities", "01_remeasure.md", "R02"),
    ("I046", "changedPages 로 쪽만 렌더", "없음 — 지도 §1-1 (다) 앵커", "07_section_index.md", "R01"),
    ("I047", "hwp_doc_save 만이 기록?", "없음 — 지도 §1-1 (자)", "07_section_index.md", "R01"),
    ("I048", "배치 convert 는 MCP 있나", "없음 — 지도 §1-1 (아) CLI 전용", "07_section_index.md", "R01"),
    ("I049", "null 과 모르겠다 구별", "없음 — 지도 §2-4", "18_field_lookup.md", "R03"),
    ("I050", "recordFields 가 전부인가", "없음 — 지도 §2-5", "05_envelope_dict.md", "R03"),
    ("I051", "didYouMean 힌트", "없음 — 지도 §4", "19_three_questions.md", "R01"),
    ("I052", "명령 가족 query", "없음 — 지도 §5, 상세는 대전", "04_boundary.md", "R09"),
    ("I053", "세션 도구 몇 개", "rhwp mcp-serve", "22_mcp_remeasure.md", "R02"),
    ("I054", "samples 는 음성 코퍼스?", "없음 — 지도 §7-2", "20_samples_index.md", "R01"),
    ("I055", "redact 가 잡는 표본", "없음 — 지도 §7-3", "20_samples_index.md", "R01"),
    ("I056", "cli_json_contract 가 고정하는 것", "없음 — 지도 §8-1", "21_contract_tests_index.md", "R01"),
    ("I057", "보안 축 계약 테스트", "없음 — 지도 §8-6", "21_contract_tests_index.md", "R01"),
    ("I058", "문서 축 권위표", "없음 — 지도 §9", "06_canonicals.md", "R01"),
    ("I059", "지도 행을 더 자세히 풀어써", "없음 — 재서술 금지, canonical 로", "12_map_vs_canonical.md", "R13"),
    ("I060", "지도 숫자 손으로 고치자", "rhwp capabilities", "01_remeasure.md", "R14"),
    ("I061", "첫 문서로 ROADMAP 을 읽자", "없음 — llms.txt 가 지도를 가리킨다", "00_first_read.md", "R01"),
    ("I062", "필드 사전을 내가 암기한 이름으로", "없음 — §2 조회", "05_envelope_dict.md", "R07"),
    ("I063", "replacedCount 0 은 실패?", "없음 — 지도 §4 데이터", "18_field_lookup.md", "R03"),
    ("I064", "notFound 는 isError 인가", "없음 — 지도 §4 데이터", "18_field_lookup.md", "R03"),
    ("I065", "overflow 필드", "없음 — 지도 §2 조회", "18_field_lookup.md", "R03"),
    ("I066", "ambiguous 필드", "없음 — 지도 §2 조회", "18_field_lookup.md", "R03"),
    ("I067", "matchCount 필드", "없음 — 지도 §2 조회", "18_field_lookup.md", "R03"),
    ("I068", "findingCount 필드", "없음 — 지도 §2 조회", "18_field_lookup.md", "R03"),
    ("I069", "hiddenCharCount 필드", "없음 — 지도 §2 조회", "18_field_lookup.md", "R03"),
    ("I070", "verify.identical 필드", "없음 — 지도 §2 조회", "18_field_lookup.md", "R03"),
    ("I071", "docId 필드", "없음 — 지도 §2 조회", "18_field_lookup.md", "R03"),
    ("I072", "closed 필드", "없음 — 지도 §2 조회", "18_field_lookup.md", "R03"),
    ("I073", "profile.recipe 필드", "없음 — 지도 §1-5 앵커", "19_three_questions.md", "R01"),
    ("I074", "missingAxes 필드", "없음 — 지도 §0 export-agent-manifest", "01_remeasure.md", "R01"),
    ("I075", "에이전트 매니페스트", "rhwp export-agent-manifest --json", "01_remeasure.md", "R01"),
    ("I076", "capabilities --search 검색", "rhwp capabilities --search <낱말>", "01_remeasure.md", "R02"),
    ("I077", "지도와 대전이 필드 정의가 다름", "없음 — 지도 §2 가 필드 사전", "04_boundary.md", "R06"),
    ("I078", "표면 플레이북 수용 기준", "없음 — agent_surface_playbook.md", "04_boundary.md", "R09"),
    ("I079", "레시피 07 을 지도에 추가", "없음 — 지도는 행만, 07 발명은 레시피 스킬", "12_map_vs_canonical.md", "R13"),
    ("I080", "실패 증상 검색", "없음 — agent_troubleshooting_guide.md", "06_canonicals.md", "R01"),
    ("I081", "선검사 스크립트", "없음 — agent_preflight_guide.md", "06_canonicals.md", "R01"),
    ("I082", "JSON 파이프라인 배치", "없음 — cli_json_pipeline_guide.md", "06_canonicals.md", "R08"),
    ("I083", "서식 함정 심화", "없음 — form_filling_guide.md", "06_canonicals.md", "R08"),
    ("I084", "경계 계약", "없음 — tech/agent_boundary_contract.md", "06_canonicals.md", "R01"),
    ("I085", "위협 모델", "없음 — tech/agent_security/threat_model.md", "06_canonicals.md", "R08"),
    ("I086", "지도 유지 규약", "없음 — 지도 말미 유지 규약", "15_pitfalls.md", "R14"),
    ("I087", "링크 검사 어떻게", "없음 — check_markdown_links.py 앵커", "15_pitfalls.md", "R01"),
    ("I088", "HWPUNIT 좌표", "없음 — 지도 §3-1", "19_three_questions.md", "R01"),
    ("I089", "이름[N] 반복 필드", "없음 — 지도 §3-1", "08_jump_to_skill.md", "R08"),
    ("I090", "병합 칸 앵커", "없음 — 지도 §3-3", "08_jump_to_skill.md", "R08"),
    ("I091", "pagesAfter 가 범위와 다름", "없음 — 지도 §3-3", "19_three_questions.md", "R01"),
    ("I092", "structuredContent 없는 도구", "없음 — 지도 §6-3", "22_mcp_remeasure.md", "R01"),
    ("I093", "hwp_batch 는 NDJSON", "없음 — 지도 §6-3", "22_mcp_remeasure.md", "R01"),
    ("I094", "닫힌 핸들 재사용", "없음 — 지도 §6-2 nextCall hwp_open", "22_mcp_remeasure.md", "R08"),
    ("I095", "387쪽 세션 이득", "없음 — 지도 §6-2 실측 앵커", "22_mcp_remeasure.md", "R01"),
    ("I096", "비밀번호 stdin", "없음 — 지도 §6-1 writeOnly", "22_mcp_remeasure.md", "R01"),
    ("I097", "개발통합 프로필", "rhwp capabilities --mcp --profile 개발통합", "19_three_questions.md", "R08"),
    ("I098", "없는 프로필 이름", "없음 — 실행 전 차단 앵커", "09_exceptions.md", "R01"),
    ("I099", "바인딩이 철회됐다는데", "없음 — 지도 §1-4", "19_three_questions.md", "R01"),
    ("I100", "첫 5분 레시피 지도", "없음 — rhwp-onboarding", "08_jump_to_skill.md", "R08"),
    ("I101", "inspect 3축", "rhwp inspect hidden-text --json", "08_jump_to_skill.md", "R08"),
    ("I102", "sanitize 메타 제거", "rhwp edit sanitize --json", "08_jump_to_skill.md", "R08"),
    ("I103", "insert-image 도장", "rhwp edit insert-image --json", "08_jump_to_skill.md", "R08"),
    ("I104", "thumbnail 미리보기", "rhwp thumbnail --json", "07_section_index.md", "R01"),
    ("I105", "export-structure 목차", "rhwp export-structure --json", "07_section_index.md", "R08"),
    ("I106", "word-count", "rhwp word-count --json", "07_section_index.md", "R01"),
    ("I107", "bookmarks", "rhwp bookmarks --json", "07_section_index.md", "R01"),
    ("I108", "charts 목록", "rhwp charts --json", "07_section_index.md", "R01"),
    ("I109", "threat-scan", "rhwp threat-scan --json", "08_jump_to_skill.md", "R08"),
    ("I110", "layout-anomaly", "rhwp layout-anomaly --json", "07_section_index.md", "R01"),
    ("I111", "audit 재현율", "rhwp audit --json", "08_jump_to_skill.md", "R08"),
    ("I112", "lineage 계보", "rhwp lineage --json", "08_jump_to_skill.md", "R08"),
    ("I113", "keygen 서명", "rhwp keygen --json", "07_section_index.md", "R01"),
    ("I114", "gate 정책", "rhwp gate --json", "07_section_index.md", "R01"),
    ("I115", "bundle 반출", "rhwp bundle export --json", "07_section_index.md", "R01"),
    ("I116", "disclose 선택 공개", "rhwp disclose redact --json", "07_section_index.md", "R01"),
    ("I117", "settle 정산", "rhwp settle propose --json", "07_section_index.md", "R01"),
    ("I118", "conformance 수준", "rhwp conformance --json", "07_section_index.md", "R01"),
    ("I119", "scan 한 파일", "rhwp scan --json", "07_section_index.md", "R01"),
    ("I120", "dump-pages 조판", "rhwp dump-pages --json", "07_section_index.md", "R01"),
]


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def write_text(path: Path, body: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not body.endswith("\n"):
        body += "\n"
    path.write_text(body, encoding="utf-8", newline="\n")


def write_json(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    text = json.dumps(obj, ensure_ascii=False, indent=2)
    path.write_text(text + "\n", encoding="utf-8", newline="\n")


def parse_front_matter(text: str) -> dict[str, str]:
    if not text.startswith("---\n"):
        return {}
    end = text.find("\n---\n", 4)
    if end < 0:
        return {}
    meta: dict[str, str] = {}
    for line in text[4:end].splitlines():
        if ":" in line:
            key, val = line.split(":", 1)
            meta[key.strip()] = val.strip()
    return meta


def days_since(iso: str) -> int | None:
    try:
        d = datetime.strptime(iso, "%Y-%m-%d").date()
    except ValueError:
        return None
    return (AS_OF - d).days


def is_stale(iso: str) -> bool:
    age = days_since(iso)
    if age is None:
        return True
    return age > STALE_DAYS


HEADING_RE = re.compile(r"^(#{2,6})\s+(.+?)\s*$")
FIELD_CELL_RE = re.compile(r"`([^`]+)`")
MD_LINK_RE = re.compile(r"\[([^\]]+)\]\(([^)]+)\)")


def slug_heading(title: str) -> str:
    title = re.sub(r"<[^>]+>", "", title)
    title = title.replace("`", "")
    title = re.sub(r"[^\w -]+", "", title, flags=re.UNICODE)
    title = re.sub(r"\s+", "-", title.strip())
    return title.lower()


def parse_map(text: str) -> dict:
    lines = text.splitlines()
    headings: list[dict] = []
    fields: list[dict] = []
    canonicals: list[dict] = []
    current_section = ""
    current_h2 = ""
    current_h3 = ""
    current_h4 = ""
    in_section2 = False
    in_section9 = False
    seen_fields: set[str] = set()
    seen_canon: set[str] = set()

    for i, line in enumerate(lines, start=1):
        hm = HEADING_RE.match(line)
        if hm:
            level = len(hm.group(1))
            title = hm.group(2).strip()
            sid = ""
            num_m = re.match(r"^([0-9]+(?:-[0-9]+)*)\.\s+", title)
            if num_m:
                sid = "S" + num_m.group(1).replace("-", "")
            elif title.startswith("유지 규약"):
                sid = "SMAINT"
            else:
                sid = f"SL{i}"
            if level == 2:
                current_h2 = title
                current_h3 = ""
                current_h4 = ""
                current_section = sid
                in_section2 = title.startswith("2.")
                in_section9 = title.startswith("9.")
            elif level == 3:
                current_h3 = title
                current_h4 = ""
                current_section = sid
            elif level == 4:
                current_h4 = title
                current_section = sid
            headings.append(
                {
                    "id": sid,
                    "level": level,
                    "title": title,
                    "line": i,
                    "anchor": slug_heading(title),
                    "h2": current_h2,
                }
            )
            continue

        if in_section2 and line.startswith("|") and "`" in line:
            cells = [c.strip() for c in line.strip().strip("|").split("|")]
            if len(cells) >= 2 and cells[0] not in {"필드", "---", ":---"}:
                names = FIELD_CELL_RE.findall(cells[0])
                ftype = cells[1] if len(cells) > 1 else ""
                if ftype in {"타입", "---"}:
                    continue
                for raw in names:
                    for name in re.split(r"\s*/\s*", raw):
                        name = name.strip()
                        if not name or name in seen_fields:
                            continue
                        if not re.match(r"^[A-Za-z_][A-Za-z0-9_\.\[\]]*$", name):
                            continue
                        seen_fields.add(name)
                        fields.append(
                            {
                                "name": name,
                                "typeHint": ftype.strip(),
                                "section": current_section or "S2",
                                "line": i,
                            }
                        )

        if in_section9:
            for label, href in MD_LINK_RE.findall(line):
                if href.startswith("http"):
                    continue
                rel = href.split("#", 1)[0]
                if not rel or rel in seen_canon:
                    continue
                if rel.endswith(".md") or rel.endswith(".txt"):
                    seen_canon.add(rel)
                    if rel.startswith("../"):
                        path = "mydocs/" + rel[3:]
                    elif "/" not in rel:
                        path = "mydocs/manual/" + rel
                    else:
                        path = "mydocs/manual/" + rel
                    canonicals.append(
                        {
                            "label": label,
                            "href": href,
                            "path": path,
                            "line": i,
                        }
                    )

    return {
        "headings": headings,
        "fields": fields,
        "canonicals": canonicals,
    }


def extract_map_binary_version(text: str) -> str:
    m = re.search(r"`rhwp (v[0-9]+\.[0-9]+\.[0-9]+)`", text)
    return m.group(1) if m else ""


def extract_package_version() -> str:
    cargo = read(REPO / "Cargo.toml")
    m = re.search(r'(?m)^version = "([^"]+)"', cargo)
    return m.group(1) if m else ""


def honesty_note() -> str:
    return (
        "이 스킬은 실 에이전트가 rhwp 참조 문서에 들어가는 진입점 라우터다. "
        "순서: llms.txt → agent_knowledge_map.md → 요청에 필요한 canonical 하나. "
        "지도는 요약·앵커만 담는다. 지도 행을 다시 쓰지 않는다. "
        "지도와 상세가 다르면 상세(canonical)를 따른다. "
        "봉투 필드 이름은 지도 §2 에서만 가져온다. "
        "last_verified 가 오래되면 추측으로 메우지 않는다. "
        "바이너리 버전이 지도와 다르면 바이너리(capabilities)가 이긴다. "
        "rhwp-codex(대전 장 항해)와 rhwp-agent-surface(3층 계약)를 다시 쓰지 않는다. "
        "gym 이 아니고 새 CLI 도 없다."
    )


def stop_rules() -> list[dict]:
    return [
        {
            "id": "R01",
            "when": "요청이 지도 한 절·정본 하나로 닫힘",
            "action": "llms.txt → 지도 해당 절 → 그 canonical 하나만 연다",
        },
        {
            "id": "R02",
            "when": "지도 수치·도구 개수를 믿기 전에 재측정이 필요",
            "action": "rhwp capabilities / capabilities --mcp / mcp-serve tools/list",
        },
        {
            "id": "R03",
            "when": "봉투 필드 이름·뜻이 필요",
            "action": "지도 §2 에서 이름을 찾는다. 없는 이름은 발명하지 않는다",
        },
        {
            "id": "R04",
            "when": f"지도 last_verified 가 {STALE_DAYS}일보다 오래됨",
            "action": "날짜를 보여주고 중단. 기억으로 사다리를 메우지 않음",
        },
        {
            "id": "R05",
            "when": "손에 든 바이너리 버전이 지도 §0 과 다름",
            "action": "바이너리가 이긴다. capabilities 로 다시 찍고 지도 숫자는 참고만",
        },
        {
            "id": "R06",
            "when": "지도 요약과 canonical 상세가 다름",
            "action": "canonical 을 따른다. 지도 행을 고쳐 쓰지 않고 상세로 점프",
        },
        {
            "id": "R07",
            "when": "§2 에 없는 필드 이름을 쓰려 함",
            "action": "중단. 철자 변형·암기 별칭을 만들지 않음",
        },
        {
            "id": "R08",
            "when": "요청이 실무 작업(채움·표·스윕·배치·세션)으로 닫힘",
            "action": "지도에서 절·정본을 고른 뒤 이웃 스킬로 점프. 지도를 더 읽지 않음",
        },
        {
            "id": "R09",
            "when": "요청이 대전 장 항해이거나 3층 계약·표면 추가",
            "action": "rhwp-codex 또는 rhwp-agent-surface 로 인계. 그 스킬을 여기서 재작성하지 않음",
        },
        {
            "id": "R10",
            "when": "지도를 처음부터 끝까지 읽으려 함",
            "action": "금지. 3문 진입으로 한 절만 고른다",
        },
        {
            "id": "R11",
            "when": "지식지도 전용 rhwp 하위명령을 만들려 함",
            "action": "금지. 이 스킬은 문서 라우터다",
        },
        {
            "id": "R12",
            "when": "gym pack 으로 지식 지도를 재현하려 함",
            "action": "금지. 실 에이전트 문서 진입점이지 gym 이 아님",
        },
        {
            "id": "R13",
            "when": "지도 기존 행을 더 자세히 풀어 쓰려 함",
            "action": "금지. 행 재서술 없이 canonical 로 보낸다",
        },
        {
            "id": "R14",
            "when": "§0·§2·§7 수치를 손으로 고치려 함",
            "action": "금지. 재측정 명령으로 실행해 갱신한다",
        },
        {
            "id": "R15",
            "when": "필드 사전을 대전이나 표면 스킬에서 재정의하려 함",
            "action": "지도 §2 가 단일 출처. 대전의 필드 장은 앵커만",
        },
        {
            "id": "R16",
            "when": "이웃 스킬 본문을 이 PR 에서 고치려 함",
            "action": "금지. 링크만",
        },
    ]


def jump_rows() -> list[dict]:
    return [
        {
            "id": "J01",
            "when": "누름틀·서식 채움·메일머지",
            "mapSection": "S11D",
            "canonical": "mydocs/manual/form_filling_guide.md",
            "skill": "rhwp-form-fill",
            "stop": "R08",
        },
        {
            "id": "J02",
            "when": "표 CSV 왕복·칸 기록",
            "mapSection": "S11B",
            "canonical": "mydocs/manual/cli_commands.md",
            "skill": "rhwp-table-exchange",
            "stop": "R08",
        },
        {
            "id": "J03",
            "when": "inspect·redact·sanitize·송신 점검",
            "mapSection": "S11F",
            "canonical": "mydocs/tech/agent_security/consumer_guide.md",
            "skill": "rhwp-security-sweep",
            "stop": "R08",
        },
        {
            "id": "J04",
            "when": "폴더 일괄·batch 축",
            "mapSection": "S11H",
            "canonical": "mydocs/manual/cli_json_pipeline_guide.md",
            "skill": "rhwp-bulk-pipeline",
            "stop": "R08",
        },
        {
            "id": "J05",
            "when": "render-diff·ir-diff 레이아웃",
            "mapSection": "S11G",
            "canonical": "mydocs/manual/cli_commands.md",
            "skill": "rhwp-visual-regression",
            "stop": "R08",
        },
        {
            "id": "J06",
            "when": "mcp-serve 부착·세션 도구",
            "mapSection": "S61",
            "canonical": "mydocs/manual/mcp_integration_guide.md",
            "skill": "rhwp-mcp-session",
            "stop": "R08",
        },
        {
            "id": "J07",
            "when": "CLI 분석·내보내기·디버그",
            "mapSection": "S50",
            "canonical": "mydocs/manual/cli_commands.md",
            "skill": "rhwp-cli",
            "stop": "R08",
        },
        {
            "id": "J08",
            "when": "처음 보는 문서 좁혀 읽기",
            "mapSection": "S11A",
            "canonical": "mydocs/manual/cli_commands.md",
            "skill": "rhwp-doc-triage",
            "stop": "R08",
        },
        {
            "id": "J09",
            "when": "run 계획·dry-run·verify",
            "mapSection": "S11D",
            "canonical": "mydocs/manual/cli_commands.md",
            "skill": "rhwp-safe-edit",
            "stop": "R08",
        },
        {
            "id": "J10",
            "when": "untrusted* 출처 표지",
            "mapSection": "S21",
            "canonical": "mydocs/tech/envelope_provenance.md",
            "skill": "rhwp-provenance",
            "stop": "R08",
        },
        {
            "id": "J11",
            "when": "replay·audit·lineage 영수증",
            "mapSection": "S22",
            "canonical": "mydocs/manual/cli_commands.md",
            "skill": "rhwp-work-receipt",
            "stop": "R08",
        },
        {
            "id": "J12",
            "when": "첫 설치·doctor·첫 5분",
            "mapSection": "S00",
            "canonical": "mydocs/manual/agent_onboarding.md",
            "skill": "rhwp-onboarding",
            "stop": "R08",
        },
        {
            "id": "J13",
            "when": "이슈·PR·기여 절차",
            "mapSection": "S90",
            "canonical": "CONTRIBUTING.md",
            "skill": "rhwp-contributor",
            "stop": "R08",
        },
        {
            "id": "J14",
            "when": "PDF/이미지 → HWPX 시험지",
            "mapSection": "S11E",
            "canonical": "mydocs/manual/cli_commands.md",
            "skill": "rhwp-exam-ingest",
            "stop": "R08",
        },
        {
            "id": "J15",
            "when": "전 명령 장 항해·실측 봉투 표본",
            "mapSection": "S50",
            "canonical": "mydocs/manual/agent_codex/00_서문.md",
            "skill": "rhwp-codex",
            "stop": "R09",
        },
        {
            "id": "J16",
            "when": "CLI/MCP 조각 추가·3층 계약",
            "mapSection": "S13",
            "canonical": "mydocs/manual/agent_surface_playbook.md",
            "skill": "rhwp-agent-surface",
            "stop": "R09",
        },
    ]


def first_read_order() -> list[dict]:
    return [
        {
            "step": 1,
            "path": CANONICAL_LLMS,
            "why": "루트 진입점. 지식 지도를 첫 문서로 가리킨다",
            "readHow": "머리 문단과 시작하기 목록만. 레시피 전체를 여기서 읽지 않는다",
        },
        {
            "step": 2,
            "path": CANONICAL_MAP,
            "why": "참조 문서의 단일 진입점. 3문 진입·§2 사전·§9 권위표",
            "readHow": "§0 실측 기준과 요청에 맞는 한 절만",
        },
        {
            "step": 3,
            "path": "<요청이 고른 canonical 하나>",
            "why": "상세 권위. 지도와 다르면 이쪽",
            "readHow": "그 파일만. 지도를 이어서 통독하지 않는다",
        },
    ]


def envelope() -> dict:
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "skill": SKILL_NAME,
        "notGym": True,
        "noNewCli": True,
        "noNewEditLogic": True,
        "routerOnly": True,
        "entryPoint": True,
        "doNotRenarrateMapRows": True,
        "canonicalWins": True,
        "binaryWinsOnVersionMismatch": True,
        "fieldDictionary": "mydocs/manual/agent_knowledge_map.md#2-봉투-필드-사전--필드-이름으로-찾는-역인덱스",
    }


def excerpt_paragraphs(text: str, source: str, limit: int) -> list[dict]:
    """원문에 그대로 있는 한 줄을 고른다. 공백을 접지 않는다."""
    out: list[dict] = []
    for line in text.splitlines():
        raw = line.rstrip()
        if len(raw) < 40:
            continue
        if raw.startswith("---") or raw.startswith("|---") or raw.startswith("| ---"):
            continue
        if raw.startswith("#") and len(raw) < 80:
            continue
        out.append(
            {
                "sourceFile": source,
                "raw": raw[:400],
                "excerpted": True,
                "fabricatedLive": False,
            }
        )
        if len(out) >= limit:
            break
    return out


def build_intents() -> list[dict]:
    rows = []
    for iid, utterance, command, reference, stop in INTENTS:
        rows.append(
            {
                "id": iid,
                "utterance": utterance,
                "command": command,
                "reference": reference,
                "stop": stop,
            }
        )
    return rows


def build_request_map() -> list[dict]:
    rows = []
    for i, (utterance, section, canonical, skill, stop) in enumerate(REQUEST_ROWS, start=1):
        rows.append(
            {
                "id": f"Q{i:03d}",
                "utterance": utterance,
                "mapSection": section,
                "canonical": canonical,
                "nextSkill": skill,
                "stop": stop,
                "readWholeMap": False,
                "renarrate": False,
            }
        )
    return rows


def build_journeys() -> list[dict]:
    journeys = []
    n = 0
    for req in REQUEST_ROWS:
        utterance, section, canonical, skill, stop = req
        n += 1
        steps = ["llms.txt", f"지도 {section}", canonical]
        if skill:
            steps.append(skill)
        journeys.append(
            {
                "id": f"Y{n:03d}",
                "title": utterance,
                "steps": steps,
                "stop": stop,
                "mapSection": section,
                "canonical": canonical,
                "nextSkill": skill,
            }
        )
    return journeys


def pick_canonical_for_heading(h: dict, parsed: dict) -> str:
    title = h["title"]
    if title.startswith("2.") or h["h2"].startswith("2."):
        return CANONICAL_MAP
    if title.startswith("6.") or h["h2"].startswith("6."):
        return "mydocs/manual/mcp_integration_guide.md"
    if title.startswith("4.") or "판정 3층" in title:
        return "mydocs/manual/mcp_integration_guide.md"
    if title.startswith("1-3") or "표면" in title:
        return "mydocs/manual/agent_surface_playbook.md"
    if title.startswith("1-2") or "실패" in title:
        return "mydocs/manual/agent_troubleshooting_guide.md"
    if "레시피" in title:
        return "mydocs/manual/recipes/01_fill_form_and_submit.md"
    if title.startswith("8.") or "계약 테스트" in title:
        return CANONICAL_MAP
    if title.startswith("7.") or "표본" in title:
        return CANONICAL_MAP
    if title.startswith("9.") or "문서 축" in title:
        if parsed["canonicals"]:
            return parsed["canonicals"][0]["path"]
        return "mydocs/manual/cli_commands.md"
    if "쓰기" in title or "누름틀" in title:
        return "mydocs/manual/form_filling_guide.md"
    if "지키기" in title or "보안" in title:
        return "mydocs/tech/agent_security/consumer_guide.md"
    if "대량" in title:
        return "mydocs/manual/cli_json_pipeline_guide.md"
    if "검증" in title:
        return "mydocs/manual/cli_commands.md"
    return "mydocs/manual/cli_commands.md"


def section_cards(parsed: dict) -> list[dict]:
    cards = []
    for h in parsed["headings"]:
        if h["level"] > 4:
            continue
        if h["level"] > 3:
            continue
        cards.append(
            {
                "id": h["id"],
                "title": h["title"],
                "level": h["level"],
                "line": h["line"],
                "anchor": h["anchor"],
                "canonical": pick_canonical_for_heading(h, parsed),
            }
        )
    return cards


def exceptions(map_verified: str, map_ver: str, pkg_ver: str) -> dict:
    age = days_since(map_verified) or 0
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        **envelope(),
        "paths": [
            {
                "id": "E01",
                "kind": "stale-last-verified",
                "stop": "R04",
                "staleDays": STALE_DAYS,
                "asOf": TODAY,
                "actualLastVerified": map_verified,
                "actualDaysSince": age,
                "actualStale": age > STALE_DAYS,
                "simulated": {
                    "lastVerified": "2025-01-01",
                    "daysSince": (AS_OF - date(2025, 1, 1)).days,
                    "stale": True,
                    "doNotFillFromMemory": True,
                },
            },
            {
                "id": "E02",
                "kind": "binary-version-mismatch",
                "stop": "R05",
                "mapBinary": map_ver,
                "packageVersion": pkg_ver,
                "mismatch": map_ver.lstrip("v") != pkg_ver,
                "winner": "binary",
                "remeasure": [c["argv"] for c in REMEASURE_COMMANDS],
                "doNotInventFields": True,
            },
            {
                "id": "E03",
                "kind": "map-vs-canonical",
                "stop": "R06",
                "winner": "canonical",
                "example": {
                    "map": CANONICAL_MAP,
                    "canonical": "mydocs/manual/cli_commands.md",
                    "rule": "플래그·종료 코드·수치의 최종 권위는 CLI 매뉴얼",
                },
            },
            {
                "id": "E04",
                "kind": "invented-field-name",
                "stop": "R07",
                "forbiddenExamples": [
                    "schema_version",
                    "untrusted_content",
                    "page_count",
                    "replaced_count",
                    "is_error",
                ],
                "dictionary": "map §2",
            },
        ],
    }


def required_refs() -> list[str]:
    return [
        "00_first_read.md",
        "01_remeasure.md",
        "02_tree.md",
        "03_request_map.md",
        "04_boundary.md",
        "05_envelope_dict.md",
        "06_canonicals.md",
        "07_section_index.md",
        "08_jump_to_skill.md",
        "09_exceptions.md",
        "10_stale_last_verified.md",
        "11_version_mismatch.md",
        "12_map_vs_canonical.md",
        "13_stop_conditions.md",
        "14_handoff.md",
        "15_pitfalls.md",
        "16_journeys.md",
        "17_intent_matrix.md",
        "18_field_lookup.md",
        "19_three_questions.md",
        "20_samples_index.md",
        "21_contract_tests_index.md",
        "22_mcp_remeasure.md",
        "23_transcripts.md",
        "24_decision_table.md",
        "25_sibling_boundary.md",
        "README.md",
    ]


def required_examples() -> list[str]:
    return [
        "README.md",
        "01_first_read.md",
        "02_remeasure.md",
        "03_field_lookup.md",
        "04_stale_last_verified.md",
        "05_version_mismatch.md",
        "06_map_vs_canonical.md",
        "07_jump_form_fill.md",
        "08_jump_table.md",
        "09_jump_security.md",
        "10_jump_batch.md",
        "11_jump_visual.md",
        "12_jump_mcp.md",
        "13_handoff_codex.md",
        "14_handoff_surface.md",
        "15_do_not_read_whole_map.md",
        "16_fail_symptom.md",
        "17_address_vocab.md",
        "18_judgment_layers.md",
        "19_invented_field.md",
        "20_provenance.md",
        "21_onboarding.md",
        "22_work_receipt.md",
        "23_safe_edit.md",
        "24_doc_triage.md",
        "25_export_png_missing.md",
        "26_profile_router.md",
        "27_session_save.md",
        "28_extract_pages_base.md",
        "29_samples_negative.md",
        "30_contract_pick.md",
        "31_reject_gym.md",
        "32_reject_new_cli.md",
        "33_canonical_cli_manual.md",
        "34_three_questions_add.md",
        "35_stop_and_jump.md",
    ]


def md_table(headers: list[str], rows: list[list[str]]) -> str:
    lines = [
        "| " + " | ".join(headers) + " |",
        "| " + " | ".join("---" for _ in headers) + " |",
    ]
    for row in rows:
        lines.append("| " + " | ".join(row) + " |")
    return "\n".join(lines)


def chapter(title: str, body_parts: list[str]) -> str:
    parts = [
        f"# {title}",
        "",
        f"이슈 #{ISSUE}. 스킬 `{SKILL_NAME}`. gym 아님. 새 CLI 없음.",
        "이 장은 지도 행을 다시 쓰지 않는다. 앵커와 다음 점프만 적는다.",
        "",
    ]
    parts.extend(body_parts)
    if not parts[-1].endswith("\n") and parts[-1] != "":
        parts.append("")
    return "\n".join(parts) + "\n"


def emit_references(parsed: dict, map_meta: dict, cards: list[dict]) -> None:
    req_rows = build_request_map()
    intents = build_intents()
    journeys = build_journeys()
    jumps = jump_rows()
    stops = stop_rules()
    fields = parsed["fields"]
    headings = parsed["headings"]

    write_text(
        REF / "00_first_read.md",
        chapter(
            "첫 읽기 순서 — llms.txt → 지도 → canonical 하나",
            [
                "실 에이전트가 rhwp 문서에 들어갈 때 순서는 고정이다.",
                "",
                md_table(
                    ["순서", "문서", "읽는 범위"],
                    [
                        ["1", "`llms.txt`", "머리 문단 + 시작하기. 레시피 전체를 여기서 소화하지 않는다"],
                        ["2", "`mydocs/manual/agent_knowledge_map.md`", "§0 과 요청에 맞는 **한 절**"],
                        ["3", "요청이 고른 canonical **하나**", "상세. 지도와 다르면 이쪽"],
                    ],
                ),
                "",
                "이 순서를 건너뛰고 ROADMAP·대전·표면 플레이북을 첫 문서로 열지 않는다.",
                "`AGENTS.md` 도 같은 진입점을 가리킨다.",
                "",
                "## 한 절만",
                "",
                "지도는 길다. §2 전수 사전을 요청과 무관하게 통독하지 않는다.",
                "3문 진입(무엇을/실패/추가)으로 절을 고르면 그 표의 **권위 열**이",
                "다음에 열 파일이다.",
                "",
                "## 이 스킬이 닫는 것",
                "",
                "- 진입 순서",
                "- 재측정 명령 세 개",
                "- 필드 이름 출처(§2)",
                "- 낡은 last_verified · 버전 불일치 · 지도≠상세",
                "- 이웃 스킬로 점프하는 시점",
                "",
                "명령 장의 실측 봉투는 `rhwp-codex` 다. 3층 계약은",
                "`rhwp-agent-surface` 다. 여기서 다시 쓰지 않는다.",
            ],
        ),
    )

    write_text(
        REF / "01_remeasure.md",
        chapter(
            "재측정 — capabilities / --mcp / tools/list",
            [
                "지도 §0 이 적어 둔 재확인 법이다. 개수는 계약이 아니다.",
                "버전이 다르면 **바이너리가 이긴다**.",
                "",
                md_table(
                    ["ID", "명령", "보는 것"],
                    [
                        ["RM01", "`rhwp capabilities`", "명령·플래그·recordFields·종료 코드"],
                        ["RM02", "`rhwp capabilities --mcp`", "MCP 무상태 도구 선언"],
                        ["RM03", "`rhwp mcp-serve` + `tools/list`", "세션 포함 실제 목록"],
                    ],
                ),
                "",
                "## tools/list 조립",
                "",
                "지도 §0 코드 블록을 그대로 쓴다. initialize → initialized →",
                "tools/list. 세션 도구는 `--mcp` 매니페스트에 없다.",
                "",
                "## 검색",
                "",
                "`rhwp capabilities --search <낱말> [--json]` 은 명령 이름·요약·",
                "하위 명령을 찾는다. `export-agent-manifest --json` 은 네 축을",
                "모으고 빠진 축은 `missingAxes` 로 밝힌다. 필드 이름은 여기서",
                "발명하지 말고 매니페스트·§2 에서 가져온다.",
                "",
                "## 하지 말 것",
                "",
                "- 지도 §0 표의 개수를 암기해 답하기",
                "- 바이너리 없이 개수를 손보기",
                "- `rhwp knowledge-map` 같은 재측정 전용 명령 발명",
            ],
        ),
    )

    write_text(
        REF / "02_tree.md",
        chapter(
            "판단 나무",
            [
                "```",
                "요청",
                " ├─ 문서 진입/어디부터? → llms.txt → 지도 §0 → 한 절",
                " ├─ 필드 이름? → 지도 §2 (없으면 중단, 발명 금지)",
                " ├─ 수치가 믿기나? → RM01/RM02/RM03",
                " ├─ last_verified 30일+? → 중단 (R04)",
                " ├─ 바이너리 ≠ 지도 버전? → 바이너리, 재측정 (R05)",
                " ├─ 지도 ≠ canonical? → canonical (R06)",
                " ├─ 실무 작업? → 절 앵커 후 이웃 스킬 (R08)",
                " ├─ 대전 장 항해? → rhwp-codex (R09)",
                " ├─ 표면 추가/3층? → rhwp-agent-surface (R09)",
                " └─ gym / 새 CLI? → 거부 (R12/R11)",
                "```",
                "",
                "나무는 라우팅만 한다. 표 행을 펼치지 않는다.",
            ],
        ),
    )

    req_md_rows = [
        [r["id"], r["utterance"], r["mapSection"], r["stop"]] for r in req_rows
    ]
    write_text(
        REF / "03_request_map.md",
        chapter(
            "요청 대조",
            [
                "발화를 지도 절과 정지 규칙에 붙인다. 명령 시퀀스를 여기서",
                "풀어 쓰지 않는다. 다음 문서는 `canonical` 열(픽스처)이다.",
                "",
                md_table(["ID", "발화", "절", "정지"], req_md_rows),
                "",
                f"행 수 {len(req_rows)}. 기계 가독은 `fixtures/request_map.json`.",
            ],
        ),
    )

    write_text(
        REF / "04_boundary.md",
        chapter(
            "형제 스킬 경계 — 대전·표면과 겹치지 않는다",
            [
                md_table(
                    ["스킬", "축", "이 스킬과의 관계"],
                    [
                        [
                            "`rhwp-knowledge-map`",
                            "문서 진입점",
                            "llms.txt → 지도 → canonical 하나",
                        ],
                        [
                            "`rhwp-codex`",
                            "대전 장 항해",
                            "명령 가족 장·실측 봉투. 필드 정의는 지도 §2",
                        ],
                        [
                            "`rhwp-agent-surface`",
                            "3층 계약",
                            "CLI JSON · MCP 무상태 · MCP 세션. 조각 추가 절차",
                        ],
                    ],
                ),
                "",
                "대전 SKILL 이 이미 말한다: 봉투 필드 정의는 대전이 아니라",
                "지식지도 §2-2 가 단일 출처다. 이 스킬은 그 출처로 **들이는**",
                "문만 닫는다.",
                "",
                "표면 플레이북은 지도 §1-3 이 가리킨다. 수용 기준·드리프트",
                "가드는 그 스킬/정본에 두고 여기서 복제하지 않는다.",
                "",
                "두 스킬 본문을 이 작업에서 고치지 않는다.",
            ],
        ),
    )

    sample_fields = fields[:12]
    field_rows = [[f["name"], f["section"], "지도 §2"] for f in sample_fields]
    write_text(
        REF / "05_envelope_dict.md",
        chapter(
            "봉투 필드 사전은 지도 §2 — 이름을 발명하지 않는다",
            [
                "필드 이름·타입·뜻은 `agent_knowledge_map.md` §2 가 사전이다.",
                "이 스킬은 사전을 옮기지 않는다. 조회 규칙만 적는다.",
                "",
                "1. 이름을 지도 §2 표에서 찾는다.",
                "2. 없으면 쓰지 않는다. 철자 변형(`schema_version`)도 금지.",
                "3. 뜻은 지도 표 셀을 읽고, 명령별 등장 맥락이 더 필요하면",
                "   그 명령의 canonical(보통 CLI 매뉴얼)로 간다.",
                "4. 대전·표면 스킬에 필드 뜻을 다시 쓰지 않는다.",
                "",
                "픽스처 `fixtures/envelope_fields.json` 은 지도에서 **추출한",
                "이름 목록**이다. 정의 문장을 복제하지 않았다.",
                "",
                "추출된 이름 보기(정의는 지도에만 있다):",
                "",
                md_table(["이름", "절", "권위"], field_rows),
                "",
                f"추출 개수 {len(fields)}. 개수는 계약이 아니다. 이름은 지도에",
                "실제로 나타난 것만.",
            ],
        ),
    )

    canon_rows = [[c["label"], c["path"]] for c in parsed["canonicals"][:24]]
    write_text(
        REF / "06_canonicals.md",
        chapter(
            "canonical 권위 — 지도 §9 축",
            [
                "지도 §9 표가 '이 지도 바깥의 권위'다. 아래는 경로 인덱스다.",
                "각 문서의 본문을 여기 옮기지 않는다.",
                "",
                md_table(["축", "경로"], canon_rows),
                "",
                "플래그·종료 코드는 `cli_commands.md`. MCP 층은",
                "`mcp_integration_guide.md`. 표면 추가는",
                "`agent_surface_playbook.md`. 실패 증상은",
                "`agent_troubleshooting_guide.md`.",
                "",
                "지도 표와 위 파일이 다르면 **파일을 따른다**.",
            ],
        ),
    )

    sec_rows = [
        [c["id"], c["title"][:40], str(c["line"])]
        for c in cards
        if c["level"] <= 3
    ]
    write_text(
        REF / "07_section_index.md",
        chapter(
            "지도 절 인덱스 — 제목·줄번호만",
            [
                "각 행은 인덱스 카드다. 표 내용을 다시 쓰지 않는다.",
                f"카드 파일은 `fixtures/cards/` ({len(cards)}장).",
                "",
                md_table(["ID", "제목", "줄"], sec_rows),
            ],
        ),
    )

    jump_md = [[j["id"], j["when"], j["skill"], j["stop"]] for j in jumps]
    write_text(
        REF / "08_jump_to_skill.md",
        chapter(
            "지도를 그만 읽고 스킬로 점프할 때",
            [
                "지도에서 절과 정본을 골랐으면 그 작업을 수행하는 스킬로 넘긴다.",
                "이 스킬 안에서 채움·표·스윕·배치·세션을 재구현하지 않는다.",
                "",
                md_table(["ID", "언제", "스킬", "정지"], jump_md),
                "",
                "점프 후 이 스킬로 돌아와 지도를 이어서 통독하지 않는다.",
                "새 질문이 생기면 다시 3문 진입으로 한 절만 고른다.",
            ],
        ),
    )

    write_text(
        REF / "09_exceptions.md",
        chapter(
            "예외 네 갈래",
            [
                md_table(
                    ["ID", "종류", "정지"],
                    [
                        ["E01", "stale last_verified", "R04"],
                        ["E02", "binary version mismatch", "R05"],
                        ["E03", "map vs canonical", "R06"],
                        ["E04", "invented field name", "R07"],
                    ],
                ),
                "",
                "상세는 10·11·12 장과 `fixtures/exceptions.json`.",
                "없는 프로필 이름은 지도 §1-5 가 이미 실행 전 차단이라고",
                "적었다. 그 행을 여기 풀어 쓰지 않는다. 앵커만 따른다.",
            ],
        ),
    )

    write_text(
        REF / "10_stale_last_verified.md",
        chapter(
            "last_verified 가 낡았을 때",
            [
                f"기준일 {TODAY}. 허용 {STALE_DAYS}일.",
                f"지도 frontmatter `last_verified` = `{map_meta['last_verified']}`.",
                f"경과일 {map_meta['daysSince']}. stale={map_meta['stale']}.",
                "",
                "실제 지도가 신선해도, 30일을 넘긴 사본을 만나면:",
                "",
                "1. 날짜를 보여 준다.",
                "2. 명령 사다리를 기억으로 메우지 않는다.",
                "3. 재측정(RM01–RM03) 전에는 §0 개수를 인용하지 않는다.",
                "",
                "시뮬레이션 픽스처는 `2025-01-01` 이다. 그 날짜를 실제",
                "지도 날짜로 바꾸어 쓰지 않는다.",
            ],
        ),
    )

    write_text(
        REF / "11_version_mismatch.md",
        chapter(
            "바이너리 버전 불일치",
            [
                f"지도 §0 표기: `{map_meta['mapBinary']}`.",
                f"이 작업나무 `Cargo.toml` package version: `{map_meta['packageVersion']}`.",
                f"불일치: `{map_meta['versionMismatch']}`.",
                "",
                "규칙(지도 §0): 버전이 다르면 바이너리가 이긴다. 문서와",
                "어긋나면 문서를 고친다. 이 스킬은 문서를 고치지 않고,",
                "에이전트에게 재측정을 시킨다.",
                "",
                "- 개수(98 명령, 181 도구, 329 필드)를 암기 인용하지 않는다.",
                "- `recordFields` 합집합은 capabilities 가 말한다.",
                "- 세션 도구 존재 여부는 tools/list 가 말한다.",
                "",
                "불일치를 숨기려고 필드 이름을 지어내지 않는다 (R07).",
            ],
        ),
    )

    write_text(
        REF / "12_map_vs_canonical.md",
        chapter(
            "지도와 상세가 다르면 상세를 따른다",
            [
                "지도 머리말이 이미 단일 출처 원칙을 적었다. 이 장은 그 문장을",
                "재서술하지 않고 적용 규칙만 고정한다.",
                "",
                md_table(
                    ["주제", "지도 역할", "이기는 문서"],
                    [
                        ["플래그·종료 코드", "앵커", "cli_commands.md"],
                        ["MCP 3층", "앵커", "mcp_integration_guide.md"],
                        ["필드 이름", "사전 본체", "지도 §2 (예외: 상세가 더 새로우면 상세)"],
                        ["표면 추가 절차", "§1-3 한 줄", "agent_surface_playbook.md"],
                        ["실패 처방", "증상→앵커", "agent_troubleshooting_guide.md"],
                    ],
                ),
                "",
                "에이전트가 지도를 고치려 하지 않는다. 상세를 읽고 작업을 진행한다.",
                "기존 행을 더 길게 풀어 쓰는 PR 은 R13 으로 거절한다.",
            ],
        ),
    )

    stop_rows = [[s["id"], s["when"], s["action"]] for s in stops]
    write_text(
        REF / "13_stop_conditions.md",
        chapter(
            "정지 규칙",
            [
                md_table(["ID", "언제", "행동"], stop_rows),
                "",
                "금지 기본값:",
                "",
                "- " + ", ".join(f"`{c}`" for c in INVENTED_COMMANDS),
                "- gym/ 트리 작성",
                "- rhwp-codex / rhwp-agent-surface 본문 재작성",
                "- DocumentCore 편집 로직",
                "- 지도 행 재서술",
            ],
        ),
    )

    write_text(
        REF / "14_handoff.md",
        chapter(
            "인계",
            [
                "이 스킬의 산출은 '다음에 열 파일 하나' 또는 '다음에 쓸 스킬",
                "하나'다. 편집 결과물이 아니다.",
                "",
                md_table(
                    ["대상", "넘기는 것", "넘기지 않는 것"],
                    [
                        ["canonical", "경로 + 절 앵커", "지도 표 전문"],
                        ["이웃 스킬", "첫 명령 힌트(정본에 있는 것)", "그 스킬 본문"],
                        ["사람", "last_verified / 버전 불일치 사실", "추측 사다리"],
                    ],
                ),
                "",
                "인계 후 지도 통독을 재개하지 않는다.",
            ],
        ),
    )

    write_text(
        REF / "15_pitfalls.md",
        chapter(
            "함정",
            [
                "1. 지도를 교본처럼 통독한다 → R10. 한 절만.",
                "2. §0 개수를 답으로 암기한다 → R02. 재측정.",
                "3. 필드 이름을 영어 관용으로 만든다 → R07.",
                "4. 대전에 필드 뜻을 다시 적는다 → R15.",
                "5. 지도와 CLI 매뉴얼이 다를 때 지도를 고친다 → R06, 상세를 따른다.",
                "6. extract-pages 쪽 번호를 search.page 와 혼동 → 지도 §3-3 앵커 후 CLI 매뉴얼.",
                "7. isError 만 보고 identical:false 를 실패로 처리 → 지도 §4 앵커.",
                "8. 세션 도구를 capabilities --mcp 에서 찾는다 → tools/list.",
                "9. gym 과제로 진입점을 시험한다 → R12.",
                "10. 지식지도 CLI 를 제안한다 → R11.",
                "",
                "유지 규약(지도 말미): 새 표면은 행만 추가. 수치는 실행해서 갱신.",
                "링크 검사: `py scripts/check_markdown_links.py mydocs/manual/agent_knowledge_map.md`.",
            ],
        ),
    )

    jrows = [[j["id"], j["title"][:36], j["stop"]] for j in journeys]
    write_text(
        REF / "16_journeys.md",
        chapter(
            "여정",
            [
                "각 여정은 첫 읽기 경로다. 살아 있는 CLI 를 여기서 돌리지 않는다.",
                "",
                md_table(["ID", "제목", "정지"], jrows),
            ],
        ),
    )

    irows = [[i["id"], i["utterance"][:36], i["stop"]] for i in intents]
    write_text(
        REF / "17_intent_matrix.md",
        chapter(
            "발화 행렬",
            [
                "발화 → 정지 규칙. 명령 칸이 '없음'이면 새 명령을 만들지 말고",
                "문서/재측정으로 닫는다.",
                "",
                md_table(["ID", "발화", "정지"], irows),
            ],
        ),
    )

    lookup_rows = []
    for name in [
        "schemaVersion",
        "source",
        "untrustedContent",
        "untrustedFields",
        "filledCount",
        "notFound",
        "ambiguous",
        "replacedCount",
        "identical",
        "changedPages",
        "matchCount",
        "findingCount",
    ]:
        hit = next((f for f in fields if f["name"] == name), None)
        if hit:
            lookup_rows.append([name, hit["section"], str(hit["line"])])
    write_text(
        REF / "18_field_lookup.md",
        chapter(
            "필드 조회 레시피",
            [
                "자주 묻는 이름만 줄번호로 붙인다. 뜻은 지도 그 줄에 있다.",
                "",
                md_table(["이름", "절", "줄"], lookup_rows),
                "",
                "목록에 없다고 이름을 만들지 않는다. §2 전체에서 다시 찾는다.",
                "그래도 없으면 R07.",
            ],
        ),
    )

    write_text(
        REF / "19_three_questions.md",
        chapter(
            "3문 진입",
            [
                "지도 §1 의 세 질문. 표 행은 지도에만 있다.",
                "",
                md_table(
                    ["질문", "절", "다음에 열 것"],
                    [
                        ["무엇을 하려는가", "§1-1", "해당 가족 표의 권위 열"],
                        ["실패했는가", "§1-2", "agent_troubleshooting_guide.md 앵커"],
                        ["추가하려는가", "§1-3", "agent_surface_playbook.md"],
                    ],
                ),
                "",
                "네 번째가 필요하면: 다른 언어(§1-4), 프로필(§1-5).",
                "레시피 처음부터 끝까지는 §1-3-1 이 파일 이름을 준다.",
                "07·08 레시피를 여기서 발명하지 않는다.",
            ],
        ),
    )

    write_text(
        REF / "20_samples_index.md",
        chapter(
            "표본 지도 인덱스",
            [
                "어떤 파일이 어떤 시험에 쓰이는지는 지도 §7 표다.",
                "이 장은 파일 목록을 복제하지 않는다.",
                "",
                "- 성격별 표본 → §7-1",
                "- 보안 축은 음성 코퍼스 → §7-2",
                "- redact 가 실제로 잡는 표본 → §7-3",
                "",
                "양성 공격 표본을 이 스킬 fixtures 에 넣지 않는다.",
                "계약 테스트가 실행 중 합성한다는 지도 문장을 따른다.",
            ],
        ),
    )

    write_text(
        REF / "21_contract_tests_index.md",
        chapter(
            "계약 테스트 지도 인덱스",
            [
                "표면을 고칠 때 어느 테스트가 red 여야 하는지는 지도 §8 이다.",
                "테스트 본문을 여기 옮기지 않는다.",
                "",
                md_table(
                    ["축", "절"],
                    [
                        ["봉투·계약", "§8-1"],
                        ["조회", "§8-2"],
                        ["편집", "§8-3"],
                        ["변환·렌더", "§8-4"],
                        ["MCP", "§8-5"],
                        ["보안", "§8-6"],
                    ],
                ),
                "",
                "이 스킬 자신의 계약은 `scripts/tests/test_agent_knowledge_map.py`",
                "와 `tests/cases/agent_knowledge_map_skill_contract.rs` 다.",
                "바이너리를 부르지 않는다.",
            ],
        ),
    )

    write_text(
        REF / "22_mcp_remeasure.md",
        chapter(
            "MCP 재측정과 세션 경계",
            [
                "`capabilities --mcp` 는 무상태 선언이다. 세션 도구는",
                "`mcp-serve` 의 `tools/list` 에만 있다. 지도 §6-2.",
                "",
                "세션의 유일한 기록 지점은 `hwp_doc_save` 라는 문장은 지도",
                "§1-1 (자)에 있다. 여기 절차를 늘여 쓰지 않는다.",
                "",
                "NDJSON 도구는 `structuredContent` 가 null 이다 (§6-3).",
                "호스트 부착 절차는 `rhwp-mcp-session` + MCP 가이드.",
                "",
                "재측정 JSON-RPC 순서는 지도 §0 코드 블록.",
            ],
        ),
    )

    write_text(
        REF / "23_transcripts.md",
        chapter(
            "발췌 계약",
            [
                "`fixtures/transcripts/` 는 살아 있는 CLI 를 다시 돌린 결과가",
                "아니다. `llms.txt` 와 `agent_knowledge_map.md` 에서 문단을",
                "잘라 온 것이다.",
                "",
                "- `excerptedFromCanonical: true`",
                "- `fabricatedLive: false`",
                "- raw 앞부분은 원문에 실제로 있다",
                "",
                "중략이 있는 블록을 JSON 으로 재구성하지 않는다.",
            ],
        ),
    )

    write_text(
        REF / "24_decision_table.md",
        chapter(
            "결정표",
            [
                md_table(
                    ["조건", "다음", "정지"],
                    [
                        ["문서가 어디 있나", "llms.txt → 지도", "R01"],
                        ["필드 이름", "지도 §2", "R03"],
                        ["숫자/개수", "재측정", "R02"],
                        ["날짜 30일+", "중단", "R04"],
                        ["버전 불일치", "바이너리", "R05"],
                        ["지도≠상세", "상세", "R06"],
                        ["없는 필드명", "중단", "R07"],
                        ["실무 작업", "이웃 스킬", "R08"],
                        ["대전/표면", "해당 스킬", "R09"],
                        ["통독", "거부", "R10"],
                        ["새 CLI", "거부", "R11"],
                        ["gym", "거부", "R12"],
                    ],
                ),
            ],
        ),
    )

    write_text(
        REF / "25_sibling_boundary.md",
        chapter(
            "이웃을 다시 쓰지 않는다",
            [
                "devel 에 있는 스킬 파일은 읽기만 한다.",
                "",
                md_table(
                    ["스킬", "역할", "이 PR"],
                    [[s, "이웃", "미수정"] for s in PEER_SKILLS_ON_DEVEL],
                ),
                "",
                "`rhwp-agent-surface` 는 형제 PR 의 축이다. 이 나무에 파일이",
                "없어도 경계를 지킨다. 만들지 않고 다시 쓰지도 않는다.",
                "",
                "`rhwp-codex` 는 이미 있다. 장 항해·생성기·신선도 검사를",
                "여기로 끌어오지 않는다.",
            ],
        ),
    )

    write_text(
        REF / "README.md",
        chapter(
            "rhwp-knowledge-map references",
            [
                "목차는 SKILL.md 레퍼런스 절과 `fixtures/skill_index.json`.",
                "생성기: `_gen_pack.py`. 지도 행을 생성하지 않고 인덱스만 만든다.",
            ],
        ),
    )


def emit_examples() -> None:
    specs = [
        (
            "01_first_read.md",
            "첫 읽기",
            [
                "요청: rhwp 문서는 어디서 시작하나.",
                "1. `llms.txt` 머리 — 지식 지도를 첫 문서로 읽으라고 적혀 있다.",
                "2. `agent_knowledge_map.md` §0 — 실측 기준과 재확인 명령.",
                "3. 질문이 '시작점'이면 여기서 멈춘다. §2 를 통독하지 않는다.",
                "정지 R01. gym 아님.",
            ],
        ),
        (
            "02_remeasure.md",
            "재측정",
            [
                "요청: 지금 바이너리에 도구가 몇 개냐.",
                "지도 §0 숫자를 읽지 말고 `rhwp capabilities` 를 친다.",
                "MCP 면 `--mcp`, 세션이면 mcp-serve tools/list.",
                "정지 R02.",
            ],
        ),
        (
            "03_field_lookup.md",
            "필드 조회",
            [
                "요청: untrustedFields 가 뭐야.",
                "지도 §2-1 표에서 이름을 찾는다. 뜻을 이 예제에 옮겨 적지 않는다.",
                "더 필요하면 `mydocs/tech/envelope_provenance.md`.",
                "다음 스킬 `rhwp-provenance`. 정지 R03 또는 R08.",
            ],
        ),
        (
            "04_stale_last_verified.md",
            "낡은 날짜",
            [
                "요청: 2025-01-01 last_verified 사본을 읽었다.",
                "오늘 2026-08-18 기준으로 30일을 넘긴다.",
                "날짜를 보여주고 사다리를 기억으로 메우지 않는다. 정지 R04.",
            ],
        ),
        (
            "05_version_mismatch.md",
            "버전 불일치",
            [
                "지도는 v0.8.3, Cargo.toml 은 0.8.4 일 수 있다.",
                "개수를 지도에서 인용하지 않는다. capabilities 를 친다.",
                "정지 R05. 필드 이름을 지어 맞추지 않는다.",
            ],
        ),
        (
            "06_map_vs_canonical.md",
            "지도 ≠ 상세",
            [
                "요청: 이 플래그의 정확한 기본값.",
                "지도는 앵커만 준다. `cli_commands.md` 해당 절을 연다.",
                "숫자가 달라도 매뉴얼을 따른다. 정지 R06.",
            ],
        ),
        (
            "07_jump_form_fill.md",
            "서식 점프",
            [
                "요청: 이 신청서 채워.",
                "llms.txt → 지도 §1-1 (라) 앵커 → form_filling_guide.md.",
                "지도를 더 읽지 않고 `rhwp-form-fill` 로 넘긴다.",
                "첫 수는 그 스킬/정본의 `rhwp fields --json`. 정지 R08.",
            ],
        ),
        (
            "08_jump_table.md",
            "표 점프",
            [
                "요청: 표를 CSV 로 뽑아 고치고 돌려.",
                "지도 §1-1 (나)/(라) 앵커만 확인하고",
                "`rhwp-table-exchange` 로 점프. 정지 R08.",
            ],
        ),
        (
            "09_jump_security.md",
            "보안 점프",
            [
                "요청: 보내도 되나.",
                "지도 §1-1 (바) → consumer_guide.md → `rhwp-security-sweep`.",
                "inspect 3축을 이 스킬에서 재설명하지 않는다. 정지 R08.",
            ],
        ),
        (
            "10_jump_batch.md",
            "배치 점프",
            [
                "요청: 폴더 수백 건 텍스트.",
                "지도 §1-1 (아) → cli_json_pipeline_guide.md →",
                "`rhwp-bulk-pipeline`. 정지 R08.",
            ],
        ),
        (
            "11_jump_visual.md",
            "시각 회귀 점프",
            [
                "요청: 편집 전후를 숫자로.",
                "지도 §1-1 (사) → `rhwp-visual-regression`. 정지 R08.",
            ],
        ),
        (
            "12_jump_mcp.md",
            "MCP 점프",
            [
                "요청: 호스트에 rhwp 붙여.",
                "지도 §6 앵커 → mcp_integration_guide.md →",
                "`rhwp-mcp-session`. tools/list 재측정은 RM03. 정지 R08.",
            ],
        ),
        (
            "13_handoff_codex.md",
            "대전 인계",
            [
                "요청: 전 명령 교본을 장 순서대로.",
                "이 스킬은 진입점만. `rhwp-codex` 로 인계한다.",
                "필드 정의는 대전이 아니라 지도 §2. 정지 R09.",
            ],
        ),
        (
            "14_handoff_surface.md",
            "표면 인계",
            [
                "요청: JSON 명령을 하나 더하고 싶다.",
                "지도 §1-3 → agent_surface_playbook.md →",
                "`rhwp-agent-surface`. 이 나무에 스킬 파일이 없어도 만들지 않는다.",
                "정지 R09.",
            ],
        ),
        (
            "15_do_not_read_whole_map.md",
            "통독 거부",
            [
                "요청: 지식 지도를 처음부터 끝까지 요약해.",
                "거부. 무엇을/실패/추가 중 하나를 고르게 한다.",
                "정지 R10.",
            ],
        ),
        (
            "16_fail_symptom.md",
            "실패 증상",
            [
                "요청: stream did not contain valid UTF-8.",
                "지도 §1-2 표의 권위 열 → troubleshooting 입력·인코딩.",
                "처방을 지도에 풀어 쓰지 않는다. 정지 R01.",
            ],
        ),
        (
            "17_address_vocab.md",
            "주소 어휘",
            [
                "요청: 페이지가 0부터야 1부터야.",
                "지도 §3-1 앵커. extract-pages 만 1 기준이라는 함정은 §3-3.",
                "상세 플래그는 CLI 매뉴얼. 정지 R01.",
            ],
        ),
        (
            "18_judgment_layers.md",
            "판정 3층",
            [
                "요청: isError false 인데 identical false.",
                "지도 §4 앵커 — 부정적 결과는 데이터.",
                "층 설명 전문은 MCP 가이드. 정지 R01 또는 R08.",
            ],
        ),
        (
            "19_invented_field.md",
            "발명 필드 거부",
            [
                "요청: page_count 로 읽으면 되지 않나.",
                "지도 §2 에 `page_count` 가 있는지 찾는다. 없으면 쓰지 않는다.",
                "`pageCount` 가 있는지는 사전이 말한다. 정지 R07.",
            ],
        ),
        (
            "20_provenance.md",
            "출처 표지",
            [
                "요청: 이 값이 문서에서 왔나.",
                "지도 §2-1 → export-provenance-map → `rhwp-provenance`.",
                "정지 R08.",
            ],
        ),
        (
            "21_onboarding.md",
            "온보딩",
            [
                "요청: 처음인데 5분 안에.",
                "지도 §0 자기서술 한 줄 → `rhwp-onboarding`.",
                "doctor 를 여기서 재구현하지 않는다. 정지 R08.",
            ],
        ),
        (
            "22_work_receipt.md",
            "작업 영수증",
            [
                "요청: 이 편집을 증명하고 싶다.",
                "지도 §2 영수증 필드 가족 앵커 → `rhwp-work-receipt`.",
                "정지 R08.",
            ],
        ),
        (
            "23_safe_edit.md",
            "안전 편집",
            [
                "요청: dry-run 하고 verify 로 저장.",
                "지도 §1-1 (라) run/--dry-run/--verify 앵커 → `rhwp-safe-edit`.",
                "정지 R08.",
            ],
        ),
        (
            "24_doc_triage.md",
            "트리아지",
            [
                "요청: 긴 HWP 를 전문 없이.",
                "지도 §1-1 (가) → `rhwp-doc-triage`. 정지 R08.",
            ],
        ),
        (
            "25_export_png_missing.md",
            "없는 기능",
            [
                "요청: PNG 로 렌더.",
                "지도 §1-1 (다) 주의 — native-skia.",
                "capabilities 의 available/requiresFeature 를 본다. 정지 R02.",
            ],
        ),
        (
            "26_profile_router.md",
            "프로필",
            [
                "요청: 행정서식만.",
                "지도 §1-5 앵커. `capabilities --mcp --profile 행정서식`.",
                "도구 목록을 여기 복제하지 않는다. 정지 R08.",
            ],
        ),
        (
            "27_session_save.md",
            "세션 저장",
            [
                "요청: 연 채로 여러 번 고쳐.",
                "지도 §1-1 (자) — 기록은 hwp_doc_save 만.",
                "`rhwp-mcp-session`. 정지 R08.",
            ],
        ),
        (
            "28_extract_pages_base.md",
            "쪽 자르기 기준",
            [
                "요청: search 가 page 13 을 줬다. 그 쪽만 잘라.",
                "지도 §3-3 ① 앵커 — from/to 는 1 기준.",
                "CLI 매뉴얼에서 플래그를 확인. 정지 R01.",
            ],
        ),
        (
            "29_samples_negative.md",
            "음성 표본",
            [
                "요청: samples 에서 injection 양성 찾아.",
                "지도 §7-2 — samples 는 음성 코퍼스.",
                "양성을 이 fixtures 에 만들지 않는다. 정지 R01.",
            ],
        ),
        (
            "30_contract_pick.md",
            "계약 테스트 고르기",
            [
                "요청: search 봉투를 바꾸면 어느 테스트가 red.",
                "지도 §8-2 앵커. 테스트 파일을 열어 확인.",
                "정지 R01.",
            ],
        ),
        (
            "31_reject_gym.md",
            "gym 거부",
            [
                "요청: gym pack 으로 지식 지도 커버리지 과제.",
                "거부. 실 에이전트 문서 경로. 정지 R12.",
            ],
        ),
        (
            "32_reject_new_cli.md",
            "새 CLI 거부",
            [
                "요청: rhwp knowledge-map --open 을 만들자.",
                "거부. 문서 라우터. 정지 R11.",
            ],
        ),
        (
            "33_canonical_cli_manual.md",
            "CLI 매뉴얼로",
            [
                "요청: info 플래그 전부.",
                "지도 §1-1 (가) 권위 열 → cli_commands.md §info.",
                "지도 표를 길게 풀어 쓰지 않는다. 정지 R01.",
            ],
        ),
        (
            "34_three_questions_add.md",
            "추가하려는가",
            [
                "요청: 도구를 하나 더하고 싶다.",
                "3문 중 '추가' — 지도 §1-3 → 표면 플레이북.",
                "`rhwp-agent-surface`. 정지 R09.",
            ],
        ),
        (
            "35_stop_and_jump.md",
            "읽고 멈추기",
            [
                "요청이 실무로 닫히면 지도에서 권위 경로를 얻은 즉시 점프한다.",
                "§5 명령 전수, §6 도구 전수는 그 요청에 필요 없다.",
                "정지 R08 또는 R10.",
            ],
        ),
    ]
    write_text(
        EXAMPLES / "README.md",
        chapter(
            "예제 목차",
            [
                "각 파일은 한 요청의 첫 읽기·점프 레시피다.",
                "살아 있는 봉투를 돌리지 않는다.",
                "",
                md_table(
                    ["파일", "제목"],
                    [[name, title] for name, title, _ in specs],
                ),
            ],
        ),
    )
    for name, title, lines in specs:
        write_text(
            EXAMPLES / name,
            chapter(
                title,
                [
                    *lines,
                    "",
                    "## 첫 읽기 점검",
                    "",
                    "1. `llms.txt` 가 지식 지도를 가리키는지 확인한다.",
                    "2. 지도 frontmatter `last_verified` 를 오늘과 비교한다 (30일).",
                    "3. 지도 §0 바이너리 표기와 `rhwp capabilities` 의 version 을 비교한다.",
                    "4. 요청에 필요한 절 하나만 연다. §2 전수 사전을 통독하지 않는다.",
                    "5. 권위 열의 canonical **하나**를 연다. 지도와 다르면 그쪽.",
                    "6. 실무 작업이면 이웃 스킬로 점프하고 이 스킬을 닫는다.",
                    "",
                    "## 하지 말 것",
                    "",
                    "- 지도 표를 이 예제에 다시 적기",
                    "- `schema_version` / `page_count` 같은 철자 변형",
                    "- `rhwp knowledge-map` 발명",
                    "- gym pack 으로 이 경로를 재현",
                    "- `rhwp-codex` 또는 `rhwp-agent-surface` 본문 수정",
                    "",
                    "## 재측정 (숫자가 필요하면)",
                    "",
                    "```",
                    "rhwp capabilities",
                    "rhwp capabilities --mcp",
                    "rhwp mcp-serve   # initialize → tools/list",
                    "```",
                    "",
                    "정본: `llms.txt`, `mydocs/manual/agent_knowledge_map.md`.",
                    "지도 행 재서술 금지. 필드 이름 발명 금지. gym 아님.",
                ],
            ),
        )


def emit_card_markdown(cards: list[dict]) -> None:
    """주요 절만 인덱스 카드로 남긴다. 표 행은 복사하지 않는다."""
    card_ref = REF / "cards"
    keep = [c for c in cards if c["level"] == 2 or (
        c["level"] == 3 and c["id"] in {
            "S11", "S12", "S13", "S14", "S15",
            "S21", "S22", "S23", "S24", "S25",
            "S31", "S32", "S33",
            "S61", "S62", "S63",
            "S71", "S72", "S73",
        }
    )]
    for c in keep:
        body = chapter(
            f"카드 {c['id']}",
            [
                f"- 제목: {c['title']}",
                f"- 지도: `{CANONICAL_MAP}` 줄 {c['line']} 앵커 `{c['anchor']}`",
                f"- 다음 정본: `{c['canonical']}`",
                "- 재서술: 금지",
                "",
                "표·수치·필드 뜻은 이 파일에 없다. 지도 그 줄 또는 정본으로 간다.",
                "이 카드를 읽은 뒤 지도 전체를 이어서 읽지 않는다.",
                "",
                "## 사용",
                "",
                "1. 요청이 이 절인지 확인한다.",
                "2. 맞으면 지도 해당 줄만 연다.",
                "3. 권위 열이 가리키는 canonical 하나를 연다.",
                "4. 실무면 이웃 스킬로 점프한다 (08_jump_to_skill.md).",
                "",
                "## 이 카드가 닫지 않는 것",
                "",
                "- 명령 시퀀스 재구현",
                "- 필드 정의 재서술",
                "- 대전 장 항해 (`rhwp-codex`)",
                "- 3층 계약 (`rhwp-agent-surface`)",
            ],
        )
        write_text(card_ref / f"{c['id']}.md", body)


def emit_fixtures(parsed: dict, map_meta: dict, cards: list[dict]) -> None:
    req = build_request_map()
    intents = build_intents()
    journeys = build_journeys()
    stops = stop_rules()
    jumps = jump_rows()
    fields = parsed["fields"]

    base = envelope()

    write_json(
        FIXT / "meta.json",
        {
            **base,
            "asOf": TODAY,
            "staleDays": STALE_DAYS,
            "mapLastVerified": map_meta["last_verified"],
            "mapBinary": map_meta["mapBinary"],
            "packageVersion": map_meta["packageVersion"],
            "versionMismatch": map_meta["versionMismatch"],
            "mapDaysSince": map_meta["daysSince"],
            "mapStale": map_meta["stale"],
            "headingCount": len(parsed["headings"]),
            "fieldCountExtracted": len(fields),
            "canonicalCount": len(parsed["canonicals"]),
        },
    )
    write_json(
        FIXT / "skill_index.json",
        {
            **base,
            "references": required_refs(),
            "examples": required_examples(),
            "forbiddenSkillsTouch": FORBIDDEN_REWRITE,
            "peerSkillsOnDevel": PEER_SKILLS_ON_DEVEL,
            "inventedCommands": INVENTED_COMMANDS,
            "firstRead": [CANONICAL_LLMS, CANONICAL_MAP],
            "fieldDictionarySection": "§2",
            "issue": ISSUE,
        },
    )
    write_json(
        FIXT / "first_read.json",
        {**base, "order": first_read_order()},
    )
    write_json(
        FIXT / "remeasure.json",
        {**base, "commands": REMEASURE_COMMANDS},
    )
    write_json(
        FIXT / "tree.json",
        {
            **base,
            "firstMove": "llms.txt → agent_knowledge_map.md → canonical 하나",
            "coreReuse": [CANONICAL_LLMS, CANONICAL_MAP],
            "forbiddenSkillsTouch": FORBIDDEN_REWRITE,
            "honesty": honesty_note(),
        },
    )
    write_json(FIXT / "request_map.json", {**base, "rows": req, "count": len(req)})
    write_json(FIXT / "stop_rules.json", {**base, "rules": stops, "count": len(stops)})
    write_json(FIXT / "exceptions.json", exceptions(
        map_meta["last_verified"], map_meta["mapBinary"], map_meta["packageVersion"]
    ))
    write_json(
        FIXT / "last_verified.json",
        {
            **base,
            "path": CANONICAL_MAP,
            "lastVerified": map_meta["last_verified"],
            "asOf": TODAY,
            "staleDays": STALE_DAYS,
            "daysSince": map_meta["daysSince"],
            "stale": map_meta["stale"],
            "simulatedStale": {
                "lastVerified": "2025-01-01",
                "daysSince": (AS_OF - date(2025, 1, 1)).days,
                "stale": True,
                "doNotTreatAsFresh": True,
                "doNotFillFromMemory": True,
            },
        },
    )
    write_json(
        FIXT / "version_mismatch.json",
        {
            **base,
            "mapBinary": map_meta["mapBinary"],
            "packageVersion": map_meta["packageVersion"],
            "mismatch": map_meta["versionMismatch"],
            "winner": "binary",
            "remeasure": ["rhwp capabilities", "rhwp capabilities --mcp", "tools/list"],
        },
    )
    write_json(
        FIXT / "envelope_fields.json",
        {
            **base,
            "source": CANONICAL_MAP,
            "section": "§2",
            "extracted": True,
            "invented": False,
            "definitionsCopied": False,
            "names": [f["name"] for f in fields],
            "count": len(fields),
            "lookupSample": [
                {"name": f["name"], "section": f["section"], "line": f["line"]}
                for f in fields
                if f["name"]
                in {
                    "schemaVersion",
                    "source",
                    "untrustedContent",
                    "untrustedFields",
                    "filledCount",
                    "notFound",
                    "ambiguous",
                    "replacedCount",
                    "identical",
                    "changedPages",
                    "matchCount",
                    "findingCount",
                }
            ],
        },
    )
    write_json(
        FIXT / "section_cards.json",
        {
            **base,
            "cards": cards,
            "count": len(cards),
            "doNotRenarrate": True,
        },
    )
    write_json(
        FIXT / "canonicals.json",
        {**base, "items": parsed["canonicals"], "count": len(parsed["canonicals"])},
    )
    write_json(FIXT / "jump_skills.json", {**base, "jumps": jumps, "count": len(jumps)})
    write_json(
        FIXT / "intent_matrix.json",
        {**base, "intents": intents, "count": len(intents)},
    )
    write_json(
        FIXT / "journeys.json",
        {**base, "journeys": journeys, "count": len(journeys)},
    )
    write_json(
        FIXT / "honesty.json",
        {
            **base,
            "note": honesty_note(),
            "doNotRenarrateMapRows": True,
            "doNotInventFieldNames": True,
            "canonicalWins": True,
            "binaryWins": True,
            "notGym": True,
            "noNewCli": True,
            "doNotRewriteCodex": True,
            "doNotRewriteSurface": True,
        },
    )
    # transcripts excerpted from llms + map
    map_text = read(MAP_PATH)
    llms_text = read(LLMS_PATH)
    excerpts = []
    for item in excerpt_paragraphs(llms_text, CANONICAL_LLMS, 12):
        excerpts.append(item)
    for item in excerpt_paragraphs(map_text, CANONICAL_MAP, 32):
        excerpts.append(item)
    records = []
    ids = []
    for i, item in enumerate(excerpts, start=1):
        tid = f"X{i:03d}"
        ids.append(tid)
        records.append({"id": tid, **item})
    write_json(
        FIXT / "transcripts.json",
        {
            **base,
            "ids": ids,
            "count": len(ids),
            "excerptedFromCanonical": True,
            "fabricatedLive": False,
            "items": records,
        },
    )

    write_json(
        FIXT / "decision_table.json",
        {
            **base,
            "rows": [
                {"when": s["when"], "stop": s["id"], "action": s["action"]}
                for s in stops
            ],
        },
    )


def main() -> None:
    if not MAP_PATH.is_file():
        raise SystemExit(f"정본 지도 없음: {MAP_PATH}")
    if not LLMS_PATH.is_file():
        raise SystemExit(f"llms.txt 없음: {LLMS_PATH}")

    map_text = read(MAP_PATH)
    meta = parse_front_matter(map_text)
    last_verified = meta.get("last_verified", "")
    map_bin = extract_map_binary_version(map_text)
    pkg = extract_package_version()
    age = days_since(last_verified)
    map_meta = {
        "last_verified": last_verified,
        "daysSince": age,
        "stale": is_stale(last_verified),
        "mapBinary": map_bin,
        "packageVersion": pkg,
        "versionMismatch": map_bin.lstrip("v") != pkg,
    }
    parsed = parse_map(map_text)
    cards = section_cards(parsed)

    emit_references(parsed, map_meta, cards)
    emit_examples()
    emit_fixtures(parsed, map_meta, cards)
    print(
        f"ok headings={len(parsed['headings'])} fields={len(parsed['fields'])} "
        f"cards={len(cards)} verified={last_verified} map={map_bin} pkg={pkg}"
    )


if __name__ == "__main__":
    main()
