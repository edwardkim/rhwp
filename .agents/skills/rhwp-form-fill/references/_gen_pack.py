#!/usr/bin/env python3
"""[#5300] rhwp-form-fill 레퍼런스·픽스처 생성기.

새 edit 로직을 발명하지 않는다. 명령·봉투·종료 코드는 cli_commands.md 와
기존 계약 테스트(fields / fill-fields / occurrence / batch fill / verify /
sanitize)가 이미 고정한 표면만 복제한다.
"""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
SKILL = Path(__file__).resolve().parents[1]
REF = SKILL / "references"
FIXT = REF / "fixtures"
# 픽스처는 스킬 아래 한 곳만. 테스트가 여기를 읽는다.

ISSUE = 5300
SCHEMA = "1.0"

# 기존 샘플. 새 HWP 바이너리를 만들지 않는다.
SAMPLES = {
    "form01": {
        "path": "samples/form-01.hwp",
        "fieldCount": 1,
        "names": ["myMsg01"],
        "note": "레시피 01·05 실측 표본. ClickHere 1개.",
    },
    "field01": {
        "path": "samples/field-01.hwp",
        "fieldCount": 11,
        "names": [
            "회사명",
            "작성자",
            "부서명",
            "전화번호",
            "이메일",
            "제목",
            "목차1",
            "목차1",
            "목차1",
            "목차1",
            "목차1",
        ],
        "note": "본문+글상자+표 셀 혼합. fill-fields 계약 표본.",
    },
    "field01memo": {
        "path": "samples/field-01-memo.hwp",
        "fieldCount": 11,
        "names": [
            "회사명",
            "작성자",
            "부서명",
            "전화번호",
            "이메일",
            "제목",
            "목차1",
            "목차1",
            "목차1",
            "목차1",
            "목차1",
        ],
        "note": "HelpState memo 지시문이 붙은 형제 문서.",
    },
    "hwp3none": {
        "path": "samples/hwp3-sample.hwp",
        "fieldCount": 0,
        "names": [],
        "note": "누름틀 없음. fieldCount 0 은 오류가 아니라 축 전환 신호.",
    },
    "reg": {
        "path": "samples/80168_regulatory_analysis.hwp",
        "fieldCount": 1070,
        "uniqueNames": 151,
        "repeat": {"피규제집단명": 14},
        "note": "규제영향분석서. 이름[N] 계약(#3476) 표본. 저장소에 없을 수 있다.",
        "optional": True,
    },
}

COMMANDS = [
    {
        "id": "fields",
        "argv": ["fields", "<서식>", "--json"],
        "writes": False,
        "issue": 3281,
        "when": "서식이 무엇을 요구하는지 읽을 때",
    },
    {
        "id": "fill-fields",
        "argv": ["edit", "fill-fields", "<서식>", "--data", "<JSON|@파일>", "-o", "<출력>", "--json"],
        "writes": True,
        "issue": 3329,
        "when": "단건 채움",
    },
    {
        "id": "fill-fields-occurrence",
        "argv": ["edit", "fill-fields", "<서식>", "--data", '{"이름[N]":"값"}', "-o", "<출력>", "--json"],
        "writes": True,
        "issue": 3476,
        "when": "같은 이름이 여러 번",
    },
    {
        "id": "batch-fill",
        "argv": [
            "batch",
            "fill",
            "--form",
            "<서식>",
            "--data",
            "<행.jsonl|.csv>",
            "--out-dir",
            "<폴더>",
            "--json",
        ],
        "writes": True,
        "issue": 3719,
        "when": "서식 1 + 데이터 N행",
    },
    {
        "id": "dry-run",
        "argv": ["…", "--dry-run", "--json"],
        "writes": False,
        "issue": 3329,
        "when": "파일을 쓰기 전 판정",
    },
    {
        "id": "verify",
        "argv": ["…", "--verify", "--json"],
        "writes": True,
        "issue": 3702,
        "when": "저장 직후 재파싱 대조",
    },
    {
        "id": "insert-image",
        "argv": [
            "edit",
            "insert-image",
            "<파일>",
            "--image",
            "<그림>",
            "--page",
            "N",
            "--x",
            "X",
            "--y",
            "Y",
            "-o",
            "<출력>",
            "--json",
        ],
        "writes": True,
        "issue": 3719,
        "when": "직인·서명",
    },
    {
        "id": "sanitize",
        "argv": ["edit", "sanitize", "<파일>", "-o", "<출력>", "--json"],
        "writes": True,
        "issue": 3719,
        "when": "제출 전 메타 제거",
    },
]

ENVELOPES = {
    "fields": {
        "required": [
            "schemaVersion",
            "source",
            "fieldCount",
            "fields",
        ],
        "fieldItem": [
            "fieldId",
            "fieldType",
            "name",
            "guide",
            "memo",
            "command",
            "value",
            "editableInForm",
            "location",
        ],
        "location": ["section", "paragraph", "nested"],
        "optional": ["textSecurity"],
        "untrustedHint": "fields[].value / guide / memo 는 문서 파생 데이터",
    },
    "fill-fields": {
        "required": [
            "schemaVersion",
            "source",
            "dryRun",
            "filledCount",
            "filled",
            "notFound",
            "ambiguous",
        ],
        "filledItem": ["name", "occurrence", "value"],
        "ambiguousItem": ["name", "matched", "total"],
        "whenSaved": ["output", "outputFormat"],
        "whenVerify": ["verify"],
        "verifyItem": ["identical", "diffCount"],
        "alsoSeen": ["changedPages", "confusable"],
    },
    "batch-fill": {
        "sameAs": "fill-fields",
        "extra": ["row"],
        "failure": ["schemaVersion", "source", "error", "exitClass", "row"],
        "stdout": "NDJSON",
        "stderr": "batch fill: N행 중 …",
    },
    "sanitize": {
        "required": [
            "schemaVersion",
            "source",
            "keepPreview",
            "removedCount",
            "removed",
            "output",
            "outputFormat",
        ],
        "removedItem": ["field", "before"],
        "idempotent": "두 번째 실행 removedCount 는 0",
    },
    "insert-image": {
        "required": [
            "schemaVersion",
            "source",
            "image",
            "page",
            "x",
            "y",
            "width",
            "height",
            "dryRun",
            "changedPages",
            "overflow",
        ],
        "whenSaved": ["binDataId", "output", "outputFormat"],
        "unit": "HWPUNIT 1/7200 inch, A4 세로 59528x84188",
    },
}

