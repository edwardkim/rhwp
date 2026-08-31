---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6420
author: kevin9327
---

# PR #6420 review - 부분 표 프레임의 누락 테두리 보정

## 검토 대상

- 원 PR head: `404cfb41c256040978afc479548f78afb64126f6`.
- 통합 적용 최종 commit: `8463255f`.
- 검증 base: `upstream/devel@3afbb066fe93724ab44309163a2e04efb954bf18`; PR 직전
  `upstream/devel@cfa4ccacab63b470771720ebed33503cdd62adb6`로 충돌 없이 rebase했다.
- 2026-08-31 재조회에서 Open/non-draft, requested reviewer는 비어 있어 `postmelee` 지정 대상이 아니다.
- 원 head의 Build & Test, Lint, Native Skia, Archive A-D, adapter/proptest가 성공했다. CodeQL의 neutral은
  실패 conclusion이 아니다.

## 변경과 검토

- 표 자신은 SOLID borderFill을 가지지만 바깥 칸이 NONE인 부분 프레임에서, 이미 칸이 그린 변은 중복하지 않고
  비어 있는 바깥 슬롯만 table border로 메운다.
- 초기에 occupancy+NONE 바깥 변 전체를 보충해 다른 표의 단 침범과 snapshot 회귀를 일으킨 이력이 있었으나,
  source head는 "바깥 SOLID가 하나라도 있는 부분 프레임"으로 조건을 좁힌 후 CI를 통과했다.
- 합성 HWPX 문서의 `issue_6311_table_border_fill_emits_left_bottom_and_title_left`가 왼쪽, 아래,
  제목 왼쪽 테두리를 render tree에서 직접 잠근다. 외부 HWP/HWPX 기준 PDF가 붙지 않은 합성 fixture이므로
  visual sweep 대상은 아니다.

## 통합 검증

- 통합 후보에서 fmt, native/WASM clippy, workspace build, all-target clippy, test-suite manifest 및
  Rust unit tier check를 실행해 모두 통과했다.
- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`
  를 완료해 `8870 passed, 46 skipped` (450.949초, exit 0)를 확인했다.
- rebase는 충돌 없이 적용됐으며 추가 로컬 회귀는 수행하지 않았다. 최종 PR head의 CI 통과를 merge 조건으로 둔다.

## 판단과 후속 comment 계획

**수용 권고.** 부분 프레임으로 범위를 제한한 구현과 render-tree 회귀가 결함을 직접 고정하며, 통합 전체
회귀가 통과했다. 통합 PR merge 뒤 source PR에는 적용한 source head, full nextest 결과와 원래 CI 성공을
간결히 남기고 integration PR 수용으로 close한다. 이 PR은 외부 기준 PDF가 없는 합성 fixture이므로
visual asset comment는 게시하지 않는다.
