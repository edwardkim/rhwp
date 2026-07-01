# PR #1715 리뷰 — #1705 어울림 표 트레일링 빈 문단 페이지 귀속 정정

- PR: #1715 `Task #1705: 어울림 표 트레일링 빈 문단을 sw 기반으로 앵커/끝 페이지에 귀속`
- 작성자: @planet6897
- 기준: `devel`
- 검토 대상 head: `17fee877b0deed7c4c5d72c9d63fd4527c67c4c4` (문서 작성 시점 참고값)
- 규모: 4 files, +98/-5
- 관련 이슈: #1705
- 문서 작성 시점 상태: `MERGEABLE`, GitHub `Build & Test` / CodeQL 진행 중
- 처리 결과: 2026-07-01 `3f22b8dd0919d82dbd9c89a8ade73249fe7a04b9` merge 완료
- 후속 처리: #1705 수동 close 완료, PR 감사 코멘트 완료

## 변경 요약

당초 #1705는 작은 표가 한글보다 과대분할된다는 이슈였지만, 후속 조사에서 실제 불일치는
어울림(floating) 표 직후 빈 문단의 문단→페이지 귀속 문제로 재정의됐다. #1700은
`wrap_around_paras`를 표의 마지막 페이지에 귀속했으나, 한글은 표 옆 wrap zone의 좁은 폭 빈 문단을
표의 첫(앵커) 페이지에 둔다.

이번 PR은 `src/document_core/queries/rendering.rs`의 `dump_page_items()` 표면화 로직을 수정한다.

- 표 문단의 첫 출현 페이지 `item_first_page` 매핑 추가
- wrap 문단의 첫 `line_seg.segment_width`를 px로 변환
- `segment_width < body_area.width * 0.9`이면 wrap zone으로 보고 표의 첫 페이지에 귀속
- 그 외 전체폭 문단은 기존처럼 표의 마지막 페이지에 귀속

레이아웃/페이지네이션 변경이 아니라 `dump-pages` / `dump_page_items()` 쿼리 출력의 페이지 매핑 정정이다.

## 변경 범위

- 코드: `src/document_core/queries/rendering.rs`
- 검증 샘플:
  - `samples/task1705/wrap_empty_para_anchor_page.hwp`
  - `samples/task1705/README.md`
  - 기존 대비 케이스: `samples/task1700/myeonjeok_wrap_10page.hwp`
- 보고서: `mydocs/report/task_m100_1705_report.md`

## 로컬 검증

임시 worktree `/private/tmp/rhwp-pr1715-review`에서 PR head를 가져와 검증했다.

- `git merge upstream/devel --no-commit --no-ff`: 충돌 없음 (`Already up to date`)
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo build --profile release-test --bin rhwp`: 통과
- `rhwp dump-pages samples/task1705/wrap_empty_para_anchor_page.hwp`: 2 pages, page 1에 `WrapAroundPara pi=2 table_pi=1 "(빈)"` 확인
- `rhwp dump-pages samples/task1700/myeonjeok_wrap_10page.hwp`: 10 pages, page 10에 `WrapAroundPara pi=2 table_pi=1 "(빈)"` 유지 확인
- `rhwp dump-pages samples/task1700/byeolpyo1_uujeong_wrap_singlepage.hwp`: 1 page, 단일 페이지 표의 `WrapAroundPara pi=2` 유지 확인
- 관련 회귀 subset:
  - `issue_1139_inline_picture_duplicate`
  - `issue_1488_rowbreak_empty_overlay_pages`
  - `issue_643`
  - `page_number_propagation`
  - 결과: 89 passed
- `cargo fmt --check`: 통과
- `git diff --check upstream/devel...HEAD`: 통과

## GitHub Actions

최종 merge 전 PR head `17fee877b0deed7c4c5d72c9d63fd4527c67c4c4` 기준:

- Render Diff preflight: success
- Canvas visual diff: success
- CodeQL preflight: success
- CodeQL Analyze: success
- CI preflight: success
- Build & Test: success (`28500953207`)
- WASM Build: skipped

중간 커밋 `9d0b98b1da4cb4c296ee4c5e2d76f9d890419198`의 오래된 Actions run은 취소 처리했다.

- Render Diff: cancelled
- CodeQL: cancelled
- CI: cancelled

최종 merge 전 최신 head 기준 GitHub Actions 통과를 확인했다.

## 리뷰 결과

Blocking finding 없음.

변경은 `dump_page_items()`의 표면화 페이지 선택에 한정되어 있고, 동봉 샘플에서 앵커 페이지 귀속과
기존 표-끝 페이지 귀속이 모두 확인됐다. 관련 wrap/query 회귀 subset도 통과했다.

## 리스크 / 후속 확인

- PR에 Rust 자동 회귀 테스트 파일은 추가되지 않았다. 실제 HWP 샘플과 로컬 `dump-pages` 검증으로 동작을 확인했다.
- `closingIssuesReferences`가 비어 있어 #1705 auto-close가 실패했다. merge 후 수동 close 처리했다.
- 최신 head의 GitHub `Build & Test`와 CodeQL 통과 확인 후 merge했다.

## 최종 판단

수용 및 merge 완료.

- PR merge: https://github.com/edwardkim/rhwp/pull/1715
- merge commit: `3f22b8dd0919d82dbd9c89a8ade73249fe7a04b9`
- #1705 close comment: https://github.com/edwardkim/rhwp/issues/1705#issuecomment-4851881453
- PR 후속 comment: https://github.com/edwardkim/rhwp/pull/1715#issuecomment-4851883890
