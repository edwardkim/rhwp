---
kind: canonical
status: active
canonical: mydocs/manual/agent_knowledge_map.md
last_verified: 2026-08-02
---

# 에이전트 지식 지도 — rhwp 참조 문서의 단일 진입점

rhwp 를 도구로 부리는 AI 에이전트·스크립트가 **첫 번째로 읽는 문서**다. 루트
[`llms.txt`](../../llms.txt)가 이 문서를 가리키고, 이 문서가 나머지 전부를 가리킨다.

**단일 출처 원칙**: 이 지도는 요약과 앵커만 담는다. 상세 서술·수치·절차의 권위는
각 canonical 문서([CLI 명령어 매뉴얼](cli_commands.md) 등)에 있으며, 이 지도와
다르면 그쪽을 따른다. 새 표면이 머지되면 해당 행만 추가한다(기존 행 재서술 금지).

## 1. 3문 진입 — 세 가지 질문으로 필요한 문서에 도착한다

### 1-1. 무엇을 하려는가 — 작업별 도구·명령 결정 표

무상태 MCP 도구는 CLI 계약의 얇은 껍데기다(선언 = `capabilities --mcp`, 실행 =
`mcp-serve`). 아래 표의 CLI 절이 곧 도구 문서다.

| 하려는 일 | 명령 (MCP 도구) | 판정 필드 | 권위 |
|---|---|---|---|
| 조사 — 규모·형식 파악 | `info --json` (`hwp_info`) | `format`·`pageCount` | [CLI 매뉴얼](cli_commands.md) §info |
| 검색 — "어느 쪽에 있나" | `search --json` (`hwp_search`) | `matchCount`·`matches[].page` | [CLI 매뉴얼](cli_commands.md) §search, [예제집 3](agent_task_playbook.md) |
| 추출 — 본문·표·구조 | `export-text`/`export-tables`/`export-structure --json` (`hwp_export_*`) | `pages[]`·`tables[]`·`structure` | [CLI 매뉴얼](cli_commands.md) §1 |
| 렌더 — 시각 확인(VLM) | `export-svg`/`export-pdf`/`export-markdown --json`·`export-png` | 매니페스트 `pages[].path` | [CLI 매뉴얼](cli_commands.md) §1 |
| 변환·검증 — 무손실 게이트 | `export-hwpx --verify --json`·`ir-diff --json` (`hwp_convert_hwpx`·`hwp_ir_diff`) | `verify{identical,diffCount}`·`categories` | [CLI 매뉴얼](cli_commands.md) §3, [예제집 6](agent_task_playbook.md) |
| 서식 채움 — 누름틀 | `fields` → `edit fill-fields` (`hwp_fields`·`hwp_fill_fields`) | `notFound`·`ambiguous`·`filledCount` | [서식 가이드 §1](form_filling_guide.md#1-fill-fields-심화) |
| 표 기록 — 격자 좌표 | `export-tables` → `edit set-cell` (`hwp_set_cell`) | `overflow`·`oldText`/`newText` | [서식 가이드 §2](form_filling_guide.md#2-set-cell-심화) |
| 치환 — 문구 일괄 교체 | `edit replace-text` → `search` 재독 (`hwp_replace_text`) | `replacedCount` | [서식 가이드 §3](form_filling_guide.md#3-replace-text-심화) |
| 생성 — JSON 명세 → HWPX | `build-from-ingest` (`hwp_build_from_ingest`) | 재독 대조(`export-text`) | [CLI 매뉴얼](cli_commands.md) §build-from-ingest, [예제집 5](agent_task_playbook.md) |
| 대량 — 아카이브 스윕 | `batch` 7축 NDJSON — 읽기 6축은 `hwp_batch`·`hwp_batch_search`, 쓰기 `convert`는 CLI 전용 | 레코드별 `error`·`exitClass` | [JSON 파이프라인 가이드](cli_json_pipeline_guide.md) |
| 세션 — 재파싱 없는 반복 | `mcp-serve` 전용: `hwp_open` → `hwp_doc_*` → `hwp_doc_save`/`hwp_close` | `docId` 핸들 | [MCP 가이드 §세션](mcp_integration_guide.md#세션-도구--재파싱-없는-반복-조회-서버-전용) |

온보딩은 명령 추측이 아니라 자기서술로 한다: `rhwp capabilities` 1회 호출로 전
명령·플래그·가용성(`available`)을 캐시한다
([시나리오 0](cli_json_pipeline_guide.md#시나리오-0--에이전트-온보딩-도구-발견)).

### 1-2. 실패했는가 — 증상별 실패 사전 앵커

대원칙([#2707](cli_commands.md#종료-코드-2707)): exit 2 = 호출 조립 버그(재시도 금지,
인자 수정) / exit 1 = 환경·입력 문제 / exit 3·4 = 오류가 아니라 검증 판정.

| 증상 | 실패 사전 앵커 |
|---|---|
| `stream did not contain valid UTF-8`, 한글 파일명 깨짐 | [입력·인코딩](agent_troubleshooting_guide.md#입력인코딩) |
| `알 수 없는 옵션`, 페이지 범위 초과, positional 중복 (exit 2) | [사용법](agent_troubleshooting_guide.md#사용법-exit-2-계열) |
| `filledCount` 성공인데 서식이 덜 채워짐, set-cell 병합 실패, 치환 후 출력 파일 없음 | [편집 응답의 오독](agent_troubleshooting_guide.md#편집-응답의-오독) |
| `--verify` 가 exit 3 — 변환 실패인가? | [검증 판정](agent_troubleshooting_guide.md#검증-판정-exit-34) |
| export-png 기능 부재, 보호 문서, 렌더 글꼴 불일치 | [환경·빌드](agent_troubleshooting_guide.md#환경빌드) |
| batch exit 1 인데 결과는 나옴, `--json` 파싱 실패 | [배치·파이프라인](agent_troubleshooting_guide.md#배치파이프라인) |
| 그 밖의 모든 것 | [그래도 안 풀리면](agent_troubleshooting_guide.md#그래도-안-풀리면) |

### 1-3. 추가하려는가 — 표면 플레이북

새 CLI `--json`·MCP 도구는 [에이전트 표면 플레이북](agent_surface_playbook.md)의
절차(이슈 → red 계약 테스트 → 구현 → 증적 2종 → PR)를 따른다. 잔여 목록·우선순위의
권위는 [#3608](https://github.com/edwardkim/rhwp/issues/3608)이다. 절차를 어긴 표면
추가는 되돌린다.

### 1-3-1. 끝에서 끝까지 예제를 따라 하고 싶은가 — 레시피

표 1-1이 명령 하나하나의 판정 필드를 알려준다면, 아래 레시피는 "처음부터 끝까지
한 번에 실행 가능한 순서"를 실측 출력과 함께 보여준다. 각 레시피는 독립 실행
가능하고, 서로 필요한 지점에서만 상호 참조한다.

| 레시피 | 다루는 것 | 핵심 명령 |
|---|---|---|
| [1 — 서식 채워서 제출용으로 만들기](recipes/01_fill_form_and_submit.md) | 누름틀 채움 → 도장 삽입 → 메타데이터 제거 | `fields`·`edit fill-fields`·`edit insert-image`·`edit sanitize` |
| [2 — 표 데이터를 CSV로 뽑아 고치고 되돌리기](recipes/02_table_csv_roundtrip.md) | 표 좌표 기반 서식의 CSV 왕복 | `export-tables`·`edit set-cell` |
| [4 — 출처를 모르는 문서를 처음 열 때](recipes/04_safety_check_untrusted_doc.md) | 본문 전체를 노출하지 않고 점진적으로 신뢰도 판정 | `info`·`digest`·`fields`(`textSecurity`)·`search`·`batch` |
| [5 — 서식 하나에 여러 사람 데이터를 한 번에 채우기](recipes/05_mail_merge_batch_fill.md) | 메일머지형 대량 서식 채움 | `batch fill` |
| [6 — 편집 전후를 눈이 아니라 숫자로 비교하기](recipes/06_visual_regression_before_after.md) | 편집이 렌더링 레이아웃에 준 영향을 정량 판정 | `render-diff` |

### 1-4. 다른 언어에서 쓰려는가 — 바인딩 가이드

바인딩은 **새 표면이 아니라 기존 계약의 재포장**이다. 판정·좌표·파싱은 전부 rhwp
본체가 하고, 바인딩은 인자 조립·봉투 파싱·종료 코드 매핑 셋만 한다. 그래서 §1-1 의
명령 표와 §2 의 봉투 필드 사전이 언어를 바꿔도 그대로 권위다.

| 언어 | 패키지 | 가이드 | 상태 |
|---|---|---|---|
| Node/TypeScript | `@rhwp/node` | [node_binding_guide.md](node_binding_guide.md) | M19 [#3776](https://github.com/edwardkim/rhwp/issues/3776) — 통합 검토 중 |
| Python | `rhwp` | [python_binding_guide.md](python_binding_guide.md) | M18 [#3762](https://github.com/edwardkim/rhwp/issues/3762) — 통합 검토 중 |

노출 기준은 손으로 고른 목록이 아니라 `capabilities` 의 `json` 선언이다 — 진단
계열처럼 `--json` 이 없는 명령은 바인딩에 함수로 없고, 필요하면 저수준 실행기로
직접 부른다. IR 모양은 `rhwp export-ir-schema`, 명령 표면은
`rhwp export-capabilities-schema`를 코드 생성의 단일 출처로 쓴다. 두 가이드가 서로
어긋나면 계약이 언어마다 갈린 것이므로 어긋난 쪽을 고친다.

## 2. 봉투 필드 사전 — 필드 이름으로 찾는 역인덱스

모든 `--json` 봉투 공통: stdout 은 순수 JSON 하나(배치는 NDJSON), 스키마는 필드
**추가만** 허용(변경·삭제는 `tests/cli_json_contract.rs` 가 잡는다).

| 필드 | 한 줄 정의 | 등장 명령 |
|---|---|---|
| `schemaVersion` | 봉투 스키마 버전 — 파싱 호환성의 기준 | 모든 `--json` 봉투 |
| `source` | 입력 파일 경로(레코드의 신원) | 모든 `--json` 봉투·batch 레코드 |
| `output` / `outputFormat` | 실제 저장된 산출물 경로/형식 — **저장했을 때만** 실린다(dry-run·치환 0건이면 없음) | `edit` 3종, `export-pdf`, `export-hwpx` |
| `bytes` | 산출물 크기(바이트) | `export-pdf`·`export-hwpx`·매니페스트 `pages[].bytes` |
| `verify{identical,diffCount}` | 변환 후 재파싱 IR 대조 결과 — `--verify` 를 준 경우에만 객체 | `export-hwpx --verify --json` |
| `identical` / `diffCount` / `categories` | IR 비교 판정 — 차이는 오류가 아니라 데이터(exit 3) | `ir-diff --json` |
| `notFound` | 문서에 없는 필드 이름(오타·범위 밖 순번) — 조용히 무시되지 않는다 | `edit fill-fields` |
| `ambiguous` | 순번 없는 이름이 여러 곳에 해당 — `{name,matched,total}`. 비어 있지 않으면 아직 끝난 게 아니다 | `edit fill-fields` |
| `overflow` | 넣은 값이 칸 폭을 넘침(채우기는 막지 않음, dry-run 에서도 검사) | `edit set-cell` (#3480) |
| `replacedCount` | 치환 건수 — 0 이면 출력 파일을 만들지 않는다 | `edit replace-text` |
| `filledCount` / `filled[]` | 채운 필드 수와 목록 `{name,occurrence,value}` | `edit fill-fields` |
| `matches[].{section,paragraph,page,charOffset}` | 매치의 구역·문단·0 기준 페이지·문자 오프셋 — 근거 인용 주소 | `search`, `batch search` |
| `matchCount` / `totalMatchCount` / `truncated` | 반환 매치 수 / 문서 전체 매치 수 / `--limit` 절단 여부. 0건은 오류가 아니다(exit 0) | `search` |
| `error` / `exitClass` | 배치 건별 실패 레코드 — 스트림은 계속되고 최종 exit 1 | `batch` 실패 레코드 |
| `dryRun` | 파일을 쓰지 않는 사전 확인 모드 여부 | `edit` 3종 |
| `docId` | 세션 핸들 — 서버 프로세스 수명과 같고 영속되지 않는다 | `hwp_open` 등 세션 도구 |

## 3. 주소 어휘 — 좌표계는 전 명령이 공유한다

- **페이지는 0 기준**: `-p`, `search` 의 `matches[].page`, `export-text` 의
  `pages[].page` 가 전부 같은 좌표계다. 사람에게 보여줄 때만 +1 한다.
  - **예외 하나 — `extract-pages` 의 `--from`/`--to` 는 1 기준이다**(첫 쪽이 1).
    다른 축의 값을 그대로 옮기면 **오류 없이 한 쪽 밀린 문서**가 나온다.
    `search` 가 `page: 1`(2쪽)을 줬다면 잘라 낼 때는 `--from 2 --to 2` 다.
- **반복 필드는 `이름[N]`**: 같은 이름이 여러 번 나오는 서식은 0 기준 순번으로
  지목한다. 순번은 `fields --json` 목록 순서와 같다 (#3476).
- **표 격자 좌표는 `export-tables` 와 동형**: `edit set-cell` 의
  `--table`/`--row`/`--col` 은 `export-tables --json` 의 `index`/`row`/`col` 과 같은
  0 기준 격자다. 병합 셀은 앵커에 한 번만 나오고, 덮인 좌표에 쓰면 앵커를 안내하며
  실패한다(보호 동작).
- **`section`/`paragraph` 는 공통 역참조 주소**: `search`·`export-tables`·
  `export-structure`·`fields`(`location.nested` 포함)가 같은 어휘로 위치를 준다.

## 4. 판정 3층 — isError 만 보면 오독한다

| 층 | 신호 | 예 |
|---|---|---|
| JSON-RPC 오류 | `error{code,message}` | 알 수 없는 메서드(-32601), params 구조 오류(-32602) |
| 도구 실행 실패 | `isError:true` | 없는 파일, 닫힌 핸들 재사용 |
| 봉투 판정(부정적 결과는 데이터) | `isError:false` + 봉투 필드 | `identical:false`, `notFound`, 치환 0건 |

CLI 대응: exit 0 ↔ `isError:false` / exit 1·2 ↔ `isError:true`(2 는 재시도 금지) /
exit 3 ↔ `isError:false` + `identical:false`. 상세는
[MCP 가이드 — 오류 의미론](mcp_integration_guide.md#오류-의미론--세-층을-혼동하지-않기).

## 5. 표본 지도 — 어떤 검증에 어떤 샘플을 쓰나 (실측 2026-07-31)

| 표본 | 특성 | 검증 용도 |
|---|---|---|
| `samples/field-01.hwp` | 누름틀 11개(`fieldCount:11`), 한컴 정답지 | `fields`/`fill-fields` 조사·채움 루프, 누름틀 guide/command 바이트 대조 |
| `samples/hwp3-sample.hwp` | HWP3 네이티브, 표 6개, "의" 276매치 | HWP3 파싱, `search` 다건 매치, `replace-text` 대량 치환 |
| `samples/table-001.hwp` | 병합 셀 20개(rowSpan/colSpan) 단일 표 | `export-tables` 병합 보존, `set-cell` 앵커 보호 판정 |
| `samples/20250130-hongbo.hwp` | 실물 보도자료 4쪽 — 표 6·그림 4·도형 1 | 복합 문서 렌더·저장 왕복·판본 대조([실사례](../report/edit_demo_hongbo/README.md)) |
| `samples/2025 행정업무운영 편람(최종).hwpx` | 393쪽·10MB 대형 실문서 | 대형 문서 `search` 성능(215ms 실측), 세션 도구의 재파싱 회피 효과 |
| `samples/hml/` | HWPML 2.9/2.91 원본 + 한컴 뷰어 기준 PDF | HML 가져오기·loss-safe 저장·시각 정합 |

## 6. 계약 테스트 지도 — `tests/*_contract.rs` 가 고정하는 계약

| 테스트 | 고정하는 계약 |
|---|---|
| `cli_json_contract.rs` | `--json` stdout 순수성·`schemaVersion`·batch NDJSON (#3237/#3238) + 드리프트 가드 |
| `batch_axes_contract.rs` | batch 신규 축(search·export-tables·fields) 레코드 = 단건 봉투와 같은 스키마 (#3346) |
| `search_json_contract.rs` | `search` 페이지 주소 봉투 (#3283) |
| `fields_json_contract.rs` | `fields` 읽기 전용 누름틀 조사 봉투 (#3281) |
| `table_extract_json_contract.rs` | `export-tables` 병합 보존 격자 봉투 (#3278) |
| `ir_diff_json_contract.rs` | `ir-diff --json` 판정 봉투 + 종료 코드 정정 (#3274) |
| `output_axis_json_contract.rs` | 산출물 축(export-pdf·export-markdown·export-hwpx) 매니페스트 (#3596) |
| `render_manifest_json_contract.rs` | `export-svg --json` 산출물 매니페스트 (#3286) |
| `genpreview_json_contract.rs` | build-from-ingest·thumbnail 생성·미리보기 축 (#3600) |
| `edit_fill_fields_contract.rs` | fill-fields dry-run 무파일·실패 시 원본 불변 (#3329) |
| `edit_field_occurrence_contract.rs` | 반복 필드 `이름[N]` 지목·`ambiguous` 보고 (#3476) |
| `edit_replace_text_contract.rs` | replace-text 치환 0건 무산출·dry-run (#3373) |
| `edit_set_cell_contract.rs` | set-cell 격자 좌표·병합 보호 (#3381) |
| `edit_fit_check_contract.rs` | `overflow` 맞춤 검사 보고 (#3480) |
| `edit_format_preserve_contract.rs` | `edit` 3종의 입력 형식 보존 산출 (#3383) |
| `mcp_server_contract.rs` | `mcp-serve` 핸드셰이크·선언-서버 드리프트 가드·isError (#3140) |
| `mcp_session_query_contract.rs` | 세션 조회·치환(hwp_doc_search·hwp_doc_replace_text) (#3601) |
| `mcp_session_edit_contract.rs` | 세션 채움·형식 보존 저장(hwp_doc_fill_fields·hwp_doc_save) (#3598) |
| `mcp_session_changed_pages_contract.rs` | 세션 편집 3종 `changedPages` — 무상태 판 동형·재조판 후 렌더 가능 (#3719 §6-1) |
| `issue_3366_thumbnail_contract.rs` | `thumbnail` 종료 코드·파싱 계약 (#3366) |
| `issue_3372_gian_form_contract.rs` | 일반기안문 표준 서식 자산의 유효성 (#3372) |
| `render_p22_web_canvas_contract.rs` | 웹 캔버스 레이어 재생이 render node 를 재구축하지 않는 계약 |
| `render_p23_pdf_export_contract.rs` | PDF export native API 경로 계약 |
| `ir_schema_contract.rs` | `export-ir-schema` 스키마 건전성 — 끊어진 참조·고아 정의·닫힌 객체 금지 (#3762) |
| `capabilities_schema_contract.rs` | `export-capabilities-schema` — 명령 표면 자기서술의 스키마 건전성 (#3776). 바인딩 타입 생성기가 이 모양에 기대므로 여기가 계약이다 |

## 유지 규약

- 새 표면(명령·MCP 도구·계약 테스트)이 머지되면 §1-1·§2·§6 에 **행만 추가**한다.
  서술이 길어지면 이 지도가 아니라 해당 canonical 문서에 쓴다.
- 링크 검사: `py scripts/check_markdown_links.py mydocs/manual/agent_knowledge_map.md`
  ([검사 가이드](markdown_link_check_guide.md)).
- 이 문서의 이슈: #3619, 로드맵 연계: [#3608](https://github.com/edwardkim/rhwp/issues/3608) M6(온보딩)·M17(품질 인프라).