STOP_RULES = [
    ("F01", "runtime exit 1", "중단. 원본 불변"),
    ("F02", "fieldCount 0", "rhwp-table-exchange"),
    ("F03", "textSecurity not clean", "rhwp-security-sweep"),
    ("F04", "조사만 요청", "fields 에서 정지"),
    ("F05", "ambiguous 비어 있지 않음", "이름[N] 재지목"),
    ("F06", "notFound 잔류", "name 을 그대로 복사"),
    ("F07", "verify + 빈 배열", "채움 완료"),
    ("F08", "제출 요청", "sanitize"),
    ("F09", "데이터 행 0", "exit 2, 상류 확인"),
    ("F10", "batch 행 실패", "행별 레코드로 격리"),
    ("F11", "name-field notFound", "게이트에서 제외"),
    ("F12", "verify identical false", "exit 3, 산출은 남음"),
    ("F13", "서식 N개 폴더", "rhwp-bulk-pipeline"),
    ("F14", "질문이 이미 답", "다음 단 금지"),
]

PITFALLS = [
    {
        "id": "P01",
        "trap": "--data 파일을 CP949 로 저장",
        "signal": "stream did not contain valid UTF-8, exit 1",
        "fix": "UTF-8 로 다시 저장",
    },
    {
        "id": "P02",
        "trap": "--name-field 컬럼이 매 행 notFound",
        "signal": "notFound 에 파일명 용도 컬럼",
        "fix": "게이트 비교에서 그 컬럼 제외. 실패가 아니다",
    },
    {
        "id": "P03",
        "trap": "batch fill 에 stdin 파일 목록",
        "signal": "아무 일도 안 일어남",
        "fix": "--form + --data 인자 축",
    },
    {
        "id": "P04",
        "trap": "ambiguous 무시",
        "signal": "14개 중 1개만 채워진 문서",
        "fix": "이름[N] 재지목 루프",
    },
    {
        "id": "P05",
        "trap": "헤더만 있는 CSV",
        "signal": "오류: --data 에 데이터 행이 없습니다, exit 2",
        "fix": "상류 명단 조회 0건부터",
    },
    {
        "id": "P06",
        "trap": "name-field 값 중복",
        "signal": "나중 행이 먼저 행을 덮어씀(_2 접미가 있음)",
        "fix": "명령이 중복 검사를 안 한다. 데이터에서 유일키를 만든다",
    },
    {
        "id": "P07",
        "trap": "머리말/각주 필드가 fields 에 없음",
        "signal": "사람이 보는 칸이 목록에 없다",
        "fix": "실재 사각지대. 이 스킬이 재귀를 넓히지 않는다",
    },
    {
        "id": "P08",
        "trap": "페이지를 1부터 셈",
        "signal": "insert-image --page 1 이 두 번째 쪽",
        "fix": "0 기준. 한컴 표기(1부터)와 혼동 금지",
    },
    {
        "id": "P09",
        "trap": "insert-image 좌표를 mm/px 로 줌",
        "signal": "도장이 점만 하거나 overflow",
        "fix": "HWPUNIT. 1mm ≈ 283.46",
    },
    {
        "id": "P10",
        "trap": "로고 셀의 기관명 필드를 채움",
        "signal": "로고와 텍스트 겹침",
        "fix": "location.nested + export-tables 로 그림 칸이면 건너뜀",
    },
    {
        "id": "P11",
        "trap": "보고만 믿고 재독 생략",
        "signal": "filledCount 는 맞는데 값이 안 보임",
        "fix": "fields 재독 또는 --verify",
    },
    {
        "id": "P12",
        "trap": "새 플래그·새 명령 발명",
        "signal": "알 수 없는 하위명령, exit 2",
        "fix": "cli_commands.md 표면만. 이 스킬은 발명 금지",
    },
]

HANDOFF = [
    {
        "when": "fieldCount 0 이고 표 빈 칸",
        "to": "rhwp-table-exchange",
        "cmd": "edit set-cell",
    },
    {
        "when": "textSecurity 또는 주입 의심",
        "to": "rhwp-security-sweep",
        "cmd": "inspect hidden-text|injection|unicode",
    },
    {
        "when": "같은 문서를 여러 edit 로 고침",
        "to": "rhwp-safe-edit",
        "cmd": "run 계획서 3층",
    },
    {
        "when": "서식 파일이 폴더에 수백",
        "to": "rhwp-bulk-pipeline",
        "cmd": "batch fields / batch fill 이 아닌 파일 목록 축",
    },
    {
        "when": "채우지 말고 문서가 뭔지만",
        "to": "rhwp-doc-triage",
        "cmd": "info → explain → …",
    },
    {
        "when": "출처 표지 소비",
        "to": "rhwp-provenance",
        "cmd": "export-provenance-map",
    },
]

