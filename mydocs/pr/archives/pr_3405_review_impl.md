---
kind: pr_review_impl
status: active
canonical: mydocs/pr/archives/pr_3405_review_impl.md
last_verified: 2026-07-27
---

# PR #3405 보정·수용 계획

## Stage 1 — 원 변경 고정과 검증 완료

- source head `5a41caff10`을 기준으로 merge simulation 및 parser·CLI·WASM 검증을 수행했다.
- 원 PR CI는 일반 CI가 성공했지만 CodeQL security aggregate가 실패라 수용 완료 상태가 아니다.

## Stage 2 — collaborator 보정 완료, remote 미반영

`ecdfd9ca4 test(crypto): 암호 HWP5 실제 fixture 회귀 고정`

- maintainer 전용 최상위 문서 변경 제거
- 실제 암호 fixture 및 공개 API·CLI·평문 저장 회귀 추가
- 일반 no-password roundtrip baseline의 암호 fixture 자동 제외와 전용 gate 기록

이 commit은 contributor의 원 기능 commit과 분리됐다. 아직 source remote에 push하지 않았고,
push 전에는 PR head·`ls-remote`·local parent SHA를 다시 대조한다.

## Stage 3 — 차단 해소 완료, 재검증 완료

1. CodeQL 57건은 2026-07-27 annotation·source·두 CLI sentinel 실행으로 과탐지임을 확인하고
   `false positive` 41건, `used in tests` 16건으로 dismiss했다. source push 뒤 새 CodeQL run에
   새 alert가 없는지 확인한다. `--password-stdin` 권장과 UI 비보존 요구사항은 유지한다.
2. `2c8dbfaf6 fix(crypto): 암호 HWP5 스트림 압축 해제 상한 적용`이 DocInfo·BodyText·즉시·지연
   BinData에 스트림별 512 MiB 복호화 후 상한을 적용했다. compressed 확장과 uncompressed 초과 회귀를
   추가했다.
3. 전용 review target에서 targeted test, IR sweep baseline diff 0, release-test 전체 exit 0, Clippy,
   WASM check를 다시 통과했다.

## Stage 4 — review 기록과 remote 준비

- `pr_3405_review.md`, 이 계획서, 오늘할일을 별도 docs commit으로 만든다.
- source branch에 LFS object가 없음을 확인한 뒤 정상 pre-push dry-run을 한다.
- 작업지시자가 승인한 경우에만 contributor source branch에 non-force push한다.

## Stage 5 — merge 후보 판정

- push 뒤 local HEAD, source ref, PR `headRefOid` 일치를 확인한다.
- code/test 보정이 포함되어 review-only fast-pass를 사용하지 않는다. 최신 head full CI와 CodeQL,
  mergeable 상태, 작업지시자 승인이 모두 필요하다.
- merge는 maintainer 권한 단계이며, merge 뒤 원 PR·관련 issue·review comment 후속 처리를 별도로
  확인한다.
