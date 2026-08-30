---
kind: pr-review
status: accepted-with-ci-condition
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6450
author: t2c-lab
---

# PR #6450 review - Gmail HTML 붙여넣기 응답없음 방지

## 검토 기준

- 원 PR head: `7caca9defd7a453cea84e8357e1646b3916fa760`
- 통합 적용 commit: `262773c29`, `70c0bc3fd`, `e369c53f5`
- 기준 base: `upstream/devel@19b89d967b1505cd4bdcdbba7d1f1413f32a1505`
- 작성 시점 원 PR은 Open/non-draft였고 최신 source head의 Build & Test와 CodeQL은 성공했다
  (CodeQL aggregate는 neutral). 최종 통합 PR 직전에 상태를 다시 확인한다.

## 변경과 메인터너 보정

- 깊은 HTML tree의 재귀 깊이와 입력 크기를 상한으로 두고, 중복 span 탐색을 제거하며, 개행 없는 장문을
  조판 전 cap 단위로 나눈다. 관련 원인은 [#6449](https://github.com/edwardkim/rhwp/issues/6449)에 기록돼 있다.
- 원 PR에는 새 동작을 직접 고정하는 Rust contract가 없었다. 통합 branch에는 production logic을 바꾸지
  않는 메인터너 테스트 보정을 추가했다. 큰 markup fallback과 4,000-character 장문 분할을 검증한다.
- `cargo test --locked --profile release-test --target-dir target/pr-review --lib
  document_core::commands::html_import::tests -- --nocapture` 결과는 `6 passed`였다.

## 판단

**수용 권고.** 입력 크기/재귀/장문 줄의 세 방어 경계를 회귀 테스트로 고정했다. HWP/HWPX fixture나
사용자-visible 기준 PDF를 추가하지 않는 HTML import 안정성 변경이므로 visual sweep 대상은 아니다.
통합 branch의 최종 head Full CI와 mergeability 통과가 merge 전 조건이다.
