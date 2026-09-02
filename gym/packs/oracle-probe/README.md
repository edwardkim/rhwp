---
kind: guide
status: active
canonical: gym/packs/oracle-probe/README.md
last_verified: 2026-08-18
---

# oracle-probe — 라이브 오라클 이중 계산

## 왜 이 pack 인가

채점은 정답을 골든 파일로 박제하지 않는다. 에이전트가 보고한 쪽수·검색
건수·필드 수·표 수를 **채점 시점에 rhwp 가 다시 계산**하고, 그 값과
등호를 본다. 이 pack 은 그 계약을 과제 단위로 드러낸다.

1. 에이전트는 문서에서 숫자 하나를 읽어 `answer.json` 에 적는다.
2. 채점기는 같은 명령을 다시 실행한다 (`info` / `explain` / `search` /
   `fields` / `export-tables` / `extract-data` / `scan` / `capabilities` /
   `verify` / `dump-pages`).
3. 두 값이 같아야 통과한다. 어제 박제한 숫자가 오늘 바이너리와 어긋나면
   과제가 틀린 것이 아니라 **오라클이 살아 있는 것**이다.

`gym/tools/oracle_probe.py` 는 이 pack 을 채점하지 않는다. 프로브는
채점기가 믿는 세 가지 전제 — 이중 계산 결정성, `{input}` / `{sub:}`
치환, 부재 산출물의 비통과 — 를 팩 픽스처 없이 감사한다. 이 pack 은
그 전제를 **실문서·실명령** 으로 구체화한다.

새 CLI 는 없다. 기존 명령과 기존 표본만 쓴다.

권위 출처: `gym/README.md` 의 "채점은 라이브다", `gym/tools/oracle_probe.py`
모듈 문서, `gym/core/checks.py` 의 `answer_eq` / `len_answer_eq` /
`value_eq`.

## 이 확장이 지키는 규칙

1. **기존 명령만.** `pack.json` requires 는 `capabilities` · `dump-pages` ·
   `explain` · `export-tables` · `extract-data` · `fields` · `info` ·
   `scan` · `search` · `verify` 뿐이다.
2. **기존 연산자만.** `answer_eq` · `len_answer_eq` · `value_eq` 만 고른다.
   전역 훑기(`deep_contains`)는 쓰지 않는다. 편집 산출물을 만들지 않는다.
3. **기존 표본만.** `samples/` 밖 파일을 만들지 않는다. table-001,
   issue2007 중첩 표, field-01 / field-01-memo, form-01 / form-02,
   국립국어원 업무계획, 수출입 현황, multi-table-001, para-001,
   exam-kor-1p, hwpx_sample2, PII 분석 표본만 축을 바꿔 다시 묻는다.
4. **라이브 오라클.** 쪽수·검색 건수·필드 수를 과제에 박제하지 않는다.
   채점기가 같은 명령을 다시 돌려 봉투 필드를 읽는다.
5. **편집 과제를 복제하지 않는다.** 누름틀을 채우지 않는다. 표를 고치지
   않는다. 산출물 `.hwp` 를 요구하지 않는다. 제출은 `answer.json` 만.
6. **사다리를 완주하지 않는다.** `replay` · `audit` · `lineage` · `gate`
   · `--deep` 을 부르지 않는다. 그 축은 automation / work-receipt /
   expert-challenges 의 일이다.
7. **원본을 덮지 않는다.** 읽기 전용 조회만 한다.

## 이중 계산이  concretamente 보이는 곳

| 에이전트가 보고하는 키 | 라이브 오라클 | 봉투 좌표 |
|------------------------|---------------|-----------|
| `pages` | `rhwp info {input} --json` | `pageCount` |
| `paragraphs` | `rhwp explain {input} --json` | `paragraphCount` |
| `tables` | `rhwp export-tables {input} --json` | `tableCount` |
| `tableLen` | 같은 명령 | `len(tables)` |
| `fields` | `rhwp fields {input} --json` | `fieldCount` |
| `fieldLen` | 같은 명령 | `len(fields)` |
| `hits` | `rhwp search … --json` | `matchCount` 또는 `len(matches)` |
| `items` | `rhwp extract-data {input} --json` | `len(items)` |
| `files` | `rhwp scan <폴더> --json` | `len(files)` |
| `commands` / `read` / `write` | `rhwp capabilities` | 배열 길이 |
| `format` | `rhwp info {input} --json` | `format` |
| `verdict` | `rhwp verify --expect-min-pages` | `verdict` |
| `dumped` | `rhwp dump-pages {input} --json` | `pageCount` |
| `mismatch` | `rhwp scan --probe --json` | `files[0].extMismatch` |

