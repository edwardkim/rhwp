---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4948 검토 - 구역 시작 secd/cold 순서 보존

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4948](https://github.com/edwardkim/rhwp/pull/4948) |
| 작성자 / source | @planet6897 / `fix/3367-secd-cold-order` |
| 원 source head | `43acd85f1705ded12f859e217e77b508a282be92` |
| 기준 devel | `76e407b127c261427854172990bde6b2e1793edf` |
| 가시성 검토 branch | `review/planet6897-20260816-r6` |
| local 적용 commit | `83d2996df4c4d8baaf0fd5dcb44f9b11ddf66d76` |
| 원 PR 상태 참고값 | `MERGEABLE` / `CLEAN` |

HWPX writer가 구역 시작 문단의 템플릿 순서를 고정해, IR에서 `cold`가 `secd`보다 앞선 문서도
`secd` 먼저 방출하던 문제를 고친다. 첫 문단의 제어 순서가 `cold → secd`인 경우만 `colPr` 블록을
앞으로 이동해 원 IR 순서를 보존한다.

## 검증

| 범위 | 명령 또는 근거 | 결과 |
| --- | --- | --- |
| 구역 순서 회귀 | `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --test issue_3367_secd_cold_order --test-threads 12 --no-fail-fast` | 1 passed |
| 누적 전체 Rust | `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` | 6,514 passed, 38 skipped, 7 slow, 378.554초 |
| 품질 | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `git diff --check upstream/devel...HEAD` | 통과 |
| 원 source CI | 최신 head의 Build & Test와 필수 분석 job | 성공; CodeQL aggregate는 `NEUTRAL` |

이 변경은 HWPX XML 구조의 순서 보존 경계다. 회귀는 HWP→HWPX→재파싱으로 다구역 `cold`/`secd`의
상대 위치를 확인한다. renderer/layout/paint 경로는 변경하지 않아 별도 pixel sweep은 적용하지 않았다.

## 판단

고정 템플릿을 일반화하지 않고 실제 IR이 역순인 경우에만 보정하며, 기존 순서는 그대로 둔다.
최신 누적 후보에서 추가 메인터너 보정이나 충돌 해소가 필요하지 않았다. **통합 수용 권고.**