# 공공·사내 서식에서 에이전트가 자주 만나는 누름틀 이름.
# 값은 발명하지 않는다. 키만 카탈로그다. 실제 name 은 항상 fields --json.
COMMON_FIELD_NAMES = [
    ("성명", "신청인·작성자 이름. 반복되면 이름[N]"),
    ("이름", "성명과 혼용. fields 의 name 을 그대로"),
    ("생년월일", "YYYY. M. D. 또는 서식 guide 를 따름"),
    ("주민등록번호", "PII. 채운 뒤 sanitize 와 별개로 redact 축을 검토"),
    ("주소", "여러 줄이 정상인 칸. overflow 는 판단 자료"),
    ("전화번호", "하이픈 포함 여부는 guide"),
    ("휴대전화", "전화번호와 별 필드인 서식이 많다"),
    ("이메일", "필드-01 표본에도 있음"),
    ("소속", "기관·부서"),
    ("부서명", "필드-01 표본"),
    ("직위", "직급과 분리된 서식 있음"),
    ("직급", "직위와 혼동 금지"),
    ("회사명", "필드-01 표본. 로고 셀이면 건너뜀"),
    ("작성자", "필드-01 표본"),
    ("제목", "필드-01 표본"),
    ("목차1", "필드-01 에서 5회 반복"),
    ("날짜", "작성일·신청일과 별 필드일 수 있음"),
    ("작성일", "날짜와 키가 다름"),
    ("신청일", "작성일과 키가 다름"),
    ("기관명", "로고 셀 함정 P10"),
    ("문서번호", "채우지 말라는 memo 가 있으면 존중"),
    ("수신", "공문 머리"),
    ("경유", "공문 머리"),
    ("제목줄", "제목과 별 키"),
    ("사업명", "계획서"),
    ("사업기간", "시작-끝 한 칸인 서식 있음"),
    ("사업비", "숫자 포맷은 guide"),
    ("담당자", "성명과 별 칸"),
    ("담당부서", "부서명과 별 칸"),
    ("피규제집단명", "규제영향분석서 14회 반복 표본"),
    ("규제목적", "장문. overflow 가능"),
    ("규제내용", "장문"),
    ("myMsg01", "form-01.hwp 실측 키"),
    ("인", "도장 칸. 텍스트 대신 insert-image"),
    ("서명", "insert-image 축"),
    ("직인", "insert-image 축"),
]


def dump(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    text = json.dumps(obj, ensure_ascii=False, indent=2)
    path.write_text(text + "\n", encoding="utf-8", newline="\n")


def skill_index() -> dict:
    refs = [
        "00_tree.md",
        "01_fields_survey.md",
        "02_fill_fields.md",
        "03_repeat_occurrence.md",
        "04_batch_fill.md",
        "05_dry_run_verify.md",
        "06_sanitize.md",
        "07_envelopes.md",
        "08_pitfalls.md",
        "09_journeys.md",
        "10_handoff.md",
        "11_failure_signals.md",
        "12_data_formats.md",
        "13_name_field.md",
        "14_insert_image.md",
        "15_axis_choice.md",
        "16_worked_traces.md",
        "17_intent_matrix.md",
        "18_field_catalog.md",
        "19_gate_recipes.md",
        "20_exit_codes.md",
        "README.md",
    ]
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "skill": "rhwp-form-fill",
        "notGym": True,
        "noNewEditLogic": True,
        "forbiddenSkillsTouch": [
            "rhwp-onboarding",
            "rhwp-mcp-session",
            "rhwp-safe-edit",
            "rhwp-provenance",
            "rhwp-doc-triage",
        ],
        "forbiddenTrees": ["gym/"],
        "references": refs,
        "coreTopics": [
            "fields survey",
            "fill-fields",
            "이름[순번]",
            "batch fill",
            "--dry-run",
            "--verify",
            "sanitize",
        ],
        "authority": [
            "mydocs/manual/cli_commands.md",
            "mydocs/manual/recipes/01_fill_form_and_submit.md",
            "mydocs/manual/recipes/05_mail_merge_batch_fill.md",
            "mydocs/manual/form_filling_guide.md",
        ],
    }


def command_ladder() -> dict:
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "ladder": COMMANDS,
        "order": [
            "fields",
            "dry-run",
            "fill-fields",
            "fill-fields-occurrence",
            "batch-fill",
            "verify",
            "insert-image",
            "sanitize",
        ],
        "notForcedTraversal": True,
    }


def envelope_keys() -> dict:
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "commands": ENVELOPES,
        "exitCodes": {
            "0": "성공 (0건 포함: fieldCount 0, filledCount 0 도 성공 봉투)",
            "1": "런타임 (파일 없음·쓰기 실패). stdout 비움. 원본 불변",
            "2": "사용법 (인자/JSON/빈 데이터 파일)",
            "3": "--verify IR 차이. 산출물은 남는다",
        },
    }


def stop_rules() -> dict:
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "rules": [
            {"id": i, "when": w, "action": a} for i, w, a in STOP_RULES
        ],
    }


def pitfalls() -> dict:
    return {"schemaVersion": SCHEMA, "issue": ISSUE, "pitfalls": PITFALLS}


def handoff() -> dict:
    return {"schemaVersion": SCHEMA, "issue": ISSUE, "handoff": HANDOFF}


def samples() -> dict:
    return {"schemaVersion": SCHEMA, "issue": ISSUE, "samples": SAMPLES}


def field_catalog() -> dict:
    items = []
    for name, note in COMMON_FIELD_NAMES:
        items.append(
            {
                "name": name,
                "note": note,
                "copyExactly": True,
                "source": "catalog-of-typical-keys, not a live fields --json",
            }
        )
    # 표본에서 온 확정 키를 다시 표기
    live = []
    for key, sample in SAMPLES.items():
        for idx, name in enumerate(sample.get("names") or []):
            live.append(
                {
                    "sample": key,
                    "path": sample["path"],
                    "occurrence": idx
                    if sample["names"].count(name) > 1
                    else 0,
                    "name": name,
                    "indexedKey": f"{name}[{sample['names'][: idx + 1].count(name) - 1}]"
                    if sample["names"].count(name) > 1
                    else name,
                }
            )
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "typicalKeys": items,
        "liveFromSamples": live,
        "rule": "에이전트는 typicalKeys 를 추측 입력으로 쓰지 않는다. fields --json 의 name 만 --data 키다.",
    }


KOR_SURNAMES = [
    "김",
    "이",
    "박",
    "최",
    "정",
    "강",
    "조",
    "윤",
    "장",
    "임",
    "한",
    "오",
    "서",
    "신",
    "권",
    "황",
    "안",
    "송",
    "류",
    "홍",
]
KOR_GIVEN = [
    "민준",
    "서연",
    "도윤",
    "하은",
    "시우",
    "지유",
    "주원",
    "서윤",
    "하준",
    "지안",
    "지호",
    "수아",
    "준서",
    "하윤",
    "건우",
    "채원",
    "우진",
    "다은",
    "현우",
    "예은",
    "선우",
    "소율",
    "연우",
    "지민",
    "유준",
    "수빈",
    "은우",
    "예린",
    "시현",
    "하린",
]