같은 표본에 다른 오라클을 붙인 짝이 있다. OP01 의 쪽수와 OP03 의 표 수는
둘 다 table-001 이지만 숫자가 다르다. OP09 의 필드 수와 OP27 의 쪽수는
둘 다 field-01 이다. 한쪽을 베껴 넣으면 다른쪽이 실패한다 — 그것이
이중 계산의 판별력이다.

스칼라(`fieldCount`)와 배열 길이(`len(fields)`)를 따로 묻는 과제
(OP09/OP10, OP30/OP31)는 오라클 내부 정합도 드러낸다. 두 경로가
어긋나면 채점기가 아니라 **명령 봉투** 가 깨진 것이다.

## 함정 (실측, 과제에 녹여 둔 것)

- **숫자를 기억하지 마라.** 어제 센 pageCount 가 오늘도 같으리라는
  보장은 바이너리 계약이지 과제 계약이 아니다.
- **표본을 바꾸면 답이 바뀐다.** table-001 의 쪽수를 form-01 에 그대로
  내면 OP25 가 실패한다.
- **검색어를 바꾸면 건수가 바뀐다.** '국어' 와 '업무' 는 같은 문서라도
  다른 오라클이다 (OP14 / OP34).
- **형식 표지는 확장자가 아니다.** `.hwp` 라도 봉투의 `format` 을
  읽어야 한다. hwpx 표본에 `hwp` 를 적으면 OP18 이 실패한다.
- **이름이 1p 라고 1 을 박제하지 마라.** OP35 는 exam-kor-1p 의 쪽수를
  info 로 다시 센다.
- **PII 표본을 마스킹하지 마라.** OP37·OP38 은 형식과 쪽수만 묻는다.
  보안 pack 의 SE01·SE02 를 복제하지 않는다.
- **제출은 answer.json 하나다.** 산출 `.hwp` 를 만들어도 채점기는
  읽지 않는다. 부재 산출물 프로브는 이 pack 의 과제가 아니라
  `oracle_probe.py` 의 단위 시험이 담당한다.

## 과제 지도

난도 1=입문 · 2=초급 · 3=중급. 보스(5) 사다리 완주는 XC 의 일이다.

### OP01–OP08 — 문서 신원 (쪽·문단·표·덤프)

| ID | 티어 | 질문 | 표본 | 오라클 |
|----|------|------|------|--------|
| OP01 | 1 | 쪽수 | table-001 | info.pageCount |
| OP02 | 1 | 문단 수 | table-001 | explain.paragraphCount |
| OP03 | 1 | 표 수 | table-001 | export-tables.tableCount |
| OP04 | 1 | '표' 검색 | table-001 | search.matchCount |
| OP05 | 2 | 쪽수 | issue2007 중첩 | info.pageCount |
| OP06 | 2 | 문단 수 | issue2007 중첩 | explain.paragraphCount |
| OP07 | 2 | 덤프 쪽수 | issue2007 중첩 | dump-pages.pageCount |
| OP08 | 2 | 표 수 | issue2007 중첩 | export-tables.tableCount |

### OP09–OP16 — 필드·검색·추출

| ID | 티어 | 질문 | 표본 | 오라클 |
|----|------|------|------|--------|
| OP09 | 2 | 누름틀 수 | field-01 | fields.fieldCount |
| OP10 | 2 | 누름틀 배열 길이 | field-01 | len(fields) |
| OP11 | 2 | 누름틀 수 | field-01-memo | fields.fieldCount |
| OP12 | 2 | 누름틀 수 | form-01 | fields.fieldCount |
| OP13 | 2 | 누름틀 수 | form-02 | fields.fieldCount |
| OP14 | 2 | '국어' 검색 | 국립국어원 | len(matches) |
| OP15 | 2 | 쪽수 | 국립국어원 | info.pageCount |
| OP16 | 2 | 추출 항목 수 | 수출입 현황 | len(items) |

