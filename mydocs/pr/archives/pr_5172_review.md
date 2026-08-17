---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-17
---

# PR #5172 검토 - HWP3 빈 셀 문단 보정

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#5172](https://github.com/edwardkim/rhwp/pull/5172) |
| 작성자 / source | `planet6897` / `fix/hwp3-empty-cell-paragraph` |
| 원 source head | `301c3b03831d6c6e3b1d05b7018120a4f72de9e1` |
| 기준 | `devel` |
| 통합 검토 branch | `review/planet6897-20260817` |
| 원 PR 상태 | 작성 시점 `OPEN` / `DIRTY`; required checks와 CodeQL은 성공 또는 해당 없음 |
| 규모 | 10 files, +481 / -11 |
| 선행 관계 | #4984 공통 변경 및 종료된 #5159 변경을 포함한 stack PR |
| 관련 이슈 | #4367 참조; #5159는 작성 시점 `CLOSED` |

## 변경 범위

- HWP3의 빈 표 셀에 최소 한 개의 빈 문단을 생성해 HWP5 LIST_HEADER 계약을 만족시킨다.
- 셀 목록을 행 우선으로 정렬하는 선행 변경과 빈 글상자 제거·다각형·사각형·수식 계약을 함께 포함한다.
- 빈 글상자와 빈 셀, 표 셀 순서 fixture 및 회귀 테스트·generated manifest를 추가·갱신한다.

이 PR은 #4984의 공통 history를 포함하는 stack PR이다. 통합 검토에서는 #4984를 먼저 적용하고,
#5172의 공통 선행 commit은 중복 적용하지 않았다. PR 내부의 `6c23b8107` merge commit은 제외하고,
고유한 `94d46efb7`, `33535410b`, `301c3b038`을 순서대로 적용했다. `unit-test-tiers.json`의 줄번호
충돌은 최종 소스 위치에 맞춰 해결한 뒤 generator check로 확인했다.

## 로컬 적용과 검증

`upstream/devel@d4cf27eeb` 기준 `review/planet6897-20260817`에 다음 순서로 누적했다.

1. #4984 고유 변경
2. #5136 고유 변경
3. #5172의 #4984 공통분을 제외한 고유 변경

최종 통합 head에서 다음을 실행했다.

- `cargo fmt --all -- --check` 통과
- `node scripts/rust-test-suite-manifest.mjs --check` 통과
- `node scripts/rust-unit-test-tiers.mjs --check` 통과
- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`
  통과: **6538 passed, 38 skipped, 8 slow**
- `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` 통과
- `git diff --check` 통과
- 신규 #4367 계약 focused test 7 passed, #5136 계약 focused test 2 passed

HWP3 parser·serializer와 fixture를 변경하므로 전체 Rust 회귀를 수행했다. renderer·Studio UI 변경은
없어 WASM·브라우저 시각 검증은 적용하지 않았다. source PR에 기록된 한컴 COM 실측은 검토했지만 Linux
검토 서버에서 COM을 재실행하지 않았다.

## 판단

빈 셀 문단 보정과 누적 HWP3 저장 계약은 현재 통합 head에서 전체 회귀·clippy와 생성 manifest 검사를
통과했다. 추가 메인터너 코드 보정은 필요하지 않다. 다만 원 PR은 작성 시점에 GitHub에서 `DIRTY`이므로
최종 통합 전 최신 head·mergeable·required checks를 다시 확인해야 한다. **로컬 통합 수용 권고.**

