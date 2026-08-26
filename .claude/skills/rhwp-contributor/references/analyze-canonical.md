# 2단 — 분석 (정본을 읽고 이슈에 기록)

구현 전에 정본과 기존 계약 시험을 읽는다. "비슷해 보이는 코드"만 보고
패치하지 않는다.

## 읽는 순서

`AGENTS.md` 문서 로딩 순서를 따른다.

1. `CLAUDE.md` → `AGENTS.md`
2. `mydocs/README.md`
3. 작업 성격에 맞는 `mydocs/manual/README.md` 선택표
4. 기여면 `CONTRIBUTING.md`, PR 이면 `pr_review_workflow.md`
5. 검증이면 `mydocs/manual/pr_review/local_validation.md` §4.3
6. 시각이면 `mydocs/manual/verification/visual_verification_governance.md`
7. 기존 `tests/cases/*` · `tests/*_contract.rs` 중 인접 계약

## 이슈에 남길 분석

- 원인 (파일·함수·계약이 어디에 있는지)
- 설계 (무엇을 바꾸고 무엇을 그대로 둘지)
- 검증 계획 (`cargo fmt --all -- --check`, clippy, 관련 test, 시각 여부)
- 비범위 (DocumentCore 편집 로직, gym, 다른 스킬, 열린 PR 파일)

## 하지 않는 분석

- DocumentCore 의 새 편집 연산자를 이 자리에서 설계하지 않는다.
  편집이 필요하면 이미 devel 에 있는 CLI (`edit`, `run`) 와
  `rhwp-safe-edit` 스킬을 **포인터로** 따른다.
- 미병합 브랜치의 명령을 정본처럼 인용하지 않는다.

## 닫는 증거

이슈 댓글 또는 본문에 정본 경로와 계약 시험 이름이 있다.

예제: [03_analyze_canonical.md](../examples/03_analyze_canonical.md).
