---
kind: pr-review
status: active
issue: 4759
pr: 4760
---

# PR #4760 리뷰 - HWPX 섹션 상호재귀 깊이 제한

## 접수와 누적 적용

| 항목 | 값 |
| --- | --- |
| PR | [#4760](https://github.com/edwardkim/rhwp/pull/4760) |
| 작성자 | `kevin9327` |
| source head | `c17c3159393097183233434d98726885b63883b3` |
| 통합 순서 / 적용 commit | 3 / `37e8f7a0b` |
| 통합 PR | [#4767](https://github.com/edwardkim/rhwp/pull/4767) |
| 검증 code candidate | `f97dd8a9b47298b1b6a1e3050045dd955d662c87` |

`parse_paragraph`를 경유하는 표, 글상자, sub-list 재귀에 깊이 제한을 두어 악성 HWPX가
native stack을 고갈시키는 것을 방지한다. RAII guard를 사용해 오류 반환과 조기 반환에서도 depth가
복구되며, 정상 깊이 문서는 계속 수용한다. 원 변경에는 메인터너 보정이 필요하지 않았다.

## 완료한 검증

- `cargo test --profile release-test --target-dir target/pr-review --lib table_nesting -- --nocapture`:
  2 passed.
- HWPX 정상·초과 depth 경로를 포함한 source 회귀와 통합 release-test nextest:
  6,021 passed, 38 skipped, 6 slow.
- `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings`, `git diff --check`,
  최신 base merge-tree: 통과.
- #4767 code candidate의 GitHub Build & Test, CodeQL, Lint, Native Skia, Canvas visual diff: 통과.

**권고: 수용.** trailing docs-only head의 fast-pass 완료 후 #4767에 반영하고,
[#4759](https://github.com/edwardkim/rhwp/issues/4759)의 자동/수동 종료 상태를 merge 뒤 확인한다.