### OP17–OP24 — 형식·스윕·자기서술·검증

| ID | 티어 | 질문 | 표본 | 오라클 |
|----|------|------|------|--------|
| OP17 | 1 | format | table-001 | info.format |
| OP18 | 2 | format | hwpx_sample2 | info.format |
| OP19 | 2 | 폴더 문서 수 | samples/hml | len(scan.files) |
| OP20 | 2 | 확장자 불일치 | samples/hml | files[0].extMismatch |
| OP21 | 1 | 명령 수 | (도구 자신) | len(capabilities.commands) |
| OP22 | 2 | 읽기 형식 수 | (도구 자신) | len(formats.read) |
| OP23 | 2 | 쓰기 형식 수 | (도구 자신) | len(formats.write) |
| OP24 | 2 | 최소 쪽수 판정 | issue2007 중첩 | verify.verdict |

### OP25–OP32 — 표본을 바꿔 같은 질문을 다시

| ID | 티어 | 질문 | 표본 | 오라클 |
|----|------|------|------|--------|
| OP25 | 1 | 쪽수 | form-01 | info.pageCount |
| OP26 | 1 | 쪽수 | form-02 | info.pageCount |
| OP27 | 1 | 쪽수 | field-01 | info.pageCount |
| OP28 | 2 | 문단 수 | field-01 | explain.paragraphCount |
| OP29 | 2 | 문단 수 | form-01 | explain.paragraphCount |
| OP30 | 2 | 표 수 | multi-table-001 | export-tables.tableCount |
| OP31 | 2 | 표 배열 길이 | multi-table-001 | len(tables) |
| OP32 | 2 | 문단 수 | para-001 | explain.paragraphCount |

### OP33–OP44 — 검색 변이·시험지·PII 신원·이중 필드

| ID | 티어 | 질문 | 표본 | 오라클 |
|----|------|------|------|--------|
| OP33 | 2 | '마케팅' 검색 | field-01 | search.matchCount |
| OP34 | 2 | '업무' 검색 | 국립국어원 | search.matchCount |
| OP35 | 1 | 쪽수 | exam-kor-1p | info.pageCount |
| OP36 | 2 | 문단 수 | exam-kor-1p | explain.paragraphCount |
| OP37 | 2 | format | PII 분석 표본 | info.format |
| OP38 | 2 | 쪽수 | PII 분석 표본 | info.pageCount |
| OP39 | 2 | format | field-01 | info.format |
| OP40 | 3 | 쪽수+표 수 | table-001 | info + export-tables |
| OP41 | 3 | 필드+쪽수 | field-01 | fields + info |
| OP42 | 3 | 검색+쪽수 | 국립국어원 | search + info |
| OP43 | 2 | 표 수 | form-01 | export-tables.tableCount |
| OP44 | 2 | 문단 수 | form-02 | explain.paragraphCount |

## 기준 풀이

각 과제의 `reference/<ID>.json` 은 채점 검사와 같은 명령을 다시 적는다.
`build_baseline.py` 가 그 명령을 실행해 `answer.json` 을 만든다. 숫자
리터럴이 기준 풀이에도 없다 — 라이브 재계산이 제출물을 만든다.

## 프로브 도구와의 관계

```
oracle_probe.py          이 pack
----------------------   --------------------------------
probe_determinism        같은 명령을 채점기가 다시 실행
probe_placeholders       기준 풀이의 {input} 치환
probe_missing_artifact   이 pack 은 answer 만 — 산출 부재는 단위 시험
--json 구조 자기점검     pack 등재 전에 모듈 표면이 살아 있는지
```

프로브가 실패하면 이 pack 의 채점 전제가 무너진다. CI 는
`scripts/tests/test_gym_oracle_probe.py` 와 `gym/tools/audit.py` 로
둘 다 막는다.

## 하지 않는 일

- 새 clap 명령, 새 `src/bin`, 새 검사 연산자를 추가하지 않는다.
- 골든 `pageCount: 1` 같은 숫자를 과제 JSON 에 적지 않는다.
- 편집·변환·영수증 사다리를 이 pack 에 끌어오지 않는다.
- `gym/README.md` 의 12 pack 표를 바꾸지 않는다. 이 pack 은 라이브
  오라클 계약을 드러내는 확장이지 운동장 입문 코스의 교체가 아니다.