def person(i: int) -> dict:
    surname = KOR_SURNAMES[i % len(KOR_SURNAMES)]
    given = KOR_GIVEN[i % len(KOR_GIVEN)]
    name = surname + given
    return {
        "성명": name,
        "myMsg01": f"{name} 귀하",
        "회사명": f"주식회사 {surname}{i:02d}",
        "작성자": name,
        "부서명": ["기획", "총무", "감사", "사업", "정보화"][i % 5] + "과",
        "전화번호": f"02-{1000 + (i % 9000):04d}-{1000 + ((i * 7) % 9000):04d}",
        "이메일": f"user{i:03d}@example.go.kr",
        "제목": f"서식 채움 표본 {i + 1:03d}",
    }


def batch_rows() -> dict:
    rows = [person(i) for i in range(40)]
    headers = [
        "성명",
        "myMsg01",
        "회사명",
        "작성자",
        "부서명",
        "전화번호",
        "이메일",
        "제목",
    ]
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "note": "메일머지 연습 행. 서식의 실제 name 과 맞춘 뒤 쓴다.",
        "rowCount": len(rows),
        "headers": headers,
        "rows": rows,
        "encoding": "utf-8",
        "emptyCsvRejected": {
            "csv": "성명,myMsg01\n",
            "exit": 2,
            "message": "오류: --data 에 데이터 행이 없습니다",
        },
        "stdinIgnored": True,
    }


OCCURRENCE_CASES = []


def occurrence_catalog() -> dict:
    cases = []
    # 목차1 x5 on field-01
    for n in range(5):
        cases.append(
            {
                "id": f"occ-mokcha-{n}",
                "sample": "samples/field-01.hwp",
                "name": "목차1",
                "total": 5,
                "key": f"목차1[{n}]",
                "bareKeyFills": 0 if n == 0 else None,
                "bareKeyAmbiguous": True,
                "outOfRange": False,
            }
        )
    cases.append(
        {
            "id": "occ-mokcha-oob",
            "sample": "samples/field-01.hwp",
            "name": "목차1",
            "total": 5,
            "key": "목차1[5]",
            "outOfRange": True,
            "landsIn": "notFound",
        }
    )
    cases.append(
        {
            "id": "occ-mokcha-neg",
            "sample": "samples/field-01.hwp",
            "name": "목차1",
            "total": 5,
            "key": "목차1[-1]",
            "outOfRange": True,
            "landsIn": "notFound",
            "note": "음수 순번은 범위 밖. 발명한 wrap-around 없음",
        }
    )
    # 규제영향분석서 14
    for n in range(14):
        cases.append(
            {
                "id": f"occ-reg-{n:02d}",
                "sample": "samples/80168_regulatory_analysis.hwp",
                "name": "피규제집단명",
                "total": 14,
                "key": f"피규제집단명[{n}]",
                "optionalSample": True,
                "outOfRange": False,
                "issue": 3476,
            }
        )
    cases.append(
        {
            "id": "occ-reg-oob",
            "sample": "samples/80168_regulatory_analysis.hwp",
            "name": "피규제집단명",
            "total": 14,
            "key": "피규제집단명[14]",
            "optionalSample": True,
            "outOfRange": True,
            "landsIn": "notFound",
        }
    )
    # unique names — no index required
    for name in ["회사명", "작성자", "부서명", "전화번호", "이메일", "제목", "myMsg01"]:
        cases.append(
            {
                "id": f"occ-unique-{name}",
                "name": name,
                "total": 1,
                "key": name,
                "indexOptional": True,
                "ambiguousIfBare": False,
            }
        )
    # synthetic addressing table 0..29 for docs (not a live document)
    for n in range(16):
        cases.append(
            {
                "id": f"occ-addr-{n:02d}",
                "name": "성명",
                "total": 16,
                "key": f"성명[{n}]",
                "synthetic": True,
                "rule": "순번은 fields --json 목록 순서, 0 기준",
            }
        )
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "zeroBased": True,
        "bareKeyMeansFirstMatch": True,
        "outOfRangeGoesToNotFound": True,
        "cases": cases,
    }


INTENTS = []


