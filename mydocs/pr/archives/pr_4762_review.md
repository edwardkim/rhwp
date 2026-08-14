---
kind: pr-review
status: active
issue: 4761
pr: 4762
---

# PR #4762 리뷰 - HWP5 묶음 도형 재귀 깊이 제한

## 접수와 누적 적용

| 항목 | 값 |
| --- | --- |
| PR | [#4762](https://github.com/edwardkim/rhwp/pull/4762) |
| 작성자 | `kevin9327` |
| source head | `a15d2a35afccacaade8432bfc4e240f715d4567e` |
| 통합 순서 / 적용 commit | 4 / `97e38c267` |
| 통합 PR | [#4767](https://github.com/edwardkim/rhwp/pull/4767) |
| 검증 code candidate | `f97dd8a9b47298b1b6a1e3050045dd955d662c87` |

HWP5 `parse_container_children`의 중첩 group 도형 재귀를 HWP3의 기존 경계와 같은 256 깊이로
제한한다. 상한을 넘는 손상 입력은 빈 자식으로 절단해 프로세스 중단을 피하고, 정상 group 중첩은
보존한다. 원 변경에는 메인터너 보정이 필요하지 않았다.

## 완료한 검증

- `cargo test --profile release-test --target-dir target/pr-review --lib group_nesting -- --nocapture`:
  2 passed.
- 전체 release-test nextest: 6,021 passed, 38 skipped, 6 slow.
- Clippy, `git diff --check`, 최신 base merge-tree: 통과.
- #4767 code candidate의 GitHub Build & Test, CodeQL, Lint, Native Skia, Canvas visual diff: 통과.

**권고: 수용.** trailing docs-only fast-pass의 aggregate와 최신 mergeability가 녹색이면
#4767로 반영하고 [#4761](https://github.com/edwardkim/rhwp/issues/4761)의 종료 상태를 merge 뒤 확인한다.
