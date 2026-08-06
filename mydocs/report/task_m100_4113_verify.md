---
kind: report
status: active
canonical: mydocs/report/task_m100_4113_verify.md
last_verified: 2026-08-06
---

# 처리 결과 — `verify` 독립 사후검증 게이트 (#4113, #3918 승격 2호)

## 분석 — 공백의 정확한 모양

편집 축의 `--verify` 는 "저장 직후 자기검증"(같은 호출 안)이고, **임의 시점·임의
파일에 기대 조건 집합을 대는 독립 명령은 없었다.** 편집 파이프라인의 마지막 관문
("됐다"를 에이전트의 자기 판단이 아니라 종료 코드로 받는 자리)이 표면 공백.
rhwp-agent 실험 표면의 `verify` 가 이 축을 검증해 두었고(#3922 — 게이트 종료 코드
왕복 계약), 그 승격이다.

## red → green

- **red 실측**: 구현 전 devel 바이너리에서
  `rhwp verify samples/field-01.hwp --expect-pages 3 --json`
  → `오류: 알 수 없는 명령입니다 - verify` + exit 2.
- **green**: 구현 후 계약 5본 전부 통과 —
  `cargo test --release --test verify_contract` → **5 passed; 0 failed** (0.89s).

## 설계 판단

- **판정은 데이터**(규칙 3): 조건별 `{kind, expected, actual, pass}` 를 봉투에 싣고
  종료 코드는 요약 — 전부 만족 0 / 불일치는 **봉투를 먼저 내고** 3(#2707 판정) /
  실행 실패 stdout 0 B + 1 / 조립 오류 2(미지 옵션 침묵 무시 금지 포함).
- **재사용만 한다**(규칙 2): `page_count`·`grep`(search 코어)·
  `collect_field_records`·`detect_format`. 새 조회 로직 0.
- **출처 표지**(규칙 4): `expectations[].actual` 은 문서 파생 값 — provenance MAP
  에 등재(expected·subject 는 호출자 값, pass·verdict 는 엔진 판정임을 note 로
  구분). 출처 스윕 호출표에는 **어떤 표본이든 항상 만족하는 기대**(부조리 문자열의
  부재)로 등재해, 스윕이 판정 결과가 아니라 표지를 보게 했다.
- 1차 조건 5축(pages·contains·not-contains·field·format)만 — rhwp-agent 의 11종
  중 잔여 축은 #4113 에서 후속 판단.
- 등재 일습: capabilities(recordFields 6종)·`--help`·MCP `hwp_verify`(조건별
  when-args, 반복 조건은 CLI 안내).

## 실측 (재현: release 바이너리, samples/field-01.hwp)

```
$ rhwp verify samples/field-01.hwp --expect-pages 3 --expect-format hwp5 --json
{"schemaVersion":"1.0","source":"samples/field-01.hwp","expectations":[
 {"kind":"pages","expected":3,"actual":3,"pass":true},
 {"kind":"format","expected":"hwp5","actual":"hwp5","pass":true}],
 "passCount":2,"failCount":0,"verdict":"pass", …}   → exit 0

$ rhwp verify samples/field-01.hwp --expect-pages 99 --json   → 봉투(verdict:fail, actual:3) 후 exit 3
$ rhwp verify 없는파일.hwp --expect-pages 1 --json            → stdout 0 B, exit 1
$ rhwp verify samples/field-01.hwp --json                     → 기대 조건 0개 usage, exit 2
```

계약 5본이 위 전 경로 + 출처 표지(untrustedContent:true,
untrustedFields 에 `expectations[].actual`)를 고정한다.

## 무회귀

- `capabilities_schema_contract` — verify 항목은 표준 키만 쓴다(17/17 green 은
  같은 워크트리의 #4114 수리에서 확인, 이 브랜치 항목도 같은 스키마 안).
- `provenance_contract` — 전수 스윕에 verify 호출 등재 후 green(수치는 PR 본문).
- 같은 저자의 열린 PR #4114 와 src/main.rs 공유 — `git merge-tree` 무충돌 사전
  실증 후 제출.