def intent_matrix() -> dict:
    rows = []

    def add(utter, cmd, ref, stop, notes=""):
        rows.append(
            {
                "id": f"I{len(rows) + 1:03d}",
                "utterance": utter,
                "command": cmd,
                "reference": ref,
                "stop": stop,
                "notes": notes,
            }
        )

    surveys = [
        "이 서식에 뭘 채워야 해?",
        "누름틀 목록 보여줘",
        "필드 이름이 뭐야?",
        "신청서 칸이 몇 개야?",
        "guide 읽어줘",
        "memo 에 뭐라고 쓰여 있어?",
        "이 양식 작성 항목 정리해줘",
        "fieldCount 확인해",
        "누름틀 없는 서식이야?",
        "표 칸 서식인지 누름틀인지 구분해",
        "한글 서식 조사해",
        "hwpx 서식 필드 뽑아",
        "같은 이름이 반복돼?",
        "피규제집단명이 몇 번 나와?",
        "성명이 몇 개야?",
        "textSecurity 봐줘",
        "이 필드 위치가 표 안이야?",
        "로고 셀인지 확인해",
        "작성 지시문만 뽑아",
        "비어 있는 누름틀만 목록",
    ]
    for u in surveys:
        add(u, "fields --json", "01_fields_survey.md", "F04")

    singles = [
        "값 채워줘",
        "신청서 작성해",
        "이 칸에 홍길동 넣어",
        "성명에 값 넣어줘",
        "한 명분만 만들어",
        "row.json 으로 채워",
        "데이터 파일로 채워",
        "회사명만 바꿔",
        "이메일 칸 채워",
        "제목 넣어",
        "작성자 칸 기록",
        "부서명 채워",
        "전화번호 넣어",
        "myMsg01 에 문구 넣어",
        "제출용으로 한 부 작성",
        "서식 한 장 완성해",
        "빈 칸 채워 저장해",
        "출력은 output/ 로",
        "원본은 건드리지 말고 채워",
        "hwpx 그대로 채워",
    ]
    for u in singles:
        add(u, "edit fill-fields --data -o --json", "02_fill_fields.md", "F07")

    repeats = [
        "같은 이름 필드 두 번째만",
        "성명 세 번째 칸",
        "이름[2] 로 채워",
        "반복 필드 전부 다르게",
        "피규제집단명 14개 다 채워",
        "목차1 다섯 칸 각각",
        "첫 매치만 채우지 마",
        "ambiguous 나왔어",
        "14개 중 1개만 채워졌어",
        "순번 지목해서 다시",
        "0번째 성명",
        "마지막 반복 칸",
        "범위 밖 순번이면?",
        "음수 인덱스 돼?",
        "같은 날짜 칸이 여러 개",
        "행마다 다른 담당자",
        "평가표 항목별 점수 칸",
        "별지 반복 블록",
        "N번째만 비워 둬",
        "나머지는 기존 값 유지",
    ]
    for u in repeats:
        add(
            u,
            'edit fill-fields --data \'{"이름[N]":"값"}\'',
            "03_repeat_occurrence.md",
            "F05",
        )

    batches = [
        "명단으로 30명분",
        "메일머지 해줘",
        "참석자마다 한 파일",
        "CSV 로 일괄 작성",
        "JSONL 로 돌려",
        "이름 필드가 파일명",
        "0001.hwp 식으로",
        "같은 서식 N부",
        "안내문 100통",
        "계약 상대방 목록",
        "수료증 일괄",
        "상장 일괄",
        "신청서 일괄 접수",
        "병렬로 돌려",
        "--threads 4",
        "실패 행만 보여줘",
        "한 줄씩 NDJSON",
        "stdin 으로 파일 목록?",
        "헤더만 있는 CSV 야",
        "동명이인 파일명",
    ]
    for u in batches:
        add(
            u,
            "batch fill --form --data --out-dir --json",
            "04_batch_fill.md",
            "F10",
        )

    dry = [
        "채우기 전에 미리보기",
        "드라이런",
        "파일 만들지 말고 확인",
        "오타 있는지 먼저",
        "notFound 만 보고 싶어",
        "실행 전에 판정",
        "같은 명령에서 dry-run 만 빼면 실행되게",
        "batch 도 미리보기",
        "out-dir 없이 dry-run?",
        "dry-run 인데 output 키가 있어?",
    ]
    for u in dry:
        add(u, "같은 인자 + --dry-run --json", "05_dry_run_verify.md", "F06")

    ver = [
        "제대로 들어갔는지 검증",
        "verify 붙여",
        "재파싱 대조",
        "identical 확인해",
        "exit 3 나면?",
        "저장 후 다시 읽어",
        "값이 화면에 있어?",
        "batch 도 verify",
        "verify 없이 저장만",
        "diffCount 봐",
    ]
    for u in ver:
        add(u, "같은 인자 + --verify --json", "05_dry_run_verify.md", "F07")

    san = [
        "제출 전 메타 지워",
        "작성자 흔적 삭제",
        "sanitize 해",
        "미리보기는 남겨",
        "배포본 만들어",
        "두 번 돌리면?",
        "본문은 건드리지 마",
        "제목 속성 비워",
        "최종수정자 지워",
        "제출용으로 정리",
    ]
    for u in san:
        add(u, "edit sanitize -o --json", "06_sanitize.md", "F08")

    images = [
        "도장 찍어",
        "직인 붙여",
        "서명 이미지",
        "100mm 지점에 도장",
        "overflow 나면?",
        "첫 쪽에 도장",
        "HWPUNIT 로",
        "페이지 번호 1이야 0이야",
        "png 직인",
        "jpg 서명",
    ]
    for u in images:
        add(u, "edit insert-image --page --x --y", "14_insert_image.md", "F08")

    hand = [
        "표 칸에 써줘",
        "누름틀이 없어",
        "set-cell 로",
        "이 문서 보내도 돼?",
        "숨은 글 있어?",
        "주입 신호",
        "폴더 전체 스윕",
        "이 hwp 뭔 문서야",
        "원본을 여러 번 고쳐",
        "계획서로 편집",
    ]
    targets = [
        ("rhwp-table-exchange", "F02"),
        ("rhwp-table-exchange", "F02"),
        ("rhwp-table-exchange", "F02"),
        ("rhwp-security-sweep", "F03"),
        ("rhwp-security-sweep", "F03"),
        ("rhwp-security-sweep", "F03"),
        ("rhwp-bulk-pipeline", "F13"),
        ("rhwp-doc-triage", "F14"),
        ("rhwp-safe-edit", "F14"),
        ("rhwp-safe-edit", "F14"),
    ]
    for u, (cmd, stop) in zip(hand, targets):
        add(u, f"handoff:{cmd}", "10_handoff.md", stop)

    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "count": len(rows),
        "intents": rows,
    }


JOURNEY_SPECS = [
    ("J01", "단건 조사만", ["fields"], "F04", "form-01"),
    ("J02", "단건 채움+검증", ["fields", "dry-run", "fill-fields", "verify"], "F07", "form-01"),
    ("J03", "단건 제출", ["fields", "fill-fields", "verify", "sanitize"], "F08", "form-01"),
    ("J04", "직인 포함 제출", ["fields", "fill-fields", "insert-image", "sanitize"], "F08", "form-01"),
    ("J05", "반복 필드 재지목", ["fields", "fill-fields", "이름[N]"], "F05", "field-01"),
    ("J06", "메일머지 3행", ["fields", "batch-fill"], "F10", "form-01"),
    ("J07", "메일머지 dry-run", ["fields", "batch-fill --dry-run"], "F10", "form-01"),
    ("J08", "메일머지 verify", ["fields", "batch-fill --verify"], "F07", "form-01"),
    ("J09", "name-field 오탐 제외", ["batch-fill --name-field 성명"], "F11", "field-01"),
    ("J10", "fieldCount 0 인계", ["fields"], "F02", "hwp3"),
    ("J11", "보안 인계", ["fields"], "F03", "untrusted"),
    ("J12", "오타 notFound", ["dry-run"], "F06", "form-01"),
    ("J13", "빈 CSV 거부", ["batch-fill"], "F09", "form-01"),
    ("J14", "CP949 실패", ["fill-fields"], "F01", "form-01"),
    ("J15", "HWPX 형식 보존", ["fill-fields"], "F07", "hwpx"),
    ("J16", "규제영향 14칸", ["fields", "이름[N]"], "F05", "reg"),
    ("J17", "로고 셀 건너뜀", ["fields", "export-tables"], "F14", "hongbo"),
    ("J18", "overflow 직인", ["insert-image"], "F08", "form-01"),
    ("J19", "sanitize 멱등", ["sanitize", "sanitize"], "F08", "filled"),
    ("J20", "폴더 수백 인계", ["batch fields"], "F13", "folder"),
]


