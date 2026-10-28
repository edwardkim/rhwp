---
kind: pr-review
status: approved
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6493
issue: 4969
author: edwardkim
---

# PR #6493 review - W10 고급 shaping과 bounded vertical layout

## 결론 - 승인, review-only 기록 검증 대기

[PR #6493](https://github.com/edwardkim/rhwp/pull/6493)은 #4969의 W10-Q2-D5, Q3, Q4를
하나의 검증 가능한 checkpoint 계보로 제출한다. 현재 코드 후보 head는
`7a67f250851babf37e8d21f242494829952ade74`이고, base는
`upstream/devel@3b301f725ab48985f19c25784b68448ed4257bfd`다. 로컬 검증과 self-review에서
차단 결함은 발견하지 않았다. 최신 코드 후보의 GitHub Full CI는 30 success, 4 expected skip,
실패·대기 0으로 끝났고 PR은 `MERGEABLE/CLEAN`이다. 이 review-only 기록의 trusted fast-pass와
최종 mergeability를 확인하기 전에는 병합하지 않는다.

## 검토 경로

- 기본 경로: `collaborator_self_merge.md`
- 적용 modifier: `intake_and_review.md`, `local_validation.md`,
  `visual_fixture_evidence.md`, `review_only_fast_pass.md`, `rework_and_exceptions.md`
- 대형 변경이므로 코드 검토·시뮬레이션·시각 증적·메인테이너 병합 판단을 분리한다.
- self PR이므로 reviewer를 지정하지 않는다.

## 변경과 경계 판단

- Q2-D5는 exact font source를 반복 준비하지 않고 page/layer 범위에서 재사용하며, key 기반 resource
  transport와 no-LineSeg atomic shaping을 연결한다.
- Q3는 variable instance의 요청·소유권·WASM adapter·bounded composer·portable publication을
  가역적으로 연결한다. 명시 요청이 없거나 지원 tuple을 벗어나면 기존 fallback을 유지한다.
- Q4는 세로쓰기 의도와 geometry를 버전이 아니라 현재 객체의 기능으로 판정하고, exact source가
  인증된 HWP5 table-cell subset에서만 atomic layout과 CanvasKit glyph replay를 활성화한다.
- malformed run, source 부재, backend capability 부재, 범위 초과는 fail-closed로 TextRun fallback을
  보존한다. partial publication과 backend-side reshaping은 허용하지 않는다.
- Q5/Q6은 이 PR 범위가 아니므로 `Refs #4969`만 사용하며 이슈를 닫지 않는다.

## self-review 결과

- production 변경의 `expect`는 선행 성공 분기에서 같은 key를 삽입하거나 `Some`을 확인한 뒤에만
  도달하는 내부 불변식이다. 외부 입력 실패는 구조화된 rejection/fallback으로 반환된다.
- cache entry 수뿐 아니라 text bytes, glyphs, clusters에 별도 상한과 checked arithmetic이 적용된다.
- 새 integration source를 만들지 않고 기존 `tests/cases/issue_4969_*.rs` 원본을 확장했다.
  generated suite·manifest·Cargo target은 추적하지 않는다.
- 새 sample, PDF, HWP/HWPX, font binary, private corpus 결과는 포함하지 않았다.
- diff는 161 files, +23,107/-329로 크지만 계획, red baseline, 구현, 측정, 최종 판정이 단계별 commit과
  기계 판독 JSON으로 연결된다. 이 계보를 분할하면 같은 atomic publication과 A/B 판정의 검토 단위가
  끊어지므로 이번 제출에서는 하나의 PR로 유지한다.
- 차단 finding: 없음.

## 로컬 검증

- `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check` - passed
- native/wasm32/workspace Clippy `-D warnings`, workspace build - passed
- source unit tier: 4,221 tests / 299 modules, base 대비 증가 없음 - passed
- integration manifest: 1,075 sources / 4,715 attrs / 48 targets - passed
- full nextest: 8,863 passed, 46 skipped, 0 failed
- CI correction 뒤 #4969 atomic activation module: 11 passed, 146 skipped - passed
- Native Skia library, missing-picture 2/2, direct PDF 4/4 - passed
- Docker Compose WASM package build - passed
- Studio unit: 1,324 passed, 1 skipped, 0 failed; production build - passed
- issue #4969 actual CanvasKit pixel replay: common horizontal 1건과 bounded vertical source 2건 - passed
- canonical native/WASM parity: 2 documents, 2 pages, mismatch 0

## 성능과 크기 판정

- 9-process A/B에서 warm layout 약 598.64배, layer build 약 339.16배 개선을 관측했다.
- 동일 Docker image와 분리된 빈 volume의 clean-room WASM A/B는 9,858,318에서 9,894,929 bytes로
  +36,611 bytes(+0.371371668%)였다. 사후 SLA는 적용하지 않았다.
- correctness, runtime parity, performance correction, causal WASM size를 모두 확보해 최종 분류는
  `qualified-bounded-subset`이다.

## 첫 CI 실패와 정정

- 최초 head `779a8ee02`의 Lint는 `header_footer_ops.rs` source-side test가 base 대비 18에서 20으로
  증가해 실패했다. 구현 오류나 포맷 실패가 아니라 새 source test를 금지한 저장소 정책 위반이다.
- 두 cache guard를 기존 `tests/cases/issue_4969_shaping_atomic_activation.rs`로 옮기면서 private cache
  슬롯 검사 대신 공개 layer JSON에서 sentinel의 표시·소실을 확인하는 integration contract로 강화했다.
- 정정 head `7a67f2508`에서 base-aware tier와 manifest, integration 모듈 11건을 재통과했다.

## 병합 전 조건

- 코드 후보 head `7a67f2508`의 Full CI는 30 success, 4 expected skip, 실패·대기 0으로 성공했다.
- Lint, Native Skia, archive A/B/C/D, frontend package, Canvas visual diff, Proptest, Adapter inter-diff,
  JavaScript/Python/Rust CodeQL이 모두 성공했다.
- 코드 후보 검증 직후 PR은 `MERGEABLE/CLEAN`, 최신 `upstream/devel@3b301f725` 대비 ahead 74,
  behind 0이며 merge-tree도 무충돌이다.
- 이 review-only 기록은 코드 후보 CI 성공 뒤 별도 commit으로 push하고, trusted fast-pass 판정과
  mergeability를 다시 확인한다.
- 병합은 최신 head 검증 뒤 메인테이너의 별도 승인에 따른다.
