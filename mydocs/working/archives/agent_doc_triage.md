# 에이전트 문서 트리아지 스킬 고도화 (#5296)

gym 이 아니다. 실사용 에이전트가 긴 HWP 를 처음부터 덤프하지 않고 좁혀 읽게 한다.
새 CLI 는 없다.

## 무엇을

`.claude/skills/rhwp-doc-triage/` 를 인덱스(`SKILL.md`) + `references/` 16장으로 나눴다.
사다리는 `info → explain → export-structure → digest → search → extract-data` 이고,
핵심은 **답이 나오면 멈춘다**.

## 왜

기존 SKILL.md 한 장은 명령을 나열하지만,

- 쪽수 밴드가 "몇 쪽/수십 쪽"으로만 적혀 진입이 흔들리고
- 정지 규칙이 약해 에이전트가 6단을 의례 순회하며 전문을 열고
- K5(보안 스윕) 인계가 로드맵 DoD 대비 빠져 있었다.

긴 편람에서 `export-text` 무제한이 기본값이 되는 실패를 스킬 구조로 막는다.

## 어떻게

- 판단 트리·명령 6단·정지 15조·인계·예산·함정·여정·반덤프 장을 분리
- 기계 픽스처 `tests/fixtures/agent_doc_triage/*.json`
- 계약 테스트 6파일 — 스킬 파일 존재, 픽스처 정합, 실 CLI 봉투 키, 정지/라우팅/여정
- capability 카탈로그에 `CAP-5296` / `rhwp-doc-triage` 등재

## 검증

```bash
cargo fmt --all -- --check
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/run-rust-test.mjs agent_doc_triage_skill_contract
node scripts/run-rust-test.mjs agent_doc_triage_fixture_schema
node scripts/run-rust-test.mjs agent_doc_triage_cli_ladder
node scripts/run-rust-test.mjs agent_doc_triage_stop_contract
node scripts/run-rust-test.mjs agent_doc_triage_routing
node scripts/run-rust-test.mjs agent_doc_triage_journeys
node scripts/run-rust-test.mjs agent_doc_triage_intent_matrix
node scripts/run-rust-test.mjs agent_doc_triage_anti_dump
```

렌더/레이아웃 변경 없음. 시각 검증 해당 없음.

## 하지 않은 것

- gym/ 미수정
- onboarding / mcp / safe-edit / provenance 스킬 미수정
- 새 서브커맨드·플래그 없음

## 사다리와 정지

순서: info → explain → export-structure → digest → search → extract-data.

정지는 성공의 기본값이다.

- S01 열기 실패
- S02 암호, 비밀번호 없음
- S03 메타 질문
- S04 종류 질문
- S05 목차 질문
- S06 훑기 (excerpt=0~2쪽 고지)
- S07 사실+쪽
- S08 검색 0건
- S09 수치+주소
- S10 긴 문서 넓은 읽기 금지
- S11 표/누름틀 인계
- S12 보안/출처 인계
- S13 예산 소진
- S14 폴더 선별
- S15 이미 답함

## 쪽수 밴드

| 밴드 | pageCount | 첫 경로 |
| --- | --- | --- |
| tiny | 1~3 | export-text --json 허용 |
| small | 4~8 | explain |
| medium | 9~30 | digest --max-chars |
| large | 31~100 | digest 800 + search limit |
| huge | 101+ | digest 600 + search --limit 20 |

표 전체는 `references/18_pagecount_routing.md`.

## 픽스처

`tests/fixtures/agent_doc_triage/`

- tree.json / command_ladder.json — 6단
- stop_rules.json — S01~S15
- envelope_keys.json — 실 CLI 키
- journeys.json / intent_matrix.json — 발화
- query_catalog.json — 검색어
- pagecount_1_220.json / sample_routing.json — 쪽수
- handoff.json / pitfalls.json / skill_index.json

## 테스트 파일

- tests/cases/agent_doc_triage_skill_contract.rs
- tests/cases/agent_doc_triage_fixture_schema.rs
- tests/cases/agent_doc_triage_cli_ladder.rs
- tests/cases/agent_doc_triage_stop_contract.rs
- tests/cases/agent_doc_triage_routing.rs
- tests/cases/agent_doc_triage_journeys.rs
- tests/cases/agent_doc_triage_intent_matrix.rs
- tests/cases/agent_doc_triage_anti_dump.rs

실 CLI 대조는 기존 샘플만 사용한다.

- samples/para-001.hwp
- samples/field-01.hwp
- samples/table-001.hwp
- samples/hwp3-sample.hwp
- samples/hwp3-sample16.hwp

## 범위 밖

- gym/
- .claude/skills/rhwp-onboarding
- .claude/skills/rhwp-mcp-session
- .claude/skills/rhwp-safe-edit
- .claude/skills/rhwp-provenance
- 새 rhwp 서브커맨드/플래그