def journeys() -> dict:
    extra_titles = [
        "신청서 한 장",
        "위임장",
        "출장 명령",
        "휴가원",
        "지출 결의",
        "품의서",
        "공문 발송",
        "보도자료 서식",
        "수료증 30부",
        "상장 12부",
        "명찰 명단",
        "참석 확인서",
        "개인정보 동의",
        "근로 계약 상대",
        "거래처 등록",
        "세금계산 요청",
        "사업계획 표지",
        "규제영향 표지",
        "평가표 채점자",
        "면접 평정",
        "출근부 월별",
        "회의록 서식",
        "회의 참석자",
        "교육 신청",
        "시설 사용 신청",
        "민원 신청",
        "정보공개 청구",
        "보조금 신청",
        "연구노트 표지",
        "실험 일지",
        "점검표",
        "안전 서약",
        "보안 서약",
        "비밀유지",
        "위촉장",
        "임명장",
        "사직서",
        "휴직 신청",
        "복직 신청",
        "주소 변경",
        "가족관계 신고",
        "재직 증명 신청",
        "경력 증명 신청",
        "원천징수 신청",
        "연말정산 추가",
        "출입증 신청",
        "차량 등록",
        "방문 신청",
        "반출 신청",
        "폐기 신청",
        "구매 요청",
        "검수 조서",
        "인수인계",
        "업무 분장",
        "비상 연락",
        "당직 일지",
        "근태 정정",
        "초과근무",
        "대체휴무",
        "교육비 청구",
        "여비 정산",
        "숙박 정산",
        "회의비 정산",
        "물품 청구",
        "비품 이동",
        "전산 장애",
        "계정 신청",
        "VPN 신청",
        "메일 신청",
        "홈페이지 게시",
        "보도 해명",
        "질의 답변",
        "국회 자료",
        "감사 자료",
        "정보공개 답변",
        "민원 회신",
        "훈령 개정 의견",
        "법령 검토",
        "계약 검토",
        "직인 날인 신청",
    ][:65]
    items = []
    for spec in JOURNEY_SPECS:
        jid, title, steps, stop, sample = spec
        items.append(
            {
                "id": jid,
                "title": title,
                "steps": steps,
                "stop": stop,
                "sample": sample,
                "notGym": True,
            }
        )
    for i, title in enumerate(extra_titles, start=21):
        kind = i % 5
        if kind == 0:
            steps, stop = ["fields"], "F04"
        elif kind == 1:
            steps, stop = ["fields", "dry-run", "fill-fields", "verify"], "F07"
        elif kind == 2:
            steps, stop = ["fields", "batch-fill"], "F10"
        elif kind == 3:
            steps, stop = ["fields", "fill-fields", "sanitize"], "F08"
        else:
            steps, stop = ["fields", "이름[N]"], "F05"
        items.append(
            {
                "id": f"J{i:02d}",
                "title": title,
                "steps": steps,
                "stop": stop,
                "sample": "live-form",
                "notGym": True,
            }
        )
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "count": len(items),
        "journeys": items,
    }


def dry_run_verify() -> dict:
    cases = []
    # dry-run matrix
    for cmd in ["fill-fields", "batch-fill", "insert-image", "sanitize"]:
        cases.append(
            {
                "id": f"dry-{cmd}",
                "command": cmd,
                "flag": "--dry-run",
                "writesFile": False,
                "hasOutputKey": False,
                "outDirStillRequired": cmd == "batch-fill",
                "sameArgvMinusFlag": True,
            }
        )
    for cmd in ["fill-fields", "batch-fill", "insert-image"]:
        cases.append(
            {
                "id": f"verify-{cmd}",
                "command": cmd,
                "flag": "--verify",
                "writesFile": True,
                "exitIfNotIdentical": 3,
                "envelope": {"verify": {"identical": True, "diffCount": 0}},
                "outputRemainsOnFail": True,
            }
        )
    cases.append(
        {
            "id": "verify-absent",
            "command": "fill-fields",
            "flag": None,
            "verifyField": None,
            "exit": 0,
            "note": "플래그 없으면 verify 는 null 이고 exit 0",
        }
    )
    # decision table: 20 combinations
    for i, (dry, ver, expect_write, expect_exit_if_diff) in enumerate(
        [
            (True, False, False, None),
            (True, True, False, None),
            (False, False, True, None),
            (False, True, True, 3),
        ]
        * 8
    ):
        cases.append(
            {
                "id": f"combo-{i:02d}",
                "dryRun": dry,
                "verify": ver,
                "writes": expect_write,
                "exitIfDiff": expect_exit_if_diff,
            }
        )
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "rule": "선검증은 실행과 같은 명령줄에서 --dry-run 하나만 뺀다",
        "cases": cases,
    }


def sanitize_cases() -> dict:
    fields = [
        "title",
        "subject",
        "author",
        "keywords",
        "comments",
        "lastSavedBy",
        "revisionNumber",
        "dateString",
        "createdAt",
        "lastSavedAt",
        "lastPrintedAt",
        "previewText",
        "previewImage",
    ]
    cases = []
    for f in fields:
        cases.append(
            {
                "id": f"san-{f}",
                "removedField": f,
                "bodyUntouched": True,
                "exportTextIdentical": True,
            }
        )
    cases.append(
        {
            "id": "san-second-run",
            "removedCount": 0,
            "idempotent": True,
            "proves": "첫 실행이 실제로 지웠다",
        }
    )
    cases.append(
        {
            "id": "san-keep-preview",
            "flag": "--keep-preview",
            "keeps": "preview image",
            "stillRemoves": "preview text always",
        }
    )
    cases.append(
        {
            "id": "san-hwpx-meta",
            "target": "Contents/content.hpf opf:metadata",
            "note": "직렬화기가 splice 하는 저작자 경로를 중립 블록으로",
        }
    )
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "bodyUntouched": True,
        "cases": cases,
        "removedFields": fields,
    }


