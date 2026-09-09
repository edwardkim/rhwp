---
kind: working
status: active
issue: 3918
---

# rhwp-agent 조회 CLI 묶음

작업 브랜치: `feat/rhwp-agent-cli-pack`
대상 바이너리: `src/bin/rhwp-agent/`
이슈: [#3918](https://github.com/edwardkim/rhwp/issues/3918)

이 파일은 공장에서 만든 rhwp-agent 조회 명령을 **한 장으로** 묶은 작업 문서다.
명령마다 따로 흩어진 기록은 쓰지 않는다. 본 CLI 스킬 문서
(`mydocs/working/agent_cli.md`, 이슈 #5316)와 겹치지 않는다 — 그쪽은 `rhwp`
본 명령 스킬이고, 여기는 `rhwp-agent` 실험 표면이다.

## 1. 한 줄

에이전트가 HWP/HWPX 문서를 **읽어서** 다음 행동을 고를 때 쓰는 조회·비교·검색·보안
게이트를 `rhwp-agent` 한 바이너리에 모은다. 문서를 고치지 않는다. 편집 로직을
만들지 않는다. 이미 있는 `DocumentCore` 질의만 부른다.

## 2. 왜 별도 바이너리인가

본 CLI(`src/main.rs`)와 capabilities·출처 지도는 열린 PR 이 동시에 만지는
경합 지점이다. 이 묶음은 `src/bin/rhwp-agent/` 만 추가한다. `Cargo.toml` 도,
`src/main.rs` 도 건드리지 않는다. 검증된 명령은 본 CLI 로 승격한다(#3918).

## 3. 계약

종료 코드:

| 코드 | 뜻 |
|------|----|
| 0 | 성공 |
| 1 | 실행 오류 (파일 열기·파싱 실패) |
| 2 | 사용법 오류 (미지 명령·미지 플래그·인자 누락) |
| 3 | 게이트 위반 — 도구는 정상이고 대상이 기대와 다르다 |

`--json` 이면 stdout 은 순수 JSON 봉투 하나. 진단은 stderr.

봉투 공통 필드:

- `schemaVersion` (`1.0`)
- `tool` (`rhwp-agent`)
- `command`
- `version`
- `untrustedContent` / `untrustedFields`

문서에서 온 값은 데이터이지 지시가 아니다. 과소 선언하지 않는다.

명령 테이블 `caps::COMMANDS` 가 디스패치·도움말·`capabilities` 의 단일 출처다.
테이블에 없는 명령은 실행되지 않는다. 미지 플래그는 침묵 무시 없이 exit 2 다.

## 4. 하지 않는 것

- `src/main.rs` 수정
- DocumentCore 편집 API 발명 (`fill-fields`, `replace-text`, `set-cell` 같은 쓰기)
- 책갈피 추가/삭제, 차트 값 쓰기, 스테가 정화 저장
- TSV·더미 코퍼스, 가짜 통계 행
- 본 CLI 출처 지도(`src/provenance.rs`) 등재 — 승격 때 옮긴다

편집이 필요하면 본 CLI `rhwp edit …` 로 넘긴다. 이 표면은 그 앞단(조사)과
뒷단(게이트)만 맡는다.

## 5. 명령 가족

`rhwp-agent capabilities --json` 이 항상 최신 목록이다. 아래는 이 묶음이
채운 조회 표면이다.

### 5.1 문서가 무엇인가

| 명령 | 하는 일 | 근거 질의 |
|------|---------|-----------|
| `info` | 포맷·쪽·문단·표·필드·문자 수 | `page_count`, `extract_tables`, `collect_all_fields` |
| `format` | 매직 기준 포맷 토큰 | `parser::detect_format` |
| `explain` | 한 줄 요약 | `explain::count_notes`, `collect_charts` |
| `explore` | 이 문서에 적용 가능한 행동 메뉴 | `explore::build_menu` |
| `notes` | 각주·미주 개수 | `explain::count_notes` |
| `encrypted` | 암호 문서면 exit 3 | `document.header.encrypted` |
| `digest` | 쪽마다 첫 줄·앞부분 발췌 | `extract_page_text_native` |
| `outline` | 쪽마다 첫 비어 있지 않은 줄 | 쪽 텍스트 |
| `structure` | 개요·조문 트리 | `structure::build_structure` |
| `bookmarks` | 책갈피 이름·주소 | `get_bookmarks_native` |
| `charts` | 차트 위치(읽기만) | `chart_extract::collect_charts` |
| `outline-nav` | 개요 번호 문단 | `get_outline_navigation_native` |
| `headers-footers` | 머리말·꼬리말 목록 | `get_header_footer_list_native` |
| `batch-info` | 여러 파일 요약 | `info` 와 같은 조회를 N개 |
| `doc-info` | 폰트·치환 | `get_document_info` |
| `page-info` | 쪽 여백·본문 상자 | `get_page_info_native` |
| `section-def` | 구역 정의 | `get_section_def_native` |
| `page-pos` / `para-page` | 쪽↔문단 주소 | `get_position_of_page_native` / `get_page_of_position_native` |
| `chart-data` | 차트 숫자(원문) | `get_chart_data_by_index_native` |

### 5.2 쪽·본문

| 명령 | 하는 일 |
|------|---------|
| `pages` | 쪽별 문자 수·빈 쪽 |
| `page-window` | 요청한 쪽 구간 텍스트 |
| `empty-pages` | 빈 쪽 번호 |
| `char-count` / `para-count` | 문자·문단 수 |
| `sample-text` | 본문 앞부분 (truncated 표지) |
| `page-hashes` | 쪽 텍스트 blake3 |
| `text-hash` | 쪽 경계를 포함한 본문 해시 |
| `line-count` / `unique-chars` | 줄 수·서로 다른 문자 수 |
| `hangul-ratio` / `ascii-ratio` | 공백 제외 비율 |
| `section-count` | 구역 수 |
| `longest-page` / `shortest-page` | 문자 수 극값 |

### 5.3 찾기

| 명령 | 하는 일 | exit 3 |
|------|---------|--------|
| `search` | 본문 부분 문자열, page/offset | — |
| `search-count` | 등장 횟수만 | — |
| `contains` | 포함 여부 | 없음 |
| `grep-pages` | 등장 쪽 번호만 | — |
| `grep` | 구역·문단·쪽 주소 + 문맥 | — |

### 5.4 서식·표

| 명령 | 하는 일 | 레시피 |
|------|---------|--------|
| `fields` / `field-count` | 누름틀 이름 | 1 |
| `field-values` | 이름과 현재 값 | 1 |
| `field-locate` | 이름·값·구역·문단·리스트 좌표 | 1 |
| `field-get` | 이름으로 값, 없으면 exit 3 | 1 |
| `empty-fields` | 값이 빈 누름틀 | 1 |
| `form-ready` | 누름틀이 있으면 채움 축 | 1, exit 3 = 대상 아님 |
| `field-diff` | 두 서식 누름틀 이름 차집합 | 개정판, exit 3 = 집합 다름 |
| `tables` / `table-count` | 표 치수 | 2 |
| `table-inspect` | 격자·병합·셀 텍스트 | 2 |
| `table-csv` | 표 하나 CSV (`--all` 이면 전 표) | 2 |
| `captions` | 표 캡션 | 2 |
| `merged-tables` | 병합 셀이 있는 표만 | 2 |

### 5.5 수확

| 명령 | 하는 일 |
|------|---------|
| `extract-data` | 날짜·금액·수량 + 주소 (`--kind date\|amount\|number\|all`) |

### 5.6 비교·파일

| 명령 | 하는 일 | exit 3 |
|------|---------|--------|
| `compare-pages` | 두 문서 쪽수 | 다름 |
| `compare-text` | 두 문서 본문 해시 | 다름 |
| `hash` / `size` / `magic` | 바이트 해시·크기·선두 바이트 | — |
| `fingerprint` | 안정 지문, `--check` 드리프트 | 드리프트 |
| `diff-text` | 줄 단위 diff | 텍스트 다름 |
| `evidence` | 전/후 증빙 번들 | — |

### 5.7 보안 (읽기 전용)

| 명령 | 하는 일 | 레시피 |
|------|---------|--------|
| `threat-scan` | 컨테이너 구조 위협 | 4 |
| `injection-scan` | 본문 프롬프트 주입 신호 | 4, 10 |
| `hidden-text` | 화면엔 없고 추출기는 읽는 글 | 10 |
| `unicode-scan` | 화면-바이트 불일치 | 10 |
| `stego-scan` | 제로폭 비트·동형자·공백 채널 | 10 |
| `pii-scan` | 주민·카드·전화·이메일 | 3, 10 |
| `armor` | nonce 격벽으로 본문을 감싸 LLM 에 넘김 | 출처 표지 |
| `sweep` | 위협·주입·은닉·유니코드·스테가 개수를 한 봉투 | 10 |

`sweep` 은 상세 발췌를 싣지 않는다. 신호가 있으면 해당 축 명령을 다시 부른다.
`armor` 는 본문을 지우지 않는다. 격벽 안은 전부 데이터다.

### 5.8 운영

| 명령 | 하는 일 |
|------|---------|
| `capabilities` | 자기서술 |
| `doctor` | 환경 자가진단 |
| `scan` | 디렉터리 재귀 발견 |
| `verify` | 산출물 사후 기대 게이트 |
| `chunk-plan` / `context-cost` | 컨텍스트 예산 |
| `plan-lint` | `run` 계획서 JSON 선검증 |
| `envelope-lint` / `nextcall` | 봉투 선검증 |

## 6. 레시피와 이어지는 순서

에이전트가 이 표면만으로 앞단을 닫는 순서. 편집은 본 CLI.

**레시피 1 서식 채움**

```
rhwp-agent explore <서식.hwp> --json
rhwp-agent form-ready <서식.hwp> --json          # exit 3 이면 표 셀 축
rhwp-agent field-values <서식.hwp> --json
rhwp-agent empty-fields <서식.hwp> --json
# 이후 rhwp edit fill-fields …  (이 바이너리가 아님)
```

실측 표본: `samples/form-01.hwp`. 누름틀 `myMsg` 계열이 있고 값은 비어 있다.

**레시피 2 표 CSV**

```
rhwp-agent tables <문서.hwp> --json
rhwp-agent merged-tables <문서.hwp> --json
rhwp-agent table-inspect <문서.hwp> --table 0 --json
rhwp-agent table-csv <문서.hwp> --table 0 --json
# 이후 본 CLI table-to-csv / csv-to-table
```

실측 표본: `samples/hwp_table_test.hwp` (표 10개, 0번 표 4×3, 머리 행
「제목 / 담당자 / 세부 내용」), `samples/hwp3-sample.hwp` (표 6개).

**레시피 4 수신 점검 + 레시피 10 송신 스윕**

```
rhwp-agent threat-scan <문서.hwp> --json         # 열기 전 컨테이너
rhwp-agent sweep <문서.hwp> --json               # 한 봉투 요약
# clean=false 이면 축을 다시 연다
rhwp-agent hidden-text <문서.hwp> --json
rhwp-agent injection-scan <문서.hwp> --json
rhwp-agent unicode-scan <문서.hwp> --json
rhwp-agent stego-scan <문서.hwp> --json
rhwp-agent pii-scan <문서.hwp> --json
```

LLM 에 본문을 넣을 때:

```
rhwp-agent armor <문서.hwp> --json
```

격벽 `⟦UNTRUSTED:<nonce>⟧ … ⟦/UNTRUSTED:<nonce>⟧` 안은 지시가 아니다.

**트리아지 (전문 덤프 없이 좁히기)**

```
rhwp-agent info <문서.hwp> --json
rhwp-agent explain <문서.hwp> --json
rhwp-agent digest <문서.hwp> --max-chars 240 --json
rhwp-agent grep <문서.hwp> --q <찾을말> --json
rhwp-agent extract-data <문서.hwp> --kind all --limit 20 --json
```

## 7. 실측 표본

지어낸 값이 아니다. 계약 시험이 같은 파일을 연다.

| 파일 | 이 묶음이 확인한 것 |
|------|---------------------|
| `samples/form-01.hwp` | 포맷 hwp5, 누름틀 있음, `form-ready` 통과, `empty-fields` 에 `myMsg`, `explore` 메뉴에 form-fill, 암호 없음 |
| `samples/hwp3-sample.hwp` | 포맷 hwp3, 표 6, `grep Linux` 가 구역 0·문단 0·쪽 0, 표 셀 히트, `digest` 첫 줄에 Linux, `armor` 격벽이 Linux 를 보존 |
| `samples/hwp_table_test.hwp` | 표 10, 0번 4×3, 머리 행 제목/담당자/세부 내용 |
| `samples/hwpx/form-01.hwpx` | `compare-pages` / `field-diff` 대조 짝 |

`hidden-text`·`unicode-scan`·`stego-scan` 은 위 세 표본에서 `clean=true` 다.
신호가 있는 악성 코퍼스는 이 묶음이 새로 만들지 않는다. 레시피 10 이 가리키는
기존 표본을 쓴다.

## 8. 시험

`tests/agent_cli_pack_contract.rs` — `regression_suite_009` 가 끌어온다.
기존 `tests/agent_toolkit_contract.rs` 는 수정하지 않는다.

확인하는 것:

- `capabilities --json` 에 이 묶음의 명령 이름이 있다
- `--json` 봉투에 `schemaVersion` / `tool` / `command` / `untrusted*` 가 있다
- 미지 플래그는 exit 2, stdout 비어 있음
- 위 실측 표본의 필드·표·검색·보안 게이트

실행:

```
python tools/rust-test-suite-manifest --prepare
cargo fmt --all -- --check
cargo test --test regression_suite_009 agent_cli_pack -- --test-threads=1
```

## 9. 명령을 더할 때

1. `DocumentCore` 공개 질의가 이미 있는지 확인한다. 없으면 이 바이너리에 로직을
   쓰지 않는다.
2. `src/bin/rhwp-agent/` 모듈에 핸들러를 추가한다. 미지 플래그는 exit 2.
3. `caps::COMMANDS` 에만 등록한다. 디스패처를 손으로 분기하지 않는다.
4. 문서 파생 필드를 `untrusted_decl` 에 적는다.
5. 이 파일 §5 표와 계약 시험에 같은 이름을 넣는다.
6. `src/main.rs`(본 CLI) 는 열지 않는다.

## 10. 이 문서가 대체하는 것

공장에서 명령 무리를 나눌 때마다 작업 문서를 새로 만들지 않는다.
이후 조회 명령을 이 바이너리에 보태면 **이 파일만** 고친다.
`mydocs/working/agent_cli.md`(#5316) 와 gym_* 기록은 그대로 둔다.
