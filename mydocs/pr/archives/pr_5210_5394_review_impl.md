---
kind: pr-review-implementation
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-18
---

# kevin9327 PR 통합 검토 구현 계획

## 기준과 적용 순서

- 기준: upstream/devel 0bc05ef81
- 가시성 branch: review/kevin9327-20260818-r1
- 적용 순서: #5210 → #5211 → #5214 → #5221 → #5227 → #5228 → #5248 → #5270 → #5272 → #5278 → #5282 → #5284 → #5315 → #5352 → #5356 → #5358 → #5360 → #5362 → #5364 → #5366 → #5369 → #5373 → #5375 → #5381 → #5385 → #5387 → #5394
- PR 내부의 merge commit은 cherry-pick하지 않았고 contributor commit history는 보존했다.
- 원 PR별 review 기록은 각각 mydocs/pr/archives/pr_N_review.md에 둔다. 통합 branch 자체만 설명하는 별도 docs-only review PR은 만들지 않는다.

## 메인터너 보정

- 5b9599ffb98c5dcc72c0e73937df076ea1e77031: oracle fixture 산출물의 CRLF를 LF로 정리하고 불필요한 PR body draft 산출물을 제거했다.
- 5f0d7e0a8482617747bdc7d7df57e98e6717a02c: oracle_probe 자리표 치환, multi PDF 기대 manifest·coverage fixture, PII reason, 보안 수신 순서 테스트와 18개 예제 명령을 정합화했다.
- 보정은 contributor 기능 commit을 amend/rebase하지 않았고, 충돌 해소와 계약·fixture 정합화에 한정했다.

## 충돌 해소 기록

- #5228: .github/workflows/ci.yml에서 oracle probe와 differential 배선을 모두 보존했다.
- #5360: tools/oracle_public/README.md의 기존 page-smoke와 M01-1 내용을 병합했다.
- #5369: tools/oracle_public/tests/__init__.py의 테스트 패키지 표식을 유지했다.
- #5375: fuzz/README.md의 nightly/CI 안내와 M04 내용을 병합했다.
- #5381, #5387, #5394: Cargo.toml의 proptest 의존성·M04 주석과 fuzz/README.md를 중복 없이 통합했다.

## 완료 검증

- Python: 1737 tests, 1 skipped, OK.
- 보안 스윕 Rust: envelopes 5/5, gate 4/4, no_raw 6/6, pii_rules 4/4, receive 5/5, skill_contract 6/6.
- property/CI Rust: prop_edit_plan 5/5, prop_hwp5_roundtrip 6/6, prop_hwpx_roundtrip 6/6, prop_roundtrip_ci 1/1.
- cargo fmt --all -- --check, git diff --check, Rust manifest/check와 unit-test tier check 통과.

## 현재 판정과 다음 조건

로컬 통합 기준 차단 결함은 없다. 다만 원격 PR은 현재 BLOCKED이고 GitHub check 상태는
merge 전 재확인이 필요하다. 원격 push·review 게시·merge는 별도 승인 전에는 수행하지 않는다.
원격 PR이 실제로 merge된 뒤에만 issue close, archive 확정, branch/worktree cleanup을 수행한다.