def failure_signals() -> dict:
    rows = [
        ("fieldCount: 0", "누름틀 없음", "rhwp-table-exchange", "F02", 0),
        ("textSecurity != clean", "은닉/주입", "레시피 04", "F03", 0),
        ("notFound 잔류", "오타·없는 이름", "fields name 복사", "F06", 0),
        ("name-field only notFound", "파일명 컬럼", "게이트 제외", "F11", 0),
        ("ambiguous 비어 있지 않음", "반복 이름", "이름[N]", "F05", 0),
        ("verify.identical false", "재파싱 차이", "render-diff", "F12", 3),
        ("데이터 행 없음", "헤더만", "상류 확인", "F09", 2),
        ("invalid JSON", "깨진 --data", "UTF-8 JSON", "F01", 2),
        ("missing file", "경로 오타", "절대 경로", "F01", 1),
        ("overflow 비어 있지 않음", "그림이 쪽 밖", "좌표/크기 조정", "F08", 0),
        ("removedCount 0", "이미 정리", "멱등", "F08", 0),
        ("stdin 무반응", "batch fill 축 혼동", "--form --data", "F10", 0),
        ("CP949", "UTF-8 아님", "재저장", "F01", 1),
        ("unknown subcommand", "발명한 명령", "cli_commands 만", "F14", 2),
        ("header/footer 누락", "사각지대", "재귀 확장 금지", "F14", 0),
    ]
    items = []
    for sig, cause, rx, stop, ex in rows:
        items.append(
            {
                "signal": sig,
                "cause": cause,
                "prescription": rx,
                "stop": stop,
                "exit": ex,
            }
        )
    return {"schemaVersion": SCHEMA, "issue": ISSUE, "signals": items}


def traces() -> list[dict]:
    out = []
    out.append(
        {
            "id": "T01",
            "title": "form-01 조사",
            "sample": "samples/form-01.hwp",
            "steps": [
                {
                    "argv": ["fields", "samples/form-01.hwp", "--json"],
                    "expect": {"fieldCount": 1, "fields.0.name": "myMsg01"},
                }
            ],
            "stop": "F04",
        }
    )
    out.append(
        {
            "id": "T02",
            "title": "form-01 dry-run 오타",
            "sample": "samples/form-01.hwp",
            "steps": [
                {
                    "argv": [
                        "edit",
                        "fill-fields",
                        "samples/form-01.hwp",
                        "--data",
                        '{"noSuchField":"x"}',
                        "--dry-run",
                        "--json",
                    ],
                    "expect": {"dryRun": True, "notFound": ["noSuchField"], "filledCount": 0},
                    "writes": False,
                }
            ],
            "stop": "F06",
            "measuredIn": "recipes/01",
        }
    )
    out.append(
        {
            "id": "T03",
            "title": "form-01 채움 실측",
            "sample": "samples/form-01.hwp",
            "steps": [
                {
                    "argv": [
                        "edit",
                        "fill-fields",
                        "samples/form-01.hwp",
                        "--data",
                        '{"myMsg01":"홍길동 귀하"}',
                        "-o",
                        "form-01_filled.hwp",
                        "--json",
                    ],
                    "expect": {
                        "filledCount": 1,
                        "notFound": [],
                        "ambiguous": [],
                        "outputFormat": "hwp5",
                    },
                }
            ],
            "stop": "F07",
            "measuredIn": "recipes/01",
        }
    )
    out.append(
        {
            "id": "T04",
            "title": "form-01 verify",
            "sample": "samples/form-01.hwp",
            "steps": [
                {
                    "argv": [
                        "edit",
                        "fill-fields",
                        "samples/form-01.hwp",
                        "--data",
                        '{"myMsg01":"홍길동 귀하"}',
                        "-o",
                        "form-01_verify.hwp",
                        "--verify",
                        "--json",
                    ],
                    "expect": {"verify.identical": True, "verify.diffCount": 0},
                }
            ],
            "stop": "F07",
        }
    )
    out.append(
        {
            "id": "T05",
            "title": "batch 2행 실측",
            "sample": "samples/form-01.hwp",
            "steps": [
                {
                    "argv": [
                        "batch",
                        "fill",
                        "--form",
                        "samples/form-01.hwp",
                        "--data",
                        "row1.jsonl",
                        "--out-dir",
                        "batch_out",
                        "--json",
                    ],
                    "ndjson": True,
                    "rows": 2,
                    "expectEach": {"filledCount": 1, "notFound": [], "ambiguous": []},
                }
            ],
            "stop": "F10",
            "measuredIn": "recipes/05",
        }
    )
    out.append(
        {
            "id": "T06",
            "title": "field-01 회사명 dry-run",
            "sample": "samples/field-01.hwp",
            "steps": [
                {
                    "argv": [
                        "edit",
                        "fill-fields",
                        "samples/field-01.hwp",
                        "--data",
                        '{"회사명":"주식회사 A"}',
                        "-o",
                        "out.hwp",
                        "--dry-run",
                        "--json",
                    ],
                    "expect": {"dryRun": True, "filledCount": 1},
                    "writes": False,
                    "contract": "tests/edit_fill_fields_contract.rs",
                }
            ],
            "stop": "F06",
        }
    )
    out.append(
        {
            "id": "T07",
            "title": "없는 필드 보고",
            "sample": "samples/field-01.hwp",
            "steps": [
                {
                    "argv": [
                        "edit",
                        "fill-fields",
                        "samples/field-01.hwp",
                        "--data",
                        '{"회사명":"A","존재하지않는필드":"B"}',
                        "--dry-run",
                        "--json",
                    ],
                    "expect": {"notFound": ["존재하지않는필드"], "filledCount": 1},
                }
            ],
            "stop": "F06",
        }
    )
    out.append(
        {
            "id": "T08",
            "title": "목차1 순번 지목",
            "sample": "samples/field-01.hwp",
            "steps": [
                {
                    "argv": [
                        "edit",
                        "fill-fields",
                        "samples/field-01.hwp",
                        "--data",
                        '{"목차1[0]":"가","목차1[1]":"나","목차1[2]":"다","목차1[3]":"라","목차1[4]":"마"}',
                        "-o",
                        "out.hwp",
                        "--json",
                    ],
                    "expect": {"ambiguous": [], "filledCount": 5},
                }
            ],
            "stop": "F05",
        }
    )
    out.append(
        {
            "id": "T09",
            "title": "fieldCount 0 은 성공 봉투",
            "sample": "samples/hwp3-sample.hwp",
            "steps": [
                {
                    "argv": ["fields", "samples/hwp3-sample.hwp", "--json"],
                    "expect": {"fieldCount": 0},
                    "exit": 0,
                }
            ],
            "stop": "F02",
        }
    )
    out.append(
        {
            "id": "T10",
            "title": "sanitize 멱등",
            "sample": "filled.hwp",
            "steps": [
                {
                    "argv": ["edit", "sanitize", "filled.hwp", "-o", "배포본.hwp", "--json"],
                    "expect": {"removedCount>=": 0},
                },
                {
                    "argv": ["edit", "sanitize", "배포본.hwp", "-o", "재확인.hwp", "--json"],
                    "expect": {"removedCount": 0},
                },
            ],
            "stop": "F08",
        }
    )
    # pad traces 11-40 as documented agent paths (not invented commands)
    titles = [
        "HWPX 입력은 hwpx 산출",
        "기본 출력명 _filled",
        "batch 순번 파일명 4자리",
        "name-field 금지문자 치환",
        "name-field 동명 _2",
        "threads 와 입력 순서 보존",
        "실패 행도 NDJSON 잔류",
        "서식 못 열면 시작 전 1회",
        "insert-image overflow 보고",
        "insert-image --page 0",
        "A4 좌표 환산 100mm",
        "verify 없는 단건 verify=null",
        "잘못된 JSON exit 2",
        "없는 파일 exit 1 stdout 빈",
        "batch stdin 무시",
        "헤더만 CSV exit 2",
        "fields 기본 출력은 비JSON",
        "memo 있는 표본",
        "nested location 배열",
        "로고 셀 판별 후 스킵",
        "보안 인계 전 채우지 않음",
        "표 칸 인계",
        "폴더 선별은 batch fields",
        "재독 fields 로 값 대조",
        "jq 게이트 단건",
        "jq 게이트 batch",
        "원본 불변 dry-run",
        "원본 불변 실패",
        "keep-preview",
        "본문 export-text 동일",
    ]
    for i, title in enumerate(titles, start=11):
        out.append(
            {
                "id": f"T{i:02d}",
                "title": title,
                "steps": [
                    {
                        "note": "기존 CLI 표면만. 새 플래그 없음",
                        "usesExistingCommand": True,
                    }
                ],
                "notGym": True,
            }
        )
    return out


