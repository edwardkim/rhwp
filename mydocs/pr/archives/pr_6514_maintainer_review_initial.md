---
doc_kind: pr_review
title: "PR #6514 메인테이너 리뷰"
status: archived
issue: 5678
pr: 6514
reviewed_at: 2026-09-01
---

# PR #6514 메인테이너 리뷰

> **후속 판정으로 대체됨:** 이 문서는 원 PR head의 최초 차단 판정을 보존한 기록이다.
> 공개 test-only API 제거와 특성화 범위 정정은 PR #6541의 `ad877288b`에서 완료됐고,
> 최종 판정과 merge 계보는 [`pr_6514_review.md`](pr_6514_review.md)를 정본으로 삼는다.

## 1. 판정

**변경 요청(현재 상태에서는 merge 보류)**

새 회귀 테스트가 확인하려는 산술 불변식과 다섯 테스트의 실행 결과는 유효하다. 그러나
integration test에서 private 구현을 직접 호출하기 위해 제품 crate에 무조건 공개되는
`fit_test_internals` 모듈과 공개 심볼을 추가한 것은 현재 저장소의 회귀 테스트 경계와 맞지 않는다.
또한 양수 자간 trim의 실제 잉크 경계를 측정하지 않은 채 해당 산술을 "결함이 아닌 선언된 계약"으로
확정해서는 안 된다.

## 2. 검토 경로와 기준점

- 기여자: `planet6897` — 기존 기여자 외부 PR 경로
- PR head: `b643b3822edccaa234133fc4cf2701910b090b8f`
- 검토 base: `upstream/devel` `336c4526e9cc5047d6dd9906ebc8d0d5ee6f2188`
- 최신 base와의 비커밋 merge simulation: 충돌 없음
- 연계 이슈: #5678 — PR은 `Refs #5678`이며 이슈는 계속 열려 있음

## 3. 발견 사항

### Blocker 1 — 테스트 전용 공개 API가 제품 경계에 추가됨

`src/renderer/composer.rs`의 `pub mod fit_test_internals`와
`src/renderer/composer/line_breaking.rs`에서 공개된 함수·상수·구조체는 `#[doc(hidden)]`이지만
Rust 가시성은 그대로 `pub`이다. `#[doc(hidden)]`은 rustdoc 노출만 감출 뿐 API를 private으로
바꾸지 않는다. 신규 integration test가 `rhwp::renderer::composer::fit_test_internals`를 외부
crate처럼 import하여 컴파일되는 사실이 이 공개성을 직접 증명한다.

특히 변경 주석은 "rhwp의 API가 아니다", "시험이 밖에서 부르는 한 가지 이유로만 pub"이라고
설명한다. 이는 `CONTRIBUTING.md`의 다음 정책과 긴장한다.

- 공개 API로 재현 가능한 테스트는 `tests/cases/`에 작성한다.
- 제품 source에 테스트 support를 추가하지 않는다.
- private 구현 불변식이나 새 내부 crate 경계가 필요하면 별도 단계에서 근거와 기준선 변경을 검토한다.

따라서 다음 중 하나가 필요하다.

1. 실제 공개 조판 경로로 작은 문단/문서를 구성하고 양수·음수 자간의 줄 나눔 결과를 검증한다.
2. private 불변식을 직접 검증해야 하는 예외라면 공개 test API를 즉시 추가하지 말고, 그 예외와
   의도된 내부 경계를 별도 검토 대상으로 제시한다.

### Blocker 2 — 양수 자간의 실제 잉크 근거 없이 논쟁 중인 동작을 계약으로 확정함

`positive_spacing_trim_can_flip_the_fit_verdict`는 산술적으로 trim이 fit 판정을 뒤집는 지점을
확인한다. 그러나 실제 glyph ink, 줄 끝 clipping/overrun, HWP/HWPX 또는 한컴 출력 정답지를
측정하지 않는다. #5678 문제 2가 요구한 것은 양수 자간 상쇄가 실제 조판에서 안전한지 확인하는
일이지, 현재 산술을 자기 자신으로 고정하는 것만은 아니다.

따라서 이 테스트는 "현재 구현의 characterization"으로는 유효하지만, 실제 잉크/오라클 근거가
확보되기 전에는 "결함이 아닌 선언된 계약"이라고 단정해서는 안 된다. 주석과 테스트 명세를
잠정 성격으로 낮추거나, 실제 공개 조판 경로와 잉크 경계 증거를 추가해야 한다.

### 비차단 관찰 — #5678 전체 해결이 아니라 부분 회귀 구속임

PR은 문제 1의 production caller 정합성이 이미 현행 코드에서 해결되었다고 설명하고, 문제 3의
per-character allocation은 변경하지 않는다. 신규 테스트도 실제 `resolved_letter_spacing_px`,
live fill 및 재-fit call site를 통과하지 않는다. 그러므로 이 변경이 수정되어 merge되더라도
#5678은 닫지 않고 나머지 조사와 최적화를 추적해야 한다.

## 4. PR #6541과의 통합 관계

collaborator PR #6541의 commit `c8708e2d88f94685dea3dd2613e3794682362b31`은 #6514 최종
aggregate diff와 동일한 stable patch-id
`9b9bd5a2c27fd4cac263681b5c8f21c94a3c6950`을 가진다. 즉 #6541에는 #6514 변경이 정확히
포함되어 있다. 현재 #6541도 open/mergeable/CLEAN이며 CI가 성공했다.

따라서 위 blocker는 #6541에도 그대로 적용된다. #6541에서 수정된 통합 patch를 검토·병합한 뒤
원본 #6514는 중복 병합하지 않고 통합 사실을 남겨 close하는 것이 안전하다.

## 5. 로컬 검증 결과

| 검증 | 결과 |
|---|---|
| 최신 `upstream/devel` 비커밋 merge simulation | PASS, 충돌 없음 |
| `git diff --cached --check` | PASS |
| 신규 integration test 집중 실행 | PASS, 5/5 |
| `cargo fmt --all -- --check` | PASS |
| native Clippy `-D warnings` | PASS |
| WASM lib Clippy `-D warnings` | PASS |
| workspace build | PASS |
| workspace all-target Clippy `-D warnings` | PASS |
| integration suite manifest check | PASS, 1098 sources / 48 targets |
| source unit-tier check | PASS, 4221 tests / 299 modules |
| PR #6514 exact-head GitHub checks | PASS |
| 통합 PR #6541 exact-head GitHub checks | PASS |

집중 실행에서는 로컬 `cargo-nextest` 0.9.137이 저장소 권장 0.9.140보다 낮다는 경고만 있었고,
테스트 결과에는 영향을 주지 않았다.

## 6. 권고 처리 순서

1. #6514에 위 두 blocker를 review comment로 전달한다.
2. 실제 통합 대상인 #6541에서 공개 test-only API 제거와 계약 표현/근거를 정정한다.
3. 정정 head를 최신 `devel`에 다시 merge simulation하고 필수 Rust 검증을 반복한다.
4. #6541 병합 후 #6514에는 통합 PR을 명시하고 중복 없이 close한다.
5. #5678은 실제 양수 자간 잉크 판정과 allocation 과제가 남으므로 open 상태를 유지한다.