def tree() -> dict:
    return {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "notGym": True,
        "noNewCli": True,
        "noNewEditLogic": True,
        "ladder": [
            "fields",
            "dry-run",
            "fill-fields",
            "batch-fill",
            "verify",
            "insert-image",
            "sanitize",
        ],
        "coreReuse": [
            "set_field_value_by_name",
            "collect_all_fields",
            "batch fill row loop calling fill-fields",
        ],
    }


def write_data_files() -> None:
    data = FIXT / "data"
    data.mkdir(parents=True, exist_ok=True)
    rows = [person(i) for i in range(12)]
    jsonl = "\n".join(json.dumps(r, ensure_ascii=False) for r in rows) + "\n"
    (data / "mailmerge_12.jsonl").write_text(jsonl, encoding="utf-8", newline="\n")
    headers = list(rows[0].keys())
    csv = [",".join(headers)]
    for r in rows:
        csv.append(",".join(r[h] for h in headers))
    (data / "mailmerge_12.csv").write_text(
        "\n".join(csv) + "\n", encoding="utf-8", newline="\n"
    )
    (data / "empty_header_only.csv").write_text(
        "성명,myMsg01\n", encoding="utf-8", newline="\n"
    )
    (data / "row_form01.json").write_text(
        json.dumps({"myMsg01": "홍길동 귀하"}, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
        newline="\n",
    )
    (data / "row_field01.json").write_text(
        json.dumps(
            {
                "회사명": "주식회사 검증",
                "작성자": "홍길동",
                "부서명": "기획과",
                "전화번호": "02-1234-5678",
                "이메일": "hong@example.go.kr",
                "제목": "서식 채움 검증",
                "목차1[0]": "배경",
                "목차1[1]": "목적",
                "목차1[2]": "범위",
                "목차1[3]": "일정",
                "목차1[4]": "예산",
            },
            ensure_ascii=False,
            indent=2,
        )
        + "\n",
        encoding="utf-8",
        newline="\n",
    )
    # 이름[N] 연습 키만. 값은 예시.
    occ = {f"성명[{i}]": person(i)["성명"] for i in range(14)}
    (data / "row_repeat_14.json").write_text(
        json.dumps(occ, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
        newline="\n",
    )


def write_all() -> None:
    FIXT.mkdir(parents=True, exist_ok=True)
    artifacts = {
        "skill_index.json": skill_index(),
        "command_ladder.json": command_ladder(),
        "envelope_keys.json": envelope_keys(),
        "stop_rules.json": stop_rules(),
        "pitfalls.json": pitfalls(),
        "handoff.json": handoff(),
        "samples.json": samples(),
        "field_catalog.json": field_catalog(),
        "batch_rows.json": batch_rows(),
        "occurrence_catalog.json": occurrence_catalog(),
        "intent_matrix.json": intent_matrix(),
        "journeys.json": journeys(),
        "dry_run_verify.json": dry_run_verify(),
        "sanitize_cases.json": sanitize_cases(),
        "failure_signals.json": failure_signals(),
        "tree.json": tree(),
    }
    traces_list = traces()
    artifacts["traces_index.json"] = {
        "schemaVersion": SCHEMA,
        "issue": ISSUE,
        "count": len(traces_list),
        "ids": [t["id"] for t in traces_list],
    }
    for name, obj in artifacts.items():
        dump(FIXT / name, obj)
    for t in traces_list:
        dump(FIXT / "traces" / f"{t['id']}.json", t)
    write_data_files()


if __name__ == "__main__":
    write_all()
    print(f"wrote fixtures under {FIXT}")
